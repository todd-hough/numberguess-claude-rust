//! Web UI handler for guess processing.
//!
//! Processes player guesses via HTML forms (HTMX).

use crate::auth::AuthenticatedUser;
use crate::core::{GameId, GuessResult, GuessingGame};
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

    // Make guess using transactional approach (concurrency-safe). The
    // post-guess state comes back from the same transaction, so no follow-up
    // fetch is needed (a concurrent request could delete the game between
    // two calls).
    let (result, game) = state
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
        GuessResult::TooLow => Ok(render_guess_form(
            token,
            game_id,
            &game,
            payload.guess,
            "too_low",
            "too-low",
            format!(
                "Too low! Your guess of {} is below the target.",
                payload.guess
            ),
        )),
        GuessResult::TooHigh => Ok(render_guess_form(
            token,
            game_id,
            &game,
            payload.guess,
            "too_high",
            "too-high",
            format!(
                "Too high! Your guess of {} is above the target.",
                payload.guess
            ),
        )),
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

/// Render the guess form for an ongoing game (too-low / too-high feedback),
/// using the post-guess state returned by the repository.
fn render_guess_form(
    token: CsrfToken,
    game_id: GameId,
    game: &GuessingGame,
    guess: i32,
    result_str: &str,
    feedback_class: &str,
    feedback_message: String,
) -> Response {
    debug!(
        game_id = %game_id,
        guess = guess,
        result = result_str,
        "Web: Guess result"
    );

    let (min, max) = game.range();

    // Calculate remaining guesses
    let remaining_guesses = game.max_guesses().and_then(|limit| {
        let remaining = limit.saturating_sub(game.guess_count());
        if remaining > 0 { Some(remaining) } else { None }
    });

    let csrf_token = token.authenticity_token().unwrap_or_default();
    let template = GuessFormTemplate {
        game_id,
        min,
        max,
        remaining_guesses,
        feedback_class: feedback_class.to_string(),
        feedback_message,
        csrf_token,
    };
    // Return token in tuple to trigger cookie setting via IntoResponseParts
    (token, template).into_response()
}
