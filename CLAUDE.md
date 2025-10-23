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
# Development - Start full stack (postgres + keycloak + oauth2-proxy + app in Docker)
make dev
# Access at http://localhost:8080 (via oauth2-proxy)
# Login with admin@local.test / password

# Development - Start only database (run app locally for faster iteration)
make dev-db
cargo run -- --server --port 4080

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
make test              # All tests (unit + integration)
make test-unit         # Unit tests only (fast, no Docker)
make test-integration  # Integration tests (starts Docker Compose, keeps running)
make test-down         # Stop integration test environment

# Debugging failed tests: Environment stays running after test-integration
docker compose logs keycloak  # Check service logs
make test-down                # Clean up when done

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
- **Web mode**: Requires PostgreSQL and authentication stack (Keycloak + oauth2-proxy + Redis)
- **Tests**: Unit tests are fast (no Docker); integration tests use `make test-integration` with Docker Compose (includes Selenium for all tests)
- **Docker**: Only needed for integration tests and optional full-stack development
- **Docker Builds**: Development/test workflows use debug builds (fast); production uses release builds (optimized). Configure via `BUILD_TYPE` env var or make targets.
- **Authentication**: All web routes require authentication via oauth2-proxy + Keycloak (OIDC)
- **Test Environment**: `make test-integration` keeps services running for debugging; use `make test-down` to clean up

## Architecture

### Authentication Architecture
The application uses an authentication proxy pattern with OAuth2/OIDC:

**Components:**
- **oauth2-proxy (Port 8080)**: Authentication gateway, external access point
- **Keycloak (Port 8090)**: OIDC identity provider with user management
- **Redis**: Session storage for oauth2-proxy
- **Application (Port 4080)**: Internal only, accessed via oauth2-proxy
- **Health Check (Port 8081)**: Internal only health endpoint

**Flow:**
1. User accesses http://localhost:8080
2. oauth2-proxy checks for valid session
3. If not authenticated, redirects to Keycloak (localhost:8090)
4. User logs in with Keycloak credentials (admin@local.test / password)
5. Keycloak redirects back to oauth2-proxy with OAuth2 code
6. oauth2-proxy exchanges code for tokens, stores session in Redis
7. oauth2-proxy forwards request to app with user headers:
   - `X-Auth-Request-User`: User ID (OIDC subject)
   - `X-Auth-Request-Email`: User's email address
   - `X-Auth-Request-Preferred-Username`: Username
   - `X-Auth-Request-Groups`: Comma-separated list of groups

**Security:**
- Network isolation: App not exposed externally
- PKCE with S256 for OAuth2 authorization code flow
- Redis-backed sessions for horizontal scalability
- All routes require authentication (no anonymous access)

### Module Organization
The codebase is organized into clear architectural layers:

#### Authentication (`src/auth/`)
User authentication and authorization:
- **mod.rs**: `AuthenticatedUser` extractor for Axum handlers, extracts user from oauth2-proxy headers

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
- **Authentication**: All web routes require OAuth2/OIDC authentication
- **Authorization**: User groups available via `AuthenticatedUser::is_in_group()`
- **Network Isolation**: Application runs on port 4080 (internal only, not exposed)
- **Session Security**: Redis-backed sessions with secure cookies (httponly, samesite=lax)
- **OAuth2 Security**: PKCE with S256 challenge method
- **Input Validation**: Prevents integer overflow
- **Range Limits**: Prevent DoS via large ranges
- **Database**: PostgreSQL with connection pooling and SQL injection protection
- **HTMX**: Loaded from CDN (consider bundling for production)

## Testing Strategy

### Authentication in Tests

**All integration tests require full authentication stack** (Keycloak + oauth2-proxy + Redis). Tests use Selenium OAuth2 authentication for all endpoints (Web UI and API).

**Authentication Approach:**
- **All Tests**: Selenium OAuth2 flow (realistic, tests full user experience)
- **Method**: Full browser-based OAuth2 authorization code flow with PKCE
- **Target**: http://localhost:8080 (oauth2-proxy)
- **Speed**: ~2-3s per login
- **Coverage**: Tests oauth2-proxy integration, session cookies, redirects

**Test Credentials:**
- Username: `admin@local.test`
- Password: `password`

**Service URLs:**
- Keycloak: http://localhost:8090 (OIDC provider)
- oauth2-proxy: http://localhost:8080 (auth gateway, external access)
- Application: http://localhost:4080 (internal only, accessed via oauth2-proxy)
- Health Check: http://localhost:8081 (internal only)

**Selenium OAuth2 Flow:**
- Used by: All integration tests (`web_ui_test.rs`, `web_endpoints_test.rs`, `api_edge_cases_test.rs`, `auth_integration_test.rs`)
- Method: Full browser-based OAuth2 authorization code flow with PKCE
- Target: http://localhost:8080 (oauth2-proxy)
- Speed: ~2-3s per login
- Coverage: Tests oauth2-proxy integration, session cookies, redirects for both web and API endpoints

