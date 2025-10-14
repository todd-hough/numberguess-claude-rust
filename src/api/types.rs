//! Request and response types for JSON API endpoints.

use crate::core::GameId;
use serde::{Deserialize, Deserializer, Serialize};

/// Custom deserializer for optional u32 that treats empty strings as None.
///
/// This handles API inputs where an empty field comes through as an empty string
/// rather than being omitted entirely. Also accepts numeric strings.
pub fn deserialize_option_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(ref s) if s.is_empty() => Ok(None),
        Some(s) => s.parse::<u32>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

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
