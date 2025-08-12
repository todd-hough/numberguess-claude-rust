# Testing Guide

## Overview
This guide covers testing strategies, best practices, and examples for the Number Guessing Game project.

## Testing Philosophy

### Principles
- Test behavior, not implementation
- Focus on edge cases and error conditions
- Keep tests simple and readable
- Each test should test one thing
- Tests should be independent

### Test Pyramid
```
        /\        E2E Tests (Few)
       /  \       - Full system tests
      /    \      
     /      \     Integration Tests (Some)
    /        \    - Module interactions
   /          \   
  /            \  Unit Tests (Many)
 /______________\ - Individual functions
```

## Running Tests

### Basic Commands
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test game::tests

# Run specific test
cargo test test_game_creation

# Run tests in single thread (for debugging)
cargo test -- --test-threads=1

# Run only library tests
cargo test --lib

# Run ignored tests
cargo test -- --ignored
```

### Test Coverage
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# Open coverage report
open tarpaulin-report.html
```

## Unit Testing

### Testing Game Logic
Location: `src/game.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation_valid_range() {
        let game = GuessingGame::new(1, 100);
        assert!(game.is_ok());
        
        let game = game.unwrap();
        assert_eq!(game.get_range(), (1, 100));
    }

    #[test]
    fn test_invalid_range_max_less_than_min() {
        let game = GuessingGame::new(100, 1);
        assert!(game.is_err());
        assert!(game.unwrap_err().contains("greater than or equal"));
    }

    #[test]
    fn test_negative_values_rejected() {
        let game = GuessingGame::new(-5, 10);
        assert!(game.is_err());
        assert!(game.unwrap_err().contains("non-negative"));
    }
}
```

### Testing with Guess Limits
```rust
#[test]
fn test_guess_limit_enforcement() {
    let mut game = GuessingGame::new_with_limit(1, 10, Some(3)).unwrap();
    game.secret_number = 5; // Set for predictable testing
    
    // Make 3 incorrect guesses
    assert_eq!(game.make_guess(1), GuessResult::TooLow);
    assert_eq!(game.make_guess(2), GuessResult::TooLow);
    assert_eq!(game.make_guess(3), GuessResult::LimitReached { 
        number: 5, 
        max_guesses: 3 
    });
    
    // Further guesses should return LimitReached
    assert_eq!(game.make_guess(5), GuessResult::LimitReached { 
        number: 5, 
        max_guesses: 3 
    });
}
```

### Edge Cases to Test
```rust
#[test]
fn test_edge_cases() {
    // Same min and max
    let game = GuessingGame::new(42, 42).unwrap();
    assert_eq!(game.secret_number, 42);
    
    // Maximum allowed value
    let game = GuessingGame::new(0, 1_000_000);
    assert!(game.is_ok());
    
    // One over maximum
    let game = GuessingGame::new(0, 1_000_001);
    assert!(game.is_err());
    
    // Zero values
    let game = GuessingGame::new(0, 0).unwrap();
    assert_eq!(game.secret_number, 0);
}
```

## Integration Testing

### Testing Web API
Create `tests/api_integration.rs`:

```rust
#[cfg(test)]
mod tests {
    use number_guessing_game::web::run_server;
    use reqwest;
    use serde_json::json;

    #[tokio::test]
    async fn test_game_creation_api() {
        // Start server in background
        tokio::spawn(async {
            run_server(3001).await;
        });
        
        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Create client
        let client = reqwest::Client::new();
        
        // Test game creation
        let response = client
            .post("http://localhost:3001/api/games")
            .json(&json!({
                "min": 1,
                "max": 100,
                "max_guesses": 10
            }))
            .send()
            .await
            .unwrap();
        
        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body["game_id"].is_number());
        assert_eq!(body["min"], 1);
        assert_eq!(body["max"], 100);
        assert_eq!(body["max_guesses"], 10);
    }
}
```

### Testing CLI Interaction
```rust
#[test]
fn test_cli_argument_parsing() {
    use clap::Parser;
    use number_guessing_game::Cli;
    
    // Test with arguments
    let cli = Cli::parse_from(&[
        "prog",
        "--min", "10",
        "--max", "50",
        "--limit", "5"
    ]);
    
    assert_eq!(cli.min, Some(10));
    assert_eq!(cli.max, Some(50));
    assert_eq!(cli.limit, Some(5));
}
```

## Property-Based Testing

Using `proptest` for randomized testing:

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.0"
```

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_valid_range_always_creates_game(
            min in 0..500_000i32,
            max in 500_000..1_000_000i32
        ) {
            let game = GuessingGame::new(min, max);
            prop_assert!(game.is_ok());
            
            let game = game.unwrap();
            let (game_min, game_max) = game.get_range();
            prop_assert_eq!(game_min, min);
            prop_assert_eq!(game_max, max);
            prop_assert!(game.secret_number >= min);
            prop_assert!(game.secret_number <= max);
        }

        #[test]
        fn test_invalid_range_always_fails(
            min in 1..1_000_000i32,
            diff in 1..1000i32
        ) {
            let max = min - diff; // max < min
            let game = GuessingGame::new(min, max);
            prop_assert!(game.is_err());
        }
    }
}
```

