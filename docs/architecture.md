# System Architecture

## Overview

The Number Guessing Game is built with a modular, layered architecture that separates concerns and enables CLI, REST API, and web interfaces to share the same core game logic.

```
┌──────────────────────────────────────────────────────────────┐
│                        User Interfaces                        │
├────────────────┬────────────────┬────────────────────────────┤
│ CLI Interface  │   REST API     │    Web UI (Browser)        │
│                │   (JSON)       │    (HTML + HTMX)           │
└────────────────┴────────────────┴────────────────────────────┘
        │               │                    │
        ▼               ▼                    ▼
┌──────────────┐ ┌───────────────┐ ┌──────────────────────┐
│ CLI Handler  │ │ API Endpoints │ │  Web UI Endpoints    │
│ (main.rs)    │ │ (/api/*)      │ │  (/game/*, /)        │
└──────────────┘ └───────────────┘ └──────────────────────┘
        │               │                    │
        │               └─────────┬──────────┘
        │                         ▼
        │                ┌──────────────────┐
        │                │  Axum Web Server │
        │                │   (src/web.rs)   │
        │                └──────────────────┘
        │                         │
        └─────────────┬───────────┘
                      ▼
            ┌───────────────────┐
            │  Game Logic Core  │
            │  (src/game.rs)    │
            └───────────────────┘
```

## Interface Layers

### 1. CLI Interface
- **Entry Point**: Direct execution via command line
- **Input**: Command-line arguments and interactive prompts
- **Output**: Terminal text output
- **Protocol**: Direct function calls

### 2. REST API Interface
- **Entry Point**: HTTP endpoints under `/api/*`
- **Input**: JSON request bodies
- **Output**: JSON response bodies
- **Protocol**: HTTP/REST
- **Clients**: curl, Postman, programmatic clients

### 3. Web UI Interface
- **Entry Point**: Browser access to `/`
- **Input**: HTML forms
- **Output**: HTML fragments via HTMX
- **Protocol**: HTTP with HTMX for dynamic updates
- **Clients**: Web browsers

## Core Components

### 1. Game Logic Module (`src/game.rs`)

**Purpose**: Pure business logic with no external dependencies (except `rand`)

**Key Types**:
```rust
pub struct GuessingGame {
    min: i32,
    max: i32,
    secret_number: i32,
    guess_count: u32,
    max_guesses: Option<u32>,
}

pub enum GuessResult {
    TooLow,
    TooHigh,
    Correct { number: i32, attempts: u32 },
    LimitReached { number: i32, max_guesses: u32 },
}
```

**Design Decisions**:
- Immutable after creation (except for guess tracking)
- No I/O operations for testability
- Result type for all fallible operations
- Comprehensive input validation

### 2. CLI Module (`src/cli.rs`)

**Purpose**: Command-line argument parsing and user input handling

**Key Components**:
- `Cli` struct: Clap-derived argument parser
- Input validation functions: `get_min_value()`, `get_max_value()`, `get_guess_limit()`
- Generic input reader: `read_input<T>()`

**Design Patterns**:
- Progressive disclosure (CLI args → interactive prompts)
- Input retry on validation failure
- Type-safe parsing with generics

### 3. Web Module (`src/web.rs`)

**Purpose**: HTTP server with both REST API and web UI support

**Architecture**:
```
HTTP Request
    ↓
Axum Router
    ├── /api/* → JSON Handlers (REST API)
    ├── /game/* → HTML Handlers (Web UI)
    └── / → Static File Server
           ↓
      Game State Management
      PostgreSQL Database
```

**State Management**:
```rust
type SharedState = PgPool;

// Games stored in PostgreSQL with schema:
// - game_id (BIGINT PRIMARY KEY)
// - min (INTEGER)
// - max (INTEGER)
// - secret_number (INTEGER)
// - guess_count (INTEGER)
// - max_guesses (INTEGER, nullable)
```

### 4. Main Entry Point (`src/main.rs`)

**Purpose**: Minimal orchestration and mode selection

**Flow**:
1. Parse CLI arguments
2. Check for `--server` flag
3. Route to appropriate handler:
   - Server mode → Initialize Tokio runtime → Connect to PostgreSQL → Run migrations → Start Axum server (serves both API and UI)
   - CLI mode → Run interactive game loop (no database required)

## REST API Interface

### API Design Principles
- **RESTful**: Resource-based URLs, HTTP verbs
- **Stateless**: Each request contains all needed information
- **JSON Format**: Consistent request/response structure
- **Idempotent**: Safe to retry requests (except game creation)

