#![allow(warnings)]

use clap::Parser;
use number_guessing_game::{
    Cli, GuessResult, GuessingGame,
    io::{prompt_guess_limit, prompt_max_value, prompt_min_value, read_input},
    web::run_server,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    if cli.server {
        // Run as web server
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in environment or .env file");

        // Read max connections from environment with validation
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u32>()
            .unwrap_or(5)
            .clamp(1, 100);

        println!("Connecting to database (max connections: {})...", max_connections);
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database");

        println!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        println!("Database initialized successfully");
        run_server(pool, cli.port).await;
    } else {
        // Run as CLI game
        run_cli_game(cli);
    }
}

fn run_cli_game(cli: Cli) {
    println!("Welcome to the Number Guessing Game!");

    // Get min and max values using I/O helper functions
    let min = prompt_min_value(cli.min);
    let max = prompt_max_value(cli.max, min);
    let guess_limit = prompt_guess_limit(cli.limit);

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
