# PostgreSQL Persistence Implementation Plan

**Date:** 2025-09-30
**Library Choice:** SQLx with compile-time checked queries
**Goal:** Replace in-memory HashMap storage with PostgreSQL database

## Overview

Replace the current `Arc<Mutex<HashMap<u64, GuessingGame>>>` in-memory storage with PostgreSQL persistence using SQLx. This will enable:
- Persistent game state across server restarts
- Proper production-ready architecture
- Compile-time SQL verification for safety

## Library Selection Rationale

**Chosen: SQLx**
- Compile-time checked queries prevent runtime SQL errors
- Built-in connection pooling
- Async-first design (perfect for tokio/axum stack)
- SQL-centric (no ORM DSL to learn)
- Migration support built-in

**Alternatives Considered:**
1. tokio-postgres + deadpool-postgres (lower-level, more control)
2. tokio-postgres standalone (minimal dependencies, no pooling)

## Architecture Changes

### Current State
```rust
// src/web.rs
type SharedState = Arc<Mutex<GameState>>;

struct GameState {
    games: HashMap<u64, GuessingGame>,
}
```

### New State
```rust
// src/web.rs
type SharedState = PgPool;  // SQLx connection pool

// src/db.rs
pub struct GameRepository {
    pool: PgPool,
}
```

## Implementation Phases

### Phase 1: Dependencies & Configuration

**Files to modify:**
- `Cargo.toml`
- `docker-compose.yml` (create)
- `.env` (create, add to .gitignore)
- `.gitignore`

**Dependencies:**
```toml
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio", "tls-rustls", "macros", "migrate"] }
dotenvy = "0.15"
```

**docker-compose.yml:**
```yaml
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: numberguess
      POSTGRES_PASSWORD: password
      POSTGRES_DB: numberguess_dev
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

**.env:**
```
DATABASE_URL=postgresql://numberguess:password@localhost:5432/numberguess_dev
```

### Phase 2: Database Schema

**Create migrations/ directory**

**Migration 1: Create games table**
File: `migrations/20250930000001_create_games_table.sql`
```sql
CREATE TABLE games (
    game_id BIGINT PRIMARY KEY,
    min_value INTEGER NOT NULL,
    max_value INTEGER NOT NULL,
    secret_number INTEGER NOT NULL,
    guess_count INTEGER NOT NULL DEFAULT 0,
    max_guesses INTEGER NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_games_created_at ON games(created_at);
```

**Migration 2: Add cleanup function**
File: `migrations/20250930000002_add_cleanup_function.sql`
```sql
-- Function to clean up old games (optional, for future use)
CREATE OR REPLACE FUNCTION cleanup_old_games(hours_old INTEGER DEFAULT 24)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM games
    WHERE created_at < NOW() - (hours_old || ' hours')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
```

### Phase 3: Database Layer

**Create new file: `src/db.rs`**

**Key components:**
1. Connection pool initialization
2. GameRepository struct
3. CRUD operations with compile-time checked queries:
   - `create_game(min, max, max_guesses) -> Result<u64, DbError>`
   - `get_game(game_id) -> Result<GuessingGame, DbError>`
   - `update_game(game_id, guess_count) -> Result<(), DbError>`
   - `delete_game(game_id) -> Result<(), DbError>`

**Type Mapping:**
- Database row → GuessingGame struct
- Need to reconstruct GuessingGame from persisted state
- secret_number is stored in DB (not regenerated)

**Error Handling:**
```rust
#[derive(Debug)]
pub enum DbError {
    NotFound,
    DatabaseError(sqlx::Error),
    ConversionError(String),
}
```

### Phase 4: Update Web Layer

**Modify: `src/web.rs`**

**Changes:**
1. Replace `type SharedState = Arc<Mutex<GameState>>` with `type SharedState = PgPool`
2. Update `run_server()` signature to accept `PgPool`
3. Modify handlers:
   - `create_game_api`: Use db operations instead of HashMap insert
   - `make_guess_api`: Use db operations instead of HashMap get_mut
   - Remove game deletion logic (games persist now)
4. Error handling updates for database errors

**Key differences:**
- No more `state.lock().unwrap()` - pool handles concurrency
- Async db operations
- Proper error propagation

### Phase 5: Application Initialization

**Modify: `src/main.rs`**

**Changes:**
1. Load environment variables with `dotenvy`
2. Create database pool on startup
3. Run migrations automatically
4. Pass pool to `run_server()`
5. Add CLI argument for database URL override

**Initialization sequence:**
```rust
// 1. Load .env
dotenvy::dotenv().ok();

// 2. Get DATABASE_URL
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

// 3. Create pool
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;

// 4. Run migrations
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;

// 5. Start server
run_server(pool, port).await;
```

### Phase 6: Testing Infrastructure

**Modify: `tests/common/containers.rs`**

**Add TestDb struct:**
```rust
use testcontainers::images::postgres::Postgres;

pub struct TestDb {
    _container: Container<'static, Postgres>,
    pub pool: PgPool,
    pub database_url: String,
}

impl TestDb {
    pub async fn new() -> Self {
        let container = testcontainers::clients::Cli::default()
            .run(Postgres::default());

        let port = container.get_host_port_ipv4(5432);
        let database_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            port
        );

        let pool = PgPool::connect(&database_url).await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .unwrap();

        Self {
            _container: container,
            pool,
            database_url,
        }
    }
}
```

**Update existing tests:**
- Replace in-memory state setup with TestDb::new()
- Test database persistence
- Test concurrent access
- Test game lifecycle with persistence

**New tests to add:**
- Database connection handling
- Migration execution
- Data persistence across requests
- Concurrent game creation/updates

### Phase 7: Offline Build Support

**Steps:**
1. Start dev database: `docker-compose up -d postgres`
2. Build with compile-time checks: `cargo build`
3. Generate offline metadata: `cargo sqlx prepare`
4. Verify sqlx-data.json created
5. Commit sqlx-data.json to git
6. Test offline build: `SQLX_OFFLINE=true cargo build`

**CI/CD Integration:**
```yaml
# .github/workflows/ci.yml
- name: Build
  run: cargo build --release
  env:
    SQLX_OFFLINE: true  # Uses committed sqlx-data.json

