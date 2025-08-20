# Integration Tests Implementation

This document explains the basic integration test implementation for the Number Guessing Game.

## Overview

The integration tests use a simple approach to:
1. Start the application as a background process
2. Run tests against its API
3. Clean up automatically when tests complete

## Test Structure

- `tests/common/containers.rs`: Contains the server management code and helper functions
- `tests/integration_test.rs`: Contains the actual integration tests

## Running Tests

To run the integration tests:

```bash
./run_integration_tests.sh
```

This will:
1. Build the application
2. Run the integration tests using `cargo test --test integration_test`

## Implementation Details

### Server Management

The `GameServerInstance` struct:
- Starts the game server as a background process
- Manages port allocation
- Waits for the server to be ready before proceeding with tests
- Automatically kills the server when tests complete

### Basic Tests

Two basic integration tests are implemented:

1. `test_basic_game_flow`: Tests the full lifecycle of a game
   - Creates a new game
   - Makes guesses until finding the correct number
   - Verifies game state at each step

2. `test_invalid_game_parameters`: Tests input validation
   - Attempts to create games with invalid parameters
   - Verifies the API rejects these attempts with appropriate error codes

## Adding New Tests

To add new integration tests:
1. Add new test functions to `tests/integration_test.rs`
2. Use the `#[test]` and `#[serial]` attributes
3. Create a new `GameServerInstance` for each test to ensure isolation

## Prerequisites

- Rust and Cargo must be installed