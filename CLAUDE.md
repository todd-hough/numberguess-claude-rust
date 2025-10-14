# Claude Code Context - Number Guessing Game

## Project Overview
A Rust-based number guessing game with both CLI and web interfaces. The game generates a random number within a user-specified range and provides feedback on guesses. Supports optional guess limits that end the game when exceeded.

## Development Environment
- Even when working on Windows we work in a bash shell
- Cargo commands may run longer than 2 minutes.  Run them without a timeout.
- Building the Docker container image in release mode takes over 6 minutes. Use timeout of at least 600000ms (10 minutes) for release builds. Debug builds are significantly faster (~2-3 minutes).

## Quick Commands

Run `make help` to see all available commands.

### Common Workflows

```bash
# Development - Start full stack (postgres + app in Docker)
make dev
# Access at http://localhost:8080

# Development - Start only database (run app locally for faster iteration)
make dev-db
cargo run -- --server --port 8080

# Stop development services
make dev-down

# Building (no database needed - uses runtime-checked SQLx)
make build
cargo build       # Direct cargo also works

# Docker Builds (release vs debug)
make docker-build        # Release build (optimized, slower to build)
make docker-build-debug  # Debug build (fast iteration, for dev/test)
make docker-rebuild      # Force rebuild with no cache (release mode)

# Note: docker-compose automatically uses debug builds for faster dev/test cycles
# To override: BUILD_TYPE=release make dev

# Devcontainer (for VS Code / devcontainer CLI users)
make dc-up         # Start devcontainer
make dc-attach     # Attach terminal to running devcontainer
make dc-down       # Stop devcontainer

# Testing
make test              # All tests (builds Docker if needed)
make test-unit         # Unit tests only (fast, no Docker)
make test-compose      # Integration tests via docker compose
make test-compose-ui   # Selenium UI tests via docker compose
make test-compose-down # Stop integration test services (auto-cleanup on exit, rarely needed)

# Running
make run-cli       # CLI game with defaults
cargo run -- --min 1 --max 100 --limit 10

make run-server    # Web server (needs postgres)

# Code Quality
make fmt           # Format code
make lint          # Run clippy

# Database & Compose
make db-shell      # PostgreSQL shell
make compose-up    # Bring up compose stack (app + postgres)
make compose-down  # Tear down compose stack
make logs          # View docker-compose logs

# Cleanup
make clean             # Clean everything
```

### Key Points
- **Build**: No database needed (SQLx uses runtime checking, not compile-time)
- **CLI mode**: No database needed at all
- **Web mode**: Requires PostgreSQL running
- **Tests**: Unit tests are fast (no Docker); integration and UI suites run via `make test-compose*` against Docker Compose
- **Docker**: Only needed for integration tests and optional full-stack development
- **Docker Builds**: Development/test workflows use debug builds (fast); production uses release builds (optimized). Configure via `BUILD_TYPE` env var or make targets.

## Architecture

### Module Organization
The codebase is organized into clear architectural layers:

