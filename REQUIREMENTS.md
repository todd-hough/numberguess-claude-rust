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

### 1. Dependencies
- **rand**: For random number generation (v0.8.5)
- **clap**: For command-line argument parsing (v4.5 with derive feature)
- **Standard library**: For I/O operations and comparison

### 2. Command-Line Interface
- **Help command**: Support `--help` flag to display usage information
- **Short flags**: `-m` for min, `-x` for max
- **Long flags**: `--min` and `--max`

### 3. Input Handling
- **Generic input function**: Reusable function for reading and parsing user input
- **Type safety**: Strong typing for numeric inputs
- **Error recovery**: Continue prompting on invalid input rather than crashing

## Usage Examples

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

## Future Enhancements (Potential)
- Difficulty levels with attempt limits
- Score tracking across sessions
- Hints system
- Multiple rounds/play again option
- Statistics (average guesses, best score, etc.)