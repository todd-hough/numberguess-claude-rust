mod common;

use common::containers::GameServerInstance;
use reqwest::blocking::Client;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GameResponse {
    game_id: u64,
    min: u32,
    max: u32,
    max_guesses: Option<u32>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuessResponse {
    result: String,
    attempts: u32,
    message: String,
}

#[test]
fn test_guess_nonexistent_game() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    let response = client
        .post(format!("{}/api/games/99999999/guess", server.url()))
        .json(&json!({"guess": 50}))
        .send()
        .unwrap();
    
    assert_eq!(response.status().as_u16(), 404, "Should return 404 for nonexistent game");
    println!("✅ Nonexistent game test passed");
}

#[test]
fn test_concurrent_games() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Create 3 games
    let game_ids: Vec<u64> = (0..3)
        .map(|_| {
            let resp = client
                .post(format!("{}/api/games", server.url()))
                .json(&json!({"min": 1, "max": 10}))
                .send()
                .unwrap();
            
            assert!(resp.status().is_success(), "Game creation should succeed");
            let game: GameResponse = resp.json().unwrap();
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
            .unwrap();
        
        assert!(resp.status().is_success(), "Guess should succeed for game {}", game_id);
        
        let guess_result: GuessResponse = resp.json().unwrap();
        println!("Game {} result: {}", game_id, guess_result.result);
    }
    
    println!("✅ Concurrent games test passed");
}

#[test]
fn test_guess_after_limit_reached() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Create game with limit=1 and min=max so we know the exact answer
    let resp = client
        .post(format!("{}/api/games", server.url()))
        .json(&json!({"min": 50, "max": 50, "max_guesses": "1"}))
        .send()
        .unwrap();
    
    assert!(resp.status().is_success(), 
        "Game creation should succeed with status 200, got {}", resp.status());
    let game: GameResponse = resp.json().unwrap();
    println!("✅ Created game with ID {} and limit {:?} (answer is 50)", game.game_id, game.max_guesses);
    
    // Make a wrong guess - since we have limit=1, this should return limit_reached
    // We know the answer is 50, so guessing 49 will definitely be wrong
    let first_guess_resp = client
        .post(format!("{}/api/games/{}/guess", server.url(), game.game_id))
        .json(&json!({"guess": 49}))
        .send()
        .unwrap();
    
    assert!(first_guess_resp.status().is_success(), "First guess should be accepted");
    let first_guess: GuessResponse = first_guess_resp.json().unwrap();
    println!("First guess result: {}, message: {}", first_guess.result, first_guess.message);
    
    // With limit=1, the first wrong guess should exhaust the limit and return limit_reached
    assert_eq!(first_guess.result, "limit_reached", 
        "First wrong guess with limit=1 should return 'limit_reached', got '{}'", 
        first_guess.result);
    
    // Try another guess - should get 404 since game is removed after limit reached
    let second_guess_resp = client
        .post(format!("{}/api/games/{}/guess", server.url(), game.game_id))
        .json(&json!({"guess": 50}))
        .send()
        .unwrap();
    
    assert_eq!(second_guess_resp.status().as_u16(), 404, 
        "Second guess should return 404 not found (game removed), got {}",
        second_guess_resp.status());
    
    println!("✅ Limit enforcement test passed");
}