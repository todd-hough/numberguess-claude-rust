//! Request and response types for JSON API endpoints.

use crate::core::GameId;
use crate::serde_helpers::deserialize_option_u32;
use serde::{Deserialize, Serialize};

// Game creation types

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub min: i32,
    pub max: i32,
    #[serde(default, deserialize_with = "deserialize_option_u32")]
    pub max_guesses: Option<u32>,
}

#[derive(Serialize)]
pub struct CreateGameResponse {
    pub game_id: GameId,
    pub min: i32,
    pub max: i32,
    pub max_guesses: Option<u32>,
    pub message: String,
}

// Guess handling types

#[derive(Deserialize)]
pub struct MakeGuessRequest {
    pub guess: i32,
}

/// Outcome of a guess as exposed by the JSON API.
///
/// Serializes to the exact legacy string values ("too_low", "too_high",
/// "correct", "limit_reached") via `rename_all = "snake_case"` — the wire
/// format is part of the external API and must not change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuessOutcome {
    TooLow,
    TooHigh,
    Correct,
    LimitReached,
}

#[derive(Serialize)]
pub struct MakeGuessResponse {
    pub result: GuessOutcome,
    pub message: String,
    pub attempts: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_outcome_serializes_to_legacy_strings() {
        // These exact strings are the external API contract (docs/api.md).
        assert_eq!(
            serde_json::to_string(&GuessOutcome::TooLow).unwrap(),
            "\"too_low\""
        );
        assert_eq!(
            serde_json::to_string(&GuessOutcome::TooHigh).unwrap(),
            "\"too_high\""
        );
        assert_eq!(
            serde_json::to_string(&GuessOutcome::Correct).unwrap(),
            "\"correct\""
        );
        assert_eq!(
            serde_json::to_string(&GuessOutcome::LimitReached).unwrap(),
            "\"limit_reached\""
        );
    }
}

// Error types

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