### Endpoint Structure

#### 1. Create Game Endpoint
```
POST /api/games
Content-Type: application/json

Request Body:
{
  "min": number,        // Required: minimum range (0-1,000,000)
  "max": number,        // Required: maximum range (0-1,000,000)
  "max_guesses": number // Optional: guess limit (null/0 for unlimited, max 100)
}

Success Response (200 OK):
{
  "game_id": number,     // Unique game identifier (u64)
  "min": number,         // Confirmed minimum
  "max": number,         // Confirmed maximum
  "max_guesses": number, // Guess limit (null if unlimited)
  "message": string      // Human-readable message
}

Error Response (400 Bad Request):
{
  "error": string        // Error description
}
```

#### 2. Make Guess Endpoint
```
POST /api/games/{game_id}/guess
Content-Type: application/json

Path Parameters:
- game_id: number (u64)  // Game identifier from creation

Request Body:
{
  "guess": number        // Required: player's guess
}

Success Response (200 OK):
{
  "result": string,      // "too_low" | "too_high" | "correct" | "limit_reached"
  "message": string,     // Human-readable feedback
  "attempts": number     // Current attempt count
}

Error Response (404 Not Found):
{
  "error": string        // "Game with ID {game_id} not found"
}
```

## Web UI Interface (HTMX)

### Design Principles
- **Server-Side Rendering**: HTML generated on server
- **Progressive Enhancement**: Works without JavaScript
- **Partial Updates**: HTMX swaps DOM fragments
- **Form-Based**: Standard HTML forms with HTMX attributes

### Endpoint Structure

#### 1. Static Files
```
GET /
Serves: static/index.html (main page with game setup form)
```

#### 2. Create Game (HTML)
```
POST /game/new
Content-Type: application/x-www-form-urlencoded

Form Data:
- min: number
- max: number
- max_guesses: number (optional)

Response: HTML fragment for game area
```

#### 3. Make Guess (HTML)
```
POST /game/{game_id}/guess
Content-Type: application/x-www-form-urlencoded

Form Data:
- guess: number

Response: HTML fragment with updated game state
```

## Data Flow Comparison

### CLI Flow
```
Terminal Input → CLI Parser → Game Logic → Terminal Output
     (sync)         (sync)       (sync)        (sync)
```

### REST API Flow
```
JSON Request → Axum Handler → Game State → JSON Response
    (async)       (async)       (mutex)       (async)
```

### Web UI Flow
```
HTML Form → HTMX Request → Axum Handler → HTML Fragment → DOM Update
  (browser)    (async)        (async)        (async)       (browser)
```

## Concurrency Model

### Web Server (API + UI)
- **Tokio Runtime**: Async I/O for handling multiple connections
- **Database Connection Pool**: SQLx PgPool for concurrent database access
- **Request Handling**: Each request runs in its own task
- **Transaction Strategy**: Database transactions with row-level locking for concurrent guess processing

### CLI
- **Single-threaded**: Synchronous execution
- **Blocking I/O**: Direct stdin/stdout operations

## Error Handling Strategy

### Validation Layers
1. **Input Parsing**: Type conversion errors
2. **Business Rules**: Range validation, limit checks
3. **State Validation**: Game existence, completion status

### Error Propagation by Interface
```
CLI:      Input Error → Retry Prompt → Success/Exit
REST API: Input Error → HTTP 400 → Error JSON
Web UI:   Input Error → HTTP 200 → Error HTML Fragment
Game:     Logic Error → Result<T, String> → Handler Decision
```

## Authentication & Network Architecture

### Service Topology

The application uses an authentication proxy pattern with OAuth2/OIDC:

```
                          ┌────────────────────┐
                          │   User/Browser     │
                          └─────────┬──────────┘
                                    │ HTTP
                                    ▼
                          ┌────────────────────┐
                          │  oauth2-proxy      │
                          │  Port: 4180        │
                          │  (External: 8080)  │
                          └─────────┬──────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
          ┌─────────────┐  ┌────────────┐  ┌─────────────┐
          │  Keycloak   │  │    App     │  │   Redis     │
          │  Port: 8090 │  │  Port 4080 │  │  Port: 6379 │
          │   (OIDC)    │  │ (internal) │  │  (sessions) │
          └─────────────┘  └────────────┘  └─────────────┘
                                 │
                                 ▼
                          ┌──────────────┐
                          │  PostgreSQL  │
                          │  Port: 5432  │
                          └──────────────┘
```

