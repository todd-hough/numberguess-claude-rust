//! API handler for game creation.
//!
//! Handles creating new game instances via JSON API.

use crate::api::error::ApiError;
use crate::api::types::{CreateGameRequest, CreateGameResponse};
use crate::auth::AuthenticatedUser;
use crate::core::validators;
use crate::db::GameRepository;
use crate::server::state::AppState;
use axum::{extract::State, response::Json};
use tracing::{debug, error, info, warn};

/// API handler for game creation (JSON).
///
/// Creates a new game with the specified parameters and returns JSON response.
/// Requires authentication via oauth2-proxy.
///
/// # Type Parameters
/// * `R` - The repository implementation (static dispatch for zero overhead)
pub async fn create_game_api<R: GameRepository>(
    State(state): State<AppState<R>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<CreateGameResponse>, ApiError> {
    debug!(
        user_id = %user.user_id,
        user_email = %user.email,
        min = payload.min,
        max = payload.max,
        max_guesses = ?payload.max_guesses,
        "API: Creating new game"
    );

    // Validate range and guess limit together (shared with the web handler)
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
            "API: Game creation failed - invalid parameters"
        );
        ApiError::Validation(e)
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
                "API: Failed to create game in database"
            );
            ApiError::Internal(e.to_string())
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