## Testing Best Practices

### 1. Test Organization
```rust
// Group related tests
mod game_creation_tests {
    #[test]
    fn valid_range() { /* ... */ }
    
    #[test]
    fn invalid_range() { /* ... */ }
}

mod guess_processing_tests {
    #[test]
    fn too_low() { /* ... */ }
    
    #[test]
    fn too_high() { /* ... */ }
    
    #[test]
    fn correct() { /* ... */ }
}
```

### 2. Test Data Builders
```rust
struct GameBuilder {
    min: i32,
    max: i32,
    max_guesses: Option<u32>,
    secret: Option<i32>,
}

impl GameBuilder {
    fn new() -> Self {
        GameBuilder {
            min: 1,
            max: 100,
            max_guesses: None,
            secret: None,
        }
    }
    
    fn with_range(mut self, min: i32, max: i32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    
    fn with_limit(mut self, limit: u32) -> Self {
        self.max_guesses = Some(limit);
        self
    }
    
    fn with_secret(mut self, secret: i32) -> Self {
        self.secret = Some(secret);
        self
    }
    
    fn build(self) -> GuessingGame {
        let mut game = GuessingGame::new_with_limit(
            self.min, 
            self.max, 
            self.max_guesses
        ).unwrap();
        
        if let Some(secret) = self.secret {
            game.secret_number = secret;
        }
        
        game
    }
}

#[test]
fn test_with_builder() {
    let game = GameBuilder::new()
        .with_range(1, 50)
        .with_limit(10)
        .with_secret(25)
        .build();
    
    assert_eq!(game.get_range(), (1, 50));
    assert_eq!(game.get_max_guesses(), Some(10));
}
```

### 3. Asserting Panic
```rust
#[test]
#[should_panic(expected = "assertion failed")]
fn test_panic_condition() {
    // Code that should panic
    assert!(false, "assertion failed");
}
```

### 4. Ignoring Expensive Tests
```rust
#[test]
#[ignore]
fn expensive_test() {
    // Run only with: cargo test -- --ignored
    for i in 0..1_000_000 {
        // Expensive operation
    }
}
```

## Manual Testing Checklist

### CLI Testing
- [ ] Start without arguments, verify prompts work
- [ ] Provide partial arguments, verify remaining prompts
- [ ] Test with invalid inputs (letters, special characters)
- [ ] Test boundary values (0, 1,000,000)
- [ ] Test guess limit functionality
- [ ] Verify error messages are helpful

### Web UI Testing
- [ ] Form validation (empty fields, invalid ranges)
- [ ] Guess submission works
- [ ] Feedback messages display correctly
- [ ] Guess counter updates
- [ ] Game completion scenarios
- [ ] Browser compatibility

### API Testing
- [ ] Create game with various parameters
- [ ] Make guesses (too low, too high, correct)
- [ ] Test error responses
- [ ] Verify JSON formatting
- [ ] Test concurrent games
- [ ] Check game cleanup after completion

## Performance Testing

### Benchmark Tests
```rust
#![feature(test)]
extern crate test;

#[cfg(test)]
mod benches {
    use test::Bencher;
    use super::*;

    #[bench]
    fn bench_game_creation(b: &mut Bencher) {
        b.iter(|| {
            GuessingGame::new(1, 1000)
        });
    }

    #[bench]
    fn bench_make_guess(b: &mut Bencher) {
        let mut game = GuessingGame::new(1, 1000).unwrap();
        b.iter(|| {
            game.make_guess(500)
        });
    }
}
```

### Load Testing
```bash
# Using Apache Bench
ab -n 1000 -c 10 http://localhost:3000/

# Using wrk
wrk -t4 -c100 -d30s http://localhost:3000/api/games
```

## Continuous Integration

### GitHub Actions Example
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - run: cargo test
    - run: cargo clippy -- -D warnings
    - run: cargo fmt -- --check
```

## Debugging Failed Tests

### Techniques
```bash
# Show test output
cargo test -- --nocapture

# Run single test
cargo test test_name -- --exact

# Use println! debugging
#[test]
fn debug_test() {
    let value = calculate_something();
    println!("Debug value: {:?}", value);
    assert_eq!(value, expected);
}

# Use dbg! macro
#[test]
fn debug_with_macro() {
    let value = dbg!(calculate_something());
    assert_eq!(value, expected);
}
```

## Test Documentation

### Documenting Test Purpose
```rust
/// Tests that the game correctly validates that maximum 
/// values must be greater than or equal to minimum values.
/// 
/// This is a critical validation to ensure game logic works.
#[test]
fn test_max_greater_than_min_validation() {
    // Test implementation
}
```

### Test Categories
Mark tests by category for selective running:
```rust
#[test]
#[cfg(feature = "integration")]
fn integration_test() { }

#[test] 
#[cfg(not(target_os = "windows"))]
fn unix_only_test() { }