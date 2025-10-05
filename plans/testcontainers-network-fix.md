# Testcontainers Network Fix - Option 2: Container IP Communication

## Problem Statement

Integration tests are failing because the game server container cannot connect to the PostgreSQL database. The issue is that `DATABASE_URL` uses `localhost:PORT`, which refers to the container's own localhost, not the host machine where PostgreSQL is accessible.

**Error**: `WaitContainer(WaitLog(EndOfStream([b"Connecting to database (max connections: 5)...\n"])))`

## Root Cause

```
Current (Broken):
┌─────────────────┐         ┌──────────────────┐
│   PostgreSQL    │         │   Game Server    │
│   Container     │         │    Container     │
│  port 5432      │         │                  │
└────────┬────────┘         │  DATABASE_URL:   │
         │                  │  localhost:32817 │
         │ mapped to        └──────────────────┘
         ↓                          ↓
    Host:32817                   FAILS!
                          (localhost in container
                           != host machine)
```

## Solution: Use Container IP Addresses

Containers on Docker's default bridge network can communicate directly via their IP addresses. This works for concurrent tests because each container gets a unique IP.

```
Fixed (Option 2):
┌─────────────────┐         ┌──────────────────┐
│   PostgreSQL    │ ←───────│   Game Server    │
│   172.17.0.2    │         │   172.17.0.3     │
│   port 5432     │         │                  │
└────────┬────────┘         │  DATABASE_URL:   │
         │                  │  172.17.0.2:5432 │
         │                  └──────────────────┘
         │                          ✓
         ↓                     WORKS!
    Host:32817              (direct container
  (for test access)          communication)
```

## Implementation Plan

### 1. Add `container_url()` method to PostgresInstance

**File**: `tests/common/containers.rs`
**Location**: After line 321 (inside `impl PostgresInstance`)

```rust
impl PostgresInstance {
    // ... existing new() method ...

    /// Get database URL for inter-container communication
    /// Uses container IP instead of localhost for container-to-container access
    pub fn container_url(&self) -> String {
        let container_id = self.container.id();

        // Use docker inspect to get the container's IP address on the bridge network
        let output = Command::new("docker")
            .args([
                "inspect",
                "-f",
                "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}",
                container_id
            ])
            .output()
            .expect("Failed to inspect PostgreSQL container");

        let ip_address = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 in IP address")
            .trim()
            .to_string();

        println!("PostgreSQL container IP: {}", ip_address);

        format!(
            "postgresql://postgres:postgres@{}:5432/postgres",
            ip_address
        )
    }
}
```

### 2. Update GameServerInstance to use container URL

**File**: `tests/common/containers.rs`
**Location**: Line 38 (in `GameServerInstance::new`)

**Before**:
```rust
let image = GameServerImage::default()
    .with_env_var("DATABASE_URL", database_url);
```

**After**:
```rust
let image = GameServerImage::default()
    .with_env_var("DATABASE_URL", database_url);

println!("Using DATABASE_URL for container: {}", database_url);
```

### 3. Update all test files to use container_url()

**Files to update**:
- `tests/integration_test.rs` (2 occurrences)
- `tests/web_endpoints_test.rs` (2 occurrences)
- `tests/web_ui_test.rs` (2 occurrences)

**Before**:
```rust
let postgres = PostgresInstance::new();
let server = GameServerInstance::new(&postgres.database_url);
```

**After**:
```rust
let postgres = PostgresInstance::new();
let server = GameServerInstance::new(&postgres.container_url());
```

## Why This Works for Concurrent Tests

### Port Isolation Mechanisms

1. **Host Side** (test process access):
   - Each container gets random host port: `localhost:32817`, `localhost:32818`, etc.
   - No conflicts - testcontainers handles this automatically

2. **Container Side** (inter-container communication):
   - Each container has unique IP on default bridge network
   - Test 1: Postgres `172.17.0.2` → GameServer `172.17.0.3`
   - Test 2: Postgres `172.17.0.4` → GameServer `172.17.0.5`
   - No conflicts - containers are network-isolated

3. **No Custom Network Needed**:
   - testcontainers-rs v0.23 doesn't support `Network::new()` API
   - Default bridge network provides sufficient isolation
   - Each test's containers are ephemeral and isolated

## Files to Modify

1. `tests/common/containers.rs` - Add `container_url()` method
2. `tests/integration_test.rs` - Update 2 tests
3. `tests/web_endpoints_test.rs` - Update 2 tests
4. `tests/web_ui_test.rs` - Update 2 tests

## Testing Plan

After implementation:

```bash
# Run single test
cargo test --test integration_test test_basic_game_flow -- --nocapture

# Run all integration tests
cargo test --test integration_test

# Run tests in parallel (verify no port conflicts)
cargo test --test integration_test --test web_endpoints_test -- --test-threads=4
```

## Expected Outcome

- ✅ All integration tests pass
- ✅ Containers communicate successfully via bridge network
- ✅ Concurrent tests run without port conflicts
- ✅ Clean container cleanup after tests
- ✅ No need for custom network configuration

## Alternative Approaches Considered

### Option 1: Custom Networks with testcontainers
- **Status**: Not available in testcontainers-rs v0.23
- **Future**: May be available in v0.25+, but not needed for this use case

### Option 3: host.docker.internal
- **Pros**: Simple configuration
- **Cons**: Platform-dependent (Docker Desktop only), unreliable on Linux
- **Verdict**: Less portable than Option 2

## References

- testcontainers-rs documentation: https://docs.rs/testcontainers/0.23/
- Docker bridge network: https://docs.docker.com/network/bridge/
- Existing `internal_url()` pattern: `tests/common/containers.rs:65-83`
