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
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tower_http::LatencyUnit;
use tracing::{info, error, debug, warn};

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

    debug!(port = port, health_port = health_port, "Configuring web server");

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

    // Main application routes with tracing middleware
    let app = Router::new()
        .nest("/api", api_routes)
        .merge(web_routes)
        .fallback_service(ServeDir::new("static"))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO).latency_unit(LatencyUnit::Millis))
        );

    // Health check server (separate port)
    let health_app = Router::new()
        .route("/health", get(health_check))
        .with_state(pool.clone());

    let main_addr = format!("0.0.0.0:{}", port);
    let health_addr = format!("0.0.0.0:{}", health_port);

    let main_listener = tokio::net::TcpListener::bind(&main_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", main_addr));

    let health_listener = tokio::net::TcpListener::bind(&health_addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", health_addr));

    // Log server startup info to stderr (structured logs)
    info!(
        main_addr = %main_addr,
        health_addr = %health_addr,
        main_port = port,
        health_port = health_port,
        "Starting web server"
    );
    info!(url = %format!("http://{}/", main_addr), "Web Interface available");
    info!("API Endpoints available");
    debug!("  POST /api/games - Create a new game");
    debug!("  POST /api/games/:game_id/guess - Make a guess");
    info!(url = %format!("http://{}/health", health_addr), "Health check available");

    // Create channels to signal when each server task has started
    let (main_ready_tx, main_ready_rx) = tokio::sync::oneshot::channel();
    let (health_ready_tx, health_ready_rx) = tokio::sync::oneshot::channel();

    // Spawn main server task
    let main_server = tokio::spawn(async move {
        debug!("Main server task started, beginning to accept connections");
        // Signal that we're about to start serving (listener is already bound and ready)
        let _ = main_ready_tx.send(());

        axum::serve(main_listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap_or_else(|_| panic!("Main server failed"));
    });

    // Spawn health check server task
    let health_server = tokio::spawn(async move {
        debug!("Health check server task started, beginning to accept connections");
        // Signal that we're about to start serving (listener is already bound and ready)
        let _ = health_ready_tx.send(());

        axum::serve(health_listener, health_app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap_or_else(|_| panic!("Health check server failed"));
    });

    // Wait for both servers to signal they're ready
    debug!("Waiting for server tasks to start accepting connections...");
    let _ = main_ready_rx.await;
    let _ = health_ready_rx.await;
    debug!("Both server tasks are now accepting connections");

    // Emit ready marker to stdout for tests/orchestration tools
    // (stdout is for program output, stderr is for logs)
    // This is only emitted AFTER both server tasks have started their accept loops
    println!("READY");

    // Wait for both servers to complete
    let _ = tokio::join!(main_server, health_server);
}

// Graceful Shutdown Signal Handler

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
    info!("Shutting down gracefully");
}

// Health Check Handler

async fn health_check(State(pool): State<SharedState>) -> StatusCode {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => {
            debug!("Health check passed");
            StatusCode::OK
        }
        Err(e) => {
            error!(error = %e, "Health check failed: database unavailable");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

// API Handlers (JSON responses)

async fn create_game_api(
    State(pool): State<SharedState>,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        min = payload.min,
        max = payload.max,
        max_guesses = ?payload.max_guesses,
        "API: Creating new game"
    );

    // Validate range using shared validator
    if let Err(e) = validators::validate_range(payload.min, payload.max) {
        warn!(
            min = payload.min,
            max = payload.max,
            error = %e,
            "API: Game creation failed - invalid range"
        );
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
                warn!(
                    limit = limit,
                    error = %e,
                    "API: Game creation failed - invalid guess limit"
                );
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
        .map_err(|e| {
            error!(
                min = payload.min,
                max = payload.max,
                max_guesses = ?guess_limit,
                error = %e,
                "API: Failed to create game in database"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() }))
        })?;

    info!(
        game_id = %game_id,
        min = payload.min,
        max = payload.max,
        max_guesses = ?guess_limit,
        "API: Game created successfully"
    );

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
    debug!(
        game_id = %game_id,
        guess = payload.guess,
        "API: Processing guess"
    );

    // Make guess using transactional approach (concurrency-safe)
    let result = db::make_guess_transactional(&pool, game_id, payload.guess)
        .await
        .map_err(|e| match e {
            db::DbError::NotFound => {
                warn!(
                    game_id = %game_id,
                    "API: Guess failed - game not found"
                );
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: format!("Game with ID {} not found", game_id),
                    }),
                )
            }
            _ => {
                error!(
                    game_id = %game_id,
                    guess = payload.guess,
                    error = %e,
                    "API: Failed to process guess"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: e.to_string(),
                    }),
                )
            }
        })?;

    let response = match result {
        GuessResult::TooLow => {
            debug!(
                game_id = %game_id,
                guess = payload.guess,
                result = "too_low",
                "API: Guess result"
            );
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
            debug!(
                game_id = %game_id,
                guess = payload.guess,
                result = "too_high",
                "API: Guess result"
            );
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
            info!(
                game_id = %game_id,
                guess = payload.guess,
                number = number,
                attempts = attempts,
                result = "correct",
                "API: Game completed - correct guess"
            );
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
            info!(
                game_id = %game_id,
                guess = payload.guess,
                number = number,
                max_guesses = max_guesses,
                result = "limit_reached",
                "API: Game completed - limit reached"
            );
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
    debug!(
        min = payload.min,
        max = payload.max,
        max_guesses = ?payload.max_guesses,
        "Web: Creating new game"
    );

    // Validate range using shared validator
    if let Err(e) = validators::validate_range(payload.min, payload.max) {
        warn!(
            min = payload.min,
            max = payload.max,
            error = %e,
            "Web: Game creation failed - invalid range"
        );
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
                warn!(
                    limit = limit,
                    error = %e,
                    "Web: Game creation failed - invalid guess limit"
                );
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
        Ok(id) => {
            info!(
                game_id = %id,
                min = payload.min,
                max = payload.max,
                max_guesses = ?guess_limit,
                "Web: Game created successfully"
            );
            id
        }
        Err(e) => {
            error!(
                min = payload.min,
                max = payload.max,
                max_guesses = ?guess_limit,
                error = %e,
                "Web: Failed to create game in database"
            );
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
    debug!(
        game_id = %game_id,
        guess = payload.guess,
        "Web: Processing guess"
    );

    // Make guess using transactional approach (concurrency-safe)
    let result = match db::make_guess_transactional(&pool, game_id, payload.guess).await {
        Ok(r) => r,
        Err(db::DbError::NotFound) => {
            warn!(
                game_id = %game_id,
                "Web: Guess failed - game not found"
            );
            return AskamaIntoResponse::into_response(GameNotFoundTemplate);
        }
        Err(e) => {
            error!(
                game_id = %game_id,
                guess = payload.guess,
                error = %e,
                "Web: Failed to process guess"
            );
            return AskamaIntoResponse::into_response(UpdateErrorTemplate);
        }
    };

    match result {
        GuessResult::TooLow | GuessResult::TooHigh => {
            let result_str = match result {
                GuessResult::TooLow => "too_low",
                GuessResult::TooHigh => "too_high",
                _ => unreachable!(),
            };
            debug!(
                game_id = %game_id,
                guess = payload.guess,
                result = result_str,
                "Web: Guess result"
            );

            // For ongoing games, fetch current state for display
            let game = match db::get_game(&pool, game_id).await {
                Ok(g) => g,
                Err(e) => {
                    error!(
                        game_id = %game_id,
                        error = %e,
                        "Web: Failed to fetch game state after guess"
                    );
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
            info!(
                game_id = %game_id,
                guess = payload.guess,
                number = number,
                attempts = attempts,
                result = "correct",
                "Web: Game completed - correct guess"
            );
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
            info!(
                game_id = %game_id,
                guess = payload.guess,
                number = number,
                max_guesses = max_guesses,
                result = "limit_reached",
                "Web: Game completed - limit reached"
            );
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
