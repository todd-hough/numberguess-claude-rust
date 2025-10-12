use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameResponse {
    pub game_id: i64,
    pub min: u32,
    pub max: u32,
    pub max_guesses: Option<u32>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuessResponse {
    pub result: String,
    pub attempts: Option<u32>,
    pub message: String,
}

/// Assert that a game response has valid structure and values
pub fn assert_valid_game_response(game: &GameResponse) {
    assert!(
        game.game_id > 0,
        "Game ID should be positive, got {}",
        game.game_id
    );
    assert!(
        game.min <= game.max,
        "Min ({}) should be <= max ({})",
        game.min,
        game.max
    );
    assert!(!game.message.is_empty(), "Game message should not be empty");
}

/// Assert that a game response matches expected range values
pub fn assert_game_in_range(game: &GameResponse, expected_min: u32, expected_max: u32) {
    assert_eq!(
        game.min, expected_min,
        "Game min should be {}, got {}",
        expected_min, game.min
    );
    assert_eq!(
        game.max, expected_max,
        "Game max should be {}, got {}",
        expected_max, game.max
    );
}

/// Assert that a guess response has valid structure and matches expected result
#[allow(dead_code)]
pub fn assert_valid_guess_response(response: &GuessResponse, expected_result: &str) {
    assert_eq!(
        response.result, expected_result,
        "Expected result '{}', got '{}'",
        expected_result, response.result
    );
    assert!(
        !response.message.is_empty(),
        "Guess response message should not be empty"
    );
}

/// Assert that a "correct" guess response has all required fields
pub fn assert_correct_guess(response: &GuessResponse) {
    assert_eq!(
        response.result, "correct",
        "Result should be 'correct', got '{}'",
        response.result
    );
    assert!(
        response.attempts.is_some(),
        "Correct guess should include attempt count"
    );
    assert!(
        response.attempts.unwrap() > 0,
        "Attempt count should be > 0, got {}",
        response.attempts.unwrap()
    );
    assert!(
        response.message.to_lowercase().contains("correct")
            || response.message.to_lowercase().contains("got it"),
        "Message should indicate success, got: '{}'",
        response.message
    );
}

/// Assert that a "too_low" or "too_high" guess response is valid
#[allow(dead_code)]
pub fn assert_incorrect_guess(response: &GuessResponse, expected_direction: &str) {
    assert!(
        response.result == "too_low" || response.result == "too_high",
        "Result should be 'too_low' or 'too_high', got '{}'",
        response.result
    );
    if !expected_direction.is_empty() {
        assert_eq!(
            response.result, expected_direction,
            "Expected '{}', got '{}'",
            expected_direction, response.result
        );
    }
    assert!(
        !response.message.is_empty(),
        "Message should not be empty for incorrect guess"
    );
}

/// Assert that a "limit_reached" response is valid
pub fn assert_limit_reached(response: &GuessResponse) {
    assert_eq!(
        response.result, "limit_reached",
        "Result should be 'limit_reached', got '{}'",
        response.result
    );
    assert!(
        response.message.to_lowercase().contains("limit")
            || response.message.to_lowercase().contains("ran out")
            || response.message.to_lowercase().contains("attempts"),
        "Message should indicate limit reached, got: '{}'",
        response.message
    );
}
