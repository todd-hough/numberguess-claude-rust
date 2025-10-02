pub mod cli;
pub mod db;
pub mod game;
pub mod web;

// Re-export commonly used items for convenience
pub use cli::{Cli, get_guess_limit, get_max_value, get_min_value, read_input};
pub use game::{GameError, GuessResult, GuessingGame};
