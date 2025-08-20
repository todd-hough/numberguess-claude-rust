# Integration Tests Implementation

This document explains the integration test implementation for the Number Guessing Game.

## Overview

The integration tests use two approaches:

1. **API Testing**: Direct HTTP requests to test the API endpoints
2. **Web UI Testing**: Browser-based testing using WebDriver to test the user interface

## Test Structure

- `tests/common/containers.rs`: Contains the server management code and helper functions
- `tests/integration_test.rs`: Contains the API integration tests
- `tests/web_ui_test.rs`: Contains the Web UI integration tests using WebDriver and Selenium

## Running Tests

To run all integration tests:

```bash
./run_integration_tests.sh
```

To run only the API tests:

```bash
cargo test --test integration_test
```

To run only the Web UI tests:

```bash
cargo test --test web_ui_test
```

## Implementation Details

### Server Management

The `GameServerInstance` struct:
- Starts the game server as a background process
- Manages port allocation
- Waits for the server to be ready before proceeding with tests
- Automatically kills the server when tests complete

### API Tests

Two basic API tests are implemented:

1. `test_basic_game_flow`: Tests the full lifecycle of a game via API calls
   - Creates a new game
   - Makes guesses until finding the correct number
   - Verifies game state at each step

2. `test_invalid_game_parameters`: Tests input validation
   - Attempts to create games with invalid parameters
   - Verifies the API rejects these attempts with appropriate error codes

### Web UI Tests

Two Web UI tests are implemented using WebDriver and Selenium:

1. `test_web_ui_game_flow`: Tests the full lifecycle of a game via browser UI
   - Navigates to the game page
   - Fills and submits the game setup form
   - Makes guesses until finding the correct number
   - Verifies feedback messages

2. `test_web_ui_invalid_inputs`: Tests input validation in the UI
   - Submits a form with invalid inputs (min > max)
   - Verifies appropriate error handling

## Dependencies

The tests depend on:
- **Docker**: For running Selenium container
- **WebDriver**: For browser automation
- **Selenium**: For browser control
- **reqwest**: For HTTP requests

## Prerequisites

- Docker must be installed and running
- Rust and Cargo must be installed

## Adding New Tests

To add new integration tests:
1. Add new test functions to the relevant test file
2. Use the `#[test]` and `#[serial]` attributes
3. Create appropriate instances for testing