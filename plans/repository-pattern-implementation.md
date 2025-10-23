# Repository Pattern Implementation Plan

## Overview
Implement a repository trait abstraction using **native async traits** with **static dispatch**. This improves code organization and creates clear abstraction boundaries.

---

## Phase 1: Create Repository Trait and PostgreSQL Implementation

### 1.1 Create Repository Trait (`src/db/repository.rs`)
Define `GameRepository` trait with native async methods:
- `async fn create(&self, min: i32, max: i32, max_guesses: Option<u32>) -> Result<GameId, DbError>`
- `async fn get(&self, game_id: GameId) -> Result<GuessingGame, DbError>`
- `async fn make_guess(&self, game_id: GameId, guess: i32) -> Result<GuessResult, DbError>`
- `async fn delete(&self, game_id: GameId) -> Result<(), DbError>`
- `async fn update(&self, game_id: GameId, game: &GuessingGame) -> Result<(), DbError>`
- `async fn health_check(&self) -> Result<(), DbError>`

Trait bounds: `Send + Sync + Clone`

### 1.2 Create PostgreSQL Repository (`src/db/postgres_repository.rs`)
- Define `PostgresGameRepository` struct with `pool: PgPool` field
- Derive `Clone` (PgPool already has Arc internally)
- Implement `GameRepository` trait
- Move existing logic from `src/db/mod.rs` into trait methods
- Keep transaction logic in `make_guess` method

### 1.3 Update `src/db/mod.rs`
- Add module declarations: `pub mod repository;` and `pub mod postgres_repository;`
- Re-export trait and PostgreSQL implementation
- Keep `DbError` type
- Mark existing standalone functions as `#[deprecated]` for backward compatibility

---

## Phase 2: Create AppState and Update Server

### 2.1 Create AppState Struct (`src/server/state.rs`)
```rust
#[derive(Clone)]
pub struct AppState<R: GameRepository> {
    pub repo: R,
}
```

### 2.2 Update Server Module (`src/server/mod.rs`)
- Add `pub mod state;` module declaration
- Import: `AppState`, `PostgresGameRepository`, `GameRepository`
- Create: `let state = AppState { repo: PostgresGameRepository::new(pool) };`
- Update router to use `AppState<PostgresGameRepository>` as state
- Add turbofish syntax to route handlers: `post(create_game_api::<PostgresGameRepository>)`

---

## Phase 3: Update API Handlers

### 3.1 Update `src/api/handlers/game.rs`
- Add generic: `pub async fn create_game_api<R: GameRepository>(...)`
- Change state: `State(state): State<AppState<R>>`
- Replace: `db::create_game(&pool, ...)` → `state.repo.create(...).await`
- Remove: `type SharedState = PgPool;`

### 3.2 Update `src/api/handlers/guess.rs`
- Add generic: `pub async fn make_guess_api<R: GameRepository>(...)`
- Change state: `State(state): State<AppState<R>>`
- Replace: `db::make_guess_transactional(&pool, ...)` → `state.repo.make_guess(...).await`
- Remove: `type SharedState = PgPool;`

### 3.3 Update `src/api/handlers/health.rs`
- Add generic: `pub async fn health_check<R: GameRepository>(...)`
- Change state: `State(state): State<AppState<R>>`
- Replace: direct PgPool query → `state.repo.health_check().await`

---

## Phase 4: Update Web Handlers

### 4.1 Update `src/web/handlers/game.rs`
- Add generic: `pub async fn create_game_web<R: GameRepository>(...)`
- Change state: `State(state): State<AppState<R>>`
- Replace: `db::create_game(&pool, ...)` → `state.repo.create(...).await`
- Remove: `type SharedState = PgPool;`

### 4.2 Update `src/web/handlers/guess.rs`
- Add generic: `pub async fn make_guess_web<R: GameRepository>(...)`
- Change state: `State(state): State<AppState<R>>`
- Replace: `db::make_guess_transactional(&pool, ...)` → `state.repo.make_guess(...).await`
- Replace: `db::get_game(&pool, ...)` → `state.repo.get(...).await`
- Remove: `type SharedState = PgPool;`

---

## Phase 5: Code Quality

### 5.1 Remove `#![allow(warnings)]` from `src/main.rs`
- Delete line 1: `#![allow(warnings)]`
- Fix any warnings that appear (unused imports, unused variables, etc.)

### 5.2 Update Documentation (`CLAUDE.md`)
- Add "Repository Pattern" section under Architecture
- Document the trait abstraction and AppState pattern
- Update module organization section

---

## Phase 6: Validation

### 6.1 Format, Build, and Lint
```bash
make fmt      # Format all code
make build    # Ensure compilation succeeds
make lint     # Check for clippy issues, fix them
```

### 6.2 Run Full Test Suite
```bash
make test     # Runs both unit and integration tests
```
Confirm no regressions in functionality.

---

## File Changes Summary

**New Files (3):**
- `src/db/repository.rs` - Trait definition
- `src/db/postgres_repository.rs` - PostgreSQL implementation
- `src/server/state.rs` - AppState struct

**Modified Files (9):**
- `src/db/mod.rs` - Module declarations, re-exports, deprecations
- `src/server/mod.rs` - Use AppState and repository
- `src/api/handlers/game.rs` - Generic handler with repository
- `src/api/handlers/guess.rs` - Generic handler with repository
- `src/api/handlers/health.rs` - Generic handler with repository
- `src/web/handlers/game.rs` - Generic handler with repository
- `src/web/handlers/guess.rs` - Generic handler with repository
- `src/main.rs` - Remove warnings suppression, fix warnings
- `CLAUDE.md` - Document pattern

**Unchanged:**
- `src/core/*` - Business logic unchanged
- `src/cli/*` - CLI mode unchanged
- `tests/*` - Tests work as-is
- `Cargo.toml` - No new dependencies needed

---

## Key Design Decisions

1. **Native async traits** (Rust 1.75+) - No `async-trait` crate needed
2. **Static dispatch** - Generic `<R: GameRepository>` for zero overhead
3. **Clone repositories** - No Arc wrapper (PgPool has Arc internally)
4. **AppState struct** - Explicit state management, extensible for future needs
5. **Deprecated functions** - Old `db::` functions marked deprecated for smooth transition

---

## Benefits

1. ✅ Clear abstraction boundary between handlers and database
2. ✅ Better code organization with explicit repository layer
3. ✅ Zero runtime overhead (native async + static dispatch)
4. ✅ No new dependencies required
5. ✅ Cleaner code without `#![allow(warnings)]`
6. ✅ Foundation for future enhancements (caching, metrics, alternative storage)

---

## Estimated Time: ~60-75 minutes

---

## Implementation Notes

- This plan uses **native async traits** available since Rust 1.75 (project uses 1.89)
- **Static dispatch** via generics provides zero-cost abstraction
- **PgPool already contains Arc** internally, so no need to wrap repository in Arc
- All handlers become generic over `R: GameRepository` but use static dispatch at compile time
- Existing `db::` module functions will be marked deprecated but not removed for compatibility
