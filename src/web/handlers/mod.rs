//! Web request handlers organized by feature.
//!
//! Each module contains handlers related to a specific game feature,
//! keeping the codebase organized and scalable.

pub mod difficulty;
pub mod game;
pub mod guess;
pub mod health;

// Re-export handler functions for easy access
pub use difficulty::difficulty_preview;
pub use game::{create_game_api, create_game_web};
pub use guess::{make_guess_api, make_guess_web};
pub use health::health_check;
