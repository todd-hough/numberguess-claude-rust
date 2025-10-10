# Contributing Guidelines

Thank you for your interest in contributing to the Number Guessing Game project! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites
- Rust 1.89.0+ (check with `rustc --version`) - Required for Rust Edition 2024
- Cargo (comes with Rust)
- Git
- PostgreSQL 12+ (for web server development)

### Setting Up Development Environment
```bash
# Clone the repository
git clone <repository-url>
cd number_guessing_game

# Build the project
cargo build

# Run tests
cargo test

# Run with formatting check
cargo fmt --check

# Run linter
cargo clippy
```

## Development Workflow

### 1. Before You Start
- Check existing issues for similar work
- For major changes, open an issue first to discuss
- Ensure your Rust toolchain is up to date

### 2. Making Changes

#### Code Style
- Run `cargo fmt` before committing
- Follow Rust naming conventions:
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants
- Keep functions small and focused (< 50 lines preferred)
- Add doc comments for public APIs

#### Commit Messages
Follow conventional commit format:
```
type(scope): brief description

Longer explanation if needed.

Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions or changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

Examples:
```
feat(game): add hint system for difficult ranges
fix(web): correct guess limit validation for API
docs(readme): update installation instructions
test(game): add edge cases for guess limit feature
```

### 3. Testing

#### Running Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test game::tests

# Run with output
cargo test -- --nocapture

# Run examples
cargo run --example demo
```

#### Writing Tests
- Add unit tests in the same file as the code
- Use descriptive test names
- Test edge cases and error conditions
- Aim for high code coverage

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let game = GuessingGame::new(1, 10).unwrap();
        
        // Act
        let result = game.make_guess(5);
        
        // Assert
        assert!(!result.is_correct());
    }
}
```

### 4. Documentation

#### Code Documentation
- Add doc comments (`///`) for public items
- Include examples in doc comments
- Document panic conditions
- Document error types

Example:
```rust
/// Creates a new guessing game with the specified range.
/// 
/// # Arguments
/// * `min` - The minimum value (inclusive)
/// * `max` - The maximum value (inclusive)
/// 
/// # Returns
/// * `Ok(GuessingGame)` - A new game instance
/// * `Err(String)` - Error message if validation fails
/// 
/// # Example
/// ```
/// let game = GuessingGame::new(1, 100).unwrap();
/// ```
pub fn new(min: i32, max: i32) -> Result<Self, String> {
    // Implementation
}
```

#### Updating Documentation
When adding features or making changes:
1. Update relevant code comments
2. Update README.md if user-facing
3. Update api.md for API changes
4. Update architecture.md for structural changes
5. Update .claude/claude.md with important context

### 5. Pull Request Process

#### Before Submitting
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy` and address warnings
- [ ] Run `cargo test` - all tests pass
- [ ] Update documentation
- [ ] Commit messages follow format
- [ ] Branch is up to date with main

#### PR Description Template
```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings
```

## Project Structure

### File Organization
```
src/
├── game.rs       # Core game logic (keep pure, no I/O)
├── cli.rs        # CLI interface (user interaction)
├── web.rs        # Web server (API + UI endpoints)
├── lib.rs        # Library exports
└── main.rs       # Entry point (minimal logic)
```

### Adding New Features

#### To Game Logic
1. Modify `src/game.rs`
2. Add tests in same file
3. Update all match statements if adding enum variants
4. Keep functions pure (no I/O)

#### To CLI Interface
1. Add new fields to `Cli` struct if needed
2. Update input validation functions
3. Modify main game loop if needed
4. Test manually with various inputs

#### To Web Interface
1. Add new endpoints in `src/web.rs`
2. Update request/response types
3. Modify `static/index.html` for UI changes
4. Test with curl and browser
5. Update api.md documentation

## Common Tasks

### Adding a New Game Option
```rust
// 1. Add to GuessingGame struct
pub struct GuessingGame {
    // ... existing fields
    pub new_option: bool,
}

// 2. Update constructor
pub fn new(/* params */) -> Result<Self, String> {
    // Validation
    Ok(GuessingGame {
        // ... existing fields
        new_option: false,
    })
}

// 3. Add CLI flag in Cli struct
#[arg(long, help = "Enable new option")]
pub new_option: bool,

// 4. Update web request types
pub struct CreateGameRequest {
    // ... existing fields
    pub new_option: Option<bool>,
}
```

### Adding API Endpoint
```rust
// 1. Define handler
async fn new_endpoint(
    State(state): State<SharedState>,
    Json(payload): Json<RequestType>,
) -> Result<Json<ResponseType>, (StatusCode, Json<ErrorResponse>)> {
    // Implementation
}

// 2. Add to router
let api_routes = Router::new()
    .route("/games/new-endpoint", post(new_endpoint))
    // ... existing routes
```

## Code Review Guidelines

### For Reviewers
- Check for correctness first
- Verify tests are included
- Ensure documentation is updated
- Look for performance issues
- Check error handling
- Verify no security issues introduced

### For Contributors
- Respond to feedback constructively
- Make requested changes promptly
- Ask for clarification if needed
- Update PR based on feedback
- Don't force push after review starts

## Troubleshooting

### Common Issues

#### Build Failures
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for dependency conflicts
cargo tree
```

#### Test Failures
```bash
# Run specific failing test
cargo test test_name -- --exact

# Check for race conditions
cargo test -- --test-threads=1
```

#### Format/Lint Issues
```bash
# Auto-fix formatting
cargo fmt

# Show clippy suggestions
cargo clippy -- -W clippy::all
```

## Release Process

### Version Bumping
1. Update version in `Cargo.toml`
2. Update CHANGELOG (if exists)
3. Create git tag: `git tag v0.2.0`
4. Build release: `cargo build --release`

### Pre-release Checklist
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Version bumped
- [ ] CHANGELOG updated
- [ ] Release notes prepared

## Getting Help

### Resources
- Rust Book: https://doc.rust-lang.org/book/
- Rust By Example: https://doc.rust-lang.org/rust-by-example/
- Axum Docs: https://docs.rs/axum/
- Clap Docs: https://docs.rs/clap/

### Contact
- Open an issue for bugs
- Start a discussion for features
- Check existing issues first

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.