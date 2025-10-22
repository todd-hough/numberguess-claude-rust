# Plan: Fix Async Test Patterns in Integration Tests

## Problem Statement

7 integration tests are using the deprecated `tokio_test::block_on()` pattern with `#[test]` instead of proper async patterns with `#[tokio::test]`. This causes runtime conflicts and test failures as documented in CLAUDE.md.

**Affected Tests:**
- `tests/web_ui_test.rs` (2 tests)
  - `test_web_ui_game_flow` - **Currently Failing**
  - `test_web_ui_invalid_inputs`
- `tests/auth_integration_test.rs` (5 tests)
  - `test_oauth2_login_flow`
  - `test_unauthenticated_web_ui_redirects_to_login`
  - `test_unauthenticated_api_returns_401`
  - `test_web_ui_endpoints_work_when_authenticated`
  - `test_api_endpoints_work_when_authenticated`

## Root Cause

Current pattern (problematic):
```rust
#[test]
fn test_something() {
    let result = tokio_test::block_on(async move {
        // async test code
    });
    assert!(result);
}
```

**Why this fails:**
1. `#[test]` creates no async runtime
2. `tokio_test::block_on()` tries to create a nested runtime
3. Causes runtime conflicts, panics, or deadlocks
4. Application uses tokio → all tests must be tokio-compatible

## Solution Pattern

Convert to proper async pattern:
```rust
#[tokio::test]
async fn test_something() {
    // Environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required");
    })
    .await
    .expect("Environment checks failed");

    // Direct async code with .await
    let result = do_async_work().await;
    assert!(result);
}
```

## Implementation Steps

### Step 1: Fix `tests/web_ui_test.rs` (2 tests)

#### 1.1 Convert `test_web_ui_game_flow`

**Before:**
```rust
#[test]
fn test_web_ui_game_flow() {
    let base_url = environment::ensure_server_ready();
    let browser_url = environment::browser_base_url();
    let selenium_url = match environment::ensure_selenium_ready() {
        Some(url) => url,
        None => { /* skip */ }
    };

    let result = tokio_test::block_on(async move {
        let driver = create_webdriver(&selenium_url).await?;
        let page = GamePage::new(&driver);
        page.goto(browser_url.as_str()).await?;
        // ... test logic
        true
    });
    assert!(result);
}
```

**After:**
```rust
#[tokio::test]
async fn test_web_ui_game_flow() {
    // Environment checks in blocking context
    let (base_url, browser_url, selenium_url) = tokio::task::spawn_blocking(|| {
        let base_url = environment::ensure_server_ready();
        let browser_url = environment::browser_base_url();
        let selenium_url = match environment::ensure_selenium_ready() {
            Some(url) => url,
            None => panic!("Selenium required for this test"),
        };
        (base_url, browser_url, selenium_url)
    })
    .await
    .expect("Environment checks failed");

    // Direct async code
    let driver = create_webdriver(&selenium_url)
        .await
        .expect("Failed to create WebDriver");

    let page = GamePage::new(&driver);
    page.goto(browser_url.as_str())
        .await
        .expect("Failed to navigate");

    // ... rest of test logic with direct .await calls
}
```

**Changes:**
- Replace `#[test]` → `#[tokio::test]`
- Add `async` to function signature
- Move environment checks into `spawn_blocking`
- Remove `tokio_test::block_on` wrapper
- Use direct `.await` calls
- Convert early returns to proper error handling

#### 1.2 Convert `test_web_ui_invalid_inputs`

Same pattern as 1.1.

### Step 2: Fix `tests/auth_integration_test.rs` (5 tests)

#### 2.1 Convert `test_oauth2_login_flow`

**Key Pattern Changes:**
```rust
// Before
#[test]
fn test_oauth2_login_flow() {
    let result = tokio_test::block_on(async move {
        let client = create_authenticated_client_selenium().await?;
        // ... test logic
        true
    });
    assert!(result);
}

// After
#[tokio::test]
async fn test_oauth2_login_flow() {
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready().expect("Selenium required");
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client_selenium()
        .await
        .expect("Failed to create authenticated client");

    // ... test logic with direct .await
}
```

#### 2.2 Convert `test_unauthenticated_web_ui_redirects_to_login`

Same pattern - note this test may have WebDriver code that needs proper await handling.

#### 2.3 Convert `test_unauthenticated_api_returns_401`

