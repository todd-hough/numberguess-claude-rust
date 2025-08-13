use crate::game::{GuessResult, GuessingGame};
use axum::{
    Router,
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::post,
};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

type SharedState = Arc<Mutex<GameState>>;

struct GameState {
    games: HashMap<u64, GuessingGame>,
}

fn deserialize_option_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(ref s) if s.is_empty() => Ok(None),
        Some(s) => s.parse::<u32>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub min: i32,
    pub max: i32,
    #[serde(default, deserialize_with = "deserialize_option_u32")]
    pub max_guesses: Option<u32>,
}

#[derive(Serialize)]
pub struct CreateGameResponse {
    pub game_id: u64,
    pub min: i32,
    pub max: i32,
    pub max_guesses: Option<u32>,
    pub message: String,
}

#[derive(Deserialize)]
pub struct MakeGuessRequest {
    pub guess: i32,
}

#[derive(Serialize)]
pub struct MakeGuessResponse {
    pub result: String,
    pub message: String,
    pub attempts: Option<u32>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn run_server(port: u16) {
    let state = Arc::new(Mutex::new(GameState {
        games: HashMap::new(),
    }));

    // API routes
    let api_routes = Router::new()
        .route("/games", post(create_game_api))
        .route("/games/{game_id}/guess", post(make_guess_api))
        .with_state(state.clone());

    // Web UI routes
    let web_routes = Router::new()
        .route("/game/new", post(create_game_web))
        .route("/game/{game_id}/guess", post(make_guess_web))
        .with_state(state.clone());

    // Combine all routes
    let app = Router::new()
        .nest("/api", api_routes)
        .merge(web_routes)
        .fallback_service(ServeDir::new("static"));

    let addr = format!("0.0.0.0:{}", port);
    println!("Starting web server on http://{}", addr);
    println!("Web Interface: http://{}/", addr);
    println!("API Endpoints:");
    println!("  POST /api/games - Create a new game");
    println!("  POST /api/games/:game_id/guess - Make a guess");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|_| panic!("Failed to start server"));
}

// API Handlers (JSON responses)

async fn create_game_api(
    State(state): State<SharedState>,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate input before creating game
    if payload.min < 0 || payload.max < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Min and max values must be non-negative (>= 0)".to_string(),
            }),
        ));
    }

    if payload.min > 1_000_000 || payload.max > 1_000_000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Min and max values cannot exceed 1,000,000".to_string(),
            }),
        ));
    }

    // Validate guess limit (max 100 for web UI)
    let guess_limit = match payload.max_guesses {
        Some(0) => None, // Treat 0 as no limit
        Some(limit) if limit > 100 => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Guess limit cannot exceed 100".to_string(),
                }),
            ));
        }
        Some(limit) => Some(limit),
        None => None,
    };

    let game = GuessingGame::new_with_limit(payload.min, payload.max, guess_limit)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let game_id = rand::rng().random::<u64>();
    let (min, max) = game.get_range();
    let max_guesses = game.get_max_guesses();

    state.lock().unwrap().games.insert(game_id, game);

    let message = match max_guesses {
        Some(limit) => format!(
            "Game created! I'm thinking of a number between {} and {} (inclusive). You have {} guesses. Make a guess by POSTing to /api/games/{}/guess",
            min, max, limit, game_id
        ),
        None => format!(
            "Game created! I'm thinking of a number between {} and {} (inclusive). Make a guess by POSTing to /api/games/{}/guess",
            min, max, game_id
        ),
    };

    Ok(Json(CreateGameResponse {
        game_id,
        min,
        max,
        max_guesses,
        message,
    }))
}