- name: Test
  run: cargo test
  # testcontainers handles PostgreSQL automatically
```

### Phase 8: Documentation

**Update: `README.md`**
- Add "Database Setup" section
- Document docker-compose usage
- Document environment variables
- Add migration instructions

**Update: `CLAUDE.md`**
- Update architecture section
- Document new database layer
- Update testing instructions
- Add migration workflow

**Update: `docs/architecture.md`**
- Document database schema
- Explain persistence layer
- Connection pooling details

**Create: `docs/database.md`**
- Schema documentation
- Migration guide
- Backup/restore procedures
- Troubleshooting

## Build Workflows

### Development Workflow
```bash
# One-time setup
docker-compose up -d
echo 'DATABASE_URL=postgresql://numberguess:password@localhost:5432/numberguess_dev' > .env

# Daily development
docker-compose up -d          # Ensure DB running
cargo build                   # Compile-time checks
cargo run -- --server         # Run with persistent storage

# When SQL queries change
cargo sqlx prepare            # Update offline data
git add sqlx-data.json        # Commit for CI
```

### Testing Workflow
```bash
# Tests handle everything automatically
cargo test

# testcontainers will:
# 1. Start PostgreSQL container
# 2. Run migrations
# 3. Execute tests
# 4. Clean up container
```

### CI/CD Workflow
```bash
# Build (no database needed)
SQLX_OFFLINE=true cargo build --release

