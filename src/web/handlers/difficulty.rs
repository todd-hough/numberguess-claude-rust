//! Difficulty preview handler.
//!
//! Provides real-time difficulty feedback as users configure game parameters.

use crate::auth::AuthenticatedUser;
use crate::core::features::difficulty;
use crate::core::validators;
use crate::web::templates::DifficultyIndicator;
use crate::web::types::DifficultyParams;
use askama::Template;
use axum::{extract::Query, response::Html};
use tracing::{debug, error};

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

    // Validate with the shared validator (silently return empty for invalid
    // inputs during typing). This also enforces MAX_RANGE, which the previous
    // ad-hoc check missed — an unbounded max like i32::MAX overflowed the
    // range-size arithmetic in calculate_difficulty.
    if let Err(e) = validators::validate_range(min, max) {
        debug!(
            min = min,
            max = max,
            error = %e,
            "Difficulty preview: Invalid range, returning empty response"
        );
        return Html(String::new());
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
    Html(template.render().unwrap_or_else(|e| {
        error!(error = %e, "Difficulty preview: template render failed");
        String::new()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_user() -> AuthenticatedUser {
        AuthenticatedUser {
            user_id: "test-user".to_string(),
            email: "test@example.com".to_string(),
            username: Some("test".to_string()),
            groups: vec![],
        }
    }

    fn params(min: Option<i32>, max: Option<i32>, max_guesses: Option<u32>) -> DifficultyParams {
        DifficultyParams {
            min,
            max,
            max_guesses,
        }
    }

    #[tokio::test]
    async fn test_valid_range_renders_indicator() {
        let Html(body) =
            difficulty_preview(test_user(), Query(params(Some(1), Some(100), Some(10)))).await;
        assert!(!body.is_empty(), "Valid range should render the indicator");
    }

    #[tokio::test]
    async fn test_max_above_limit_returns_empty() {
        // Regression: max = i32::MAX previously overflowed `max - min + 1`
        // in calculate_difficulty (panic in debug builds).
        let Html(body) =
            difficulty_preview(test_user(), Query(params(Some(0), Some(i32::MAX), None))).await;
        assert!(body.is_empty(), "Out-of-range max should return empty");

        // Just above the documented limit is also rejected
        let Html(body) =
            difficulty_preview(test_user(), Query(params(Some(0), Some(1_000_001), None))).await;
        assert!(body.is_empty(), "max > MAX_RANGE should return empty");
    }

    #[tokio::test]
    async fn test_invalid_ranges_return_empty() {
        for (min, max) in [(Some(-1), Some(100)), (Some(100), Some(1))] {
            let Html(body) = difficulty_preview(test_user(), Query(params(min, max, None))).await;
            assert!(
                body.is_empty(),
                "Invalid range {min:?}..{max:?} should return empty"
            );
        }
    }
}
