//! Authentication integration tests.
//!
//! Tests the authentication mechanisms using Selenium OAuth2 flow
//! for both web UI and API endpoints.

mod common;

use common::{auth_helpers, environment, page_objects::GamePage};
use reqwest::StatusCode;

const TEST_USERNAME: &str = "admin@local.test";
const TEST_PASSWORD: &str = "password";

// =============================================================================
// OAuth2 Login Flow Tests (Selenium)
// =============================================================================

#[tokio::test]
async fn test_oauth2_login_flow() {
    use auth_helpers::create_webdriver;

    // Environment checks in blocking context
    let (browser_url, selenium_url) = tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        let browser_url = environment::browser_base_url();
        let selenium_url = environment::ensure_selenium_ready();
        (browser_url, selenium_url)
    })
    .await
    .expect("Environment checks failed");

    // Create WebDriver with direct await
    let driver = create_webdriver(&selenium_url)
        .await
        .expect("Failed to create WebDriver");

    let page = GamePage::new(&driver);

    // Navigate to protected page - should redirect to Keycloak
    page.goto(browser_url.as_str())
        .await
        .expect("Failed to navigate to app");

    println!("Navigated to app, checking for Keycloak redirect");

    // Verify we're on Keycloak login page
    let on_login_page = page
        .is_on_login_page()
        .await
        .expect("Failed to check login page");

    assert!(on_login_page, "Not redirected to Keycloak login page");

    println!("Redirected to Keycloak login page");

    // Perform login
    page.login(TEST_USERNAME, TEST_PASSWORD)
        .await
        .expect("Failed to login");

    println!("Successfully logged in");

    // Verify we're back at the app (not on Keycloak)
    let current_url = driver
        .current_url()
        .await
        .expect("Failed to get current URL");

    assert!(
        !current_url.as_str().contains("keycloak"),
        "Still on Keycloak after login: {current_url}"
    );

    println!("Redirected back to application");

    // Verify session cookie is set
    let cookies = driver
        .get_all_cookies()
        .await
        .expect("Failed to get cookies");

    let has_session_cookie = cookies.iter().any(|c| c.name == "_oauth2_proxy");

    assert!(has_session_cookie, "Session cookie not set");

    println!("Session cookie set");

    // Cleanup
    page.quit().await.ok();

    println!("OAuth2 login flow test passed");
}

// =============================================================================
// Unauthenticated Request Tests
// =============================================================================

#[tokio::test]
async fn test_unauthenticated_web_ui_redirects_to_login() {
    // Environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_unauthenticated_client();

    // Try to access protected page without authentication
    let response = client
        .get("http://localhost:8080")
        .send()
        .await
        .expect("Should send request");

    // oauth2-proxy should redirect to Keycloak (302) or return 401
    assert!(
        response.status().is_redirection() || response.status() == StatusCode::UNAUTHORIZED,
        "Unauthenticated request should be redirected or rejected, got: {}",
        response.status()
    );

    // If it's a redirect, verify it's to Keycloak
    if response.status().is_redirection()
        && let Some(location) = response.headers().get("location")
    {
        let location_str = location.to_str().unwrap_or("");
        assert!(
            location_str.contains("keycloak") || location_str.contains("oauth2"),
            "Redirect should be to Keycloak login, got: {location_str}"
        );
        println!("Redirected to Keycloak: {location_str}");
    }

    println!("Unauthenticated web UI redirect test passed");
}

#[tokio::test]
async fn test_unauthenticated_api_returns_401() {
    // Environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_unauthenticated_client();

    // Try to create a game via oauth2-proxy without authentication
    let response = client
        .post("http://localhost:8080/api/games")
        .json(&serde_json::json!({
            "min": 1,
            "max": 100
        }))
        .send()
        .await
        .expect("Should send request");

    // oauth2-proxy should redirect or return 401
    assert!(
        response.status().is_redirection() || response.status() == StatusCode::UNAUTHORIZED,
        "Unauthenticated API request should be rejected, got: {}",
        response.status()
    );

    println!("Unauthenticated API 401 test passed");
}

// =============================================================================
// Authenticated Endpoint Tests (Web UI via Selenium)
// =============================================================================

#[tokio::test]
async fn test_web_ui_endpoints_work_when_authenticated() {
    use auth_helpers::create_webdriver;

    // Environment checks in blocking context
    let (browser_url, selenium_url) = tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        let browser_url = environment::browser_base_url();
        let selenium_url = environment::ensure_selenium_ready();
        (browser_url, selenium_url)
    })
    .await
    .expect("Environment checks failed");

    // Create WebDriver with direct await
    let driver = create_webdriver(&selenium_url)
        .await
        .expect("Failed to create WebDriver");

    let page = GamePage::new(&driver);

    // Navigate and login
    page.goto(browser_url.as_str())
        .await
        .expect("Failed to navigate");

    page.login(TEST_USERNAME, TEST_PASSWORD)
        .await
        .expect("Failed to login");

    println!("Logged in successfully");

    // Try to start a game (tests POST /game/new)
    page.start_game(1, 100, Some(10))
        .await
        .expect("Failed to start game");

    println!("Started game successfully");

    // Verify game interface is visible
    let game_started = page
        .is_game_started()
        .await
        .expect("Failed to check if game started");

    assert!(
        game_started,
        "Game interface not visible after starting game"
    );

    println!("Game interface visible");

    // Cleanup
    page.quit().await.ok();

    println!("Authenticated web UI endpoints test passed");
}

// =============================================================================
// Authenticated Endpoint Tests (API via Selenium OAuth2)
// =============================================================================

#[tokio::test]
async fn test_api_endpoints_work_when_authenticated() {
    // Environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client
    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create authenticated client");

    println!("Created authenticated API client");

    // Test POST /api/games
    let create_response = client
        .post("http://localhost:8080/api/games")
        .json(&serde_json::json!({
            "min": 1,
            "max": 100,
            "max_guesses": "10"
        }))
        .send()
        .await
        .expect("Failed to send create game request");

    assert!(
        create_response.status().is_success(),
        "Create game failed with status: {}",
        create_response.status()
    );

    let game: serde_json::Value = create_response
        .json()
        .await
        .expect("Failed to parse game response");

    let game_id = game["game_id"]
        .as_u64()
        .expect("Game response missing game_id");

    println!("Created game with ID: {game_id}");

    // Test POST /api/games/{id}/guess
    let guess_response = client
        .post(format!("http://localhost:8080/api/games/{game_id}/guess"))
        .json(&serde_json::json!({
            "guess": 50
        }))
        .send()
        .await
        .expect("Failed to send guess request");

    assert!(
        guess_response.status().is_success(),
        "Guess failed with status: {}",
        guess_response.status()
    );

    println!("Made guess successfully");

    println!("Authenticated API endpoints test passed");
}
