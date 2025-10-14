//! Web UI request handlers.
//!
//! Contains handlers for HTML/HTMX web interface organized by feature.

pub mod difficulty;
pub mod game;
pub mod guess;

// Re-export handler functions for easy access
pub use difficulty::difficulty_preview;
pub use game::create_game_web;
pub use guess::make_guess_web;
