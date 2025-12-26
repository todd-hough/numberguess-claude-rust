# CSRF Troubleshooting Session - 2025-12-25

## Bugs Fixed

### 1. CSRF Cookie Not Being Set (CRITICAL)
Handlers extracted `CsrfToken` but didn't return it in response tuple.
**Fix**: Return `(token, template)` in all handlers.

### 2. CSRF Cookie Name Mismatch
Test expected `"x-csrf-token"`, library default was `"Csrf_Token"`.
**Fix**: Added `.with_cookie_name("x-csrf-token")` in server config.

### 3. Test Client Missing Cookie Store
Client didn't persist cookies between requests.
**Fix**: Added `cookies` feature to reqwest, used `cookie_provider(jar)`.

### 4. Silent Timeout Bug
`wait_for_oauth2_redirect()` returned `Ok(())` on timeout.
**Fix**: Return error with URL on timeout, increased timeouts to 30s.

### 5. Quote Mismatch in Test
Test used single quotes, template uses double quotes.
**Fix**: Updated test to use double quotes.

### 6. Tests Missing CSRF Tokens
Several integration tests were POSTing without including `authenticity_token`.
**Fix**: Updated tests to GET index page first, extract token, include in POSTs.

### 7. web_ui_test Invalid Input Test
`submit_game_setup()` waits for `.guess-form` which doesn't appear on error.
**Fix**: Use direct submit click + wait_for_feedback() for invalid input tests.

## Files Modified
- `src/server/mod.rs` - cookie name config
- `src/web/handlers/game.rs` - return token in tuple
- `src/web/handlers/guess.rs` - return token in tuple
- `tests/common/auth_helpers.rs` - cookie jar, timeouts, error messages
- `tests/csrf_test.rs` - renamed test, quote fix
- `tests/web_endpoints_test.rs` - added CSRF tokens to all 3 POST tests
- `tests/web_ui_test.rs` - fixed invalid input test to not expect success
- `Cargo.toml` - added `cookies` feature

## Test Status (as of last run)

### Passing Tests
- ✅ api_edge_cases_test (5 tests)
- ✅ auth_integration_test (5 tests)
- ✅ cli_test (6 tests)
- ✅ concurrency_test (3 tests)
- ✅ csrf_test (2 tests)
- ✅ integration_test (2 ignored - superseded)
- ✅ web_endpoints_test (4 tests)

### Needs Verification
- ⚠️ web_ui_test (2 tests) - Selenium connection failed on last run (resource issue)

## Next Steps
1. Run `make test-integration` to verify web_ui_test passes
2. If all pass, clean up this troubleshooting doc
3. Commit CSRF changes to csrf-update branch