### Integration Test Architecture & Networking

**CRITICAL**: Integration tests MUST use async patterns. Do NOT use `tokio_test::block_on()` or `reqwest::blocking::Client` as they cause tokio runtime conflicts.

**Docker Compose Service Topology:**
```
┌─────────────────────────────────────────────────────────────┐
│ Host Machine (Test Runner)                                   │
│                                                               │
│  Integration Tests (Rust)                                    │
│    ├─ Use `#[tokio::test]` + async/await                    │
│    ├─ Access via localhost:8080, localhost:4444, etc.       │
│    └─ Selenium OAuth2 authentication flow                   │
│                                                               │
│        │ HTTP requests                                       │
│        ▼                                                      │
└─────────────────────────────────────────────────────────────┘
         │
         │ Port mapping (localhost:8080 → oauth2-proxy:4180)
         ▼
┌─────────────────────────────────────────────────────────────┐
│ Docker Bridge Network (numberguess_default)                  │
│                                                               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │  Selenium    │    │ oauth2-proxy │◄───│  Keycloak    │  │
│  │  :4444       │───▶│  :4180       │    │  :8090       │  │
│  │ (Chrome)     │    │ (Auth Proxy) │    │  (OIDC)      │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                    │                   │          │
│         │                    ▼                   ▼          │
│         │             ┌──────────────┐    ┌──────────────┐ │
│         └────────────▶│     App      │    │    Redis     │ │
│                       │   :4080      │    │   :6379      │ │
│                       └──────────────┘    └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Network Addressing - CRITICAL DIFFERENCE:**

**Inside Docker Compose (service-to-service)**:
- Services use Docker hostnames: `oauth2-proxy:4180`, `keycloak:8090`, `redis:6379`, `app:4080`
- Selenium (running in Docker) MUST use these hostnames to reach other services
- Example: Selenium navigates to `http://oauth2-proxy:4180` for OAuth2 flow

**Outside Docker Compose (tests on host)**:
- Use `localhost` + exposed port: `localhost:8080`, `localhost:4444`, `localhost:6379`
- Port mapping: `localhost:8080` → `oauth2-proxy:4180` (internal)
- Tests access services via exposed ports on localhost

**Port Mapping Table:**
| Service | Internal (Docker) | External (localhost) | Purpose |
|---------|------------------|---------------------|---------|
| oauth2-proxy | oauth2-proxy:4180 | localhost:8080 | Auth gateway (external access) |
| keycloak | keycloak:8090 | localhost:8090 | OIDC provider |
| app | app:4080 | (not exposed) | Application (internal only) |
| app health | app:8081 | localhost:8081 | Health check endpoint |
| redis | redis:6379 | localhost:6379 | Session storage |
| selenium | selenium:4444 | localhost:4444 | Browser automation |

**Environment Variables:**
```bash
# Where tests (on host) connect to Selenium
SELENIUM_REMOTE_URL=http://localhost:4444

# Where tests (on host) access the application (via oauth2-proxy)
GAME_SERVER_BASE_URL=http://localhost:8080

# Where Selenium (in Docker) accesses oauth2-proxy (Docker hostname!)
GAME_SERVER_BROWSER_URL=http://oauth2-proxy:4180
```

**Why These Variables Exist:**
- Tests run on **host** → use `localhost:8080`, `localhost:4444`
- Selenium runs in **Docker** → must use `oauth2-proxy:4180` (Docker hostname)
- Without `GAME_SERVER_BROWSER_URL`, Selenium would try `localhost:4180` which doesn't exist in Docker

**Async Requirement - DO NOT USE BLOCKING PATTERNS:**

**❌ WRONG - Causes Runtime Conflicts:**
```rust
#[test]  // Wrong! Not async
fn test_something() {
    let client = tokio_test::block_on(  // Wrong! Nested runtime
        create_authenticated_client_selenium()
    ).unwrap();

    let response = tokio_test::block_on(async {  // Wrong! Multiple block_on
        client.get(url).send().await
    }).unwrap();
}
```

**✅ CORRECT - Proper Async Pattern:**
```rust
#[tokio::test]  // Correct! Tokio test
async fn test_something() {  // Correct! Async function
    // Environment checks in blocking context (they use blocking client)
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required");
    })
    .await
    .expect("Environment checks failed");

    // Create async authenticated client
    let client = auth_helpers::create_authenticated_client_selenium()
        .await  // Correct! Direct await
        .expect("Failed to create client");

    // Make requests with await
    let response = client
        .get("http://localhost:8080")
        .send()
        .await  // Correct! Direct await
        .expect("Request failed");
}
```

