# Test Readiness Signal Analysis

## Problem

After implementing `tracing` for structured logging, integration tests that rely on stdout to detect when the server is ready are failing.

### Root Cause

**Before (println!):**
- Logs went to **stdout**
- Test waits for: `WaitFor::message_on_stdout("Starting web server on")`
- ✅ Tests detected server readiness

**After (tracing):**
- Logs go to **stderr** (tracing default)
- Test still waits for stdout message
- ❌ Tests timeout waiting for stdout message that never comes

### Affected Code

**`tests/common/containers.rs:24`**
```rust
fn ready_conditions(&self) -> Vec<WaitFor> {
    vec![WaitFor::message_on_stdout("Starting web server on")]
}
```

## Solution Options

### Option 1: Change Tracing to Write to Stdout (❌ Not Recommended)

**Approach**: Configure `tracing-subscriber` to write to stdout instead of stderr

**Pros:**
- Minimal test changes
- Tests work immediately

**Cons:**
- ❌ Breaks Unix convention (logs should go to stderr)
- ❌ Makes it harder to separate logs from program output
- ❌ Could break scripts that rely on clean stdout
- ❌ Not idiomatic Rust/Unix behavior

**Verdict**: Bad practice, avoid

---

### Option 2: Add Explicit Stdout Marker (✅ Recommended)

**Approach**: Print a single marker to stdout when server is ready, keep logs on stderr

**Pros:**
- ✅ Maintains proper stdout/stderr separation
- ✅ Clear, explicit "ready" signal for tests and orchestration
- ✅ Works with testcontainers and other tools
- ✅ Minimal code changes
- ✅ Follows Unix conventions

**Cons:**
- Adds one extra line of code

**Implementation:**
```rust
// In src/web.rs, after server binds
println!("READY"); // Explicit stdout marker for tests

info!(
    main_addr = %main_addr,
    health_addr = %health_addr,
    "Starting web server"
);
```

**Test Update:**
```rust
fn ready_conditions(&self) -> Vec<WaitFor> {
    vec![WaitFor::message_on_stdout("READY")]
}
```

**Verdict**: ✅ Best practice

---

### Option 3: Use Health Check Endpoint (⚠️ Alternative)

**Approach**: Remove stdout dependency, rely entirely on HTTP health check

**Pros:**
- ✅ Most realistic (production-like)
- ✅ Tests actual functionality
- ✅ No stdout/stderr concerns

**Cons:**
- ⚠️ Slower (polling overhead)
- ⚠️ `testcontainers` ready_conditions still useful for logs
- ⚠️ Requires removing WaitFor condition or changing to stderr

**Implementation:**
```rust
fn ready_conditions(&self) -> Vec<WaitFor> {
    vec![] // No log-based ready condition
}

// Already exists:
wait_for_server_ready(&url, 30)
    .expect("Server should become ready");
```

**Verdict**: ⚠️ Works but slower; Option 2 is better

---

### Option 4: Wait for Stderr Message (⚠️ Possible)

**Approach**: Change tests to wait for stderr message

**Pros:**
- Logs stay on stderr (proper)

**Cons:**
- ⚠️ `testcontainers` `WaitFor::message_on_stderr` may not be available
- ⚠️ Depends on testcontainers API version
- ⚠️ Less clear as "ready signal" (logs could be anything)

**Verdict**: ⚠️ Check if API supports it, but Option 2 is clearer

---

## Recommended Solution: Option 2

### Implementation Plan

1. **Add explicit stdout marker in `src/web.rs`**
   - After server binds successfully
   - Before or after structured logs
   - Simple, clear message: `"READY"` or `"SERVER_READY"`

2. **Update test wait condition in `tests/common/containers.rs`**
   - Change from: `"Starting web server on"`
   - Change to: `"READY"`

3. **Benefits:**
   - Tests work immediately
   - Logs properly separated (stderr for logs, stdout for ready signal)
   - Clear contract for server readiness
   - Useful for Docker, Kubernetes, systemd, etc.

### Code Changes

**File: `src/web.rs`**

**Location**: After server binds, before or after first log

**Option A** (Before logs - fastest signal):
```rust
let main_listener = tokio::net::TcpListener::bind(&main_addr)
    .await
    .unwrap_or_else(|_| panic!("Failed to bind to {}", main_addr));

let health_listener = tokio::net::TcpListener::bind(&health_addr)
    .await
    .unwrap_or_else(|_| panic!("Failed to bind to {}", health_addr));

// Explicit ready marker for tests/orchestration
println!("READY");

info!(
    main_addr = %main_addr,
    health_addr = %health_addr,
    "Starting web server"
);
```

**Option B** (After logs - more info first):
```rust
info!(
    main_addr = %main_addr,
    health_addr = %health_addr,
    "Starting web server"
);
info!("Web Interface: http://{}/", main_addr);
info!("API Endpoints:");
info!("  POST /api/games - Create a new game");
info!("  POST /api/games/:game_id/guess - Make a guess");
info!("Health Check: http://{}/health", health_addr);

// Explicit ready marker for tests/orchestration (last, so all setup is done)
println!("READY");
```

**Recommendation**: Option B (after all logs), so "READY" truly means ready

**File: `tests/common/containers.rs`**

```rust
fn ready_conditions(&self) -> Vec<WaitFor> {
    vec![WaitFor::message_on_stdout("READY")]
}
```

### Alternative Marker Messages

Instead of just `"READY"`, could use:
- `"SERVER_READY"` - More explicit
- `"NUMBER_GUESSING_GAME_READY"` - App-specific
- JSON: `{"status":"ready","port":8080}` - Structured

**Recommendation**: `"READY"` - Simple, clear, universal

---

## Testing the Fix

```bash
# Build Docker image
docker build -t numberguess-claude-rust:latest .

# Run integration tests
cargo test --test integration_test
cargo test --test concurrency_test
cargo test --test web_endpoints_test
```

Should see container start with:
```
[INFO] Connecting to database...
[INFO] Starting web server...
READY    ← stdout marker
```

---

## Documentation Updates

Update [CLAUDE.md](../CLAUDE.md) to document:
- Server emits `"READY"` to stdout when fully initialized
- Useful for Docker, Kubernetes, process managers
- Logs go to stderr (structured via tracing)
- Program output (ready signal) goes to stdout

---

## Edge Cases Considered

1. **What if binding fails?**
   - No `println!("READY")` emitted
   - Test timeout = correct behavior ✅

2. **What if migrations fail?**
   - Server panics before READY
   - Test fails = correct behavior ✅

3. **Multiple servers in one process?**
   - Only one READY needed (main server readiness)
   - Health check server is secondary ✅

4. **Process managers (systemd, Docker)?**
   - Can use `READY` signal too
   - Benefit beyond just tests ✅

---

## Summary

**Problem**: Tests wait for stdout, logs now on stderr
**Solution**: Add explicit `println!("READY")` marker
**Why**: Proper separation of concerns, clear contract, minimal change
**Status**: Ready to implement ✅
