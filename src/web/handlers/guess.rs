//! Web UI handler for guess processing.
//!
//! Processes player guesses via HTML forms (HTMX).

use crate::auth::AuthenticatedUser;
use crate::core::{GameId, GuessResult};
use crate::db::{DbError, GameRepository};
use crate::server::state::AppState;
use crate::web::templates::{
    GameCompleteTemplate, GameNotFoundTemplate, GuessFormTemplate, UpdateErrorTemplate,
};
use crate::web::types::MakeGuessRequest;
use axum::{
    extract::{Form, Path, State},
    response::IntoResponse,
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
) -> impl IntoResponse {
    // Verify CSRF token
    if token.verify(&payload.authenticity_token).is_err() {
        warn!("Web: CSRF token verification failed");
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Invalid CSRF token",
        ).into_response();
    }

    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
        game_id = %game_id,
        guess = payload.guess,
        "Web: Processing guess"
    );

    // Make guess using transactional approach (concurrency-safe)
    let result = match state.repo.make_guess(game_id, payload.guess).await {
        Ok(r) => r,
        Err(DbError::NotFound) => {
            warn!(
                game_id = %game_id,
                "Web: Guess failed - game not found"
            );
            return GameNotFoundTemplate.into_response();
        }
        Err(e) => {
            error!(
                game_id = %game_id,
                guess = payload.guess,
                error = %e,
                "Web: Failed to process guess"
            );
            return UpdateErrorTemplate.into_response();
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
            let game = match state.repo.get(game_id).await {
                Ok(g) => g,
                Err(e) => {
                    error!(
                        game_id = %game_id,
                        error = %e,
                        "Web: Failed to fetch game state after guess"
                    );
                    return UpdateErrorTemplate.into_response();
                }
            };

            let (min, max) = game.get_range();
            let max_guesses = game.get_max_guesses();
            let guess_count = game.get_guess_count();

            // Calculate remaining guesses
            let remaining_guesses = max_guesses.and_then(|limit| {
                let remaining = limit.saturating_sub(guess_count);
                if remaining > 0 {
                    Some(remaining)
                } else {
                    None
                }
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
            (token, template).into_response()
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
            template.into_response()
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
                message: format!(
                    "Sorry! You've reached the limit of {} guesses!",
                    max_guesses
                ),
                number,
                attempts: None,
            };
            template.into_response()
        }
    }
}