use std::io::{self, Write};
use clap::Parser;
use number_guessing_game::{GuessingGame, GuessResult};

#[derive(Parser, Debug)]
#[command(name = "number_guessing_game")]
#[command(about = "A fun number guessing game", long_about = None)]
struct Cli {
    #[arg(short, long, help = "Minimum number (inclusive)")]
    min: Option<i32>,
    
    #[arg(short = 'x', long, help = "Maximum number (inclusive)")]
    max: Option<i32>,
}

fn read_input<T>(prompt: &str) -> T 
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    loop {
        print!("{}", prompt);
        io::stdout().flush().expect("Failed to flush stdout");
        
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        
        match input.trim().parse() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}

fn get_valid_max(min: i32) -> i32 {
    loop {
        let max: i32 = read_input("Enter maximum number (inclusive): ");
        if max >= min {
            return max;
        } else {
            println!("Maximum must be greater than or equal to minimum. Please try again.");
        }
    }
}

fn main() {
    println!("Welcome to the Number Guessing Game!");
    
    let cli = Cli::parse();
    
    // Get min value from CLI or prompt user
    let min: i32 = match cli.min {
        Some(m) => {
            println!("Using minimum value from command line: {}", m);
            m
        },
        None => read_input("Enter minimum number (inclusive): ")
    };
    
    // Get max value from CLI or prompt user
    let max: i32 = match cli.max {
        Some(m) => {
            if m >= min {
                println!("Using maximum value from command line: {}", m);
                m
            } else {
                println!("Maximum from command line ({}) is less than minimum ({}). Please provide a valid maximum.", m, min);
                get_valid_max(min)
            }
        },
        None => get_valid_max(min)
    };
    
    println!("I'm thinking of a number between {} and {} (inclusive)...", min, max);
    
    // Create the game
    let mut game = GuessingGame::new(min, max)
        .expect("Failed to create game");
    
    // Main game loop
    loop {
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
        }
    }
    
    println!("Thanks for playing!");
}