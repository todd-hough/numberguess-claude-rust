//! Request types for web UI (HTML forms).

use crate::serde_helpers::deserialize_option_u32;
use serde::Deserialize;

// Game creation types

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    pub min: i32,
    pub max: i32,
    #[serde(default, deserialize_with = "deserialize_option_u32")]
    pub max_guesses: Option<u32>,
    #[serde(default)]
    pub authenticity_token: String,
}

// Guess handling types

#[derive(Deserialize)]
pub struct MakeGuessRequest {
    pub guess: i32,
    #[serde(default)]
    pub authenticity_token: String,
}

// Difficulty preview types

#[derive(Debug, Deserialize)]
pub struct DifficultyParams {
    pub min: Option<i32>,
    pub max: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_option_u32")]
    pub max_guesses: Option<u32>,
}
