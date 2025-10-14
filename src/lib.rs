pub mod cli;
pub mod db;
pub mod features;
pub mod game;
pub mod game_id;
pub mod io;
pub mod templates;
pub mod validators;
pub mod web;

// Re-export commonly used items for convenience
pub use cli::Cli;
pub use game::{GameError, GuessResult, GuessingGame};
pub use game_id::GameId;