# Test (testcontainers handles database)
cargo test
```

## Database Schema

### Games Table
```sql
CREATE TABLE games (
    game_id         BIGINT PRIMARY KEY,      -- Random u64 ID
    min_value       INTEGER NOT NULL,        -- Game range minimum
    max_value       INTEGER NOT NULL,        -- Game range maximum
    secret_number   INTEGER NOT NULL,        -- Target number
    guess_count     INTEGER NOT NULL,        -- Current attempts
    max_guesses     INTEGER NULL,            -- Optional limit (NULL = unlimited)
    created_at      TIMESTAMP NOT NULL,      -- Game creation time
    updated_at      TIMESTAMP NOT NULL       -- Last guess time
);
```

### Indexes
- PRIMARY KEY on game_id (automatic)
- INDEX on created_at (for cleanup queries)

## Migration Strategy

### Current State → PostgreSQL
1. **No data migration needed** (in-memory state is ephemeral)
2. **API remains unchanged** (game_id, requests/responses identical)
3. **Deployment:** Start new version with PostgreSQL, old games lost (acceptable)

### Rollback Plan
If issues arise:
1. Keep old code in git branch
2. Revert to previous version
3. In-memory state restores immediately

## Security Considerations

### Database Credentials
- ✅ Use .env file (not committed)
- ✅ Environment variables in production
- ✅ Connection pooling limits concurrent connections
- ⚠️ TODO: Add connection timeout configuration
- ⚠️ TODO: Add retry logic for transient failures

### SQL Injection
- ✅ SQLx parameterized queries prevent injection
- ✅ Compile-time checking ensures query validity
- ✅ Type safety prevents data type attacks

### Data Exposure
- ⚠️ secret_number stored in plaintext (acceptable for game)
- ⚠️ TODO: Consider adding game expiration
- ⚠️ TODO: Consider rate limiting on game creation

## Performance Considerations

### Connection Pooling
- Default: 5 max connections
- Each request borrows from pool
- Returns to pool after handler completes
- Prevents connection exhaustion

### Query Performance
- game_id is PRIMARY KEY (indexed automatically)
- SELECT by game_id is O(1) lookup
- created_at indexed for cleanup queries
- No complex JOINs needed

### Expected Load
- Small payload per game (~40 bytes)
- Simple CRUD operations
- No complex transactions
- Suitable for thousands of concurrent games

## Testing Strategy

### Unit Tests
- Database layer (src/db.rs)
- Type conversions
- Error handling

### Integration Tests
- Full game lifecycle with persistence
- Concurrent game operations
- Migration execution
- Connection pool behavior

### Test Isolation
- Each test gets fresh PostgreSQL container
- Migrations run per container
- No test pollution
- Parallel test execution safe

## Rollout Checklist

### Pre-Implementation
- [x] Review plan
- [ ] Confirm library choice (SQLx)
- [ ] Verify docker-compose setup
- [ ] Review schema design

### Implementation
- [ ] Phase 1: Dependencies
- [ ] Phase 2: Migrations
- [ ] Phase 3: Database layer
- [ ] Phase 4: Web layer updates
- [ ] Phase 5: App initialization
- [ ] Phase 6: Testing
- [ ] Phase 7: Offline support
- [ ] Phase 8: Documentation

### Validation
- [ ] All existing tests pass
- [ ] New database tests pass
- [ ] Manual testing with PostgreSQL
- [ ] Offline build works
- [ ] CI/CD pipeline works

### Deployment
- [ ] Update deployment docs
- [ ] Configure production DATABASE_URL
- [ ] Set up production PostgreSQL
- [ ] Deploy new version
- [ ] Monitor for errors

## Known Limitations

1. **No automatic game cleanup** - Games persist forever (mitigation: add cleanup job later)
2. **No distributed locking** - Single server only (acceptable for current scale)
3. **No replication/backup** - Production needs backup strategy
4. **Compile-time checks need DB** - Or use offline mode with sqlx-data.json

## Future Enhancements

1. **Game expiration** - Auto-delete games older than X hours
2. **Game statistics** - Track total games, average guesses, etc.
3. **User accounts** - Associate games with users
4. **Leaderboards** - Best scores across all players
5. **Game history** - Store completed game results

## References

- SQLx documentation: https://docs.rs/sqlx/latest/sqlx/
- testcontainers-rs: https://github.com/testcontainers/testcontainers-rs
- PostgreSQL docs: https://www.postgresql.org/docs/
- Axum examples: https://github.com/tokio-rs/axum/tree/main/examples

---

**Plan Author:** Claude Code
**Plan Status:** Ready for Implementation
**Estimated Effort:** 4-6 hours for full implementation and testing
