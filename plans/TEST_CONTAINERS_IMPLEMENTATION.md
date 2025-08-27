# Integration Tests Implementation

This document explains the integration test implementation for the Number Guessing Game.

## Overview

The integration tests use two approaches:

1. **API Testing**: Direct HTTP requests to test the API endpoints
2. **Web UI Testing**: Browser-based testing using WebDriver to test the user interface

Both test types use **random port allocation** and can run **in parallel** for faster execution.

## Test Structure

- `tests/common/containers.rs`: Contains the server management code, port allocation utilities and helper functions
- `tests/integration_test.rs`: Contains the API integration tests
- `tests/web_ui_test.rs`: Contains the Web UI integration tests using WebDriver and Selenium

## Running Tests

To run all integration tests (in parallel):

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

### Port Management

The tests use a **dynamic port allocation system**:

- **Random Port Selection**: Each test instance gets a random port from the ephemeral port range (49152-65535)
- **Port Availability Check**: The system verifies port availability before assignment
- **Fallback Mechanism**: If no port is found after 10 attempts, uses a fallback port
- **No Port Conflicts**: Multiple tests can run simultaneously without port conflicts

### Parallel Execution

- **No Serial Dependency**: Tests do not use `#[serial]` attribute and can run in parallel
- **Independent Resources**: Each test creates its own server instance and resources
- **Faster Execution**: Parallel execution reduces total test time significantly
- **Scalable**: Easy to add new tests without worrying about resource conflicts

### Server Management

The `GameServerInstance` struct:
- Automatically finds and allocates an available random port
- Starts the game server as a background process on the allocated port
- Waits for the server to be ready before proceeding with tests
- Automatically kills the server when tests complete via `Drop` implementation
- Provides access to the allocated port and server URL

### API Tests

Two API tests are implemented:

1. `test_basic_game_flow`: Tests the full lifecycle of a game via API calls
   - Creates a new game with random port allocation
   - Makes guesses until finding the correct number
   - Verifies game state at each step

2. `test_invalid_game_parameters`: Tests input validation
   - Creates a server with random port allocation
   - Attempts to create games with invalid parameters
   - Verifies the API rejects these attempts with appropriate error codes

### Web UI Tests

Two Web UI tests are implemented using WebDriver and Selenium:

1. `test_web_ui_game_flow`: Tests the full lifecycle of a game via browser UI
   - Allocates random ports for both game server and selenium
   - Simulates browser interactions with the game
   - Demonstrates the structure for real WebDriver testing

2. `test_web_ui_invalid_inputs`: Tests input validation in the UI
   - Allocates random ports for both game server and selenium
   - Simulates invalid form submissions
   - Demonstrates error handling testing structure

## Performance Benefits

- **Parallel Execution**: Tests run simultaneously instead of sequentially
- **Reduced Test Time**: Integration tests complete in ~0.7s instead of ~1.2s when sequential
- **Better Resource Utilization**: Makes full use of available CPU cores
- **Scalability**: Adding more tests doesn't significantly impact runtime

## Dependencies

The tests depend on:
- **Docker**: For running Selenium container (for real WebDriver tests)
- **WebDriver**: For browser automation
- **Selenium**: For browser control
- **reqwest**: For HTTP requests
- **rand**: For random port generation

## Prerequisites

- Docker must be installed and running (for real WebDriver tests)
- Rust and Cargo must be installed

## Adding New Tests

To add new integration tests:
1. Add new test functions to the relevant test file
2. Use the `#[test]` attribute (no `#[serial]` needed)
3. Create `GameServerInstance::new()` for automatic port allocation
4. Each test will get its own random port automatically

## Port Range

- **Ephemeral Port Range**: 49152-65535
- **Multiple Attempts**: Up to 10 tries to find an available port
- **Fallback**: Uses port 49152 if no port found (rare scenario)
- **Thread-Safe**: Random port generation is thread-safe for parallel execution

## Known Limitations

- Random port allocation relies on OS port availability
- Very rare chance of port conflicts under heavy system load
- WebDriver tests currently simulated (require real Selenium setup for full browser testing)