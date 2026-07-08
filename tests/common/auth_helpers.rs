//! Authentication helpers for integration tests.
//!
//! Provides tier-aware authentication: Selenium OAuth2 login (full tier) or
//! pass-through clients for the nginx mock-auth proxy (light tier, MOCK_AUTH=1).

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
    let timeout = Duration::from_secs(30); // Increased for slow systems
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if driver.query(By::Id("kc-login")).nowait().exists().await? {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let current_url = driver.current_url().await?;
    Err(WebDriverError::Timeout(format!(
        "Timeout waiting for Keycloak login page. Current URL: {}",
        current_url.as_str()
    )))
}

/// Wait for OAuth2 redirect back to application.
async fn wait_for_oauth2_redirect(driver: &WebDriver) -> WebDriverResult<()> {
    let timeout = Duration::from_secs(30); // Increased for slow systems
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let url = driver.current_url().await?;
        let url_str = url.as_str();
        // We're back at the app when URL doesn't contain keycloak or oauth2/callback
        if !url_str.contains("keycloak") && !url_str.contains("oauth2/callback") {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Get final URL for error message
    let final_url = driver.current_url().await?;
    Err(WebDriverError::Timeout(format!(
        "Timeout waiting for OAuth2 redirect. Still at: {}",
        final_url.as_str()
    )))
}

/// Extract oauth2-proxy session cookie from WebDriver.
async fn extract_session_cookie(driver: &WebDriver) -> WebDriverResult<Cookie> {
    let cookies = driver.get_all_cookies().await?;

    for cookie in &cookies {
        if cookie.name == "_oauth2_proxy" {
            return Ok(cookie.clone());
        }
    }

    // Provide helpful error with available cookies
    let cookie_names: Vec<&str> = cookies.iter().map(|c| c.name.as_str()).collect();
    let current_url = driver.current_url().await?;
    Err(WebDriverError::FatalError(format!(
        "oauth2-proxy session cookie not found. URL: {}, Available cookies: {:?}",
        current_url.as_str(),
        cookie_names
    )))
}

/// Create an authenticated reqwest client for the active test tier.
///
/// - **Full tier** (default): performs a full OAuth2 login via Selenium,
///   extracts the oauth2-proxy session cookie, and returns a client carrying it.
/// - **Light tier** (`MOCK_AUTH=1`, `make test-func`): the nginx mock-auth
///   proxy injects the identity headers, so no login is needed — returns a
///   plain client with a cookie store (still required for CSRF cookies).
///
/// # Example
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = create_authenticated_client().await?;
/// let resp = client.get("http://localhost:8080")
///     .send()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_authenticated_client() -> Result<reqwest::Client, Box<dyn Error>> {
    // Cookie store is always enabled: full tier stores the oauth2-proxy
    // session cookie, and both tiers need it for CSRF token cookies.
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());

    if !environment::is_mock_auth() {
        // DESIGN CHOICE — no session-cookie caching, one full browser OAuth2
        // login (~2-3s) per call. A per-binary cache (tokio OnceCell) was
        // tried and removed: in the current tier split each full-tier binary
        // makes at most ONE call here (auth_integration_test has one call
        // site; web_ui_test drives the browser directly), so a cache never
        // gets a second hit and only adds complexity. If the full tier ever
        // gains multiple authenticated-client tests per binary, reintroduce
        // caching then — and keep such tests in the light tier when they
        // don't assert on the real auth stack (see CLAUDE.md "Two-Tier
        // Integration Test Architecture").
        let caps = DesiredCapabilities::chrome();
        let selenium_url = environment::selenium_url();
        let driver = WebDriver::new(&selenium_url, caps).await?;

        // Perform OAuth2 login
        let session_cookie = login_with_keycloak_selenium(&driver).await?;

        // Close WebDriver (quit() consumes the driver and handles cleanup)
        driver.quit().await?;

        // Add the oauth2-proxy session cookie to the jar
        let cookie_url = "http://localhost:8080".parse::<reqwest::Url>().unwrap();
        jar.add_cookie_str(
            &format!("_oauth2_proxy={}; Path=/", session_cookie.value),
            &cookie_url,
        );
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .cookie_provider(jar)
        .build()?;

    Ok(client)
}

/// Helper function to create a WebDriver for Selenium tests.
pub async fn create_webdriver(selenium_url: &str) -> WebDriverResult<WebDriver> {
    // Selenium only exists in the full tier. Fail with a clear message instead
    // of a confusing connection error if a browser test is run in mock mode
    // (e.g. `MOCK_AUTH=1 cargo test` without the Makefile's --test filter).
    assert!(
        !environment::is_mock_auth(),
        "This test requires Selenium and the full auth stack, which do not run in mock mode. \
         Run it via `make test-auth` (without MOCK_AUTH)."
    );
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
