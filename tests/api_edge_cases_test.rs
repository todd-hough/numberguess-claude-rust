mod common;

use common::assertions::{
    assert_valid_game_response, assert_game_in_range, assert_limit_reached,
    GameResponse, GuessResponse,
};
use common::containers::{GameServerInstance, PostgresInstance};
use reqwest::blocking::Client;
use serde_json::json;

#[test]
fn test_guess_nonexistent_game() {
    let postgres = PostgresInstance::new();
    let server = GameServerInstance::new(&postgres.container_url());
    let client = Client::new();
    
    let response = client
        .post(format!("{}/api/games/99999999/guess", server.url()))
        .json(&json!({"guess": 50}))
        .send()
        .expect("Should send POST request to guess on nonexistent game");
    
    assert_eq!(response.status().as_u16(), 404, "Should return 404 for nonexistent game");
    println!("✅ Nonexistent game test passed");
}

#[test]
fn test_concurrent_games() {
    let postgres = PostgresInstance::new();
    let server = GameServerInstance::new(&postgres.container_url());
    let client = Client::new();
    
    // Create 3 games
    let game_ids: Vec<i64> = (0..3)
        .map(|_| {
            let resp = client
                .post(format!("{}/api/games", server.url()))
                .json(&json!({"min": 1, "max": 10}))
                .send()
                .expect("Should send POST request to create game");

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_else(|_| "Could not read body".to_string());
                panic!("Game creation failed with status {}: {}", status, body);
            }
            let game: GameResponse = resp.json().expect("Should parse JSON game response");
            game.game_id
        })
        .collect();
    
    println!("✅ Created {} concurrent games", game_ids.len());
    
    // Make guess to each game
    for game_id in &game_ids {
        let resp = client
            .post(format!("{}/api/games/{}/guess", server.url(), game_id))
            .json(&json!({"guess": 5}))
            .send()
            .expect("Should send POST request to make guess");

        assert!(resp.status().is_success(), "Guess should succeed for game {}", game_id);

        let guess_result: GuessResponse = resp.json().expect("Should parse JSON guess response");
        println!("Game {} result: {}", game_id, guess_result.result);
    }
    
    println!("✅ Concurrent games test passed");
}

#[test]
fn test_guess_after_limit_reached() {
    let postgres = PostgresInstance::new();
    let server = GameServerInstance::new(&postgres.container_url());
    let client = Client::new();
    
    // Create game with limit=1 and min=max so we know the exact answer
    let resp = client
        .post(format!("{}/api/games", server.url()))
        .json(&json!({"min": 50, "max": 50, "max_guesses": "1"}))
        .send()
        .expect("Should send POST request to create game");

    assert!(resp.status().is_success(),
        "Game creation should succeed with status 200, got {}", resp.status());
    let game: GameResponse = resp.json().expect("Should parse JSON game response");
    println!("✅ Created game with ID {} and limit {:?} (answer is 50)", game.game_id, game.max_guesses);

    // Validate game structure
    assert_valid_game_response(&game);
    assert_game_in_range(&game, 50, 50);
    
    // Make a wrong guess - since we have limit=1, this should return limit_reached
    // We know the answer is 50, so guessing 49 will definitely be wrong
    let first_guess_resp = client
        .post(format!("{}/api/games/{}/guess", server.url(), game.game_id))
        .json(&json!({"guess": 49}))
        .send()
        .expect("Should send POST request for first guess");

    assert!(first_guess_resp.status().is_success(), "First guess should be accepted");
    let first_guess: GuessResponse = first_guess_resp.json().expect("Should parse JSON response for first guess");
    println!("First guess result: {}, message: {}", first_guess.result, first_guess.message);

    // With limit=1, the first wrong guess should exhaust the limit and return limit_reached
    assert_limit_reached(&first_guess);
    
    // Try another guess - should get 404 since game is removed after limit reached
    let second_guess_resp = client
        .post(format!("{}/api/games/{}/guess", server.url(), game.game_id))
        .json(&json!({"guess": 50}))
        .send()
        .expect("Should send POST request for second guess");
    
    assert_eq!(second_guess_resp.status().as_u16(), 404, 
        "Second guess should return 404 not found (game removed), got {}",
        second_guess_resp.status());
    
    println!("✅ Limit enforcement test passed");
}