mod common;

use common::assertions::{
    GameResponse, GuessResponse, assert_correct_guess, assert_game_in_range,
    assert_valid_game_response,
};
use common::environment;
use reqwest::blocking::Client;
use serde_json::json;

// NOTE: This test is superseded by authenticated API tests in api_edge_cases_test.rs and auth_integration_test.rs
// Keeping for reference but ignoring since all endpoints now require authentication
#[test]
#[ignore = "Superseded by authenticated API tests"]
fn test_basic_game_flow() {
    let base_url = environment::ensure_server_ready();

    println!("Using game server at {}", base_url);

    // Create HTTP client
    let client = Client::new();

    // Step 1: Create a new game with simple parameters
    let game_data = json!({
        "min": 1,
        "max": 10,
        "limit": 5
    });

    let create_response = client
        .post(format!("{}/api/games", base_url))
        .json(&game_data)
        .send()
        .expect("Should send POST request to create game");

    assert!(
        create_response.status().is_success(),
        "Game creation should return a success status"
    );

    let game: GameResponse = create_response
        .json()
        .expect("Should parse JSON game response");

    println!("Game created with ID: {}", game.game_id);

    // Use comprehensive assertion helpers
    assert_valid_game_response(&game);
    assert_game_in_range(&game, 1, 10);

    // Step 2: Try all possible numbers in range
    // This ensures we'll find the correct number regardless of what it is
    let game_id = game.game_id;
    let mut result = String::new();

    for guess_num in 1..=10 {
        println!("Making guess: {}", guess_num);

        let guess_response = client
            .post(format!("{}/api/games/{}/guess", base_url, game_id))
            .json(&json!({ "guess": guess_num }))
            .send()
            .expect("Should send POST request to make guess");

        if !guess_response.status().is_success() {
            panic!("Guess failed with status: {}", guess_response.status());
        }

        let guess_result: GuessResponse = guess_response
            .json()
            .expect("Should parse JSON guess response");

        println!("Guess result: {}", guess_result.result);
        result = guess_result.result.clone();

        if guess_result.result == "correct" {
            println!("Found the correct number: {}", guess_num);

            // Use comprehensive assertion for correct guess
            assert_correct_guess(&guess_result);
            break;
        }
    }

    // We should have found the correct answer among all the numbers we tried
    assert_eq!(
        result, "correct",
        "Should eventually find the correct number"
    );

    println!("Basic game flow test passed at {}", base_url);
}

// NOTE: This test is superseded by authenticated API tests
// Keeping for reference but ignoring since all endpoints now require authentication
#[test]
#[ignore = "Superseded by authenticated API tests"]
fn test_invalid_game_parameters() {
    let base_url = environment::ensure_server_ready();

    println!("Using game server at {}", base_url);

    // Create HTTP client
    let client = Client::new();

    // Test invalid game parameters
    let invalid_game_data = vec![
        // Min > Max
        json!({
            "min": 100,
            "max": 10,
            "limit": 5
        }),
        // Min negative (should be rejected)
        json!({
            "min": -10,
            "max": 10,
            "limit": 5
        }),
        // Max exceeds allowed limit
        json!({
            "min": 1,
            "max": 2000000, // Over 1,000,000
            "limit": 5
        }),
    ];

    for (i, game_data) in invalid_game_data.iter().enumerate() {
        println!("Testing invalid game data case {}: {:?}", i, game_data);

        let create_response = client
            .post(format!("{}/api/games", base_url))
            .json(game_data)
            .send()
            .expect("Should send POST request with invalid game parameters");

        assert!(
            create_response.status().as_u16() >= 400,
            "Invalid game should be rejected with 4xx status, got {}",
            create_response.status()
        );
    }

    println!("Invalid game parameters test passed at {}", base_url);
}
