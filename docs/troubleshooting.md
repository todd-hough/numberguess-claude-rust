# Troubleshooting Guide

## Common Issues and Solutions

### CLI Mode Issues

#### Issue: "Invalid input. Please try again" Loop
**Symptoms**: The program keeps rejecting valid-looking input

**Solutions**:
1. Ensure you're entering only numbers (no letters or special characters)
2. Check for trailing spaces or newlines
3. Verify the number is within the valid range (0 to 1,000,000)

```bash
# Good input
50

# Bad input
50a
fifty
50.5
```

#### Issue: Command-line Arguments Not Working
**Symptoms**: `--min` or `--max` arguments are ignored

**Solutions**:
```bash
# Correct usage
cargo run -- --min 1 --max 100

# Note the -- separator when using cargo run
# Without cargo:
./number_guessing_game --min 1 --max 100
```

#### Issue: Guess Limit Not Applied
**Symptoms**: Game doesn't end after reaching guess limit

**Check**:
- Verify limit was set correctly: `--limit 10`
- Ensure limit is greater than 0
- Maximum limit is 1000 for CLI

### Web Server Issues

#### Issue: "Address already in use" Error
**Symptoms**: Server fails to start with binding error

**Solutions**:
```bash
# Find process using port 3000
lsof -i :3000  # Linux/macOS
netstat -ano | findstr :3000  # Windows

# Kill the process
kill -9 <PID>  # Linux/macOS
taskkill /PID <PID> /F  # Windows

# Or use a different port
cargo run -- --server --port 8080
```

#### Issue: Cannot Access Web Interface
**Symptoms**: Browser shows "connection refused" or "site can't be reached"

**Check**:
1. Server is running: Look for "Starting web server on http://0.0.0.0:3000"
2. Correct URL: `http://localhost:3000` (not https)
3. Firewall settings: Port 3000 might be blocked
4. Try `http://127.0.0.1:3000` instead of localhost

#### Issue: API Returns 404 for All Requests
**Symptoms**: All API calls return "Not Found"

**Solutions**:
- Verify correct endpoint: `/api/games` not `/games`
- Check game ID exists and is correct
- Ensure server is running (`--server` flag)

### API Integration Issues

#### Issue: "Game not found" Error
**Symptoms**: Valid game ID returns 404

**Causes**:
- Game was already completed (games are removed after winning/losing)
- Game ID is incorrect
- Server was restarted (games are stored in memory)

**Solution**:
Create a new game and use the fresh game ID

#### Issue: JSON Parse Errors
**Symptoms**: 400 Bad Request with parsing error

**Check your JSON**:
```bash
# Correct
curl -X POST http://localhost:3000/api/games \
  -H "Content-Type: application/json" \
  -d '{"min": 1, "max": 100}'

# Wrong - missing quotes
-d '{min: 1, max: 100}'

# Wrong - wrong content type
-H "Content-Type: text/plain"
```

#### Issue: Guess Limit Exceeds 100
**Symptoms**: API returns error about guess limit

**Solution**:
Web API has a maximum guess limit of 100. Use a value between 1-100 or omit for unlimited.

### Web UI Issues

#### Issue: Form Doesn't Submit
**Symptoms**: Clicking "Start New Game" does nothing

**Check**:
1. JavaScript console for errors (F12 in browser)
2. HTMX library loaded correctly
3. Network tab shows request being made

#### Issue: Page Refreshes Instead of Updating
**Symptoms**: Full page reload on form submission

**Causes**:
- HTMX not loaded
- Network error

**Solutions**:
1. Check internet connection (HTMX loads from CDN)
2. Check browser console for CDN errors
3. Consider downloading HTMX locally

#### Issue: Guess Counter Not Showing
**Symptoms**: No remaining guesses displayed

**Check**:
- Guess limit was set when creating game
- Browser is receiving updated HTML fragments

### Build and Development Issues

#### Issue: Build Fails with Dependency Errors
```bash
# Clean and rebuild
cargo clean
cargo update
cargo build

# Check for version conflicts
cargo tree -d
```

#### Issue: Tests Failing
```bash
# Run specific test for details
cargo test test_name -- --nocapture

# Check for race conditions
cargo test -- --test-threads=1
```

#### Issue: Clippy Warnings
```bash
# See all warnings
cargo clippy -- -W clippy::all

# Auto-fix some issues
cargo fix
```

### Performance Issues

#### Issue: High Memory Usage
**Symptoms**: Server memory grows continuously

**Causes**:
- Games not being cleaned up
- Memory leak in game storage

**Solutions**:
1. Restart server periodically
2. Monitor active games count
3. Implement game timeout (future enhancement)

#### Issue: Slow Response Times
**Check**:
1. Server CPU usage
2. Network latency
3. Number of concurrent games

**Solutions**:
- Restart server
- Use release build: `cargo build --release`
- Check for blocking operations

### Debugging Tips

#### Enable Logging
```bash
# Set Rust log level
RUST_LOG=debug cargo run -- --server

# Log to file
cargo run -- --server 2>&1 | tee debug.log
```

#### Test Individual Components
```bash
# Test game logic only
cargo test game::tests

# Test web endpoints
curl -v http://localhost:3000/api/games

# Test examples
cargo run --example demo
```

#### Check System Resources
```bash
# Memory usage
free -h  # Linux
vm_stat  # macOS

# Port availability
netstat -tulpn  # Linux
netstat -an | grep LISTEN  # macOS
```

## Error Messages Reference

| Error | Cause | Solution |
|-------|-------|----------|
| "Min and max values must be non-negative" | Negative numbers provided | Use values >= 0 |
| "Maximum must be greater than or equal to minimum" | max < min | Ensure max >= min |
| "Guess limit cannot exceed 100" | Web limit > 100 | Use limit <= 100 |
| "Game with ID X not found" | Invalid/expired game | Create new game |
| "Failed to bind to address" | Port in use | Use different port |
| "Invalid input" | Non-numeric input | Enter numbers only |

## Getting Help

If issues persist:

1. **Check Documentation**:
   - [README.md](../README.md) for basic usage
   - [api.md](api.md) for API details
   - [architecture.md](architecture.md) for system design

2. **Debugging Steps**:
   - Enable debug logging
   - Check example implementations
   - Verify with curl commands

3. **Report Issues**:
   - Include error messages
   - Provide steps to reproduce
   - List system information
   - Share relevant logs