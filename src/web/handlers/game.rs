//! Web UI handler for game creation.
//!
//! Handles creating new game instances via HTML forms (HTMX).

use crate::auth::AuthenticatedUser;
use crate::core::validators;
use crate::db::GameRepository;
use crate::server::state::AppState;
use crate::web::error::WebError;
use crate::web::templates::{GameStartedTemplate, IndexTemplate};
use crate::web::types::CreateGameRequest;
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use axum_csrf::CsrfToken;
use tracing::{debug, error, info, warn};

/// Web UI handler for the main index page.
pub async fn index_web(token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap_or_default();
    // Return token in tuple to trigger cookie setting via IntoResponseParts
    (token, IndexTemplate { csrf_token })
}

/// Web UI handler for game creation (HTML).
///
/// Creates a new game and returns HTML response for HTMX.
/// Requires authentication via oauth2-proxy.
///
/// # Type Parameters
/// * `R` - The repository implementation (static dispatch for zero overhead)
pub async fn create_game_web<R: GameRepository>(
    token: CsrfToken,
    State(state): State<AppState<R>>,
    user: AuthenticatedUser,
    Form(payload): Form<CreateGameRequest>,
) -> Result<impl IntoResponse, WebError> {
    // Verify CSRF token
    if token.verify(&payload.authenticity_token).is_err() {
        warn!("Web: CSRF token verification failed");
        return Err(WebError::InvalidCsrf);
    }

    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
        min = payload.min,
        max = payload.max,
        max_guesses = ?payload.max_guesses,
        "Web: Creating new game"
    );

    // Validate range and guess limit together (shared with the API handler)
    let guess_limit = validators::validate_new_game_params(
        payload.min,
        payload.max,
        payload.max_guesses,
        validators::MAX_WEB_GUESS_LIMIT,
    )
    .map_err(|e| {
        warn!(
            min = payload.min,
            max = payload.max,
            max_guesses = ?payload.max_guesses,
            error = %e,
            "Web: Game creation failed - invalid parameters"
        );
        WebError::from(e)
    })?;

    // Create game in database
    let game_id = state
        .repo
        .create(payload.min, payload.max, guess_limit)
        .await
        .map_err(|e| {
            error!(
                min = payload.min,
                max = payload.max,
                max_guesses = ?guess_limit,
                error = %e,
                "Web: Failed to create game in database"
            );
            WebError::ErrorMessage(e.to_string())
        })?;

    info!(
        user_id = %user.user_id,
        user_email = %user.email,
        game_id = %game_id,
        min = payload.min,
        max = payload.max,
        max_guesses = ?guess_limit,
        "Web: Game created successfully"
    );

    let csrf_token = token.authenticity_token().unwrap_or_default();
    let template = GameStartedTemplate {
        game_id,
        min: payload.min,
        max: payload.max,
        max_guesses: guess_limit,
        csrf_token,
    };
    // Return token in tuple to trigger cookie setting via IntoResponseParts
    Ok((token, template))
}
