//! API handler for guess processing.
//!
//! Processes player guesses via JSON API.

use crate::api::error::ApiError;
use crate::api::types::{GuessOutcome, MakeGuessRequest, MakeGuessResponse};
use crate::auth::AuthenticatedUser;
use crate::core::{GameId, GuessResult};
use crate::db::{DbError, GameRepository};
use crate::server::state::AppState;
use axum::{
    extract::{Path, State},
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
) -> Result<Json<MakeGuessResponse>, ApiError> {
    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
        game_id = %game_id,
        guess = payload.guess,
        "API: Processing guess"
    );

    // Make guess using transactional approach (concurrency-safe).
    // The post-guess state is also returned; the JSON API doesn't need it.
    let (result, _game) = state
        .repo
        .make_guess(game_id, payload.guess)
        .await
        .map_err(|e| {
            match &e {
                DbError::NotFound => warn!(
                    game_id = %game_id,
                    "API: Guess failed - game not found"
                ),
                e => error!(
                    game_id = %game_id,
                    guess = payload.guess,
                    error = %e,
                    "API: Failed to process guess"
                ),
            }
            ApiError::from_db_for_game(game_id)(e)
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
                result: GuessOutcome::TooLow,
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
                result: GuessOutcome::TooHigh,
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
                result: GuessOutcome::Correct,
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
                result: GuessOutcome::LimitReached,
                message: format!(
                    "Sorry, you've reached the limit of {max_guesses} guesses! The number was {number}."
                ),
                attempts: Some(max_guesses),
            }
        }
    };

    Ok(Json(response))
}