This one uses a simpler pattern with HTTP client:
```rust
// Before
#[test]
fn test_unauthenticated_api_returns_401() {
    let response = tokio_test::block_on(async {
        let client = reqwest::Client::new();
        client.get(url).send().await
    }).unwrap();
}

// After
#[tokio::test]
async fn test_unauthenticated_api_returns_401() {
    let base_url = tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready()
    })
    .await
    .expect("Environment check failed");

    let client = reqwest::Client::new();
    let url = format!("{}/api/games", base_url);
    let response = client.get(&url)
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 401);
}
```

#### 2.4 Convert `test_web_ui_endpoints_work_when_authenticated`

Same pattern as 2.1 - includes Selenium auth flow.

#### 2.5 Convert `test_api_endpoints_work_when_authenticated`

Same pattern as 2.1 - includes API client auth flow.

### Step 3: Update Dependencies (if needed)

Check `Cargo.toml` and remove `tokio-test` if it's no longer used:
```bash
grep -r "tokio_test" tests/
# If no results, remove from Cargo.toml
```

### Step 4: Verify Changes

1. **Compile check:**
   ```bash
   cargo test --no-run --tests
   ```

2. **Run individual tests:**
   ```bash
   make test-integration
   cargo test --test web_ui_test -- --nocapture --test-threads=1
   cargo test --test auth_integration_test -- --nocapture --test-threads=1
   ```

3. **Run full test suite:**
   ```bash
   make test
   ```

## Common Patterns to Watch For

### Pattern 1: Early Returns with `println!` + `return false`
```rust
// Before
if let Err(e) = page.goto(url).await {
    println!("Failed to navigate: {}", e);
    return false;
}

// After
page.goto(url)
    .await
    .expect("Failed to navigate");
// OR with better error messages:
page.goto(url)
    .await
    .unwrap_or_else(|e| panic!("Failed to navigate to {}: {}", url, e));
```

### Pattern 2: Environment Checks
```rust
// Always wrap in spawn_blocking:
tokio::task::spawn_blocking(|| {
    environment::ensure_server_ready();
    environment::ensure_selenium_ready().expect("Selenium required");
})
.await
.expect("Environment checks failed");
```

### Pattern 3: Optional Selenium Tests
```rust
// Before
let selenium_url = match environment::ensure_selenium_ready() {
    Some(url) => url,
    None => {
        println!("Skipping test - Selenium not available");
        return;
    }
};

// After - if test requires Selenium, don't make it optional:
let selenium_url = tokio::task::spawn_blocking(|| {
    environment::ensure_selenium_ready()
        .expect("Selenium required for this test. Run via 'make test-integration'")
})
.await
.expect("Environment check failed");
```

### Pattern 4: WebDriver Cleanup
```rust
// Before (in error paths)
let _ = page.quit().await;
return false;

// After
// Let WebDriver drop naturally, or use explicit cleanup:
drop(driver);
// OR for graceful shutdown:
page.quit().await.ok(); // Ignore cleanup errors
```

## Testing Strategy

1. **Fix one test at a time** - Start with `test_web_ui_game_flow` (currently failing)
2. **Run after each fix** - Verify the individual test works
3. **Use test-integration environment** - Keep it running for fast iteration:
   ```bash
   make test-integration  # Starts environment
   cargo test --test web_ui_test test_web_ui_game_flow -- --nocapture
   # Fix, re-run, repeat
   make test-down  # Clean up when done
   ```

## Success Criteria

- [ ] All 7 tests converted to `#[tokio::test]` + `async fn`
- [ ] No uses of `tokio_test::block_on` remain
- [ ] All tests pass with `make test-integration`
- [ ] No runtime conflicts or panics
- [ ] Environment checks properly wrapped in `spawn_blocking`
- [ ] Error handling is clear and descriptive

## Rollback Plan

If issues arise:
1. Git stash current changes
2. Review CLAUDE.md async patterns section
3. Test individual pattern changes in isolation
4. Re-apply with corrections

## Notes

- Environment check functions use blocking client, so they MUST be in `spawn_blocking`
- All other async code (WebDriver, reqwest) should use direct `.await`
- The tests already work in concept - just need proper async runtime setup
- This aligns with existing patterns in `web_endpoints_test.rs` and `concurrency_test.rs` which already use `#[tokio::test]`

## References

- CLAUDE.md: "Integration Test Architecture & Networking" section
- Existing correct examples: `tests/web_endpoints_test.rs`, `tests/concurrency_test.rs`
- Tokio docs: https://docs.rs/tokio/latest/tokio/attr.test.html
