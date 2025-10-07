use crate::db;
use crate::game::GuessResult;
use crate::game_id::GameId;
use crate::templates::*;
use crate::validators;
use askama_axum::IntoResponse as AskamaIntoResponse;
use axum::{
    Router,
    extract::{Form, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::PgPool;
use tower_http::services::ServeDir;

type SharedState = PgPool;

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
    pub game_id: GameId,
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

pub async fn run_server(pool: PgPool, port: u16) {
    let health_port = 8081;

    // API routes
    let api_routes = Router::new()
        .route("/games", post(create_game_api))
        .route("/games/{game_id}/guess", post(make_guess_api))
        .with_state(pool.clone());

    // Web UI routes
    let web_routes = Router::new()
        .route("/game/new", post(create_game_web))
        .route("/game/{game_id}/guess", post(make_guess_web))
        .with_state(pool.clone());

    // Main application routes
    let app = Router::new()
        .nest("/api", api_routes)
        .merge(web_routes)
        .fallback_service(ServeDir::new("static"));

    // Health check server (separate port)
    let health_app = Router::new()
        .route("/health", get(health_check))
        .with_state(pool.clone());

    let main_addr = format!("0.0.0.0:{}", port);
    let health_addr = format!("0.0.0.0:{}", health_port);

    println!("Starting web server on http://{}", main_addr);
    println!("Web Interface: http://{}/", main_addr);
    println!("API Endpoints:");
    println!("  POST /api/games - Create a new game");
    println!("  POST /api/games/:game_id/guess - Make a guess");
    println!("Health Check: http://{}/health", health_addr);

    let main_listener = tokio::net::TcpListener::bind(&main_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", main_addr));

    let health_listener = tokio::net::TcpListener::bind(&health_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", health_addr));

    // Run both servers concurrently with graceful shutdown
    tokio::select! {
        result = axum::serve(main_listener, app).with_graceful_shutdown(shutdown_signal()) => {
            result.unwrap_or_else(|_| panic!("Main server failed"));
        }
        result = axum::serve(health_listener, health_app).with_graceful_shutdown(shutdown_signal()) => {
            result.unwrap_or_else(|_| panic!("Health check server failed"));
        }
    }
}

// Graceful Shutdown Signal Handler

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
    println!("Shutting down gracefully...");
}

// Health Check Handler

async fn health_check(State(pool): State<SharedState>) -> StatusCode {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

// API Handlers (JSON responses)

async fn create_game_api(
    State(pool): State<SharedState>,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate range using shared validator
    if let Err(e) = validators::validate_range(payload.min, payload.max) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        ));
    }

    // Validate guess limit using shared validator
    let guess_limit = if let Some(limit) = payload.max_guesses {
        match validators::validate_guess_limit(limit, validators::MAX_WEB_GUESS_LIMIT) {
            Ok(validated) => validated,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: e }),
                ));
            }
        }
    } else {
        None
    };

    // Create game in database
    let game_id = db::create_game(&pool, payload.min, payload.max, guess_limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let message = match guess_limit {
        Some(limit) => format!(
            "Game created! I'm thinking of a number between {} and {} (inclusive). You have {} guesses. Make a guess by POSTing to /api/games/{}/guess",
            payload.min, payload.max, limit, game_id
        ),
        None => format!(
            "Game created! I'm thinking of a number between {} and {} (inclusive). Make a guess by POSTing to /api/games/{}/guess",
            payload.min, payload.max, game_id
        ),
    };

    Ok(Json(CreateGameResponse {
        game_id,
        min: payload.min,
        max: payload.max,
        max_guesses: guess_limit,
        message,
    }))
}

