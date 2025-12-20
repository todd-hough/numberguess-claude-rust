//! Request types for web UI (HTML forms).

use serde::{Deserialize, Deserializer};

/// Custom deserializer for optional u32 that treats empty strings as None.
///
/// This handles HTML form inputs where an empty field comes through as an empty string
/// rather than being omitted entirely.
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
