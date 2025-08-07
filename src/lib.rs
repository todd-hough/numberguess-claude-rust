pub mod game;
pub mod cli;

// Re-export commonly used items for convenience
pub use game::{GuessingGame, GuessResult};
pub use cli::{Cli, read_input, get_min_value, get_max_value};