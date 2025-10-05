# Claude Code Context - Number Guessing Game

## Project Overview
A Rust-based number guessing game with both CLI and web interfaces. The game generates a random number within a user-specified range and provides feedback on guesses. Supports optional guess limits that end the game when exceeded.

## Development Environment
- Even when working on Windows we work in a bash shell
- Cargo commands may run longer than 2 minutes.  Run them without a timeout.
- Building the Docker container image takes over 6 minutes. Use timeout of at least 600000ms (10 minutes) for `docker build` commands.

## Quick Commands

### Choose Your Tool: `make` or `just`
Both command runners are available with identical functionality. Use whichever you prefer:
- **make**: Pre-installed on most systems, traditional choice
- **just**: Modern alternative with better syntax (install: `cargo install just`)

Run `make` or `just --list` to see all available commands.

### Common Workflows

```bash
# Development - Start full stack (postgres + app in Docker)
make dev          # or: just dev
# Access at http://localhost:3000

# Development - Start only database (run app locally for faster iteration)
make dev-db       # or: just dev-db
cargo run -- --server --port 3000

# Stop development services
make dev-down     # or: just dev-down

# Building (no database needed - uses runtime-checked SQLx)
make build        # or: just build
cargo build       # Direct cargo also works

# Testing
make test              # All tests (builds Docker if needed)
make test-unit         # Unit tests only (fast, no Docker)
make test-integration  # Integration tests with testcontainers

# Or with just:
just test
just test-unit
just test-integration
just test-verbose      # With output for debugging

# Running
make run-cli           # CLI game with defaults
cargo run -- --min 1 --max 100 --limit 10

make run-server        # Web server (needs postgres)
just run-server 8080   # With custom port

# Code Quality
make fmt               # Format code
make lint              # Run clippy
just check             # Format + lint

# Database
make db-shell          # PostgreSQL shell
make logs              # View docker-compose logs

# Cleanup
make clean             # Clean everything
```

### Key Points
- **Build**: No database needed (SQLx uses runtime checking, not compile-time)
- **CLI mode**: No database needed at all
- **Web mode**: Requires PostgreSQL running
- **Tests**: Unit tests are fast (no Docker), integration tests use testcontainers
- **Docker**: Only needed for integration tests and optional full-stack development

## Architecture

