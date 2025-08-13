# Claude Code Context - Number Guessing Game

## Project Overview
A Rust-based number guessing game with both CLI and web interfaces. The game generates a random number within a user-specified range and provides feedback on guesses. Supports optional guess limits that end the game when exceeded.

## Development Environment
- Even when working on Windows we work in a bash shell

## Quick Commands
```bash
# Build and test
cargo build --release
cargo test --lib
cargo clippy

# Run CLI game
cargo run -- --min 1 --max 100 --limit 10

# Run web server
cargo run -- --server --port 3000

# Format code
cargo fmt
```

## Architecture

### Core Modules
- **src/game.rs**: Pure game logic, no I/O. Contains `GuessingGame` struct and `GuessResult` enum
- **src/cli.rs**: CLI argument parsing (clap) and user input handling
- **src/web.rs**: Axum-based web server with REST API and HTMX frontend
- **src/main.rs**: Minimal entry point, mode selection (CLI vs Web)
- **static/index.html**: Web UI with HTMX for dynamic updates

### Key Design Patterns
1. **Separation of Concerns**: Game logic isolated from I/O
2. **Result Types**: Extensive use of `Result<T, String>` for error handling
3. **State Management**: Web server uses `Arc<Mutex<HashMap>>` for concurrent game sessions
4. **Validation**: Input validation at multiple layers (CLI, web, game logic)

## Important Constraints

### Numeric Limits
- Range: 0 to 1,000,000 (inclusive)
- Guess limits: Max 1000 (CLI), Max 100 (Web/API)
- Negative numbers not allowed

### Web API
- Games stored in memory (lost on restart)
- Game IDs are random u64 values
- Games auto-removed when completed
- JSON request/response format

### Security Considerations
- Input validation prevents integer overflow
- Range limits prevent DoS via large ranges
- No persistent storage (stateless between restarts)
- HTMX from CDN (consider bundling for production)

## Testing Strategy
```bash
# Unit tests in src/game.rs
cargo test --lib

# Integration test via examples
cargo run --example demo
cargo run --example web_client  # requires server running

# Manual web UI test
cargo run -- --server
# Visit http://localhost:3000
```

## Common Tasks

### Adding a New Feature
1. Update game logic in `src/game.rs`
2. Add CLI support in `src/cli.rs` if needed
3. Update web handlers in `src/web.rs`
4. Modify HTML in `static/index.html` for UI changes
5. Write tests in the relevant module
6. Update documentation (README.md, docs/api.md, docs/requirements.md)

### Modifying Game Rules
- Core logic in `game.rs::GuessingGame::make_guess()`
- Add new `GuessResult` variants as needed
- Update all match statements handling `GuessResult`

### Debugging Web Issues
- Check browser console for HTMX errors
- Use `curl` to test API endpoints directly
- Server logs to stdout by default
- Consider adding `tracing` for production

## Code Style
- Use `cargo fmt` before commits
- Follow Rust naming conventions
- Keep functions small and focused
- Document public APIs with doc comments
- Use descriptive variable names

## Known Issues & TODOs
- See docs/security-todo.md for security improvements
- No rate limiting on API endpoints
- No request size limits
- Games remain in memory until completed (potential memory leak)
- No persistent storage option

## File Structure
```
├── .claude/
│   └── claude.md    # This file
├── src/
│   ├── game.rs      # Core game logic
│   ├── cli.rs       # CLI interface
│   ├── web.rs       # Web server
│   ├── lib.rs       # Library exports
│   └── main.rs      # Entry point
├── static/
│   └── index.html   # Web UI
├── examples/        # Usage examples
├── docs/            # All documentation
│   ├── api.md       # API documentation
│   ├── architecture.md # System design
│   ├── contributing.md # Dev guidelines
│   ├── requirements.md # Technical specs
│   └── security-todo.md # Security TODOs
├── target/          # Build artifacts
└── README.md        # Main documentation
```

## Dependencies to Know
- **clap**: CLI parsing with derive macros (v4.5.45)
- **axum**: Modern web framework (v0.8.4)
- **tokio**: Async runtime (v1.47.1)
- **serde**: JSON serialization (v1.0.219)
- **tower-http**: Static file serving (v0.6.6)
- **rand**: Random number generation (v0.9.2)

## Version Information
- **Rust Version**: 1.89.0 (29483883e 2025-08-04)
- **Rust Edition**: 2024
- **Last Updated**: Dependencies updated to latest versions (Aug 2025)

## Performance Considerations
- Each game stores minimal state (5 fields)
- O(1) game lookup via HashMap
- No database queries
- Static files served directly

## Deployment Notes
- Single binary output
- No external dependencies at runtime
- Configurable port via CLI
- Binds to 0.0.0.0 (all interfaces)

## Quick Fixes

### "Game not found" errors
- Games are removed after completion
- Check if game_id is valid
- Verify game hasn't already ended

### Input validation failures
- Check range: 0 to 1,000,000
- Ensure max >= min
- Verify positive integers only

### Build issues
```bash
cargo clean
cargo update
cargo build --release
```

## Related Documentation
- **../README.md**: User-facing documentation
- **../docs/api.md**: REST API specification
- **../docs/requirements.md**: Detailed technical requirements
- **../docs/security-todo.md**: Security improvements needed
- **../docs/architecture.md**: Detailed system design
- **../docs/contributing.md**: Development guidelines
- **../docs/**: All documentation and guides
```