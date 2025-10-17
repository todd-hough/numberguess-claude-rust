//! API handler for game creation.
//!
//! Handles creating new game instances via JSON API.

use crate::api::types::{CreateGameRequest, CreateGameResponse, ErrorResponse};
use crate::auth::AuthenticatedUser;
use crate::core::validators;
use crate::db;
use axum::{extract::State, http::StatusCode, response::Json};
use sqlx::PgPool;
use tracing::{debug, error, info, warn};

type SharedState = PgPool;

/// API handler for game creation (JSON).
///
/// Creates a new game with the specified parameters and returns JSON response.
/// Requires authentication via oauth2-proxy.
pub async fn create_game_api(
    State(pool): State<SharedState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
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
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
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
                return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })));
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    info!(
        game_id = %game_id,
        user_id = %user.user_id,
        user_email = %user.email,
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
