//! REST API module.
//!
//! Provides JSON API endpoints for the number guessing game.
//! All handlers return JSON responses.

pub mod handlers;
pub mod types;

// Re-export commonly used items
pub use handlers::{create_game_api, health_check, make_guess_api};
pub use types::{
    CreateGameRequest, CreateGameResponse, ErrorResponse, MakeGuessRequest, MakeGuessResponse,
};
