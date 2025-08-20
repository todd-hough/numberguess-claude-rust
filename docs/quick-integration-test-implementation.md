# Quick Integration Test Implementation Guide

## Priority Tests to Implement (Maximum Impact, Minimum Effort)

### Phase 1: CLI Testing (New Coverage) - 30 minutes
Create `tests/cli_test.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_basic_flow() {
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.args(&["--min", "5", "--max", "5", "--limit", "1"])
        .write_stdin("5\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("You got it!"));
}

#[test] 
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("number_guessing_game").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A fun number guessing game"));
}
```

### Phase 2: API Edge Cases - 20 minutes
Create `tests/api_edge_cases_test.rs`:

```rust
#[test]
fn test_guess_nonexistent_game() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    let response = client
        .post(format!("{}/api/games/99999999/guess", server.url()))
        .json(&json!({"guess": 50}))
        .send()
        .unwrap();
    
    assert_eq!(response.status(), 404);
}

#[test]
fn test_concurrent_games() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Create 3 games
    let game_ids: Vec<u64> = (0..3)
        .map(|_| {
            let resp = client
                .post(format!("{}/api/games", server.url()))
                .json(&json!({"min": 1, "max": 10}))
                .send()
                .unwrap();
            let game: GameResponse = resp.json().unwrap();
            game.game_id
        })
        .collect();
    
    // Make guess to each game
    for game_id in &game_ids {
        let resp = client
            .post(format!("{}/api/games/{}/guess", server.url(), game_id))
            .json(&json!({"guess": 5}))
            .send()
            .unwrap();
        assert!(resp.status().is_success());
    }
}

#[test]
fn test_guess_after_limit_reached() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Create game with limit=1
    let resp = client
        .post(format!("{}/api/games", server.url()))
        .json(&json!({"min": 1, "max": 10, "limit": 1}))
        .send()
        .unwrap();
    let game: GameResponse = resp.json().unwrap();
    
    // Make wrong guess to exhaust limit
    client
        .post(format!("{}/api/games/{}/guess", server.url(), game.game_id))
        .json(&json!({"guess": 11}))
        .send()
        .unwrap();
    
    // Try another guess - should fail
    let resp = client
        .post(format!("{}/api/games/{}/guess", server.url(), game.game_id))
        .json(&json!({"guess": 5}))
        .send()
        .unwrap();
    
    assert!(resp.status().is_client_error());
}
```

### Phase 3: Static File Serving - 10 minutes
Add to `tests/web_endpoints_test.rs`:

```rust
#[test]
fn test_static_file_serving() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Test root serves index.html
    let resp = client
        .get(server.url())
        .send()
        .unwrap();
    
    assert!(resp.status().is_success());
    let body = resp.text().unwrap();
    assert!(body.contains("Number Guessing Game"));
    assert!(body.contains("<!DOCTYPE html>"));
}

#[test]
fn test_web_form_endpoints() {
    let server = GameServerInstance::new();
    let client = Client::new();
    
    // Test form submission to /game/new
    let resp = client
        .post(format!("{}/game/new", server.url()))
        .form(&[("min", "1"), ("max", "10"), ("max_guesses", "5")])
        .send()
        .unwrap();
    
    assert!(resp.status().is_success());
    let body = resp.text().unwrap();
    // Should return HTML with game interface
    assert!(body.contains("guess"));
}
```

## Test Helper Utilities
Add to `tests/common/mod.rs`:

```rust
pub mod test_helpers {
    use serde_json::json;
    use reqwest::blocking::Client;
    
    pub fn create_game_quick(client: &Client, base_url: &str) -> u64 {
        let resp = client
            .post(format!("{}/api/games", base_url))
            .json(&json!({"min": 1, "max": 10}))
            .send()
            .unwrap();
        let game: GameResponse = resp.json().unwrap();
        game.game_id
    }
    
    pub fn make_guess_quick(client: &Client, base_url: &str, game_id: u64, guess: i32) -> GuessResponse {
        let resp = client
            .post(format!("{}/api/games/{}/guess", base_url, game_id))
            .json(&json!({"guess": guess}))
            .send()
            .unwrap();
        resp.json().unwrap()
    }
}
```

## Execution Time Estimates

| Test File | Tests | Estimated Time |
|-----------|-------|----------------|
| cli_test.rs | 2 | ~1.5s |
| api_edge_cases_test.rs | 3 | ~0.8s |
| web_endpoints_test.rs | 2 | ~0.5s |
| **Total** | **7 new tests** | **~2.8s** |

Combined with existing 4 tests: **~4.5s total**

## Running the Tests

```bash
# Run all integration tests in parallel
cargo test --tests

# Run specific test file
cargo test --test cli_test
cargo test --test api_edge_cases_test

# Run with output
cargo test --tests -- --nocapture

# Measure actual time
time cargo test --tests
```

## Key Benefits of This Approach

1. **Fast**: All tests complete in < 5 seconds
2. **High Coverage**: Covers all major interfaces (CLI, API, Web)
3. **Real Issues**: Tests catch actual bugs (nonexistent games, limit enforcement)
4. **Maintainable**: Simple, focused tests that are easy to understand
5. **CI-Friendly**: Quick enough to run on every commit

## What We're NOT Testing (Intentionally)

1. **Performance**: Separate benchmark suite if needed
2. **Browser JavaScript**: Would require real Selenium setup
3. **Network failures**: Better suited for unit tests with mocks
4. **Database**: No database in this application
5. **Authentication**: No auth in current implementation

## Next Steps After Implementation

1. ✅ Run tests to establish baseline
2. ✅ Add to CI pipeline
3. ✅ Monitor for flaky tests
4. ⬜ Add more edge cases only if bugs are found
5. ⬜ Consider contract tests if API consumed by others