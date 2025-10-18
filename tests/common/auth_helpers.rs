//! Authentication helpers for integration tests.
//!
//! Provides two authentication strategies:
//! 1. Selenium-based OAuth2 flow (for Web UI tests)
//! 2. Programmatic Direct Access Grants (for API tests)

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;
use thirtyfour::prelude::*;

// Constants for authentication
const KEYCLOAK_TOKEN_URL: &str =
    "http://localhost:8090/realms/numberguess/protocol/openid-connect/token";
const TEST_CLIENT_ID: &str = "test-client";
const TEST_CLIENT_SECRET: &str = "test-secret-do-not-use-in-production";
const TEST_USERNAME: &str = "admin@local.test";
const TEST_PASSWORD: &str = "password";
const OAUTH2_PROXY_URL: &str = "http://localhost:8080";
const SELENIUM_URL: &str = "http://localhost:4444";

/// Response from Keycloak token endpoint
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    expires_in: u64,
    #[allow(dead_code)]
    refresh_token: Option<String>,
}

// =============================================================================
// Programmatic Authentication (Direct Access Grants)
// Used for fast API tests that bypass oauth2-proxy
// =============================================================================

/// Get an access token from Keycloak using Direct Access Grants (ROPC flow).
///
/// This bypasses oauth2-proxy and gets a JWT token directly from Keycloak.
/// The token can be used with Bearer authentication against the app on port 4080.
///
/// # Example
/// ```no_run
/// let token = get_access_token("admin@local.test", "password").unwrap();
/// // Use token in Authorization: Bearer header
/// ```
pub fn get_access_token(username: &str, password: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let params = [
        ("grant_type", "password"),
        ("client_id", TEST_CLIENT_ID),
        ("client_secret", TEST_CLIENT_SECRET),
        ("username", username),
        ("password", password),
    ];

    let response = client
        .post(KEYCLOAK_TOKEN_URL)
        .form(&params)
        .send()?
        .error_for_status()?;

    let token_response: TokenResponse = response.json()?;
    Ok(token_response.access_token)
}

/// Create a reqwest client with programmatic authentication (Bearer token).
///
/// This client adds an Authorization: Bearer header to all requests.
/// Use this for API tests that access the app directly on port 4080.
///
/// # Example
/// ```no_run
/// let client = create_authenticated_client_programmatic().unwrap();
/// let resp = client.post("http://localhost:4080/api/games")
///     .json(&body)
///     .send()
///     .unwrap();
/// ```
pub fn create_authenticated_client_programmatic() -> Result<Client, Box<dyn Error>> {
    let token = get_access_token(TEST_USERNAME, TEST_PASSWORD)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .default_headers(headers)
        .build()?;

    Ok(client)
}

// =============================================================================
// Selenium-Based OAuth2 Authentication
// Used for Web UI tests that test the full OAuth2 flow through oauth2-proxy
// =============================================================================

/// Perform OAuth2 login via Selenium and return session cookie.
///
/// This performs the full OAuth2 authorization code flow:
/// 1. Navigate to oauth2-proxy protected page
/// 2. Get redirected to Keycloak login page
/// 3. Fill in credentials and submit
/// 4. Wait for OAuth2 callback redirect
/// 5. Extract session cookie
///
/// The session cookie can then be used with reqwest for subsequent requests.
///
/// # Example
/// ```no_run
/// use thirtyfour::prelude::*;
/// # async fn example() -> WebDriverResult<()> {
/// let driver = create_webdriver("http://localhost:4444").await?;
/// let cookie = login_with_keycloak_selenium(&driver).await?;
/// # Ok(())
/// # }
/// ```
pub async fn login_with_keycloak_selenium(
    driver: &WebDriver,
) -> WebDriverResult<Cookie> {
    // Navigate to protected page - will redirect to Keycloak
    driver.goto(OAUTH2_PROXY_URL).await?;

    // Wait for redirect to Keycloak login page
    wait_for_keycloak_login_page(driver).await?;

    // Fill in username
    let username_field = driver.find(By::Id("username")).await?;
    username_field.send_keys(TEST_USERNAME).await?;

    // Fill in password
    let password_field = driver.find(By::Id("password")).await?;
    password_field.send_keys(TEST_PASSWORD).await?;

    // Submit login form
    let submit_button = driver.find(By::Id("kc-login")).await?;
    submit_button.click().await?;

    // Wait for OAuth2 redirect back to app
    wait_for_oauth2_redirect(driver).await?;

    // Extract oauth2-proxy session cookie
    extract_session_cookie(driver).await
}

