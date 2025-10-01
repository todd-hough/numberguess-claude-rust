# Claude Code Context - Number Guessing Game

## Project Overview
A Rust-based number guessing game with both CLI and web interfaces. The game generates a random number within a user-specified range and provides feedback on guesses. Supports optional guess limits that end the game when exceeded.

## Development Environment
- Even when working on Windows we work in a bash shell
- Cargo commands may run longer than 2 minutes.  Run them without a timeout.

## Quick Commands
```bash
# Build and test (automatically builds Docker image if needed)
cargo build
cargo test

# Or use Makefile for convenience
make test              # Run all tests (checks Docker image first)
make test-unit         # Unit tests only (no Docker)
make test-web-ui       # Web UI tests only
make docker-rebuild    # Force rebuild Docker image

# Run test with output for troubleshooting
cargo test -- --nocapture

# Run CLI game
cargo run -- --min 1 --max 100 --limit 10

# Run web server
cargo run -- --server --port 3000

# Format code
cargo fmt
cargo clippy
```

## Architecture

### Core Modules
- **src/game.rs**: Pure game logic, no I/O. Contains `GuessingGame` struct and `GuessResult` enum
- **src/cli.rs**: CLI argument parsing (clap) and user input handling
- **src/db.rs**: PostgreSQL database layer with runtime-checked SQLx queries
- **src/web.rs**: Axum-based web server with REST API and HTMX frontend
- **src/main.rs**: Minimal entry point, mode selection (CLI vs Web), database initialization
- **static/index.html**: Web UI with HTMX for dynamic updates
- **migrations/**: SQLx database migrations

### Key Design Patterns
1. **Separation of Concerns**: Game logic isolated from I/O, database layer separate from web layer
2. **Result Types**: Extensive use of `Result<T, String>` for error handling
3. **State Management**: Web server uses PostgreSQL with SQLx connection pooling (`PgPool`)
4. **Validation**: Input validation at multiple layers (CLI, web, game logic, database)

## Important Constraints

### Numeric Limits
- Range: 0 to 1,000,000 (inclusive)
- Guess limits: Max 1000 (CLI), Max 100 (Web/API)
- Negative numbers not allowed

### Web API
- Games stored in memory (lost on restart)
- Game IDs are random u64 values
- Games auto-removed when completed
- JSON request/response format

### Security Considerations
- Input validation prevents integer overflow
- Range limits prevent DoS via large ranges
- No persistent storage (stateless between restarts)
- HTMX from CDN (consider bundling for production)

## Testing Strategy
```bash
# Unit tests in src/game.rs
cargo test --lib

# Integration tests with test containers (requires Docker)
cargo test --test smoke_test   # Basic connectivity test
cargo test --test game_lifecycle_test   # API functionality
cargo test --test concurrent_games_test # Concurrency testing
cargo test --test error_handling_test   # Error scenarios
cargo test --test cli_integration_test  # CLI testing
cargo test --test stress_test           # Performance/stress testing

# Integration test via examples (legacy)
cargo run --example demo
cargo run --example web_client  # requires server running

# Manual web UI test
cargo run -- --server
# Visit http://localhost:3000
```

## Common Tasks

### Adding a New Feature
1. Update game logic in `src/game.rs`
2. Add CLI support in `src/cli.rs` if needed
3. Update web handlers in `src/web.rs`
4. Modify HTML in `static/index.html` for UI changes
5. Write tests in the relevant module
6. Update documentation (README.md, docs/api.md, docs/requirements.md)

### Modifying Game Rules
- Core logic in `game.rs::GuessingGame::make_guess()`
- Add new `GuessResult` variants as needed
- Update all match statements handling `GuessResult`

### Debugging Web Issues
- Check browser console for HTMX errors
- Use `curl` to test API endpoints directly
- Server logs to stdout by default
- Consider adding `tracing` for production

## Code Style
- Use `cargo fmt` before commits
- Follow Rust naming conventions
- Keep functions small and focused
- Document public APIs with doc comments
- Use descriptive variable names

## Known Issues & TODOs
- See docs/security-todo.md for security improvements
- No rate limiting on API endpoints
- No request size limits
- Games remain in memory until completed (potential memory leak)
- No persistent storage option

## File Structure
```
├── .claude/
│   └── claude.md    # This file
├── src/
│   ├── game.rs      # Core game logic
│   ├── cli.rs       # CLI interface
│   ├── web.rs       # Web server
│   ├── lib.rs       # Library exports
│   └── main.rs      # Entry point
├── static/
│   └── index.html   # Web UI
├── tests/           # Integration tests with test containers
│   ├── common/      # Shared test infrastructure
│   │   ├── mod.rs   # Module declarations
│   │   ├── containers.rs # Docker container configurations
│   │   ├── fixtures.rs   # Test data and scenarios
│   │   └── assertions.rs # Custom test assertions
│   ├── fixtures/    # Test data files
│   │   └── test_data.json
│   ├── smoke_test.rs           # Basic connectivity tests
│   ├── game_lifecycle_test.rs  # API functionality tests
│   ├── concurrent_games_test.rs # Concurrency tests
│   ├── error_handling_test.rs  # Error scenario tests
│   ├── cli_integration_test.rs # CLI interface tests
│   └── stress_test.rs          # Performance/stress tests
├── examples/        # Usage examples
├── docs/            # All documentation
│   ├── api.md       # API documentation
│   ├── architecture.md # System design
│   ├── contributing.md # Dev guidelines
│   ├── requirements.md # Technical specs
│   └── security-todo.md # Security TODOs
├── target/          # Build artifacts
├── Dockerfile       # Container configuration
├── .dockerignore    # Docker build exclusions
├── run_integration_tests.sh  # Test runner (Linux/macOS)
├── run_integration_tests.bat # Test runner (Windows)
└── README.md        # Main documentation
```

## Database Setup

### PostgreSQL Integration
The web server now uses PostgreSQL for persistent storage instead of in-memory HashMap.

**Setup for Development:**
```bash
# Option 1: Use docker-compose (if available)
docker compose up -d postgres

# Option 2: Use existing PostgreSQL instance
# Just set DATABASE_URL in .env file

# Create .env file (already done, but customize if needed)
echo 'DATABASE_URL=postgresql://postgres:postgres@localhost:5432/postgres' > .env

# Build and run (migrations run automatically)
cargo build
cargo run -- --server
```

**Database Schema:**
- **games table**: Stores active game state (game_id, min/max values, secret_number, guess_count, max_guesses)
- **Migrations**: Located in `migrations/` directory, run automatically on startup
- **Connection pooling**: Max 5 connections via SQLx PgPool

**Key Points:**
- Runtime-checked queries (no compile-time database required)
- Automatic migration execution on server startup
- Games persist across server restarts
- Completed games are automatically deleted from database
- Environment variable `DATABASE_URL` required for web mode

## Dependencies to Know

### Runtime Dependencies
- **clap**: CLI parsing with derive macros (v4.5.45)
- **axum**: Modern web framework (v0.8.4)
- **tokio**: Async runtime (v1.47.1)
- **serde**: JSON serialization (v1.0.219)
- **tower-http**: Static file serving (v0.6.6)
- **rand**: Random number generation (v0.9.2)
- **sqlx**: PostgreSQL driver with runtime-checked queries (v0.8)
- **dotenvy**: .env file support (v0.15)

### Test Dependencies  
- **testcontainers**: Docker container management for tests (v0.23)
- **reqwest**: HTTP client for API testing (v0.12.23)
- **assert_cmd**: CLI testing framework (v2.0)
- **predicates**: Test assertion predicates (v3.0)
- **serial_test**: Sequential test execution (v3.0)
- **tokio-test**: Async testing utilities (v0.4)

## Version Information
- **Rust Version**: 1.89.0 (29483883e 2025-08-04)
- **Rust Edition**: 2024
- **Last Updated**: Dependencies updated to latest versions (Aug 2025)

## Performance Considerations
- Each game stores minimal state (5 fields)
- O(1) game lookup via HashMap
- No database queries
- Static files served directly

## Deployment Notes
- Single binary output
- No external dependencies at runtime
- Configurable port via CLI
- Binds to 0.0.0.0 (all interfaces)

## Quick Fixes

### "Game not found" errors
- Games are removed after completion
- Check if game_id is valid
- Verify game hasn't already ended

### Input validation failures
- Check range: 0 to 1,000,000
- Ensure max >= min
- Verify positive integers only

### Build issues
```bash
cargo clean
cargo update
cargo build --release
```

## Related Documentation
- **../README.md**: User-facing documentation
- **../docs/api.md**: REST API specification
- **../docs/requirements.md**: Detailed technical requirements
- **../docs/security-todo.md**: Security improvements needed
- **../docs/architecture.md**: Detailed system design
- **../docs/contributing.md**: Development guidelines
- **../docs/**: All documentation and guides
```