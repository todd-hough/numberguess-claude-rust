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

#[derive(Serialize)]
pub struct MakeGuessResponse {
    pub result: String,
    pub message: String,
    pub attempts: Option<u32>,
}

// Error types

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
