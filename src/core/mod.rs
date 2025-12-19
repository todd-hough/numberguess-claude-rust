//! Core game logic module.
//!
//! Contains all business logic with no I/O dependencies.
//! This includes game mechanics, validation, type definitions, and features.

pub mod errors;
pub mod features;
pub mod game;
pub mod game_id;
pub mod validators;

// Re-export commonly used items for convenience
pub use errors::{GameError, GuessResult};
pub use game::GuessingGame;
pub use game_id::GameId;
pub use validators::{
    MAX_CLI_GUESS_LIMIT, MAX_RANGE, MAX_WEB_GUESS_LIMIT, validate_guess_limit,
    validate_max_gte_min, validate_max_value, validate_min_value, validate_range,
};
