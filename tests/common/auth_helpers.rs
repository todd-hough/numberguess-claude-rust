//! Authentication helpers for integration tests.
//!
//! Provides Selenium-based OAuth2 authentication for all tests (Web UI and API).

use reqwest::header::{HeaderMap, HeaderValue};
use std::error::Error;
use std::time::Duration;
use thirtyfour::prelude::*;

use super::environment;

// Constants for authentication
const TEST_USERNAME: &str = "admin@local.test";
const TEST_PASSWORD: &str = "password";

// =============================================================================
// Selenium-Based OAuth2 Authentication
// Used for all tests that require authentication
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
pub async fn login_with_keycloak_selenium(driver: &WebDriver) -> WebDriverResult<Cookie> {
    // Navigate to protected page - will redirect to Keycloak
    // Use browser_base_url which respects GAME_SERVER_BROWSER_URL environment variable
    // (Selenium running in Docker needs http://oauth2-proxy:8080, not localhost)
    let oauth2_proxy_url = environment::browser_base_url();
    driver.goto(&oauth2_proxy_url).await?;

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
        if driver.query(By::Id("kc-login")).nowait().exists().await? {
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
pub async fn create_authenticated_client_selenium() -> Result<reqwest::Client, Box<dyn Error>> {
    // Create temporary WebDriver
    let caps = DesiredCapabilities::chrome();
    let selenium_url =
        environment::selenium_url().ok_or_else(|| "SELENIUM_REMOTE_URL not set".to_string())?;
    let driver = WebDriver::new(&selenium_url, caps).await?;

    // Perform OAuth2 login
    let session_cookie = login_with_keycloak_selenium(&driver).await?;

    // Close WebDriver (quit() consumes the driver and handles cleanup)
    driver.quit().await?;

    // Convert Selenium cookie to reqwest header
    let cookie_str = format!("_oauth2_proxy={}", session_cookie.value);

    // Create async reqwest client (no runtime conflicts!)
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, HeaderValue::from_str(&cookie_str)?);

    let client = reqwest::Client::builder()
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
/// let resp = client.get("http://localhost:8080").send().await.unwrap();
/// assert!(resp.status().is_redirection() || resp.status() == 401);
/// ```
pub fn create_unauthenticated_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects
        .build()
        .expect("Failed to create unauthenticated client")
}
