use axum::{
    extract::{Path, State, Form},
    http::StatusCode,
    response::{Html, Json, IntoResponse},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::Rng;
use tower_http::services::ServeDir;
use crate::game::{GuessingGame, GuessResult};

type SharedState = Arc<Mutex<GameState>>;

struct GameState {
    games: HashMap<u64, GuessingGame>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub min: i32,
    pub max: i32,
}

#[derive(Serialize)]
pub struct CreateGameResponse {
    pub game_id: u64,
    pub min: i32,
    pub max: i32,
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
        .route("/games/:game_id/guess", post(make_guess_api))
        .with_state(state.clone());

    // Web UI routes
    let web_routes = Router::new()
        .route("/game/new", post(create_game_web))
        .route("/game/:game_id/guess", post(make_guess_web))
        .with_state(state.clone());

    // Combine all routes
    let app = Router::new()
        .nest_service("/", ServeDir::new("static"))
        .nest("/api", api_routes)
        .merge(web_routes);

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
    
    let game = GuessingGame::new(payload.min, payload.max)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: e }),
            )
        })?;

    let game_id = rand::thread_rng().gen::<u64>();
    let (min, max) = game.get_range();

    state.lock().unwrap().games.insert(game_id, game);

    Ok(Json(CreateGameResponse {
        game_id,
        min,
        max,
        message: format!("Game created! I'm thinking of a number between {} and {} (inclusive). Make a guess by POSTing to /api/games/{}/guess", min, max, game_id),
    }))
}

async fn make_guess_api(
    State(state): State<SharedState>,
    Path(game_id): Path<u64>,
    Json(payload): Json<MakeGuessRequest>,
) -> Result<Json<MakeGuessResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut state = state.lock().unwrap();
    
    let game = state.games.get_mut(&game_id)
        .ok_or_else(|| {
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
            message: format!("Too low! Your guess of {} is below the target.", payload.guess),
            attempts: Some(guess_count),
        },
        GuessResult::TooHigh => MakeGuessResponse {
            result: "too_high".to_string(),
            message: format!("Too high! Your guess of {} is above the target.", payload.guess),
            attempts: Some(guess_count),
        },
        GuessResult::Correct { number, attempts } => {
            // Remove the completed game from state
            state.games.remove(&game_id);
            
            MakeGuessResponse {
                result: "correct".to_string(),
                message: format!("You got it! The number was {}. It took you {} guesses.", number, attempts),
                attempts: Some(attempts),
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
        return Html(format!(r#"
            <h1>🎯 Number Guessing Game</h1>
            <div id="setup-area">
                <div id="feedback" class="active too-high">
                    Error: Min and max values must be non-negative (>= 0)
                </div>
                <button onclick="location.reload()" class="new-game-btn">Try Again</button>
            </div>
        "#)).into_response();
    }
    
    if payload.min > 1_000_000 || payload.max > 1_000_000 {
        return Html(format!(r#"
            <h1>🎯 Number Guessing Game</h1>
            <div id="setup-area">
                <div id="feedback" class="active too-high">
                    Error: Min and max values cannot exceed 1,000,000
                </div>
                <button onclick="location.reload()" class="new-game-btn">Try Again</button>
            </div>
        "#)).into_response();
    }
    
    let game = match GuessingGame::new(payload.min, payload.max) {
        Ok(g) => g,
        Err(e) => {
            return Html(format!(r#"
                <h1>🎯 Number Guessing Game</h1>
                <div id="setup-area">
                    <div id="feedback" class="active too-high">
                        Error: {}
                    </div>
                    <button onclick="location.reload()" class="new-game-btn">Try Again</button>
                </div>
            "#, e)).into_response();
        }
    };

    let game_id = rand::thread_rng().gen::<u64>();
    let (min, max) = game.get_range();

    state.lock().unwrap().games.insert(game_id, game);

    let html = format!(
        r#"<h1>🎯 Number Guessing Game</h1>
        <div id='game-area' class='active'>
            <div class='game-info'>
                <h2>Game Started!</h2>
                <p>I'm thinking of a number between</p>
                <p class='range-display'>{} and {}</p>
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
        </div>"#, min, max, game_id, min, max);
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
            return Html(format!(r#"
                <div id="feedback" class="active too-high">
                    Game not found. It may have expired or been completed.
                </div>
                <a href="/" class="new-game-link">← Start a New Game</a>
            "#)).into_response();
        }
    };

    let result = game.make_guess(payload.guess);
    let (min, max) = game.get_range();
    
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
                
                <div id='feedback' class='active too-low'>
                    Too low! Your guess of {} is below the target.
                </div>"#, game_id, min, max, payload.guess);
            Html(html).into_response()
        },
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
                
                <div id='feedback' class='active too-high'>
                    Too high! Your guess of {} is above the target.
                </div>"#, game_id, min, max, payload.guess);
            Html(html).into_response()
        },
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
    }
}