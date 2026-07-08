//! CLI game runner.
//!
//! Contains the main game loop for CLI mode.

use crate::cli::args::Cli;
use crate::cli::io::{prompt_guess_limit, prompt_max_value, prompt_min_value, read_input};
use crate::core::{GuessResult, GuessingGame};

/// Run the CLI game with the provided arguments.
pub fn run_cli_game(cli: Cli) {
    println!("Welcome to the Number Guessing Game!");

    // Get min and max values using I/O helper functions
    let min = prompt_min_value(cli.min);
    let max = prompt_max_value(cli.max, min);
    let guess_limit = prompt_guess_limit(cli.limit);

    println!("I'm thinking of a number between {min} and {max} (inclusive)...");

    if let Some(limit) = guess_limit {
        println!("You have {limit} guesses to find the number!");
    }

    // Create the game
    let mut game =
        GuessingGame::new_with_limit(min, max, guess_limit).expect("Failed to create game");

    // Main game loop
    loop {
        // Show remaining guesses if there's a limit
        if let Some(max_guesses) = game.get_max_guesses() {
            let remaining = max_guesses.saturating_sub(game.get_guess_count());
            if remaining > 0 {
                println!("Guesses remaining: {remaining}");
            }
        }

        // Get user's guess
        let guess: i32 = read_input("Enter your guess: ");

        // Process the guess
        match game.make_guess(guess) {
            GuessResult::TooLow => println!("Too low!"),
            GuessResult::TooHigh => println!("Too high!"),
            GuessResult::Correct { number, attempts } => {
                println!("You got it! The number was {number}.");
                println!("It took you {attempts} guesses.");
                break;
            }
            GuessResult::LimitReached {
                number,
                max_guesses,
            } => {
                println!("Sorry, you've reached the limit of {max_guesses} guesses!");
                println!("The number was {number}.");
                println!("Better luck next time!");
                break;
            }
        }
    }

    println!("Thanks for playing!");
}