**Why Blocking Fails:**
1. `#[tokio::test]` initializes a tokio runtime
2. `tokio_test::block_on()` tries to create a nested runtime → **panic or deadlock**
3. `reqwest::blocking::Client` has DNS resolution issues in tokio context
4. Application uses tokio → all tests must be tokio-compatible

**Test Pattern Summary:**
- ✅ Use `#[tokio::test]` for all integration tests
- ✅ Use `async fn` for test functions
- ✅ Use `reqwest::Client` (async, not blocking)
- ✅ Use `.await` for all async operations
- ✅ Wrap blocking environment checks in `tokio::task::spawn_blocking`
- ❌ Never use `tokio_test::block_on()`
- ❌ Never use `reqwest::blocking::Client`
- ❌ Never use `#[test]` with async code

**Running Tests:**
```bash
# Unit tests (no authentication needed)
cargo test --lib
# or
make test-unit

# Integration tests (includes auth stack + Selenium)
make test-integration
# Environment stays running for debugging - use 'make test-down' to stop

# Full suite (unit + integration)
make test
# Note: Runs unit tests first, then integration tests

# Stop integration test environment
make test-down
```

**IMPORTANT - Integration Test Startup Time:**
Integration tests take ~5 minutes to start all services (postgres, redis, keycloak, oauth2-proxy, selenium, app). The `make test-integration` command uses `docker compose up --wait` which automatically waits for all health checks to pass before running tests.

**To monitor service startup progress:**
```bash
# In one terminal: Start integration tests
make test-integration

# In another terminal: Monitor service health in real-time
watch -n 2 'docker compose ps'

# Or check specific service logs
docker compose logs -f keycloak    # Slowest service (~30-60s)
docker compose logs -f app
docker compose logs -f oauth2-proxy

# Check all services are healthy
docker compose ps --format "table {{.Service}}\t{{.Status}}\t{{.Health}}"
```

The startup sequence completes when all services show "healthy" status. The `make test-integration` command will automatically wait for this before running tests.

**Test Startup Sequence:**
Docker Compose now handles service orchestration automatically via health checks and dependencies:
1. postgres, redis start first (parallel)
2. keycloak starts after postgres is healthy (30-60s - imports realm configuration)
3. app starts after postgres is healthy
4. oauth2-proxy starts after keycloak, redis, and app are all healthy
5. selenium starts after oauth2-proxy and app are healthy
6. `docker compose up --wait` ensures all services are healthy before tests run
7. Tests run with `--test-threads=1` (prevents session conflicts)
8. Environment stays running after tests complete (for debugging)

**Troubleshooting Tests:**

**Debugging Failed Tests:**
Since `make test-integration` keeps the environment running after tests:
1. Run `make test-integration` - tests run, environment stays up
2. Check service logs: `docker compose logs keycloak`, `docker compose logs app`, etc.
3. Inspect running containers: `docker compose ps`
4. Re-run specific tests: `cargo test --test auth_integration_test`
5. Clean up when done: `make test-down`

Authentication Issues:
- **Keycloak not ready**: Check `docker compose logs keycloak` (imports realm on startup)
- **Login fails**: Verify realm import succeeded, check user exists
- **Session issues**: Check Redis is running: `docker compose exec redis redis-cli ping`
- **401 errors**: Verify auth headers are being sent (check auth_helpers)

Service Health Checks:
- Keycloak: http://localhost:8090/health/ready
- oauth2-proxy: http://localhost:8080 (should redirect to Keycloak)
- App: http://localhost:8081/health
- Redis: TCP connection to localhost:6379

Common Issues:
- **"Keycloak not responding"**: Keycloak can take 30-60s to start (Docker Compose waits automatically)
- **"Session cookie not found"**: OAuth2 flow may have failed, check Keycloak logs
- **Test timeouts**: Use `--test-threads=1` to avoid parallel execution conflicts
- **Port conflicts**: Ensure ports 4080, 5432, 6379, 8080, 8081, 8090 are available
- **Services not starting**: Check `docker compose -f docker-compose.yml -f docker-compose.integration.yml --profile integration ps`

**Performance Notes:**
- **Full stack startup**: ~90 seconds (Keycloak is slowest component, health checks handle timing)
- **Subsequent runs**: Instant if services already running (idempotent `docker compose up`)
- **All tests**: 2-3s overhead per test for OAuth2 login (Selenium-based)
- **Total test suite**: ~3-5 minutes with full auth stack

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
│   ├── auth/        # Authentication
│   │   └── mod.rs       # AuthenticatedUser extractor
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
├── keycloak/        # Keycloak configuration
│   └── realm-export.json # Realm configuration with users and groups
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
- For all new features, determine the dependencies and plan to implement the dependencies one at a time with testing to prove each dependency is working before moving on to the next step.
- When transitioning from planning to implementation, ALWAYS write the plan to a document in the plans directory. Once all work is done on the feature then ask the user if you should clean up the plan document.