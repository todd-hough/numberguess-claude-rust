# Number Guessing Game

A fun and interactive number guessing game that can be played via command-line or as a web service with REST API.

## Features

- **CLI Mode**: Interactive command-line gameplay (no database required)
- **Web Server Mode**: REST API and web UI for browser-based play
- **PostgreSQL Persistence**: Game state persists across server restarts
- **Configurable Range**: Set custom min/max values for the guessing range
- **Guess Limit**: Optional limit on number of guesses per game
- **Input Validation**: Comprehensive validation with helpful error messages
- **Web Interface**: Modern, responsive UI with real-time feedback
- **Docker Support**: Easy setup with Docker Compose for local development

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd number_guessing_game

# Build the project
make build

# Or build for release
cargo build --release
```

### Prerequisites
- **Rust**: 1.89.0 or later
- **PostgreSQL**: Required for web server mode (can use Docker Compose)
- **Docker**: Optional, for running full stack or tests
- **Make**: For convenient command running

## Usage

### Command-Line Mode

CLI mode does not require a database.

```bash
# Quick start with make
make run-cli

# Or use cargo directly with custom options
cargo run -- --min 1 --max 100

# With guess limit
cargo run -- --min 1 --max 50 --limit 10

# With all options
cargo run -- --min 1 --max 100 --limit 5
```

### Web Server Mode

Web server requires PostgreSQL. Use one of these approaches:

**Option 1: Full Stack (easiest for quick start)**
```bash
# Starts both PostgreSQL and app server in Docker
make dev

# Access the web interface at http://localhost:3000
# Stop with: make dev-down
```

**Option 2: Local Development (faster iteration)**
```bash
# Start just the database
make dev-db

# Run the app locally (in another terminal)
make run-server

# Or with custom port
cargo run -- --server --port 8080
```

**Option 3: Use Your Own PostgreSQL**
```bash
# Copy and configure environment file
cp .env.example .env
# Edit .env with your database credentials

# Run the server
cargo run -- --server --port 3000
```

Then visit `http://localhost:3000` in your browser for the web interface.

## Command-Line Options

- `-m, --min <MIN>`: Minimum number (inclusive, 0 to 1,000,000)
- `-x, --max <MAX>`: Maximum number (inclusive, 0 to 1,000,000)
- `-l, --limit <LIMIT>`: Maximum number of guesses allowed (optional)
- `-s, --server`: Run as a web server
- `-p, --port <PORT>`: Port for the web server (default: 3000)
- `-h, --help`: Display help information

## Guess Limit Feature

The game now supports an optional guess limit that restricts the number of attempts:

### CLI Mode
- Use `--limit` flag to set a maximum number of guesses
- Interactive prompt asks if you want to set a limit when not provided
- Maximum limit is 1000 for CLI mode
- Use 0 or leave blank for unlimited guesses

### Web UI
- Optional "Guess Limit" field on the new game form
- Maximum of 100 guesses for web games
- Leave blank or enter 0 for unlimited guesses
- **Remaining guesses counter**: Displays "Guesses remaining: X" throughout gameplay
  - Shows initial count when game starts
  - Updates after each guess to show remaining attempts
  - Only displayed when a guess limit is set

### API
- Include `"max_guesses"` in the game creation request
- Set to `null` or `0` for unlimited guesses
- Maximum of 100 guesses for API games

## REST API

### Create a New Game

```bash
POST /api/games
Content-Type: application/json

{
  "min": 1,
  "max": 100,
  "max_guesses": 10  // Optional, null for unlimited
}
```

Response:
```json
{
  "game_id": 12345678901234567,
  "min": 1,
  "max": 100,
  "max_guesses": 10,
  "message": "Game created! I'm thinking of a number between 1 and 100 (inclusive). You have 10 guesses."
}
```

### Make a Guess

```bash
POST /api/games/{game_id}/guess
Content-Type: application/json

{
  "guess": 50
}
```

Possible responses:

**Too Low/High:**
```json
{
  "result": "too_low",
  "message": "Too low! Your guess of 50 is below the target.",
  "attempts": 3
}
```

**Correct:**
```json
{
  "result": "correct",
  "message": "You got it! The number was 42. It took you 5 guesses.",
  "attempts": 5
}
```

**Limit Reached:**
```json
{
  "result": "limit_reached",
  "message": "Sorry, you've reached the limit of 10 guesses! The number was 42.",
  "attempts": 10
}
```

## Examples

### Using the Library

```rust
use number_guessing_game::{GuessingGame, GuessResult};

// Create game with guess limit
let mut game = GuessingGame::new_with_limit(1, 100, Some(5)).unwrap();

// Make guesses
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

### Running Example Programs

```bash
# Demo of library usage
cargo run --example demo

