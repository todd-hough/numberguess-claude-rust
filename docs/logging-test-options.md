# Logging Test Options - Comparison Guide

This document compares different approaches for testing logging in the Number Guessing Game project.

## Summary Comparison

| Approach | Use Case | Pros | Cons | Best For |
|----------|----------|------|------|----------|
| **Shell Script** | Manual testing | Simple, visual | Not automated, platform-dependent | Quick demos |
| **tracing-test** | Unit tests | Assert log content, isolated | Requires `--nocapture`, RAM usage | Testing specific log messages |
| **test-log** | Debugging tests | Auto initialization, minimal setup | Less assertion capability | Debugging during development |
| **Integration Tests** | End-to-end | Most realistic, full startup | Slower, requires database | CI/CD pipelines |

## Option 1: Shell Script (Current Approach)

**File**: `test_logging.sh`

### Pros
✅ Visual demonstration
✅ Easy to understand
✅ No additional dependencies
✅ Good for documentation/demos

### Cons
❌ Not automated in CI/CD
❌ Platform-dependent (bash)
❌ Manual verification
❌ Can't assert specific conditions

### Example
```bash
./test_logging.sh
```

**When to use**: Quick manual verification, demonstrating logging to users

---

## Option 2: `tracing-test` Crate

**Dependency**: `tracing-test = "0.2"`

### Pros
✅ Assert specific log messages
✅ Unit test integration
✅ Captures logs in memory
✅ Works with `cargo test`

### Cons
❌ Requires `--nocapture` flag
❌ Memory usage with verbose logs
❌ Integration tests need `no-env-filter` feature
❌ Focused on log content, not behavior

### Example
```rust
#[test]
#[tracing_test::traced_test]
fn test_database_connection_log() {
    info!(max_connections = 5, "Connecting to database");

    assert!(logs_contain("Connecting to database"));
    assert!(logs_contain("max_connections"));
}
```

**Usage**:
```bash
cargo test test_database_connection_log -- --nocapture
```

**When to use**: Testing that specific log messages are emitted in unit tests

---

## Option 3: `test-log` Crate

**Dependency**: `test-log = { version = "0.2", features = ["trace"] }`

### Pros
✅ Automatic tracing initialization
✅ Respects `RUST_LOG` env var
✅ Works with async tests (`#[test_log::test(tokio::test)]`)
✅ Minimal code changes

### Cons
❌ Less assertion capability
❌ Mainly for debugging, not validation
❌ Another test attribute to remember

### Example
```rust
#[test_log::test]
fn test_with_logging() {
    info!("Test started");
    // Your test logic
    assert!(true);
}

#[test_log::test(tokio::test)]
async fn test_async() {
    info!("Async test with logging");
    assert!(true);
}
```

**Usage**:
```bash
RUST_LOG=debug cargo test
cargo test -- --nocapture
```

**When to use**: Debugging tests, seeing logs during development

---

## Option 4: Integration Tests with Process Spawning

**File**: `tests/logging_test.rs`

### Pros
✅ Most realistic (full server startup)
✅ Tests actual production behavior
✅ Can test different RUST_LOG levels
✅ Automated in CI/CD

### Cons
❌ Slower (spawns processes)
❌ Requires database setup
❌ More complex to write
❌ Platform considerations (Windows/Linux)

### Example
```rust
#[test]
#[ignore] // Requires database
fn test_server_startup_logs() {
    // Start server, capture stderr
    let child = Command::new("cargo")
        .args(&["run", "--", "--server"])
        .env("RUST_LOG", "info")
        .stderr(Stdio::piped())
        .spawn()?;

    // Read and assert logs
    assert!(logs.contains("Starting web server"));
}
```

**Usage**:
```bash
cargo test --test logging_test -- --ignored
```

**When to use**: CI/CD validation, end-to-end testing, verifying production behavior

---

## Option 5: Manual Subscriber in Tests (Not Recommended)

### Pros
✅ No extra dependencies
✅ Full control

### Cons
❌ Subscriber can only be initialized once
❌ Tests must run serially
❌ Complex setup code
❌ Not thread-safe

### Example
```rust
#[test]
fn test_logging() {
    // Only works once per test run!
    tracing_subscriber::fmt()
        .with_test_writer()
        .init();

    info!("Test log");
}
```

**When to use**: Avoid this approach - use `tracing-test` or `test-log` instead

---

## Recommendations

### For This Project

**Immediate (No changes needed)**:
- ✅ Keep `test_logging.sh` for manual demos
- ✅ Use existing integration tests for behavior

**Future Enhancements**:
1. **Add `test-log`** for debugging during development:
   ```toml
   [dev-dependencies]
   test-log = { version = "0.2", features = ["trace"] }
   ```
   Replace `#[test]` with `#[test_log::test]` in tests where you want to see logs

2. **Add `tracing-test`** for specific log validation:
   ```toml
   [dev-dependencies]
   tracing-test = "0.2"
   ```
   Use for critical logs (errors, startup messages)

3. **Keep integration tests** for end-to-end validation

### Recommended Setup

```toml
# Cargo.toml
[dev-dependencies]
# ... existing dependencies ...
test-log = { version = "0.2", features = ["trace"] }  # For debugging
tracing-test = "0.2"                                   # For assertions
```

### Usage Patterns

**For debugging a failing test**:
```rust
#[test_log::test]
fn my_failing_test() {
    // Logs automatically visible with --nocapture
}
```

**For asserting log content**:
```rust
#[test]
#[tracing_test::traced_test]
fn test_error_handling() {
    error!(game_id = 123, "Game not found");
    assert!(logs_contain("Game not found"));
}
```

**For integration/E2E**:
```rust
#[test]
#[ignore]
fn test_server_startup() {
    // Spawn process, check real logs
}
```

---

## Examples Provided

1. **`test_logging.sh`** - Shell script (manual)
2. **`examples/tracing_test_example.rs`** - tracing-test demo
3. **`examples/test_log_example.rs`** - test-log demo
4. **`tests/logging_test.rs`** - Integration test examples

## Try Them Out

```bash
# Shell script
./test_logging.sh

# tracing-test example (requires adding dependency)
cargo test --example tracing_test_example -- --nocapture

# test-log example (requires adding dependency)
cargo test --example test_log_example -- --nocapure

# Integration tests
cargo test --test logging_test -- --ignored --nocapture
```

## References

- **tracing-test**: https://docs.rs/tracing-test/
- **test-log**: https://docs.rs/test-log/
- **Tracing FAQ**: https://github.com/tokio-rs/tracing/blob/master/tracing/FAQ.md
- **Testing Tracing**: https://tokio.rs/tokio/topics/tracing
