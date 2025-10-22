mod common;

use common::assertions::{
    GameResponse, GuessResponse, assert_game_in_range, assert_limit_reached,
    assert_valid_game_response,
};
use common::{auth_helpers, environment};
use serde_json::json;

// API tests use Selenium OAuth2 authentication via oauth2-proxy (port 8080)
// oauth2-proxy validates the session cookie and adds X-Forwarded-* headers before proxying to the app
const API_BASE_URL: &str = "http://localhost:8080";

#[tokio::test]
async fn test_guess_nonexistent_game() {
    // Run environment checks in blocking context to avoid runtime conflicts
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required for authentication");
    })
    .await
    .expect("Environment checks failed");

    // Use Selenium OAuth2 authentication for API tests
    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    let response = client
        .post(format!("{}/api/games/99999999/guess", API_BASE_URL))
        .json(&json!({"guess": 50}))
        .send()
        .await
        .expect("Should send POST request to guess on nonexistent game");

    assert_eq!(
        response.status().as_u16(),
        404,
        "Should return 404 for nonexistent game"
    );
    println!("✅ Nonexistent game test passed");
}

#[tokio::test]
async fn test_concurrent_games() {
    // Run environment checks in blocking context to avoid runtime conflicts
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required for authentication");
    })
    .await
    .expect("Environment checks failed");

    // Use Selenium OAuth2 authentication for API tests
    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Create 3 games (converted from map to for loop for async/await)
    let mut game_ids: Vec<i64> = Vec::new();
    for _ in 0..3 {
        let resp = client
            .post(format!("{}/api/games", API_BASE_URL))
            .json(&json!({"min": 1, "max": 10}))
            .send()
            .await
            .expect("Should send POST request to create game");

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "Could not read body".to_string());
            panic!("Game creation failed with status {}: {}", status, body);
        }
        let game: GameResponse = resp.json().await.expect("Should parse JSON game response");
        game_ids.push(game.game_id);
    }

    println!("✅ Created {} concurrent games", game_ids.len());

    // Make guess to each game
    for game_id in &game_ids {
        let resp = client
            .post(format!("{}/api/games/{}/guess", API_BASE_URL, game_id))
            .json(&json!({"guess": 5}))
            .send()
            .await
            .expect("Should send POST request to make guess");

        assert!(
            resp.status().is_success(),
            "Guess should succeed for game {}",
            game_id
        );

        let guess_result: GuessResponse = resp.json().await.expect("Should parse JSON guess response");
        println!("Game {} result: {}", game_id, guess_result.result);
    }

    println!("✅ Concurrent games test passed");
}

#[tokio::test]
async fn test_guess_after_limit_reached() {
    // Run environment checks in blocking context to avoid runtime conflicts
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required for authentication");
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Create game with limit=1 and min=max so we know the exact answer
    let resp = client
        .post(format!("{}/api/games", API_BASE_URL))
        .json(&json!({"min": 50, "max": 50, "max_guesses": "1"}))
        .send()
        .await
        .expect("Should send POST request to create game");

    assert!(
        resp.status().is_success(),
        "Game creation should succeed with status 200, got {}",
        resp.status()
    );
    let game: GameResponse = resp.json().await.expect("Should parse JSON game response");
    println!(
        "✅ Created game with ID {} and limit {:?} (answer is 50)",
        game.game_id, game.max_guesses
    );

    // Validate game structure
    assert_valid_game_response(&game);
    assert_game_in_range(&game, 50, 50);

    // Make a wrong guess - since we have limit=1, this should return limit_reached
    // We know the answer is 50, so guessing 49 will definitely be wrong
    let first_guess_resp = client
        .post(format!("{}/api/games/{}/guess", API_BASE_URL, game.game_id))
        .json(&json!({"guess": 49}))
        .send()
        .await
        .expect("Should send POST request for first guess");

    assert!(
        first_guess_resp.status().is_success(),
        "First guess should be accepted"
    );
    let first_guess: GuessResponse = first_guess_resp
        .json()
        .await
        .expect("Should parse JSON response for first guess");
    println!(
        "First guess result: {}, message: {}",
        first_guess.result, first_guess.message
    );

    // With limit=1, the first wrong guess should exhaust the limit and return limit_reached
    assert_limit_reached(&first_guess);

    // Try another guess - should get 404 since game is removed after limit reached
    let second_guess_resp = client
        .post(format!("{}/api/games/{}/guess", API_BASE_URL, game.game_id))
        .json(&json!({"guess": 50}))
        .send()
        .await
        .expect("Should send POST request for second guess");

    assert_eq!(
        second_guess_resp.status().as_u16(),
        404,
        "Second guess should return 404 not found (game removed), got {}",
        second_guess_resp.status()
    );

    println!("✅ Limit enforcement test passed");
}

