//! Web UI handler for guess processing.
//!
//! Processes player guesses via HTML forms (HTMX).

use crate::auth::AuthenticatedUser;
use crate::core::{GameId, GuessResult};
use crate::db::{DbError, GameRepository};
use crate::server::state::AppState;
use crate::web::error::WebError;
use crate::web::templates::{GameCompleteTemplate, GuessFormTemplate};
use crate::web::types::MakeGuessRequest;
use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Response},
};
use axum_csrf::CsrfToken;
use tracing::{debug, error, info, warn};

/// Web UI handler for making a guess (HTML).
///
/// Processes a guess and returns HTML response for HTMX.
/// Requires authentication via oauth2-proxy.
///
/// # Type Parameters
/// * `R` - The repository implementation (static dispatch for zero overhead)
pub async fn make_guess_web<R: GameRepository>(
    token: CsrfToken,
    State(state): State<AppState<R>>,
    user: AuthenticatedUser,
    Path(game_id): Path<GameId>,
    Form(payload): Form<MakeGuessRequest>,
) -> Result<Response, WebError> {
    // Verify CSRF token
    if token.verify(&payload.authenticity_token).is_err() {
        warn!("Web: CSRF token verification failed");
        return Err(WebError::InvalidCsrf);
    }

    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
        game_id = %game_id,
        guess = payload.guess,
        "Web: Processing guess"
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
                    "Web: Guess failed - game not found"
                );
                WebError::GameNotFound
            }
            e => {
                error!(
                    game_id = %game_id,
                    guess = payload.guess,
                    error = %e,
                    "Web: Failed to process guess"
                );
                WebError::UpdateFailed
            }
        })?;

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
            let game = state.repo.get(game_id).await.map_err(|e| {
                error!(
                    game_id = %game_id,
                    error = %e,
                    "Web: Failed to fetch game state after guess"
                );
                WebError::UpdateFailed
            })?;

            let (min, max) = game.range();
            let max_guesses = game.max_guesses();
            let guess_count = game.guess_count();

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

            let csrf_token = token.authenticity_token().unwrap_or_default();
            let template = GuessFormTemplate {
                game_id,
                min,
                max,
                remaining_guesses,
                feedback_class,
                feedback_message,
                csrf_token,
            };
            // Return token in tuple to trigger cookie setting via IntoResponseParts
            Ok((token, template).into_response())
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
                "Web: Game completed - correct guess"
            );
            let template = GameCompleteTemplate {
                feedback_class: "correct".to_string(),
                emoji: "🎉 Congratulations! You got it!".to_string(),
                message: String::new(),
                number,
                attempts: Some(attempts),
            };
            Ok(template.into_response())
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
                "Web: Game completed - limit reached"
            );
            let template = GameCompleteTemplate {
                feedback_class: "limit-reached".to_string(),
                emoji: "❌".to_string(),
                message: format!("Sorry! You've reached the limit of {max_guesses} guesses!"),
                number,
                attempts: None,
            };
            Ok(template.into_response())
        }
    }
}
