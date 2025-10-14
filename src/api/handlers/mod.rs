//! API request handlers.
//!
//! Contains handlers for JSON API endpoints organized by feature.

pub mod game;
pub mod guess;
pub mod health;

// Re-export handler functions for easy access
pub use game::create_game_api;
pub use guess::make_guess_api;
pub use health::health_check;