### Core Modules
- **src/game.rs**: Pure game logic, no I/O. Contains `GuessingGame` struct and `GuessResult` enum
- **src/cli.rs**: CLI argument parsing using clap (no validation or I/O)
- **src/validators.rs**: Shared validation logic used by both CLI and web layers (pure functions, no I/O)
- **src/io.rs**: User input/output helpers for CLI interactions
- **src/templates.rs**: Askama template structs for type-safe HTML rendering
- **src/game_id.rs**: Type-safe newtype wrapper for game IDs
- **src/db.rs**: PostgreSQL database layer with runtime-checked SQLx queries
- **src/web.rs**: Axum-based web server with REST API and HTMX frontend
- **src/main.rs**: Minimal entry point, mode selection (CLI vs Web), database initialization
- **static/index.html**: Web UI with HTMX for dynamic updates
- **templates/**: Askama HTML templates (compile-time checked)
- **migrations/**: SQLx database migrations

### Key Design Patterns
1. **Separation of Concerns**: Game logic isolated from I/O, validation separate from presentation, database layer separate from web layer
2. **Shared Validation**: Single source of truth for validation logic in `validators` module
3. **Type Safety**: Newtype pattern for `GameId`, compile-time template checking with Askama
4. **Result Types**: Extensive use of `Result<T, String>` for error handling with safe type conversions
5. **State Management**: Web server uses PostgreSQL with SQLx connection pooling (`PgPool`)
6. **Module Organization**: Clear boundaries between argument parsing, validation, I/O, and business logic
7. **Template-Based HTML**: Askama templates provide compile-time checked, type-safe HTML rendering

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
2. Add validation logic in `src/validators.rs` if needed
3. Add CLI support:
   - Argument parsing in `src/cli.rs`
   - User I/O in `src/io.rs`
4. Update web handlers in `src/web.rs` (uses shared validators)
5. Modify HTML in `static/index.html` for UI changes
6. Write tests in the relevant module
7. Update documentation (README.md, docs/api.md, docs/requirements.md)

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
│   ├── cli.rs       # CLI argument parsing (clap only)
│   ├── validators.rs # Shared validation logic (no I/O)
│   ├── io.rs        # User input/output helpers
│   ├── templates.rs # Askama template structs
│   ├── game_id.rs   # Type-safe game ID wrapper
│   ├── db.rs        # PostgreSQL database layer
│   ├── web.rs       # Web server
│   ├── lib.rs       # Library exports
│   └── main.rs      # Entry point
├── static/
│   └── index.html   # Web UI
├── templates/       # Askama HTML templates
│   ├── error.html
│   ├── game_started.html
│   ├── guess_form.html
│   ├── game_complete.html
│   ├── game_not_found.html
│   └── update_error.html
├── migrations/      # SQLx database migrations
│   ├── 20250930000001_create_games_table.sql
│   └── 20250930000002_add_cleanup_function.sql
├── tests/           # Integration tests with test containers
│   ├── common/      # Shared test infrastructure
│   │   ├── mod.rs   # Module declarations
│   │   ├── containers.rs # Docker container configurations
│   │   ├── fixtures.rs   # Test data and scenarios
│   │   └── assertions.rs # Custom test assertions
│   ├── fixtures/    # Test data files
│   │   └── test_data.json
│   ├── api_edge_cases_test.rs  # API edge cases
│   ├── cli_test.rs             # CLI interface tests
│   ├── integration_test.rs     # Basic integration tests
│   ├── web_endpoints_test.rs   # Web endpoint tests
│   └── web_ui_test.rs          # Web UI tests
├── examples/        # Usage examples
├── docs/            # All documentation
│   ├── api.md       # API documentation
│   ├── architecture.md # System design
│   ├── contributing.md # Dev guidelines
│   ├── requirements.md # Technical specs
│   └── security-todo.md # Security TODOs
├── plans/           # Implementation plans
│   └── code-improvement-suggestions.md
├── target/          # Build artifacts
├── Dockerfile       # Container configuration
├── docker-compose.yml # Docker Compose for development
├── .dockerignore    # Docker build exclusions
├── .env             # Environment variables (DATABASE_URL)
├── .env.example     # Example environment configuration
├── Makefile         # Make command runner
├── justfile         # Just command runner (modern alternative)
├── build.rs         # Build script (Docker image for tests)
├── run_integration_tests.sh  # Test runner (Linux/macOS)
├── run_integration_tests.bat # Test runner (Windows)
└── README.md        # Main documentation
```

## Database Setup

### PostgreSQL Integration
The web server uses PostgreSQL for persistent storage.

**Quick Start:**
```bash
# Option 1: Full stack in Docker (easiest for manual testing)
make dev              # or: just dev
# Access at http://localhost:3000

# Option 2: Database only (run app locally for faster iteration)
make dev-db           # or: just dev-db
cargo run -- --server

# Option 3: Use existing PostgreSQL instance
# Copy .env.example to .env and set your DATABASE_URL
cp .env.example .env
# Edit .env with your database credentials
```

**Database Schema:**
- **games table**: Stores active game state (game_id, min/max values, secret_number, guess_count, max_guesses)
- **Migrations**: Located in `migrations/` directory, run automatically on server startup
- **Connection pooling**: Max 5 connections via SQLx PgPool

**Key Points:**
- **Runtime-checked queries**: No database needed at compile time (cargo build works without DB)
- **Automatic migrations**: Migrations run on server startup
- **Persistent storage**: Games persist across server restarts
- **Auto-cleanup**: Completed games are automatically deleted from database
- **Environment variables**: DATABASE_URL required for web mode (see `.env.example`)

**Default Credentials (docker-compose):**
- User: `numberguess`
- Password: `password`
- Database: `numberguess_dev`
- Connection string: `postgresql://numberguess:password@localhost:5432/numberguess_dev`

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
- **thiserror**: Error derive macros (v2.0)
- **askama**: Type-safe compile-time HTML templates (v0.12)
- **askama_axum**: Askama integration with Axum (v0.4)

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