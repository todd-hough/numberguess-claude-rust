# Number Guessing Game - Requirements

## Application Overview
A number guessing game that can be run either as a command-line application or as a REST web service. In CLI mode, players interact directly through the terminal. In server mode, the game provides HTTP API endpoints for creating games and making guesses, allowing multiple concurrent game sessions.

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
  - Limit reached message when max guesses exceeded
- **Guess counter**: Track and display number of attempts at game completion
- **Guess limit**: Optional maximum number of allowed guesses
  - Configurable via CLI flag (`--limit`)
  - Interactive prompt when not provided
  - Different limits for CLI (1000) and Web (100)
  - Game ends when limit is reached, revealing the answer

### 3. User Interface
- **Welcome message**: Display game title on startup
- **Clear prompts**: Provide clear input prompts for all user interactions
- **Error handling**: Display helpful error messages for invalid inputs
- **Game completion**: Show final statistics and thank you message

### 4. Web Service Mode
- **REST API**: HTTP endpoints for game operations
- **Session Management**: Support multiple concurrent games with unique IDs
- **JSON Communication**: Request and response bodies in JSON format
- **Game Lifecycle**: Games persist in memory until completed
- **Error Handling**: Appropriate HTTP status codes and error messages

## Technical Requirements

### 1. Architecture
- **Modular Design**: Separation of concerns into distinct library modules
- **Game Module** (`src/game.rs`): 
  - Core game logic and state management
  - `GuessingGame` struct for game instance with optional guess limit
  - `GuessResult` enum for game outcomes (TooLow, TooHigh, Correct, LimitReached)
  - Methods for checking remaining guesses and game state
  - No I/O operations or external dependencies (except rand)
- **CLI Module** (`src/cli.rs`):
  - Command-line argument parsing with clap
  - User input handling functions
  - Input validation helpers
  - Server mode configuration (--server, --port)
  - All I/O operations
- **Web Module** (`src/web.rs`):
  - REST API endpoints implementation
  - Game session management with HashMap storage
  - Async request handlers using Axum
  - JSON serialization/deserialization
  - Random numeric game ID generation
- **Library Entry** (`src/lib.rs`):
  - Exposes both game and cli modules
  - Re-exports commonly used types for convenience
- **Main Application** (`src/main.rs`):
  - Minimal orchestration layer
  - Mode selection (CLI vs Web Server)
  - Async runtime initialization for web mode
  - CLI game loop for interactive mode

### 2. Dependencies
- **rand**: For random number generation (v0.8.5)
- **clap**: For command-line argument parsing (v4.5 with derive feature)
- **axum**: Web framework for REST API (v0.7)
- **tokio**: Async runtime for web server (v1 with full features)
- **serde**: Serialization framework (v1.0)
- **serde_json**: JSON serialization (v1.0)
- **tower**: Middleware and service utilities (v0.4)
- **tower-http**: HTTP-specific middleware (v0.5 with CORS)
- **Standard library**: For I/O operations and comparison

[Dev Dependencies]
- **reqwest**: HTTP client for testing (v0.11 with json)

### 3. Command-Line Interface
- **Help command**: Support `--help` flag to display usage information
- **Game parameters**:
  - `-m, --min`: Minimum number (inclusive)
  - `-x, --max`: Maximum number (inclusive)
  - `-l, --limit`: Maximum number of guesses allowed (optional)
- **Server mode**:
  - `-s, --server`: Run as web server
  - `-p, --port`: Server port (default: 3000)

### 4. Input Handling
- **Generic input function**: Reusable function for reading and parsing user input
- **Type safety**: Strong typing for numeric inputs
- **Error recovery**: Continue prompting on invalid input rather than crashing
- **Separation**: Input handling isolated in CLI module

### 5. Testing
- **Unit tests**: Comprehensive tests for game logic in `game.rs`
- **Test coverage**: Game creation, range validation, guess processing, state tracking
- **Testability**: Core logic separated from I/O for easy testing
- **Examples**: 
  - `examples/demo.rs`: Library usage demonstration
  - `examples/web_client.rs`: HTTP client for testing web API

## Usage Examples

### Command-Line Usage
```bash
# Fully automated with CLI parameters
cargo run -- --min 1 --max 100

# With guess limit
cargo run -- --min 1 --max 100 --limit 10

# Partial automation (will prompt for max and limit)
cargo run -- --min 1

# Fully interactive (will prompt for all)
cargo run

# Display help
cargo run -- --help
```

### Web Server Usage
```bash
# Start server on default port 3000
cargo run -- --server

# Start server on custom port
cargo run -- --server --port 8080
```

### REST API Usage
```bash
# Create a new game without limit
curl -X POST http://localhost:3000/api/games \
  -H "Content-Type: application/json" \
  -d '{"min": 1, "max": 100}'

# Create a new game with 10 guess limit
curl -X POST http://localhost:3000/api/games \
  -H "Content-Type: application/json" \
  -d '{"min": 1, "max": 100, "max_guesses": 10}'

# Make a guess (use game_id from previous response)
curl -X POST http://localhost:3000/api/games/{game_id}/guess \
  -H "Content-Type: application/json" \
  -d '{"guess": 50}'
```

### Library Usage
```rust
use number_guessing_game::game::{GuessingGame, GuessResult};

// Create a new game with optional guess limit
let mut game = GuessingGame::new_with_limit(1, 100, Some(10)).unwrap();

// Make a guess
match game.make_guess(50) {
    GuessResult::TooLow => println!("Too low!"),
    GuessResult::TooHigh => println!("Too high!"),
    GuessResult::Correct { number, attempts } => {
        println!("Correct! Found {} in {} attempts", number, attempts);
    },
    GuessResult::LimitReached { number, max_guesses } => {
        println!("Limit reached! The number was {}. Max guesses: {}", number, max_guesses);
    }
}
```

## API Endpoints

### POST /api/games
Creates a new game session.
- **Request**: `{"min": 1, "max": 100, "max_guesses": 10}` (max_guesses is optional)
- **Response**: `{"game_id": 12345678901234567, "min": 1, "max": 100, "max_guesses": 10, "message": "..."}`

### POST /api/games/{game_id}/guess
Makes a guess for an existing game.
- **Request**: `{"guess": 50}`
- **Response**: `{"result": "too_low|too_high|correct|limit_reached", "message": "...", "attempts": number}`

## Future Enhancements (Potential)
- ~~Difficulty levels with attempt limits~~ ✓ Implemented as guess limit feature
- Score tracking across sessions
- Hints system
- Multiple rounds/play again option
- Statistics (average guesses, best score, etc.)
- Persistent storage for game sessions
- WebSocket support for real-time gameplay
- Authentication and user profiles
- Leaderboards