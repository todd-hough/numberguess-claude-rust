mod common;

use common::{auth_helpers, environment};

#[tokio::test]
async fn test_static_file_serving() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client using Selenium OAuth2 flow
    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Test root serves index.html
    let resp = client
        .get("http://localhost:8080")
        .send()
        .await
        .expect("Should send GET request to root URL");

    assert!(
        resp.status().is_success(),
        "Root URL should return successful response"
    );
    let body = resp.text().await.expect("Should get response body as text");
    assert!(
        body.contains("Number Guessing Game"),
        "Response should contain game title"
    );
    assert!(body.contains("<!DOCTYPE html>"), "Response should be HTML");

    println!("Static file serving test passed");
}

#[tokio::test]
async fn test_web_form_endpoints() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client using Selenium OAuth2 flow
    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Get CSRF token from index page
    let resp = client
        .get("http://localhost:8080")
        .send()
        .await
        .expect("Should GET index");
    let body = resp.text().await.expect("Should get body");
    let token = body
        .split("name=\"authenticity_token\" value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("Should find CSRF token");

    // Test form submission to /game/new with CSRF token
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[
            ("min", "1"),
            ("max", "10"),
            ("max_guesses", "5"),
            ("authenticity_token", token),
        ])
        .send()
        .await
        .expect("Should send POST request with form data");

    assert!(resp.status().is_success(), "Form submission should succeed");
    let body = resp.text().await.expect("Should get response body as text");

    // Should return HTML with game interface
    assert!(
        body.contains("guess"),
        "Response should contain game interface"
    );
    assert!(
        body.contains("form"),
        "Response should contain form element"
    );

    println!("Web form endpoints test passed");
}

#[tokio::test]
async fn test_remaining_guesses_display() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client using Selenium OAuth2 flow
    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Get CSRF token from index page
    let resp = client
        .get("http://localhost:8080")
        .send()
        .await
        .expect("Should GET index");
    let body = resp.text().await.expect("Should get body");
    let token = body
        .split("name=\"authenticity_token\" value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("Should find CSRF token")
        .to_string();

    // Create a game with a guess limit of 5
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[
            ("min", "1"),
            ("max", "100"),
            ("max_guesses", "5"),
            ("authenticity_token", &token),
        ])
        .send()
        .await
        .expect("Should create game with guess limit");

    assert!(resp.status().is_success(), "Game creation should succeed");
    let body = resp.text().await.expect("Should get response body");

    // Verify initial "Guesses remaining: 5" is displayed
    assert!(
        body.contains("Guesses remaining:"),
        "Should display 'Guesses remaining' label"
    );
    assert!(
        body.contains("<strong>5</strong>"),
        "Should display initial count of 5"
    );

    // Extract game ID from the response
    let game_id = body
        .split("hx-post='/game/")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .expect("Should find game ID in response");

    // Make first guess (same token works for session)
    let resp = client
        .post(format!("http://localhost:8080/game/{}/guess", game_id))
        .form(&[("guess", "50"), ("authenticity_token", &token)])
        .send()
        .await
        .expect("Should make first guess");

    assert!(resp.status().is_success(), "First guess should succeed");
    let body = resp.text().await.expect("Should get response body");

    // Verify remaining count decreased to 4
    assert!(
        body.contains("Guesses remaining:"),
        "Should display 'Guesses remaining' after guess"
    );
    assert!(
        body.contains("<strong>4</strong>"),
        "Should show 4 guesses remaining after first guess"
    );

    // Make second guess
    let resp = client
        .post(format!("http://localhost:8080/game/{}/guess", game_id))
        .form(&[("guess", "75"), ("authenticity_token", &token)])
        .send()
        .await
        .expect("Should make second guess");

    assert!(resp.status().is_success(), "Second guess should succeed");
    let body = resp.text().await.expect("Should get response body");

    // Verify remaining count decreased to 3
    assert!(
        body.contains("<strong>3</strong>"),
        "Should show 3 guesses remaining after second guess"
    );

    // Make third guess
    let resp = client
        .post(format!("http://localhost:8080/game/{}/guess", game_id))
        .form(&[("guess", "25"), ("authenticity_token", &token)])
        .send()
        .await
        .expect("Should make third guess");

    assert!(resp.status().is_success(), "Third guess should succeed");
    let body = resp.text().await.expect("Should get response body");

    // Verify remaining count decreased to 2
    assert!(
        body.contains("<strong>2</strong>"),
        "Should show 2 guesses remaining after third guess"
    );

    println!("Remaining guesses display test passed");
}

#[tokio::test]
async fn test_no_remaining_guesses_display_without_limit() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client using Selenium OAuth2 flow
    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Get CSRF token from index page
    let resp = client
        .get("http://localhost:8080")
        .send()
        .await
        .expect("Should GET index");
    let body = resp.text().await.expect("Should get body");
    let token = body
        .split("name=\"authenticity_token\" value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("Should find CSRF token")
        .to_string();

    // Create a game WITHOUT a guess limit
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[("min", "1"), ("max", "100"), ("authenticity_token", &token)])
        .send()
        .await
        .expect("Should create game without guess limit");

    assert!(resp.status().is_success(), "Game creation should succeed");
    let body = resp.text().await.expect("Should get response body");

    // Verify "Guesses remaining" is NOT displayed when there's no limit
    assert!(
        !body.contains("Guesses remaining:"),
        "Should not display 'Guesses remaining' when no limit is set"
    );

    // Extract game ID from the response
    let game_id = body
        .split("hx-post='/game/")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .expect("Should find game ID in response");

    // Make a guess
    let resp = client
        .post(format!("http://localhost:8080/game/{}/guess", game_id))
        .form(&[("guess", "50"), ("authenticity_token", &token)])
        .send()
        .await
        .expect("Should make guess");

    assert!(resp.status().is_success(), "Guess should succeed");
    let body = resp.text().await.expect("Should get response body");

    // Verify "Guesses remaining" is still NOT displayed after a guess
    assert!(
        !body.contains("Guesses remaining:"),
        "Should not display 'Guesses remaining' after guess when no limit"
    );

    println!("No remaining guesses display without limit test passed");
}