### Network Boundaries

**Docker Network (Internal Communication)**:
- Services communicate using Docker hostnames
- Examples: `oauth2-proxy:4180`, `keycloak:8090`, `redis:6379`, `app:4080`
- Docker's internal DNS resolves service names to container IPs
- Network name: `numberguess_default` (created by Docker Compose)

**Host Network (External Access)**:
- External access via `localhost` + exposed port
- Port mapping defined in `docker-compose.yml`
- Only specific ports are exposed to the host

### Port Mapping

| Service | Internal Port | Internal Hostname | External Port | External Access | Purpose |
|---------|--------------|------------------|---------------|-----------------|---------|
| oauth2-proxy | 4180 | oauth2-proxy:4180 | 8080 | localhost:8080 | Auth gateway (main entry) |
| keycloak | 8090 | keycloak:8090 | 8090 | localhost:8090 | OIDC provider |
| app (main) | 4080 | app:4080 | (not exposed) | (internal only) | Application server |
| app (health) | 8081 | app:8081 | 8081 | localhost:8081 | Health check |
| redis | 6379 | redis:6379 | 6379 | localhost:6379 | Session storage |
| postgres | 5432 | postgres:5432 | 5432 | localhost:5432 | Database |

**Critical Network Isolation**:
- Application (port 4080) is **never exposed** to host
- All external access goes through oauth2-proxy (port 8080)
- This prevents unauthorized direct access to the application

### Authentication Flow

```
1. User → localhost:8080/
   ├─ oauth2-proxy checks session
   └─ No session → Redirect to Keycloak

2. User → Keycloak login (localhost:8090)
   ├─ Enter credentials
   └─ OAuth2 authorization code flow (PKCE)

3. Keycloak → oauth2-proxy/callback?code=xxx
   ├─ oauth2-proxy exchanges code for tokens
   ├─ Creates session in Redis
   └─ Sets cookie (_oauth2_proxy)

4. oauth2-proxy → app:4080
   ├─ Adds headers:
   │  ├─ X-Auth-Request-User
   │  ├─ X-Auth-Request-Email
   │  ├─ X-Auth-Request-Preferred-Username
   │  └─ X-Auth-Request-Groups
   └─ Proxies request to application
```

### Testing Network Topology

Integration tests run on the host but use Selenium in Docker:

```
Host Machine (Tests)
├─ Accesses localhost:8080 (oauth2-proxy)
├─ Accesses localhost:4444 (Selenium)
└─ Sets environment variables

Docker Network (Services + Selenium)
├─ Selenium navigates to oauth2-proxy:4180
└─ (NOT localhost - inside Docker!)
```

**Why This Configuration**:
- Tests run on **host** → use `localhost:8080`
- Selenium runs in **Docker** → must use `oauth2-proxy:4180`
- Environment variable `GAME_SERVER_BROWSER_URL` controls this
- Without it, Selenium would try `localhost` which doesn't work in Docker

## Security Architecture

### Input Validation
- **Numeric Bounds**: Prevent integer overflow
- **Range Limits**: Max 1,000,000 to prevent DoS
- **Guess Limits**: Max 100 (web) / 1000 (CLI)

### Web Security
- **No Authentication**: Public game sessions
- **PostgreSQL Database**: Persistent storage with parameterized queries (SQL injection protection via SQLx)
- **JSON Parsing**: Serde handles malformed input
- **Static Files**: Served from controlled directory
- **Database Migrations**: Schema versioning with SQLx migrations

### Current Vulnerabilities
- No rate limiting
- No request size limits
- Database growth possible (games persist until completed)
- HTMX loaded from CDN
- No CORS configuration
- Database credentials in environment variables

## Performance Characteristics

### Database Storage
- **Per Game**: Database row with 6 columns (game_id, min, max, secret_number, guess_count, max_guesses)
- **Storage Growth**: Linear with active games, automatic cleanup on completion
- **Connection Pool**: Configurable (default: 5 connections, max: 100)

### Time Complexity
- **Game Creation**: O(1) database insert with index
- **Guess Processing**: O(1) database query + update in transaction
- **Game Lookup**: O(1) indexed primary key lookup
- **Game Cleanup**: O(1) database delete on completion

### Scalability Limits
- **Concurrent Games**: Limited by database storage and connection pool
- **Requests/Second**: Limited by database throughput and connection pool size
- **Single Instance**: Can support multiple instances with shared PostgreSQL database

