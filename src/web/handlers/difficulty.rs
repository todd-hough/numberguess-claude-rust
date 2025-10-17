//! Difficulty preview handler.
//!
//! Provides real-time difficulty feedback as users configure game parameters.

use crate::auth::AuthenticatedUser;
use crate::core::features::difficulty;
use crate::web::templates::DifficultyIndicator;
use crate::web::types::DifficultyParams;
use askama::Template;
use axum::{extract::Query, response::Html};
use tracing::debug;

/// Handles difficulty preview requests from the game setup form.
///
/// This endpoint is called via HTMX when users adjust game parameters
/// (min, max, guess limit). It calculates and returns an HTML fragment
/// showing the difficulty level, optimal guesses, and helpful guidance.
///
/// Returns an empty response for invalid inputs to avoid showing errors
/// while the user is still typing.
///
/// Requires authentication via oauth2-proxy.
pub async fn difficulty_preview(
    _user: AuthenticatedUser,
    Query(params): Query<DifficultyParams>,
) -> Html<String> {
    // Extract parameters with defaults
    let min = params.min.unwrap_or(1);
    let max = params.max.unwrap_or(100);

    // Validate range (silently return empty for invalid inputs during typing)
    if min < 0 || max < 0 || max < min {
        debug!(
            min = min,
            max = max,
            "Difficulty preview: Invalid range, returning empty response"
        );
        return Html("".to_string());
    }

    // Calculate difficulty information
    let info = difficulty::calculate_difficulty(min, max, params.max_guesses);

    debug!(
        min = info.min,
        max = info.max,
        range_size = info.range_size,
        optimal_guesses = info.optimal_guesses,
        guess_limit = ?info.guess_limit,
        buffer = info.buffer,
        level = ?info.level,
        "Difficulty preview calculated"
    );

    // Render template
    let template = DifficultyIndicator { info };
    Html(template.render().unwrap_or_default())
}
