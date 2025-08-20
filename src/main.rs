#![allow(warnings)]

use clap::Parser;
use number_guessing_game::{
    Cli, GuessResult, GuessingGame, get_guess_limit, get_max_value, get_min_value, read_input,
    web::run_server,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.server {
        // Run as web server
        run_server(cli.port).await;
    } else {
        // Run as CLI game
        run_cli_game(cli);
    }
}

fn run_cli_game(cli: Cli) {
    println!("Welcome to the Number Guessing Game!");

    // Get min and max values using CLI helper functions
    let min = get_min_value(cli.min);
    let max = get_max_value(cli.max, min);
    let guess_limit = get_guess_limit(cli.limit);

    println!(
        "I'm thinking of a number between {} and {} (inclusive)...",
        min, max
    );

    if let Some(limit) = guess_limit {
        println!("You have {} guesses to find the number!", limit);
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
                println!("Guesses remaining: {}", remaining);
            }
        }

        // Get user's guess
        let guess: i32 = read_input("Enter your guess: ");

        // Process the guess
        match game.make_guess(guess) {
            GuessResult::TooLow => println!("Too low!"),
            GuessResult::TooHigh => println!("Too high!"),
            GuessResult::Correct { number, attempts } => {
                println!("You got it! The number was {}.", number);
                println!("It took you {} guesses.", attempts);
                break;
            }
            GuessResult::LimitReached {
                number,
                max_guesses,
            } => {
                println!(
                    "Sorry, you've reached the limit of {} guesses!",
                    max_guesses
                );
                println!("The number was {}.", number);
                println!("Better luck next time!");
                break;
            }
        }
    }

    println!("Thanks for playing!");
}
