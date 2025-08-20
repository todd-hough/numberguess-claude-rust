# Integration Test Plan for Number Guessing Game

## Current Test Coverage

### Existing Tests
1. **API Tests** (`tests/integration_test.rs`)
   - ✅ `test_basic_game_flow` - Creates game, makes guesses, finds correct answer
   - ✅ `test_invalid_game_parameters` - Tests validation for invalid min/max/limit values

2. **Web UI Tests** (`tests/web_ui_test.rs`)
   - ✅ `test_web_ui_game_flow` - Simulated browser test for game flow
   - ✅ `test_web_ui_invalid_inputs` - Simulated browser test for input validation

## Interface Points Analysis

### 1. CLI Interface
- **Entry Point**: Binary execution with command-line arguments
- **Input**: stdin for user guesses
- **Output**: stdout for game messages

### 2. REST API Endpoints
- `POST /api/games` - Create new game
- `POST /api/games/{game_id}/guess` - Make a guess

### 3. Web UI Endpoints  
- `POST /game/new` - Create game via HTML form
- `POST /game/{game_id}/guess` - Make guess via HTML form
- `GET /` - Serve static HTML/CSS/JS files

### 4. Server Management
- Server startup on specified port
- Graceful shutdown
- Port binding errors

## Proposed Additional Integration Tests

### Priority 1: Critical Path Coverage (Fast, Essential)

#### 1. CLI Integration Tests (`tests/cli_test.rs`)
```rust
// Test 1: Basic CLI game flow with arguments
test_cli_basic_game_flow()
- Run binary with --min 1 --max 10 --limit 5
- Send guesses via stdin
- Verify output messages
- Ensure game ends correctly

// Test 2: CLI with default values
test_cli_default_values()
- Run binary without arguments
- Verify default range (1-100)
- Test single guess interaction

// Test 3: CLI help and version
test_cli_help_output()
- Run with --help
- Verify help text is displayed
```

#### 2. API Edge Cases (`tests/api_edge_cases_test.rs`)
```rust
// Test 1: Non-existent game ID
test_api_guess_nonexistent_game()
- Try to make guess for invalid game_id
- Expect 404 or appropriate error

// Test 2: Multiple concurrent games
test_api_concurrent_games()
- Create 3 games simultaneously
- Make guesses to each
- Verify isolation between games

// Test 3: Guess limit reached
test_api_guess_limit_exhausted()
- Create game with limit=2
- Make 2 wrong guesses
- Verify game ends and rejects further guesses

// Test 4: Boundary value guesses
test_api_boundary_guesses()
- Create game min=50, max=60
- Test guesses at boundaries (49, 50, 60, 61)
- Verify proper validation
```

#### 3. Web Endpoints (`tests/web_endpoints_test.rs`)
```rust
// Test 1: HTML form game creation
test_web_create_game_form()
- POST to /game/new with form data
- Verify HTML response contains game UI
- Extract game_id from response

// Test 2: HTML form guess submission
test_web_make_guess_form()
- Create game via web endpoint
- POST guess via form submission
- Verify HTML response with feedback

// Test 3: Static file serving
test_static_file_serving()
- GET /
- Verify index.html is served
- GET /index.html
- Verify correct content-type headers
```

### Priority 2: Extended Coverage (Nice to Have)

#### 4. Server Lifecycle Tests (`tests/server_lifecycle_test.rs`)
```rust
// Test 1: Port already in use
test_server_port_conflict()
- Start first server on port X
- Try to start second server on same port
- Verify appropriate error handling

// Test 2: Server shutdown cleanup
test_server_graceful_shutdown()
- Start server
- Create active game
- Shutdown server
- Verify clean termination
```

#### 5. Performance Tests (`tests/performance_test.rs`)
```rust
// Test 1: Many games creation
test_many_games_creation()
- Create 100 games rapidly
- Verify all succeed
- Check response times remain reasonable

// Test 2: Many guesses per game
test_many_guesses_single_game()
- Create game with large range (1-1000)
- Make 50 guesses rapidly
- Verify all are processed correctly
```

## Test Execution Strategy

### Fast Test Suite (< 5 seconds total)
Run these on every commit:
1. All Priority 1 tests
2. Existing API tests
3. Use parallel execution with random ports

### Extended Test Suite (< 30 seconds total)
Run these before releases:
1. All Priority 1 tests
2. All Priority 2 tests
3. Performance tests with reduced iterations

## Implementation Approach

### Test Utilities to Add
```rust
// In tests/common/mod.rs
pub mod cli_helpers {
    // Function to run CLI with args and capture output
    pub fn run_cli_with_args(args: Vec<&str>) -> CliOutput
    
    // Function to run CLI with stdin input
    pub fn run_cli_with_input(args: Vec<&str>, input: &str) -> CliOutput
}

pub mod api_helpers {
    // Function to create game and return ID
    pub fn quick_create_game(client: &Client, min: i32, max: i32) -> u64
    
    // Function to make guess and return result
    pub fn make_guess(client: &Client, game_id: u64, guess: i32) -> GuessResponse
}
```

## Coverage Goals

### Must Have (Minimum Viable Coverage)
- ✅ Basic API happy path
- ✅ API input validation
- ⬜ CLI basic operation
- ⬜ Non-existent game handling
- ⬜ Concurrent games isolation
- ⬜ Static file serving

### Should Have (Good Coverage)
- ⬜ CLI with various arguments
- ⬜ Guess limit enforcement
- ⬜ Boundary value testing
- ⬜ Web form endpoints
- ⬜ Multiple games management

### Nice to Have (Comprehensive Coverage)
- ⬜ Port conflict handling
- ⬜ Performance benchmarks
- ⬜ Server lifecycle management
- ⬜ Error recovery scenarios

## Test Execution Times (Target)

| Test Category | Target Time | Tests Count |
|--------------|-------------|-------------|
| API Tests | < 1s | 6-8 tests |
| CLI Tests | < 2s | 3-4 tests |
| Web Tests | < 1s | 3-4 tests |
| Edge Cases | < 1s | 4-5 tests |
| **Total Fast Suite** | **< 5s** | **~20 tests** |

## Benefits of This Approach

1. **Fast Feedback**: All critical paths tested in < 5 seconds
2. **Comprehensive**: Covers all major interface points
3. **Maintainable**: Clear test organization and naming
4. **Scalable**: Easy to add new tests in appropriate categories
5. **Parallel Execution**: Tests use random ports for concurrent running
6. **CI/CD Friendly**: Fast suite for commits, extended for releases

## Next Steps

1. Implement Priority 1 tests first
2. Add common test utilities for CLI and API helpers
3. Run tests in parallel for speed
4. Add performance tests as optional extended suite
5. Document any flaky tests and mitigation strategies