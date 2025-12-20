mod common;

use common::{auth_helpers, environment};
use reqwest::header;

#[tokio::test]
async fn test_csrf_protection_enforcement() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required for authentication");
    })
    .await
    .expect("Environment checks failed");

    // 1. Create an authenticated client using Selenium OAuth2 flow
    let client = auth_helpers::create_authenticated_client_selenium()
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

    // 4. Perform legitimate GET to obtain a valid CSRF token and cookie
    let resp = client
        .get("http://localhost:8080")
        .send()
        .await
        .expect("Should GET index");

    assert!(resp.status().is_success());
    
    // Extract CSRF cookie (clone before consuming response)
    let csrf_cookie = resp.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|h| h.to_str().ok())
        .find(|c| c.contains("x-csrf-token"))
        .and_then(|c| c.split(';').next())
        .expect("Should find x-csrf-token cookie")
        .to_string();
    
    // Extract token from HTML body
    let body = resp.text().await.expect("Should get body");
    let token = body
        .split("name=\"authenticity_token\" value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("Should find authenticity_token in HTML");

    // 5. Attempt POST with VALID CSRF token and cookie
    let resp = client
        .post("http://localhost:8080/game/new")
        .header(header::COOKIE, csrf_cookie)
        .form(&[
            ("min", "1"),
            ("max", "100"),
            ("authenticity_token", token),
        ])
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
async fn test_csrf_token_rotation() {
    // Run environment checks
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required for authentication");
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create client");

    // 1. Get initial token
    let resp = client.get("http://localhost:8080").send().await.unwrap();
    let body1 = resp.text().await.unwrap();
    let token1 = body1
        .split("name=\"authenticity_token\" value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap()
        .to_string();

    // 2. Start game (submits token1)
    let resp = client
        .post("http://localhost:8080/game/new")
        .form(&[
            ("min", "1"),
            ("max", "100"),
            ("authenticity_token", &token1),
        ])
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());
    let body2 = resp.text().await.unwrap();
    
    // 3. Extract new token from game_started response
    let token2 = body2
        .split("name='authenticity_token' value='")
        .nth(1)
        .and_then(|s| s.split('\'').next())
        .expect("Should find new token in response")
        .to_string();

    assert_ne!(token1, token2, "CSRF token should rotate after use");

    // 4. Use new token for a guess
    let game_id = body2
        .split("hx-post='/game/")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap();

    let resp = client
        .post(format!("http://localhost:8080/game/{}/guess", game_id))
        .form(&[
            ("guess", "50"),
            ("authenticity_token", &token2),
        ])
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success(), "Second request with rotated token should succeed");

    println!("CSRF token rotation test passed");
}
