use number_guessing_game::game::{GuessingGame, GuessResult};

fn main() {
    println!("Demo: Using the number guessing game library");
    
    // Create a game with range 1-10
    let mut game = GuessingGame::new(1, 10).unwrap();
    let (min, max) = game.get_range();
    println!("Game created with range {}-{}", min, max);
    
    // Simulate some guesses
    let guesses = [3, 7, 5, 2, 8, 4, 6];
    
    for guess in guesses {
        println!("\nGuessing: {}", guess);
        match game.make_guess(guess) {
            GuessResult::TooLow => println!("  -> Too low!"),
            GuessResult::TooHigh => println!("  -> Too high!"),
            GuessResult::Correct { number, attempts } => {
                println!("  -> Correct! The number was {} found in {} attempts", number, attempts);
                break;
            },
            GuessResult::LimitReached { number, max_guesses } => {
                println!("  -> Limit reached! The number was {} (max guesses: {})", number, max_guesses);
                break;
            }
        }
    }
    
    println!("\nTotal guesses made: {}", game.get_guess_count());
}