#[tokio::test]
async fn test_zero_limit_means_unlimited() {
    // Run environment checks in blocking context to avoid runtime conflicts
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required for authentication");
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Create game WITHOUT max_guesses field (omitted = unlimited)
    let resp = client
        .post(format!("{}/api/games", API_BASE_URL))
        .json(&json!({
            "min": 1,
            "max": 100
            // max_guesses omitted = unlimited
        }))
        .send()
        .await
        .expect("Should send POST request to create game");

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        panic!("Game creation failed with {}: {}", status, body);
    }

    let game: GameResponse = resp.json().await.expect("Should parse JSON game response");

    // Verify max_guesses is None (unlimited)
    assert_eq!(
        game.max_guesses, None,
        "max_guesses should be None when omitted"
    );
    println!("✅ Created game {} with unlimited guesses", game.game_id);

    // Make many guesses to verify it's truly unlimited
    for i in 1..=15 {
        let guess_resp = client
            .post(format!("{}/api/games/{}/guess", API_BASE_URL, game.game_id))
            .json(&json!({"guess": i}))
            .send()
            .await
            .expect("Should send guess");

        assert!(
            guess_resp.status().is_success(),
            "Guess {} should succeed with unlimited limit",
            i
        );

        let guess_result: GuessResponse = guess_resp.json().await.expect("Should parse guess response");

        // Should never get limit_reached with unlimited
        assert_ne!(
            guess_result.result, "limit_reached",
            "Should not reach limit with unlimited guesses"
        );

        if guess_result.result == "correct" {
            println!(
                "✅ Found correct answer after {} guesses (unlimited worked)",
                i
            );
            break;
        }
    }

    println!("✅ Unlimited (omitted max_guesses) test passed");
}

#[tokio::test]
async fn test_web_rejects_excessive_guess_limit() {
    // Run environment checks in blocking context to avoid runtime conflicts
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required for authentication");
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // Web API should reject max_guesses > 100 (send as strings for API compatibility)
    let excessive_limits = vec!["101", "150", "1000"];

    for limit in excessive_limits {
        println!("Testing excessive limit: {}", limit);

        let resp = client
            .post(format!("{}/api/games", API_BASE_URL))
            .json(&json!({
                "min": 1,
                "max": 10,
                "max_guesses": limit
            }))
            .send()
            .await
            .expect("Should send POST request");

        let status = resp.status();
        assert!(
            status.as_u16() >= 400 && status.as_u16() < 500,
            "Should reject max_guesses={} with 4xx error (got {})",
            limit,
            status
        );

        // Log error message
        if let Ok(body) = resp.text().await {
            println!(
                "  Correctly rejected with status {} and message: {}",
                status, body
            );
        }
    }

    // Verify exactly 100 is accepted (boundary)
    let resp = client
        .post(format!("{}/api/games", API_BASE_URL))
        .json(&json!({
            "min": 1,
            "max": 10,
            "max_guesses": "100"
        }))
        .send()
        .await
        .expect("Should send POST request");

    assert!(
        resp.status().is_success(),
        "Should accept max_guesses=100 (exactly at limit)"
    );
    let game: GameResponse = resp.json().await.expect("Should parse game response");
    assert_eq!(game.max_guesses, Some(100));
    println!("✅ Accepted max_guesses=100 (at boundary)");

    println!("✅ Excessive limit rejection test passed");
}