async fn make_guess_api(
    State(state): State<SharedState>,
    Path(game_id): Path<u64>,
    Json(payload): Json<MakeGuessRequest>,
) -> Result<Json<MakeGuessResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut state = state.lock().unwrap();

    let game = state.games.get_mut(&game_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Game with ID {} not found", game_id),
            }),
        )
    })?;

    let result = game.make_guess(payload.guess);

    let guess_count = game.get_guess_count();

    let response = match result {
        GuessResult::TooLow => MakeGuessResponse {
            result: "too_low".to_string(),
            message: format!(
                "Too low! Your guess of {} is below the target.",
                payload.guess
            ),
            attempts: Some(guess_count),
        },
        GuessResult::TooHigh => MakeGuessResponse {
            result: "too_high".to_string(),
            message: format!(
                "Too high! Your guess of {} is above the target.",
                payload.guess
            ),
            attempts: Some(guess_count),
        },
        GuessResult::Correct { number, attempts } => {
            // Remove the completed game from state
            state.games.remove(&game_id);

            MakeGuessResponse {
                result: "correct".to_string(),
                message: format!(
                    "You got it! The number was {}. It took you {} guesses.",
                    number, attempts
                ),
                attempts: Some(attempts),
            }
        }
        GuessResult::LimitReached {
            number,
            max_guesses,
        } => {
            // Remove the completed game from state
            state.games.remove(&game_id);

            MakeGuessResponse {
                result: "limit_reached".to_string(),
                message: format!(
                    "Sorry, you've reached the limit of {} guesses! The number was {}.",
                    max_guesses, number
                ),
                attempts: Some(max_guesses),
            }
        }
    };

    Ok(Json(response))
}

// Web UI Handlers (HTML responses for HTMX)

async fn create_game_web(
    State(state): State<SharedState>,
    Form(payload): Form<CreateGameRequest>,
) -> impl IntoResponse {
    // Validate input before creating game
    if payload.min < 0 || payload.max < 0 {
        return Html(
            r#"
            <h1>🎯 Number Guessing Game</h1>
            <div id="setup-area">
                <div id="feedback" class="active too-high">
                    Error: Min and max values must be non-negative (>= 0)
                </div>
                <button onclick="location.reload()" class="new-game-btn">Try Again</button>
            </div>
        "#
            .to_string(),
        )
        .into_response();
    }

    if payload.min > 1_000_000 || payload.max > 1_000_000 {
        return Html(
            r#"
            <h1>🎯 Number Guessing Game</h1>
            <div id="setup-area">
                <div id="feedback" class="active too-high">
                    Error: Min and max values cannot exceed 1,000,000
                </div>
                <button onclick="location.reload()" class="new-game-btn">Try Again</button>
            </div>
        "#
            .to_string(),
        )
        .into_response();
    }

    // Validate guess limit (max 100 for web UI)
    let guess_limit = match payload.max_guesses {
        Some(0) => None, // Treat 0 as no limit
        Some(limit) if limit > 100 => {
            return Html(
                r#"
                <h1>🎯 Number Guessing Game</h1>
                <div id="setup-area">
                    <div id="feedback" class="active too-high">
                        Error: Guess limit cannot exceed 100
                    </div>
                    <button onclick="location.reload()" class="new-game-btn">Try Again</button>
                </div>
            "#
                .to_string(),
            )
            .into_response();
        }
        Some(limit) => Some(limit),
        None => None,
    };

    let game = match GuessingGame::new_with_limit(payload.min, payload.max, guess_limit) {
        Ok(g) => g,
        Err(e) => {
            return Html(format!(
                r#"
                <h1>🎯 Number Guessing Game</h1>
                <div id="setup-area">
                    <div id="feedback" class="active too-high">
                        Error: {}
                    </div>
                    <button onclick="location.reload()" class="new-game-btn">Try Again</button>
                </div>
            "#,
                e
            ))
            .into_response();
        }
    };

    let game_id = rand::rng().random::<u64>();
    let (min, max) = game.get_range();
    let max_guesses = game.get_max_guesses();

    state.lock().unwrap().games.insert(game_id, game);

    let guess_info = match max_guesses {
        Some(limit) => format!(
            "<p>You have <strong>{}</strong> guesses to find it!</p>",
            limit
        ),
        None => String::new(),
    };

    let html = format!(
        r#"<h1>🎯 Number Guessing Game</h1>
        <div id='game-area' class='active'>
            <div class='game-info'>
                <h2>Game Started!</h2>
                <p>I'm thinking of a number between</p>
                <p class='range-display'>{} and {}</p>
                {}
            </div>
            
            <div id='game-content'>
                <form class='guess-form' 
                      hx-post='/game/{}/guess' 
                      hx-target='#game-content' 
                      hx-swap='innerHTML'>
                    <div class='guess-input-group'>
                        <input type='number' 
                               name='guess' 
                               min='{}' 
                               max='{}' 
                               placeholder='Enter your guess' 
                               required 
                               autofocus>
                        <button type='submit'>
                            Guess
                            <span class='htmx-indicator'>
                                <span class='spinner'></span>
                            </span>
                        </button>
                    </div>
                </form>
                
                <div id='game-feedback'>
                    <!-- Feedback will appear here -->
                </div>
            </div>
        </div>"#,
        min, max, guess_info, game_id, min, max
    );
    Html(html).into_response()
}

