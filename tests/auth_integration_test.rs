//! Authentication integration tests.
//!
//! Tests the authentication mechanisms using both Selenium OAuth2 flow
//! and programmatic Direct Access Grants.

mod common;

use common::{auth_helpers, environment, page_objects::GamePage};
use reqwest::StatusCode;

const TEST_USERNAME: &str = "admin@local.test";
const TEST_PASSWORD: &str = "password";

// =============================================================================
// OAuth2 Login Flow Tests (Selenium)
// =============================================================================

#[test]
fn test_oauth2_login_flow() {
    let browser_url = environment::browser_base_url();
    let selenium_url = match environment::ensure_selenium_ready() {
        Some(url) => url,
        None => {
            println!("Skipping OAuth2 login test - Selenium not available");
            return;
        }
    };

    let result = tokio_test::block_on(async move {
        use auth_helpers::create_webdriver;

        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                eprintln!("Failed to create WebDriver: {}", e);
                return false;
            }
        };

        let page = GamePage::new(&driver);

        // Navigate to protected page - should redirect to Keycloak
        if let Err(e) = page.goto(browser_url.as_str()).await {
            eprintln!("Failed to navigate to app: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Navigated to app, checking for Keycloak redirect");

        // Verify we're on Keycloak login page
        let on_login_page = match page.is_on_login_page().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to check login page: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        if !on_login_page {
            eprintln!("Not redirected to Keycloak login page");
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Redirected to Keycloak login page");

        // Perform login
        if let Err(e) = page.login(TEST_USERNAME, TEST_PASSWORD).await {
            eprintln!("Failed to login: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Successfully logged in");

        // Verify we're back at the app (not on Keycloak)
        let current_url = match driver.current_url().await {
            Ok(url) => url,
            Err(e) => {
                eprintln!("Failed to get current URL: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        if current_url.as_str().contains("keycloak") {
            eprintln!("Still on Keycloak after login: {}", current_url);
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Redirected back to application");

        // Verify session cookie is set
        let cookies = match driver.get_all_cookies().await {
            Ok(cookies) => cookies,
            Err(e) => {
                eprintln!("Failed to get cookies: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        let has_session_cookie = cookies.iter().any(|c| c.name == "_oauth2_proxy");

        if !has_session_cookie {
            eprintln!("Session cookie not set");
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Session cookie set");

        if let Err(e) = page.quit().await {
            eprintln!("Failed to quit WebDriver: {}", e);
        }

        true
    });

    assert!(result, "OAuth2 login flow should complete successfully");
    println!("✅ OAuth2 login flow test passed");
}

// =============================================================================
// Programmatic Authentication Tests
// =============================================================================

#[test]
fn test_programmatic_authentication_works() {
    environment::ensure_server_ready();

    // Get access token
    let token = auth_helpers::get_access_token(TEST_USERNAME, TEST_PASSWORD)
        .expect("Should get access token");

    println!("✓ Got access token from Keycloak");

    // Verify token is a JWT (3 parts separated by dots)
    assert_eq!(token.split('.').count(), 3, "Token should be valid JWT");

    // Verify we can use the token to make authenticated requests
    let client = auth_helpers::create_authenticated_client_programmatic()
        .expect("Should create authenticated client");

    println!("✓ Created authenticated client with bearer token");

    // Try to create a game via API (directly to app on port 4080)
    let response = client
        .post("http://localhost:4080/api/games")
        .json(&serde_json::json!({
            "min": 1,
            "max": 100,
            "max_guesses": null
        }))
        .send()
        .expect("Should send request");

    assert!(
        response.status().is_success(),
        "API request with bearer token should succeed"
    );

    println!("✓ Successfully made authenticated API request");
    println!("✅ Programmatic authentication test passed");
}

// =============================================================================
// Unauthenticated Request Tests
// =============================================================================

#[test]
fn test_unauthenticated_web_ui_redirects_to_login() {
    environment::ensure_server_ready();

    let client = auth_helpers::create_unauthenticated_client();

    // Try to access protected page without authentication
    let response = client
        .get("http://localhost:8080")
        .send()
        .expect("Should send request");

    // oauth2-proxy should redirect to Keycloak (302) or return 401
    assert!(
        response.status().is_redirection() || response.status() == StatusCode::UNAUTHORIZED,
        "Unauthenticated request should be redirected or rejected, got: {}",
        response.status()
    );

    // If it's a redirect, verify it's to Keycloak
    if response.status().is_redirection() {
        if let Some(location) = response.headers().get("location") {
            let location_str = location.to_str().unwrap_or("");
            assert!(
                location_str.contains("keycloak") || location_str.contains("oauth2"),
                "Redirect should be to Keycloak login, got: {}",
                location_str
            );
            println!("✓ Redirected to Keycloak: {}", location_str);
        }
    }

    println!("✅ Unauthenticated web UI redirect test passed");
}

#[test]
fn test_unauthenticated_api_returns_401() {
    environment::ensure_server_ready();

    let client = auth_helpers::create_unauthenticated_client();

    // Try to create a game via oauth2-proxy without authentication
    let response = client
        .post("http://localhost:8080/api/games")
        .json(&serde_json::json!({
            "min": 1,
            "max": 100
        }))
        .send()
        .expect("Should send request");

    // oauth2-proxy should redirect or return 401
    assert!(
        response.status().is_redirection() || response.status() == StatusCode::UNAUTHORIZED,
        "Unauthenticated API request should be rejected, got: {}",
        response.status()
    );

    println!("✅ Unauthenticated API 401 test passed");
}

// =============================================================================
// Authenticated Endpoint Tests (Web UI via Selenium)
// =============================================================================

#[test]
fn test_web_ui_endpoints_work_when_authenticated() {
    let browser_url = environment::browser_base_url();
    let selenium_url = match environment::ensure_selenium_ready() {
        Some(url) => url,
        None => {
            println!("Skipping web UI authenticated test - Selenium not available");
            return;
        }
    };

    let result = tokio_test::block_on(async move {
        use auth_helpers::create_webdriver;

        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                eprintln!("Failed to create WebDriver: {}", e);
                return false;
            }
        };

        let page = GamePage::new(&driver);

        // Navigate and login
        if let Err(e) = page.goto(browser_url.as_str()).await {
            eprintln!("Failed to navigate: {}", e);
            let _ = page.quit().await;
            return false;
        }

        if let Err(e) = page.login(TEST_USERNAME, TEST_PASSWORD).await {
            eprintln!("Failed to login: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Logged in successfully");

        // Try to start a game (tests POST /game/new)
        if let Err(e) = page.start_game(1, 100, Some(10)).await {
            eprintln!("Failed to start game: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Started game successfully");

        // Verify game interface is visible
        let game_started = match page.is_game_started().await {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to check if game started: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        if !game_started {
            eprintln!("Game interface not visible after starting game");
            let _ = page.quit().await;
            return false;
        }

        println!("✓ Game interface visible");

        if let Err(e) = page.quit().await {
            eprintln!("Failed to quit WebDriver: {}", e);
        }

        true
    });

    assert!(
        result,
        "Authenticated web UI endpoints should work correctly"
    );
    println!("✅ Authenticated web UI endpoints test passed");
}

// =============================================================================
// Authenticated Endpoint Tests (API via Programmatic)
// =============================================================================

#[test]
fn test_api_endpoints_work_when_authenticated() {
    environment::ensure_server_ready();

    let client = auth_helpers::create_authenticated_client_programmatic()
        .expect("Should create authenticated client");

    println!("✓ Created authenticated API client");

    // Test POST /api/games
    let create_response = client
        .post("http://localhost:4080/api/games")
        .json(&serde_json::json!({
            "min": 1,
            "max": 100,
            "max_guesses": 10
        }))
        .send()
        .expect("Should send create game request");

    assert!(
        create_response.status().is_success(),
        "Create game should succeed with authentication"
    );

    let game: serde_json::Value = create_response
        .json()
        .expect("Should parse game response");

    let game_id = game["game_id"]
        .as_u64()
        .expect("Game should have game_id");

    println!("✓ Created game with ID: {}", game_id);

    // Test POST /api/games/{id}/guess
    let guess_response = client
        .post(format!("http://localhost:4080/api/games/{}/guess", game_id))
        .json(&serde_json::json!({
            "guess": 50
        }))
        .send()
        .expect("Should send guess request");

    assert!(
        guess_response.status().is_success(),
        "Guess should succeed with authentication"
    );

    println!("✓ Made guess successfully");

    println!("✅ Authenticated API endpoints test passed");
}