## Deployment Architecture

### Build Output
```
target/release/number_guessing_game
├── Single static binary
├── No runtime dependencies
└── Embedded static files
```

### Runtime Requirements
- **OS**: Any (Windows, Linux, macOS)
- **Network**: Port binding capability
- **Memory**: ~10MB base + connection pool overhead
- **CPU**: Single core sufficient
- **Database**: PostgreSQL 12+ (for web server mode only, CLI mode has no database requirement)

### Configuration
- **CLI Arguments**: Runtime configuration (--server, --port)
- **Environment Variables**: Required for web mode (DATABASE_URL, optional: DB_MAX_CONNECTIONS, RUST_LOG)
- **Config Files**: .env file support via dotenvy
- **Hardcoded Values**: Port default (3000), health port (8081), connection pool default (5), limits

## Testing Architecture

### Unit Tests
- **Location**: Inline with modules (`#[cfg(test)]`)
- **Coverage**: Game logic, validation
- **Strategy**: Pure functions, no I/O

### Integration Tests
- **Examples**: `demo.rs`, `web_client.rs`
- **Manual Testing**: Web UI interaction, API testing with curl
- **Missing**: Automated E2E tests

## API Testing Examples

### REST API Testing
```bash
# Create game
curl -X POST http://localhost:3000/api/games \
  -H "Content-Type: application/json" \
  -d '{"min": 1, "max": 100, "max_guesses": 10}'

# Make guess
curl -X POST http://localhost:3000/api/games/12345/guess \
  -H "Content-Type: application/json" \
  -d '{"guess": 50}'
```

### Web UI Testing
```bash
# Start server
cargo run -- --server

# Open browser to http://localhost:3000
# Fill form and play game via UI
```

### CLI Testing
```bash
# Interactive mode
cargo run

# With parameters
cargo run -- --min 1 --max 100 --limit 10
```

## Future Architecture Considerations

### Implemented Features
1. ✅ **Database Integration**: PostgreSQL with persistent storage and migrations

### Potential Improvements
1. **Session Management**: User authentication and authorization
2. **WebSocket Support**: Real-time updates and multiplayer features
3. **Microservices**: Separate game engine service
4. **Caching Layer**: Redis for frequently accessed game state
5. **Load Balancing**: Multiple instance support with connection pooling
6. **API Versioning**: `/api/v1/games` structure
7. **OpenAPI Spec**: Auto-generated API documentation
8. **GraphQL**: Alternative to REST API
9. **Read Replicas**: Database scaling for high read loads

### Scaling Strategy
```
Current: Single/Multiple Processes → PostgreSQL Database
Phase 1: Multiple Processes → PostgreSQL + Redis Cache
Phase 2: Load Balanced Instances → Shared PostgreSQL + Redis
Phase 3: Microservices → Game Service + API Gateway + Message Queue
```

## Module Dependencies

```
main.rs
  ├── cli.rs (use)
  ├── db.rs (use via lib.rs)
  ├── game.rs (use via lib.rs)
  └── web.rs (use via lib.rs)

lib.rs
  ├── game.rs (pub mod)
  ├── game_id.rs (pub mod)
  ├── cli.rs (pub mod)
  ├── validators.rs (pub mod)
  ├── io.rs (pub mod)
  ├── templates.rs (pub mod)
  ├── db.rs (pub mod)
  └── web.rs (pub mod)

web.rs
  ├── db.rs (use)
  ├── game.rs (use)
  ├── game_id.rs (use)
  ├── validators.rs (use)
  └── templates.rs (use)

db.rs
  ├── game.rs (use)
  └── game_id.rs (use)

cli.rs
  └── (no internal deps)

validators.rs
  └── (no internal deps)

io.rs
  └── (no internal deps)

game.rs
  └── (no internal deps)

game_id.rs
  └── (no internal deps)

templates.rs
  └── game_id.rs (use)
```

## Technology Stack Rationale

### Why Rust?
- Memory safety without garbage collection
- Performance for game logic
- Strong type system for validation
- Single binary deployment

### Why Axum?
- Modern async design
- Tower middleware ecosystem
- Type-safe routing
- Good performance

### Why HTMX?
- Progressive enhancement
- No build step required
- Server-side rendering
- Lightweight (12KB)

### Why Clap?
- Derive macro simplicity
- Automatic help generation
- Type-safe parsing
- Well-maintained