async fn make_guess_web(
    State(state): State<SharedState>,
    Path(game_id): Path<u64>,
    Form(payload): Form<MakeGuessRequest>,
) -> impl IntoResponse {
    let mut state = state.lock().unwrap();

    let game = match state.games.get_mut(&game_id) {
        Some(g) => g,
        None => {
            return Html(
                r#"
                <div id="feedback" class="active too-high">
                    Game not found. It may have expired or been completed.
                </div>
                <a href="/" class="new-game-link">← Start a New Game</a>
            "#
                .to_string(),
            )
            .into_response();
        }
    };

    let result = game.make_guess(payload.guess);
    let (min, max) = game.get_range();
    let max_guesses = game.get_max_guesses();
    let guess_count = game.get_guess_count();

    // Calculate remaining guesses
    let remaining_info = match max_guesses {
        Some(limit) => {
            let remaining = limit.saturating_sub(guess_count);
            if remaining > 0 {
                format!(
                    "<p style='color: #666; font-weight: 600;'>Guesses remaining: {}</p>",
                    remaining
                )
            } else {
                String::new()
            }
        }
        None => String::new(),
    };

    match result {
        GuessResult::TooLow => {
            let html = format!(
                r#"<form class='guess-form' 
                      hx-post='/game/{}/guess' 
                      hx-target='#game-content' 
                      hx-swap='innerHTML'>
                    <div class='guess-input-group'>
                        <input type='number' 
                               name='guess' 
                               min='{}' 
                               max='{}' 
                               placeholder='Enter your guess' 
                               value=''
                               required 
                               autofocus>
                        <button type='submit'>
                            Guess
                            <span class='htmx-indicator'>
                                <span class='spinner'></span>
                            </span>
                        </button>
                    </div>
                </form>
                
                {}
                <div id='feedback' class='active too-low'>
                    Too low! Your guess of {} is below the target.
                </div>"#,
                game_id, min, max, remaining_info, payload.guess
            );
            Html(html).into_response()
        }
        GuessResult::TooHigh => {
            let html = format!(
                r#"<form class='guess-form' 
                      hx-post='/game/{}/guess' 
                      hx-target='#game-content' 
                      hx-swap='innerHTML'>
                    <div class='guess-input-group'>
                        <input type='number' 
                               name='guess' 
                               min='{}' 
                               max='{}' 
                               placeholder='Enter your guess' 
                               value=''
                               required 
                               autofocus>
                        <button type='submit'>
                            Guess
                            <span class='htmx-indicator'>
                                <span class='spinner'></span>
                            </span>
                        </button>
                    </div>
                </form>
                
                {}
                <div id='feedback' class='active too-high'>
                    Too high! Your guess of {} is above the target.
                </div>"#,
                game_id, min, max, remaining_info, payload.guess
            );
            Html(html).into_response()
        }
        GuessResult::Correct { number, attempts } => {
            // Remove the completed game
            state.games.remove(&game_id);

            Html(format!(r#"
                <div id="feedback" class="active correct">
                    🎉 Congratulations! You got it!<br>
                    The number was {}.<br>
                    It took you {} {} to find it!
                </div>
                <button onclick="window.location.href='/'" class="new-game-btn">Start New Game</button>
            "#, number, attempts, if attempts == 1 { "guess" } else { "guesses" })).into_response()
        }
        GuessResult::LimitReached {
            number,
            max_guesses,
        } => {
            // Remove the completed game
            state.games.remove(&game_id);

            Html(format!(r#"
                <div id="feedback" class="active limit-reached" style="color: #e74c3c;">
                    ❌ Sorry! You've reached the limit of {} guesses!<br>
                    The number was {}.<br>
                    Better luck next time!
                </div>
                <button onclick="window.location.href='/'" class="new-game-btn">Start New Game</button>
            "#, max_guesses, number)).into_response()
        }
    }
}
