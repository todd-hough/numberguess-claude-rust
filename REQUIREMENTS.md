# Number Guessing Game - Requirements

## Application Overview
A command-line number guessing game where the computer generates a random number within a user-specified range, and the player attempts to guess it with feedback provided after each guess.

## Functional Requirements

### 1. Range Configuration
- **Command-line parameters**: Accept optional `--min` and `--max` parameters via CLI
- **Interactive fallback**: If parameters not provided via CLI, prompt user at runtime
- **Validation**: Maximum value must be greater than or equal to minimum value
- **Range display**: Show the selected range to the user before game starts

### 2. Game Mechanics
- **Random number generation**: Generate a random integer within the specified range (inclusive)
- **User input**: Accept integer guesses from the user
- **Input validation**: Handle invalid inputs gracefully with error messages
- **Feedback system**: 
  - "Too low!" when guess is below target
  - "Too high!" when guess is above target
  - Success message when correct
- **Guess counter**: Track and display number of attempts at game completion

### 3. User Interface
- **Welcome message**: Display game title on startup
- **Clear prompts**: Provide clear input prompts for all user interactions
- **Error handling**: Display helpful error messages for invalid inputs
- **Game completion**: Show final statistics and thank you message

## Technical Requirements

### 1. Architecture
- **Modular Design**: Separation of concerns into distinct library modules
- **Game Module** (`src/game.rs`): 
  - Core game logic and state management
  - `GuessingGame` struct for game instance
  - `GuessResult` enum for game outcomes
  - No I/O operations or external dependencies (except rand)
- **CLI Module** (`src/cli.rs`):
  - Command-line argument parsing with clap
  - User input handling functions
  - Input validation helpers
  - All I/O operations
- **Library Entry** (`src/lib.rs`):
  - Exposes both game and cli modules
  - Re-exports commonly used types for convenience
- **Main Application** (`src/main.rs`):
  - Minimal orchestration layer
  - Combines CLI and game modules
  - Contains only the main game loop

### 2. Dependencies
- **rand**: For random number generation (v0.8.5)
- **clap**: For command-line argument parsing (v4.5 with derive feature)
- **Standard library**: For I/O operations and comparison

### 3. Command-Line Interface
- **Help command**: Support `--help` flag to display usage information
- **Short flags**: `-m` for min, `-x` for max
- **Long flags**: `--min` and `--max`

### 4. Input Handling
- **Generic input function**: Reusable function for reading and parsing user input
- **Type safety**: Strong typing for numeric inputs
- **Error recovery**: Continue prompting on invalid input rather than crashing
- **Separation**: Input handling isolated in CLI module

### 5. Testing
- **Unit tests**: Comprehensive tests for game logic in `game.rs`
- **Test coverage**: Game creation, range validation, guess processing, state tracking
- **Testability**: Core logic separated from I/O for easy testing
- **Examples**: Demo application in `examples/` directory showcasing library usage

## Usage Examples

### Command-Line Usage
```bash
# Fully automated with CLI parameters
cargo run -- --min 1 --max 100

# Partial automation (will prompt for max)
cargo run -- --min 1

# Fully interactive (will prompt for both)
cargo run

# Display help
cargo run -- --help
```

### Library Usage
```rust
use number_guessing_game::game::{GuessingGame, GuessResult};

// Create a new game
let mut game = GuessingGame::new(1, 100).unwrap();

// Make a guess
match game.make_guess(50) {
    GuessResult::TooLow => println!("Too low!"),
    GuessResult::TooHigh => println!("Too high!"),
    GuessResult::Correct { number, attempts } => {
        println!("Correct! Found {} in {} attempts", number, attempts);
    }
}
```

## Future Enhancements (Potential)
- Difficulty levels with attempt limits
- Score tracking across sessions
- Hints system
- Multiple rounds/play again option
- Statistics (average guesses, best score, etc.)