# Web API client example
cargo run --example web_client
```

## Project Structure

```
number_guessing_game/
├── src/
│   ├── main.rs      # Application entry point
│   ├── lib.rs       # Library exports
│   ├── game.rs      # Core game logic
│   ├── cli.rs       # CLI argument parsing (clap)
│   ├── io.rs        # User input/output helpers
│   ├── validators.rs # Shared validation logic
│   ├── templates.rs # Askama template structs
│   ├── game_id.rs   # Type-safe game ID wrapper
│   ├── db.rs        # PostgreSQL database layer
│   └── web.rs       # Web server and API endpoints
├── static/
│   └── index.html   # Web UI
├── templates/       # Askama HTML templates
│   ├── error.html
│   ├── game_started.html
│   ├── guess_form.html
│   ├── game_complete.html
│   ├── game_not_found.html
│   └── update_error.html
├── migrations/      # Database migrations
│   ├── 20250930000001_create_games_table.sql
│   └── 20250930000002_add_cleanup_function.sql
├── tests/           # Integration and UI tests (Compose-backed)
│   ├── common/      # Shared test utilities
│   └── *_test.rs    # Integration test suites
├── docs/            # Detailed documentation
├── Makefile         # Make command runner with shortcuts (dc-up, dc-down, dc-attach)
├── docker-compose.yml # Docker orchestration
├── .env.example     # Environment configuration template
└── build.rs         # Build script for Docker image
```

## Development

### Quick Reference

```bash
# View all available commands
make help

# Common commands
make dev           # Start full stack for manual testing
make dev-db        # Start only database (run app locally)
make dev-down      # Stop services
make dc-up         # Boot the devcontainer (requires devcontainer CLI)
make dc-down       # Stop devcontainer
make dc-attach     # Attach terminal to devcontainer
make compose-up    # Bring up app + postgres via docker compose
make compose-down  # Tear down compose stack
make build         # Build application (no DB needed!)
make test          # Run all tests
make test-unit     # Fast unit tests (no Docker)
make test-compose  # Run API integration tests via docker compose
make test-compose-ui # Run Selenium UI tests via docker compose
make test-compose-down # Stop integration test services (auto-cleanup, rarely needed)
make fmt           # Format code
make lint          # Run linter
make clean         # Clean everything
```

### Running Tests

```bash
# Run all tests (builds Docker image if needed)
make test

# Run unit tests only (fast, no Docker required)
make test-unit

# Run integration tests against docker compose stack
make test-compose

# Run Selenium-powered UI tests
make test-compose-ui

# Run with output (using cargo directly)
cargo test -- --nocapture

# Manually stop integration test services (optional - they auto-cleanup on exit)
make test-compose-down
```

**Note**: Compose-backed integration tests (`make test-compose` and `make test-compose-ui`) automatically clean up services on exit. They also reset the database via `scripts/reset-db.sh` for deterministic results. Use `make test-compose-down` only if you need to manually stop services (e.g., after interrupting tests with Ctrl+C).

### Building for Release

```bash
cargo build --release
```

The optimized binary will be in `target/release/number_guessing_game`

### Development Workflow

**For web development with live reload:**
```bash
# Terminal 1: Start database
make dev-db

# Terminal 2: Run app locally (restart on changes)
cargo run -- --server

# Or use cargo-watch for auto-reload
cargo install cargo-watch
cargo watch -x 'run -- --server'
```

**For full-stack testing:**
```bash
# Start everything in Docker
make dev

# View logs
make logs

# Access database
make db-shell
```

### Code Quality

```bash
# Format code
make fmt

# Run linter
make lint
```

## Documentation

### For Developers
- [**architecture.md**](docs/architecture.md) - System design and technical architecture
- [**contributing.md**](docs/contributing.md) - Contribution guidelines and development workflow
- [**api.md**](docs/api.md) - REST API specification and examples
- [**requirements.md**](docs/requirements.md) - Detailed technical requirements and specifications

### For AI Assistants
- [**.claude/claude.md**](.claude/claude.md) - Optimized context for Claude Code and AI assistants

### Additional Resources
- [**security-todo.md**](docs/security-todo.md) - Security improvements and considerations
- [**docs/**](docs/) - Detailed guides and tutorials

## Quick Links

| Document | Purpose |
|----------|---------|
| [Architecture](docs/architecture.md) | System design, components, data flow |
| [API Reference](docs/api.md) | REST endpoints, request/response formats |
| [Contributing](docs/contributing.md) | How to contribute, code style, PR process |
| [Requirements](docs/requirements.md) | Feature specifications, technical details |
| [Claude Context](.claude/claude.md) | AI-optimized project context |

## License

[Your License Here]

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](docs/contributing.md) for details on:
- Setting up your development environment
- Code style and standards
- Testing requirements
- Pull request process

For major changes, please open an issue first to discuss what you would like to change.
