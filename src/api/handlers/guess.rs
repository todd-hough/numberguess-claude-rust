//! API handler for guess processing.
//!
//! Processes player guesses via JSON API.

use crate::api::types::{ErrorResponse, MakeGuessRequest, MakeGuessResponse};
use crate::auth::AuthenticatedUser;
use crate::core::{GameId, GuessResult};
use crate::db::{DbError, GameRepository};
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use tracing::{debug, error, info, warn};

/// API handler for making a guess (JSON).
///
/// Processes a guess and returns the result as JSON.
/// Requires authentication via oauth2-proxy.
///
/// # Type Parameters
/// * `R` - The repository implementation (static dispatch for zero overhead)
pub async fn make_guess_api<R: GameRepository>(
    State(state): State<AppState<R>>,
    user: AuthenticatedUser,
    Path(game_id): Path<GameId>,
    Json(payload): Json<MakeGuessRequest>,
) -> Result<Json<MakeGuessResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
        game_id = %game_id,
        guess = payload.guess,
        "API: Processing guess"
    );

    // Make guess using transactional approach (concurrency-safe)
    let result = state
        .repo
        .make_guess(game_id, payload.guess)
        .await
        .map_err(|e| match e {
            DbError::NotFound => {
                warn!(
                    game_id = %game_id,
                    "API: Guess failed - game not found"
                );
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: format!("Game with ID {game_id} not found"),
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
                user_id = %user.user_id,
                user_email = %user.email,
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
                    "You got it! The number was {number}. It took you {attempts} guesses."
                ),
                attempts: Some(attempts),
            }
        }
        GuessResult::LimitReached {
            number,
            max_guesses,
        } => {
            info!(
                user_id = %user.user_id,
                user_email = %user.email,
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
                    "Sorry, you've reached the limit of {max_guesses} guesses! The number was {number}."
                ),
                attempts: Some(max_guesses),
            }
        }
    };

    Ok(Json(response))
}
