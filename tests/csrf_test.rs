mod common;

use common::{auth_helpers, environment};

#[tokio::test]
async fn test_csrf_protection_enforcement() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // 1. Create an authenticated client using Selenium OAuth2 flow
    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create authenticated client");

    // 2. Attempt POST without CSRF token
    // This should fail because we haven't provided the authenticity_token field
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[("min", "1"), ("max", "100")])
        .send()
        .await
        .expect("Should send POST request");

    // axum-csrf returns 400 Bad Request when token is missing or invalid
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "POST without CSRF token should be rejected"
    );

    // 3. Attempt POST with INVALID CSRF token
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[
            ("min", "1"),
            ("max", "100"),
            ("authenticity_token", "invalid-token-value"),
        ])
        .send()
        .await
        .expect("Should send POST request");

    assert_eq!(
        resp.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "POST with invalid CSRF token should be rejected"
    );

    // 4. Perform legitimate GET to obtain a valid CSRF token
    // The cookie jar will automatically store the x-csrf-token cookie from the response
    let resp = client
        .get("http://localhost:8080")
        .send()
        .await
        .expect("Should GET index");

    assert!(resp.status().is_success());

    // Extract token from HTML body
    let body = resp.text().await.expect("Should get body");
    let token = body
        .split("name=\"authenticity_token\" value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("Should find authenticity_token in HTML");

    // 5. Attempt POST with VALID CSRF token
    // The cookie jar automatically sends both oauth2-proxy session and x-csrf-token cookies
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[("min", "1"), ("max", "100"), ("authenticity_token", token)])
        .send()
        .await
        .expect("Should send legitimate POST request");

    assert!(
        resp.status().is_success(),
        "POST with valid CSRF token should succeed, got {}",
        resp.status()
    );

    println!("CSRF protection enforcement test passed");
}

#[tokio::test]
async fn test_csrf_token_reuse_within_session() {
    // Run environment checks
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create client");

    // 1. Get initial token
    let resp = client.get("http://localhost:8080").send().await.unwrap();
    let body1 = resp.text().await.unwrap();
    let token = body1
        .split("name=\"authenticity_token\" value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap()
        .to_string();

    // 2. Start game (first use of token)
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[("min", "1"), ("max", "100"), ("authenticity_token", &token)])
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "First POST with token should succeed"
    );
    let body2 = resp.text().await.unwrap();

    // 3. Extract game_id from response
    let game_id = body2
        .split("hx-post='/game/")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .expect("Should find game_id in response");

    // 4. Make a guess using the SAME token (axum_csrf uses per-session tokens)
    let resp = client
        .post(format!("http://localhost:8080/game/{game_id}/guess"))
        .form(&[("guess", "50"), ("authenticity_token", &token)])
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "Second POST with same token should succeed (per-session tokens)"
    );

    // 5. Make another guess to confirm token continues to work
    let resp = client
        .post(format!("http://localhost:8080/game/{game_id}/guess"))
        .form(&[("guess", "25"), ("authenticity_token", &token)])
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "Third POST with same token should succeed"
    );

    println!("CSRF token reuse within session test passed");
}
