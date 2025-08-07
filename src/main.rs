use clap::Parser;
use number_guessing_game::{
    GuessingGame, GuessResult,
    Cli, read_input, get_min_value, get_max_value
};

fn main() {
    println!("Welcome to the Number Guessing Game!");
    
    let cli = Cli::parse();
    
    // Get min and max values using CLI helper functions
    let min = get_min_value(cli.min);
    let max = get_max_value(cli.max, min);
    
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