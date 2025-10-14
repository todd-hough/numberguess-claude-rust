//! Web UI handler for game creation.
//!
//! Handles creating new game instances via HTML forms (HTMX).

use crate::core::validators;
use crate::db;
use crate::web::templates::{ErrorTemplate, GameStartedTemplate};
use crate::web::types::CreateGameRequest;
use askama_axum::IntoResponse as AskamaIntoResponse;
use axum::{
    extract::{Form, State},
    response::IntoResponse,
};
use sqlx::PgPool;
use tracing::{debug, error, info, warn};

type SharedState = PgPool;

/// Web UI handler for game creation (HTML).
///
/// Creates a new game and returns HTML response for HTMX.
pub async fn create_game_web(
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
        let template = ErrorTemplate { error_message: &e };
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
                let template = ErrorTemplate { error_message: &e };
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
