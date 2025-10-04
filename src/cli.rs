//! CLI argument parsing using clap.
//!
//! This module handles command-line argument parsing only.
//! Validation logic is in the validators module.
//! User I/O logic is in the io module.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "number_guessing_game")]
#[command(about = "A fun number guessing game", long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Minimum number (inclusive)")]
    pub min: Option<i32>,

    #[arg(short = 'x', long, help = "Maximum number (inclusive)")]
    pub max: Option<i32>,

    #[arg(short = 'l', long, help = "Maximum number of guesses allowed")]
    pub limit: Option<u32>,

    #[arg(short, long, help = "Run as a web server")]
    pub server: bool,

    #[arg(short, long, default_value = "8080", help = "Port for the web server")]
    pub port: u16,
}
