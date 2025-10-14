//! Guess handling handlers.
//!
//! Processes player guesses for both API and web UI.

use crate::db;
use crate::game::GuessResult;
use crate::game_id::GameId;
use crate::templates::{GameCompleteTemplate, GameNotFoundTemplate, GuessFormTemplate, UpdateErrorTemplate};
use crate::web::types::{ErrorResponse, MakeGuessRequest, MakeGuessResponse};
use askama_axum::IntoResponse as AskamaIntoResponse;
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::PgPool;
use tracing::{debug, error, info, warn};

type SharedState = PgPool;

/// API handler for making a guess (JSON).
///
/// Processes a guess and returns the result as JSON.
pub async fn make_guess_api(
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

/// Web UI handler for making a guess (HTML).
///
/// Processes a guess and returns HTML response for HTMX.
pub async fn make_guess_web(
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
            let remaining_guesses = max_guesses.and_then(|limit| {
                let remaining = limit.saturating_sub(guess_count);
                if remaining > 0 { Some(remaining) } else { None }
            });

            let (feedback_class, feedback_message) = match result {
                GuessResult::TooLow => (
                    "too-low".to_string(),
                    format!(
                        "Too low! Your guess of {} is below the target.",
                        payload.guess
                    ),
                ),
                GuessResult::TooHigh => (
                    "too-high".to_string(),
                    format!(
                        "Too high! Your guess of {} is above the target.",
                        payload.guess
                    ),
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
                message: format!(
                    "Sorry! You've reached the limit of {} guesses!",
                    max_guesses
                ),
                number,
                attempts: None,
            };
            AskamaIntoResponse::into_response(template)
        }
    }
}
