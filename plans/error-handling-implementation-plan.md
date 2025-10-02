# Error Handling Implementation Plan

*Generated: 2025-10-01*

## Objective
Address issues 1-3 from code-improvement-suggestions.md by implementing consistent, type-safe error handling using `thiserror`.

## Library Recommendation: thiserror

**Why thiserror over anyhow?**

After researching current Rust error handling best practices (2025), I recommend **thiserror** for this project:

1. **Hybrid architecture** - The codebase is both a library and application
2. **Typed errors required** - Web handlers need specific error types to generate appropriate HTTP responses
3. **Better API design** - Library exports benefit from structured error types
4. **No context propagation needed** - Unlike large applications, our errors are local and specific

**Industry consensus (2025):**
- Use `thiserror` for libraries with typed, structured errors
- Use `anyhow` for pure applications where callers don't care about error details
- This project sits in the middle, leaning towards library design

## Issues to Address

### Issue #1: Replace String errors with proper error types
**Location:** `src/game.rs:15`, multiple functions

**Current:** `Result<Self, String>` with formatted error messages
**Target:** `Result<Self, GameError>` with typed enum variants

### Issue #2: DbError could use thiserror
**Location:** `src/db.rs:6-22`

**Current:** Manual `Display` and `Error` trait implementations
**Target:** Derive-based implementation with automatic conversions

### Issue #3: Inconsistent error handling in web handlers
**Location:** `src/web.rs:434, 480, 526, 542`

**Current:** Silent failures with `let _ =`
**Target:** Consistent error logging even when ignoring results

## Implementation Plan

### Phase 1: Add thiserror Dependency
**Files:** `Cargo.toml`

```toml
[dependencies]
thiserror = "2.0"
```

**Verification:** `cargo build`

---

### Phase 2: Create GameError Type
**Files:** `src/game.rs`

**Steps:**
1. Add `use thiserror::Error;` at top of file
2. Create comprehensive `GameError` enum:

```rust
#[derive(Error, Debug)]
pub enum GameError {
    #[error("Minimum value ({0}) must be non-negative (>= 0)")]
    NegativeMin(i32),

    #[error("Maximum value ({0}) must be non-negative (>= 0)")]
    NegativeMax(i32),

    #[error("Maximum ({max}) must be greater than or equal to minimum ({min})")]
    InvalidRange { min: i32, max: i32 },

    #[error("Minimum value ({value}) exceeds maximum allowed value ({limit})")]
    MinExceedsLimit { value: i32, limit: i32 },

    #[error("Maximum value ({value}) exceeds maximum allowed value ({limit})")]
    MaxExceedsLimit { value: i32, limit: i32 },

    #[error("Range between min ({min}) and max ({max}) is too large")]
    RangeTooLarge { min: i32, max: i32 },

    #[error("Secret number ({secret}) must be between min ({min}) and max ({max})")]
    SecretOutOfRange { secret: i32, min: i32, max: i32 },
}
```