#### Core (`src/core/`)
Pure business logic with no I/O dependencies:
- **game.rs**: Core game logic - `GuessingGame` struct and `GuessResult` enum
- **game_id.rs**: Type-safe newtype wrapper for game IDs
- **validators.rs**: Shared validation logic used by CLI, API, and Web layers
- **features/**: Feature modules (e.g., difficulty calculator)

#### API (`src/api/`)
REST API with JSON endpoints:
- **handlers/**: API request handlers (game creation, guessing, health check)
- **types.rs**: Request/response types for JSON API

#### Web UI (`src/web/`)
HTML/HTMX interface:
- **handlers/**: Web UI handlers (game creation, guessing, difficulty preview)
- **templates.rs**: Askama template structs for type-safe HTML rendering
- **types.rs**: Form request types for web UI

#### CLI (`src/cli/`)
Command-line interface:
- **args.rs**: Argument parsing using clap
- **io.rs**: User input/output helpers
- **runner.rs**: CLI game loop

#### Database (`src/db/`)
- **mod.rs**: PostgreSQL layer with runtime-checked SQLx queries

#### Server (`src/server/`)
- **mod.rs**: Server initialization, routing configuration, and startup

#### Other
- **src/main.rs**: Entry point, mode selection (CLI vs Server), database initialization
- **static/index.html**: Web UI with HTMX for dynamic updates
- **templates/**: Askama HTML templates (compile-time checked)
- **migrations/**: SQLx database migrations

### Key Design Patterns
1. **Layered Architecture**: Clear separation between Core, API, Web UI, CLI, Database, and Server layers
2. **Separation of Concerns**: Game logic isolated from I/O, validation separate from presentation, database layer separate from web layer
3. **Shared Validation**: Single source of truth for validation logic in `core/validators` module
4. **API vs Web UI Separation**: JSON API handlers in `src/api/`, HTML handlers in `src/web/`
5. **Type Safety**: Newtype pattern for `GameId`, compile-time template checking with Askama
6. **Result Types**: Extensive use of `Result<T, String>` for error handling with safe type conversions
7. **State Management**: Web server uses PostgreSQL with SQLx connection pooling (`PgPool`)
8. **Module Organization**: Clear boundaries between core logic, API, web UI, CLI, database, and server
9. **Template-Based HTML**: Askama templates provide compile-time checked, type-safe HTML rendering
10. **Structured Logging**: Uses `tracing` framework for async-aware, structured system logs while preserving user-facing CLI output

## Important Constraints

### Numeric Limits
- Range: 0 to 1,000,000 (inclusive)
- Guess limits: Max 1000 (CLI), Max 100 (Web/API)
- Negative numbers not allowed

### Web API & UI
- Games persisted in PostgreSQL database
- Game IDs are random u64 values
- Games auto-removed when completed
- JSON request/response format
- **Web UI Features**:
  - HTMX-powered dynamic updates without page reloads
  - Remaining guesses counter (displays "Guesses remaining: X" when limit is set)
  - Styled with `.guesses-remaining` CSS class (blue background, prominent display)
  - Counter shows initial count at game start and updates after each guess

### Security Considerations
- Input validation prevents integer overflow
- Range limits prevent DoS via large ranges
- No persistent storage (stateless between restarts)
- HTMX from CDN (consider bundling for production)

## Testing Strategy
```bash
# Unit tests in src/game.rs
cargo test --lib

# Integration tests via docker compose (API)
make test-compose

# Web UI tests via docker compose (Selenium)
make test-compose-ui

# Full suite via make
make test
```

## Common Tasks

### Adding a New Feature
1. Update game logic in `src/core/game.rs`
2. Add validation logic in `src/core/validators.rs` if needed
3. Add CLI support:
   - Argument parsing in `src/cli/args.rs`
   - User I/O in `src/cli/io.rs`
   - Game loop in `src/cli/runner.rs`
4. Add API support:
   - Request/response types in `src/api/types.rs`
   - Handlers in `src/api/handlers/`
5. Add Web UI support:
   - Form types in `src/web/types.rs`
   - Handlers in `src/web/handlers/`
   - Templates in `src/web/templates.rs` and `templates/`
6. Modify HTML in `static/index.html` for UI changes
7. Write tests in the relevant module
8. Update documentation (README.md, docs/api.md, docs/requirements.md)

### Modifying Game Rules
- Core logic in `src/core/game.rs::GuessingGame::make_guess()`
- Add new `GuessResult` variants as needed
- Update all match statements handling `GuessResult` in API, Web, and CLI layers

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
│   ├── core/        # Core business logic (no I/O)
│   │   ├── mod.rs
│   │   ├── game.rs      # Game logic (GuessingGame, GuessResult)
│   │   ├── game_id.rs   # Type-safe game ID wrapper
│   │   ├── validators.rs # Shared validation logic
│   │   └── features/    # Feature modules
│   │       ├── mod.rs
│   │       └── difficulty/
│   │           ├── mod.rs
│   │           ├── calculator.rs
│   │           └── types.rs
│   ├── api/         # REST API (JSON endpoints)
│   │   ├── mod.rs
│   │   ├── types.rs     # API request/response types
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── game.rs  # Game creation API handler
│   │       ├── guess.rs # Guess processing API handler
│   │       └── health.rs # Health check handler
│   ├── web/         # Web UI (HTML/HTMX)
│   │   ├── mod.rs
│   │   ├── templates.rs # Askama template structs
│   │   ├── types.rs     # Web form types
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── game.rs       # Game creation web handler
│   │       ├── guess.rs      # Guess processing web handler
│   │       └── difficulty.rs # Difficulty preview handler
│   ├── cli/         # Command-line interface
│   │   ├── mod.rs
│   │   ├── args.rs      # CLI argument parsing (clap)
│   │   ├── io.rs        # User input/output helpers
│   │   └── runner.rs    # CLI game loop
│   ├── db/          # Database layer
│   │   └── mod.rs       # PostgreSQL operations (SQLx)
│   ├── server/      # Server setup
│   │   └── mod.rs       # Server initialization & routing
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
│   ├── update_error.html
│   └── difficulty_indicator.html
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
├── Makefile         # Make command runner with shortcuts (dc-up, dc-down, dc-attach)
├── build.rs         # Build script (Docker image for tests)
├── run_integration_tests.sh  # Test runner (Linux/macOS)
├── run_integration_tests.bat # Test runner (Windows)
└── README.md        # Main documentation
```

## Logging Configuration

### Structured Logging with Tracing
The application uses the `tracing` framework for structured, async-aware logging of system events.

**Key Principles:**
- **System logs** (database, web server, errors): Use `tracing` macros (`info!`, `error!`, etc.)
- **User-facing output** (CLI prompts, game feedback): Use `println!` for direct console output
- **Environment-controlled**: Configure verbosity via `RUST_LOG` environment variable

**Configuration:**
```bash
# Default: info level for the application
RUST_LOG=number_guessing_game=info cargo run -- --server

# Debug level for troubleshooting
RUST_LOG=number_guessing_game=debug cargo run -- --server

# Trace level for detailed diagnostics
RUST_LOG=trace cargo run -- --server

# Multiple modules with different levels
RUST_LOG=sqlx=warn,number_guessing_game=debug cargo run -- --server

# Only show errors
RUST_LOG=error cargo run -- --server
```

**Log Levels (most to least verbose):**
1. `trace` - Very detailed, function-level tracing
2. `debug` - Debugging information
3. `info` - General informational messages (default)
4. `warn` - Warning messages
5. `error` - Error messages only

**Structured Fields:**
Logs include contextual information as structured fields:
```
INFO number_guessing_game: Connecting to database max_connections=5
INFO number_guessing_game::web: Starting web server main_addr="0.0.0.0:3000" health_addr="0.0.0.0:8081"
ERROR number_guessing_game::web: Failed to make guess game_id=12345 error="Database error"
```

**Development Tips:**
- Start with `info` level for normal operation
- Use `debug` when troubleshooting issues
- Use `trace` for deep debugging (very verbose)
- In `.env` file: Set `RUST_LOG=number_guessing_game=info` (see `.env.example`)

**Stdout vs Stderr:**
- **Stderr**: Structured logs via `tracing` (for monitoring, debugging)
- **Stdout**: Program output only - emits `"READY"` when server is fully initialized
- This separation follows Unix conventions and enables:
  - Process managers (Docker, Kubernetes, systemd) to detect readiness
  - Integration tests to wait for server startup
  - Clean separation of logs from application output

## Database Setup

### PostgreSQL Integration
The web server uses PostgreSQL for persistent storage.

**Quick Start:**
```bash
# Option 1: Full stack in Docker (easiest for manual testing)
make dev
# Access at http://localhost:8080

# Option 2: Database only (run app locally for faster iteration)
make dev-db
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

**Database Configuration:**
The database name and credentials are centralized via environment variables in `.env` file:
- `POSTGRES_USER`: Database username (default: `numberguess`)
- `POSTGRES_PASSWORD`: Database password (default: `password`)
- `POSTGRES_DB`: Database name (default: `numberguess_dev`)
- `DATABASE_URL`: Full connection string (uses the variables above)

All configuration files (docker-compose.yml, Makefile) reference these environment variables, providing a single source of truth. To change the database name, simply update `.env` and all components will use the new value.

**Default Credentials:**
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
- **tracing**: Structured, async-aware logging framework (v0.1)
- **tracing-subscriber**: Log collection and formatting with env-filter (v0.3)

### Test Dependencies  
- **Docker Compose**: Orchestrates Postgres/app/Selenium for integration tests (see `docker-compose.integration.yml`)
- **reqwest**: HTTP client for API testing (v0.12.23)
- **assert_cmd**: CLI testing framework (v2.0)
- **predicates**: Test assertion predicates (v3.0)
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
- NEVER change an external API such as a REST endpoint without explicit approval to make the change
- ALWAYS document external API so the document becomes a reference for behavior