/// Wait for Keycloak login page to appear.
async fn wait_for_keycloak_login_page(driver: &WebDriver) -> WebDriverResult<()> {
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if driver
            .query(By::Id("kc-login"))
            .nowait()
            .exists()
            .await?
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(WebDriverError::Timeout(
        "Timeout waiting for Keycloak login page".to_string(),
    ))
}

/// Wait for OAuth2 redirect back to application.
async fn wait_for_oauth2_redirect(driver: &WebDriver) -> WebDriverResult<()> {
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let url = driver.current_url().await?;
        let url_str = url.as_str();
        // We're back at the app when URL doesn't contain keycloak or oauth2/callback
        if !url_str.contains("keycloak") && !url_str.contains("oauth2/callback") {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Extract oauth2-proxy session cookie from WebDriver.
async fn extract_session_cookie(driver: &WebDriver) -> WebDriverResult<Cookie> {
    let cookies = driver.get_all_cookies().await?;

    for cookie in cookies {
        if cookie.name == "_oauth2_proxy" {
            return Ok(cookie);
        }
    }

    Err(WebDriverError::FatalError(
        "oauth2-proxy session cookie not found".to_string(),
    ))
}

/// Create a reqwest client with Selenium-based authentication (session cookie).
///
/// This performs a full OAuth2 login via Selenium, extracts the session cookie,
/// and creates a reqwest client with that cookie.
///
/// Use this for Web UI tests that access oauth2-proxy on port 8080.
///
/// # Example
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = create_authenticated_client_selenium().await?;
/// let resp = client.get("http://localhost:8080")
///     .send()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_authenticated_client_selenium() -> Result<Client, Box<dyn Error>> {
    // Create temporary WebDriver
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new(SELENIUM_URL, caps).await?;

    // Perform OAuth2 login
    let session_cookie = login_with_keycloak_selenium(&driver).await?;

    // Close WebDriver
    driver.quit().await?;

    // Convert Selenium cookie to reqwest header
    let cookie_str = format!("_oauth2_proxy={}", session_cookie.value);

    // Create reqwest blocking client with cookie header
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        HeaderValue::from_str(&cookie_str)?,
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .default_headers(headers)
        .build()?;

    Ok(client)
}

/// Helper function to create a WebDriver for Selenium tests.
pub async fn create_webdriver(selenium_url: &str) -> WebDriverResult<WebDriver> {
    let caps = DesiredCapabilities::chrome();
    WebDriver::new(selenium_url, caps).await
}

// =============================================================================
// Unauthenticated Client
// Used for testing 401/redirect responses
// =============================================================================

/// Create a reqwest client without any authentication.
///
/// Use this for testing that unauthenticated requests are properly rejected.
///
/// # Example
/// ```no_run
/// let client = create_unauthenticated_client();
/// let resp = client.get("http://localhost:8080").send().unwrap();
/// assert!(resp.status().is_redirection() || resp.status() == 401);
/// ```
pub fn create_unauthenticated_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects
        .build()
        .expect("Failed to create unauthenticated client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires Keycloak to be running
    fn test_get_access_token() {
        let token = get_access_token(TEST_USERNAME, TEST_PASSWORD)
            .expect("Should get access token");
        assert!(!token.is_empty());
        // JWT tokens have 3 parts separated by dots
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    #[ignore] // Requires Keycloak to be running
    fn test_create_authenticated_client_programmatic() {
        let client = create_authenticated_client_programmatic()
            .expect("Should create authenticated client");

        // Verify client was created successfully
        assert!(client.get("http://localhost").build().is_ok());
    }
}