async fn make_guess_api(
    State(pool): State<SharedState>,
    Path(game_id): Path<GameId>,
    Json(payload): Json<MakeGuessRequest>,
) -> Result<Json<MakeGuessResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Make guess using transactional approach (concurrency-safe)
    let result = db::make_guess_transactional(&pool, game_id, payload.guess)
        .await
        .map_err(|e| match e {
            db::DbError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Game with ID {} not found", game_id),
                }),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            ),
        })?;

    let response = match result {
        GuessResult::TooLow => {
            MakeGuessResponse {
                result: "too_low".to_string(),
                message: format!(
                    "Too low! Your guess of {} is below the target.",
                    payload.guess
                ),
                attempts: None, // Attempts not included for ongoing game
            }
        }
        GuessResult::TooHigh => {
            MakeGuessResponse {
                result: "too_high".to_string(),
                message: format!(
                    "Too high! Your guess of {} is above the target.",
                    payload.guess
                ),
                attempts: None, // Attempts not included for ongoing game
            }
        }
        GuessResult::Correct { number, attempts } => {
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
    State(pool): State<SharedState>,
    Form(payload): Form<CreateGameRequest>,
) -> impl IntoResponse {
    // Validate range using shared validator
    if let Err(e) = validators::validate_range(payload.min, payload.max) {
        let template = ErrorTemplate {
            error_message: &e,
        };
        return AskamaIntoResponse::into_response(template);
    }

    // Validate guess limit using shared validator
    let guess_limit = if let Some(limit) = payload.max_guesses {
        match validators::validate_guess_limit(limit, validators::MAX_WEB_GUESS_LIMIT) {
            Ok(validated) => validated,
            Err(e) => {
                let template = ErrorTemplate {
                    error_message: &e,
                };
                return AskamaIntoResponse::into_response(template);
            }
        }
    } else {
        None
    };

    // Create game in database
    let game_id = match db::create_game(&pool, payload.min, payload.max, guess_limit).await {
        Ok(id) => id,
        Err(e) => {
            let err_str = e.to_string();
            let template = ErrorTemplate {
                error_message: &err_str,
            };
            return AskamaIntoResponse::into_response(template);
        }
    };

    let template = GameStartedTemplate {
        game_id,
        min: payload.min,
        max: payload.max,
        max_guesses: guess_limit,
    };
    AskamaIntoResponse::into_response(template)
}

async fn make_guess_web(
    State(pool): State<SharedState>,
    Path(game_id): Path<GameId>,
    Form(payload): Form<MakeGuessRequest>,
) -> impl IntoResponse {
    // Make guess using transactional approach (concurrency-safe)
    let result = match db::make_guess_transactional(&pool, game_id, payload.guess).await {
        Ok(r) => r,
        Err(db::DbError::NotFound) => {
            return AskamaIntoResponse::into_response(GameNotFoundTemplate);
        }
        Err(e) => {
            eprintln!("Failed to make guess for game {}: {}", game_id, e);
            return AskamaIntoResponse::into_response(UpdateErrorTemplate);
        }
    };

    match result {
        GuessResult::TooLow | GuessResult::TooHigh => {
            // For ongoing games, fetch current state for display
            let game = match db::get_game(&pool, game_id).await {
                Ok(g) => g,
                Err(_) => {
                    return AskamaIntoResponse::into_response(UpdateErrorTemplate);
                }
            };

            let (min, max) = game.get_range();
            let max_guesses = game.get_max_guesses();
            let guess_count = game.get_guess_count();

            // Calculate remaining guesses
            let remaining_guesses = max_guesses.map(|limit| {
                let remaining = limit.saturating_sub(guess_count);
                if remaining > 0 {
                    Some(remaining)
                } else {
                    None
                }
            }).flatten();

            let (feedback_class, feedback_message) = match result {
                GuessResult::TooLow => (
                    "too-low".to_string(),
                    format!("Too low! Your guess of {} is below the target.", payload.guess),
                ),
                GuessResult::TooHigh => (
                    "too-high".to_string(),
                    format!("Too high! Your guess of {} is above the target.", payload.guess),
                ),
                _ => unreachable!(),
            };

            let template = GuessFormTemplate {
                game_id,
                min,
                max,
                remaining_guesses,
                feedback_class,
                feedback_message,
            };
            AskamaIntoResponse::into_response(template)
        }
        GuessResult::Correct { number, attempts } => {
            let template = GameCompleteTemplate {
                feedback_class: "correct".to_string(),
                emoji: "🎉 Congratulations! You got it!".to_string(),
                message: String::new(),
                number,
                attempts: Some(attempts),
            };
            AskamaIntoResponse::into_response(template)
        }
        GuessResult::LimitReached {
            number,
            max_guesses,
        } => {
            let template = GameCompleteTemplate {
                feedback_class: "limit-reached".to_string(),
                emoji: "❌".to_string(),
                message: format!("Sorry! You've reached the limit of {} guesses!", max_guesses),
                number,
                attempts: None,
            };
            AskamaIntoResponse::into_response(template)
        }
    }
}