3. Extract validation logic to private helper (addresses suggestion #4):

```rust
fn validate_range(min: i32, max: i32) -> Result<(), GameError> {
    if min < 0 {
        return Err(GameError::NegativeMin(min));
    }
    if max < 0 {
        return Err(GameError::NegativeMax(max));
    }
    if max < min {
        return Err(GameError::InvalidRange { min, max });
    }
    if min > MAX_ALLOWED {
        return Err(GameError::MinExceedsLimit { value: min, limit: MAX_ALLOWED });
    }
    if max > MAX_ALLOWED {
        return Err(GameError::MaxExceedsLimit { value: max, limit: MAX_ALLOWED });
    }
    if max.saturating_sub(min) == i32::MAX {
        return Err(GameError::RangeTooLarge { min, max });
    }
    Ok(())
}
```

4. Update function signatures:
   - `new() -> Result<Self, GameError>`
   - `new_with_limit() -> Result<Self, GameError>`
   - `from_db() -> Result<Self, GameError>`

5. Replace all validation code with `validate_range()` calls
6. Add secret number validation in `from_db()`
7. Update all tests to work with `GameError`

**Expected changes:**
- ~80 lines modified (validation logic, function signatures, error returns)
- ~15 test updates (error message assertions)

---

### Phase 3: Modernize DbError
**Files:** `src/db.rs`

**Steps:**
1. Add `use thiserror::Error;` at top
2. Replace entire `DbError` implementation:

```rust
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Game not found")]
    NotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Game validation error: {0}")]
    GameError(#[from] crate::game::GameError),
}
```

3. Remove manual `impl Display` and `impl std::error::Error`
4. Simplify `impl From<sqlx::Error>` (thiserror handles most of it):

```rust
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound,
            e => DbError::DatabaseError(e),
        }
    }
}
```

5. Update error conversions:
   - `create_game()`: `.map_err(|e| DbError::GameError(e))?` → automatic with `#[from]`
   - `get_game()`: `.map_err(|e| DbError::GameError(e))` → automatic

**Expected changes:**
- ~30 lines deleted (manual implementations)
- ~10 lines added (thiserror derives)
- Net: -20 lines, cleaner code

---

### Phase 4: Consistent Error Logging in Web Handlers
**Files:** `src/web.rs`

**Steps:**
1. Find all instances of `let _ = db::delete_game(...)`
2. Replace with proper logging:

```rust
if let Err(e) = db::delete_game(&pool, game_id).await {
    eprintln!("Failed to delete completed game {}: {}", game_id, e);
}
```

3. Find all instances of `if let Err(_) = db::update_game(...)`
4. Replace with named error and logging:

```rust
if let Err(e) = db::update_game(&pool, game_id, &game).await {
    eprintln!("Failed to update game {}: {}", game_id, e);
    return Html(/* error response */).into_response();
}
```

**Locations to update:**
- Line 434: `update_game` in TooLow handler
- Line 480: `update_game` in TooHigh handler
- Line 526: `delete_game` in Correct handler
- Line 542: `delete_game` in LimitReached handler

**Expected changes:**
- 4 error handling sites improved
- ~10 lines added for logging

---

### Phase 5: Update CLI Error Handling
**Files:** `src/cli.rs`

**Steps:**
1. Read current error handling patterns
2. Update any `String` error handling to use `GameError`
3. Ensure error messages remain user-friendly
4. Update error display in CLI output

**Expected changes:**
- Minimal, as CLI mostly uses library functions
- May need to update error message formatting

---

### Phase 6: Update Library Exports
**Files:** `src/lib.rs`

**Steps:**
1. Export `GameError` for library users:

```rust
pub use game::{GuessingGame, GuessResult, GameError};
```

2. Consider exporting `DbError` if needed by external users

**Expected changes:**
- 1-2 lines modified

---

### Phase 7: Testing & Verification

**Test plan:**
1. **Unit tests:** `cargo test --lib`
   - Verify all game.rs tests pass
   - Check error type assertions

2. **Build verification:** `cargo build`
   - Ensure no compilation errors
   - Check for unused imports/code

3. **Integration tests:** `cargo test`
   - Run all integration tests
   - Verify database error handling

4. **Full test suite:** `./run_integration_tests.sh`
   - Test with Docker containers
   - Verify web API error responses

5. **Manual testing:**
   - CLI: `cargo run -- --min 1 --max 100`
     - Test invalid inputs (negative, out of range)
   - Web: `cargo run -- --server`
     - Test API error responses
     - Verify error logging appears in console

---

## Success Criteria

✅ All code compiles without warnings
✅ All tests pass (unit + integration)
✅ No more `String` error types in game logic
✅ `DbError` uses thiserror derives
✅ All web handler errors are logged consistently
✅ Error messages remain clear and user-friendly
✅ No regression in functionality

## Risk Assessment

**Low risk changes:**
- Adding thiserror dependency (well-established crate)
- Creating GameError type (additive change)
- Extracting validation helper (refactor, same logic)

**Medium risk changes:**
- Updating DbError (changes error conversion flow)
- Updating web handlers (could affect error responses)

**Mitigation:**
- Comprehensive testing at each phase
- Keep error messages identical where possible
- Run integration tests frequently

## Estimated Effort

- **Phase 1:** 5 minutes
- **Phase 2:** 30-45 minutes (most complex)
- **Phase 3:** 15-20 minutes
- **Phase 4:** 15 minutes
- **Phase 5:** 10 minutes
- **Phase 6:** 5 minutes
- **Phase 7:** 20-30 minutes (testing)

**Total:** ~2 hours

## Additional Benefits

Beyond addressing issues 1-3, this implementation also:

1. **Addresses suggestion #4** - Extracts duplicate validation logic
2. **Improves maintainability** - Easy to add new error variants
3. **Better debugging** - Structured errors with full context
4. **Type safety** - Compiler catches missing error cases
5. **Production readiness** - Professional error handling throughout

## References

- [thiserror documentation](https://docs.rs/thiserror)
- Rust Error Handling Guide 2025
- Code improvement suggestions: items 1-4
