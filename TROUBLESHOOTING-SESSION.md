# Troubleshooting Session - 2025-12-19

## Problem
After adding CSRF protection, attempted to remove emojis from log/test messages, which broke integration tests.

## Root Causes Found

### 1. Borrow Checker Error in `tests/csrf_test.rs` (FIXED)
- Line 65-75: `resp.headers()` borrowed `resp`, then `resp.text().await` tried to consume it
- Fix: Clone the cookie string before consuming the response

### 2. Timing Issue in `tests/common/page_objects.rs` (FIXED)
- `submit_game_setup()` used a fixed 500ms sleep, insufficient for slow DB operations
- Fix: Wait for `.guess-form` element to appear (up to 30s) instead of fixed sleep

### 3. CSRF Token Not Being Verified (FIXED)
- `axum_csrf` doesn't auto-reject invalid tokens - you must call `token.verify()` explicitly
- Handlers were extracting `CsrfToken` but not verifying it
- Fix: Added `authenticity_token` field to request structs and verification in handlers

## Files Modified

1. **tests/csrf_test.rs** - Fixed borrow checker error (lines 65-75)

2. **tests/common/page_objects.rs** - Changed `submit_game_setup()` to wait for element

3. **src/web/types.rs** - Added `authenticity_token: String` field to:
   - `CreateGameRequest`
   - `MakeGuessRequest`

4. **src/web/handlers/game.rs** - Added CSRF verification at start of `create_game_web`:
   ```rust
   if token.verify(&payload.authenticity_token).is_err() {
       warn!("Web: CSRF token verification failed");
       return (StatusCode::BAD_REQUEST, "Invalid CSRF token").into_response();
   }
   ```

5. **src/web/handlers/guess.rs** - Added same CSRF verification to `make_guess_web`

## Current State
- Code compiles successfully (`cargo build` passes)
- Unit tests should pass (`make test-unit`)
- Integration tests not fully verified due to resource constraints on laptop
- Docker image needs rebuild (`make build`) before integration tests

## Next Steps
1. Run `make test-down` to clean up any running containers
2. Run `make build` to rebuild Docker image with fixes
3. Run `make test-integration` to verify all tests pass
4. If CSRF tests still fail, check that templates include the hidden field (they do - already verified)

## Commands to Resume
```bash
# Clean up
make test-down

# Verify unit tests
make test-unit

# Rebuild and run integration tests
make build
make test-integration
```
