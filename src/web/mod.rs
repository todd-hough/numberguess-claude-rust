//! Web UI module.
//!
//! Provides HTML/HTMX interface for the number guessing game.
//! All handlers return HTML responses rendered via Askama templates.

pub mod handlers;
pub mod templates;
pub mod types;

// Re-export commonly used items
pub use handlers::{create_game_web, difficulty_preview, make_guess_web};
pub use templates::{
    DifficultyIndicator, ErrorTemplate, GameCompleteTemplate, GameNotFoundTemplate,
    GameStartedTemplate, GuessFormTemplate, UpdateErrorTemplate,
};
