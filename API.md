# Number Guessing Game - Web Service Documentation

## Running the Server

```bash
# Start the server on default port 3000
cargo run -- --server

# Start the server on a custom port
cargo run -- --server --port 8080
```

## Web Interface

The server provides a web-based user interface at the root path:
- **URL:** `http://localhost:3000/`
- **Features:** 
  - Interactive game UI with HTMX
  - Real-time feedback without page reloads
  - Responsive design with modern styling

## REST API Endpoints

### 1. Create a New Game

**Endpoint:** `POST /api/games`

**Request Body:**
```json
{
  "min": 1,
  "max": 100
}
```

**Response:**
```json
{
  "game_id": 12345678901234567,
  "min": 1,
  "max": 100,
  "message": "Game created! I'm thinking of a number between 1 and 100 (inclusive). Make a guess by POSTing to /api/games/12345678901234567/guess"
}
```

**Error Response (400 Bad Request):**
```json
{
  "error": "Maximum (10) must be greater than or equal to minimum (20)"
}
```

### 2. Make a Guess

**Endpoint:** `POST /api/games/{game_id}/guess`

**Request Body:**
```json
{
  "guess": 50
}
```

**Response (Too Low):**
```json
{
  "result": "too_low",
  "message": "Too low! Your guess of 50 is below the target.",
  "attempts": 2
}
```

**Response (Too High):**
```json
{
  "result": "too_high",
  "message": "Too high! Your guess of 50 is above the target.",
  "attempts": 3
}
```

**Response (Correct):**
```json
{
  "result": "correct",
  "message": "You got it! The number was 42. It took you 5 guesses.",
  "attempts": 5
}
```

**Error Response (404 Not Found):**
```json
{
  "error": "Game with ID 12345678901234567 not found"
}
```

## Example Usage with curl

```bash
# Create a new game
curl -X POST http://localhost:3000/api/games \
  -H "Content-Type: application/json" \
  -d '{"min": 1, "max": 100}'

# Make a guess (replace {game_id} with actual ID from previous response)
curl -X POST http://localhost:3000/api/games/{game_id}/guess \
  -H "Content-Type: application/json" \
  -d '{"guess": 50}'
```

## Example Usage with the Test Client

```bash
# First, start the server in one terminal
cargo run -- --server

# In another terminal, run the test client
cargo run --example web_client
```

## Notes

- **Web Interface**: Visit `http://localhost:3000/` for the interactive web UI
- **API Access**: REST API endpoints are available at `/api/*` paths
- Games are stored in memory and will be lost when the server restarts
- Each game has a unique random numeric ID that must be used for making guesses
- Games are automatically removed from memory once they are completed (correct guess)
- Multiple games can be active simultaneously for different users