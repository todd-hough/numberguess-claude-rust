//! CLI interface module.
//!
//! Handles command-line argument parsing, user I/O, and the CLI game loop.

pub mod args;
pub mod io;
pub mod runner;

// Re-export commonly used items
pub use args::Cli;
pub use runner::run_cli_game;
