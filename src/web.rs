use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::Rng;
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

    let app = Router::new()
        .route("/", get(root))
        .route("/games", post(create_game))
        .route("/games/:game_id/guess", post(make_guess))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("Starting web server on http://{}", addr);
    println!("API Endpoints:");
    println!("  POST /games - Create a new game (body: {{\"min\": 1, \"max\": 100}})");
    println!("  POST /games/:game_id/guess - Make a guess (body: {{\"guess\": 50}})");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));
    
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|_| panic!("Failed to start server"));
}

async fn root() -> &'static str {
    "Number Guessing Game API - POST to /games to start a new game"
}

async fn create_game(
    State(state): State<SharedState>,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, (StatusCode, Json<ErrorResponse>)> {
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
        message: format!("Game created! I'm thinking of a number between {} and {} (inclusive). Make a guess by POSTing to /games/{}/guess", min, max, game_id),
    }))
}

async fn make_guess(
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
    
    let response = match result {
        GuessResult::TooLow => MakeGuessResponse {
            result: "too_low".to_string(),
            message: format!("Too low! Your guess of {} is below the target.", payload.guess),
            attempts: None,
        },
        GuessResult::TooHigh => MakeGuessResponse {
            result: "too_high".to_string(),
            message: format!("Too high! Your guess of {} is above the target.", payload.guess),
            attempts: None,
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