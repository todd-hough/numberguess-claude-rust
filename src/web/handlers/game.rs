//! Web UI handler for game creation.
//!
//! Handles creating new game instances via HTML forms (HTMX).

use crate::auth::AuthenticatedUser;
use crate::core::validators;
use crate::db::GameRepository;
use crate::server::state::AppState;
use crate::web::templates::{ErrorTemplate, GameStartedTemplate};
use crate::web::types::CreateGameRequest;
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use tracing::{debug, error, info, warn};

/// Web UI handler for game creation (HTML).
///
/// Creates a new game and returns HTML response for HTMX.
/// Requires authentication via oauth2-proxy.
///
/// # Type Parameters
/// * `R` - The repository implementation (static dispatch for zero overhead)
pub async fn create_game_web<R: GameRepository>(
    State(state): State<AppState<R>>,
    user: AuthenticatedUser,
    Form(payload): Form<CreateGameRequest>,
) -> impl IntoResponse {
    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
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
        let template = ErrorTemplate { error_message: &e };
        return template.into_response();
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
                let template = ErrorTemplate { error_message: &e };
                return template.into_response();
            }
        }
    } else {
        None
    };

    // Create game in database
    let game_id = match state
        .repo
        .create(payload.min, payload.max, guess_limit)
        .await
    {
        Ok(id) => {
            info!(
                user_id = %user.user_id,
                user_email = %user.email,
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
            return template.into_response();
        }
    };

    let template = GameStartedTemplate {
        game_id,
        min: payload.min,
        max: payload.max,
        max_guesses: guess_limit,
    };
    template.into_response()
}
