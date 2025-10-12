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
