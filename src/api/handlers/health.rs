//! Health check handler.
//!
//! Provides a health endpoint that verifies database connectivity.

use crate::db::GameRepository;
use crate::server::state::AppState;
use axum::{extract::State, http::StatusCode};
use tracing::{debug, error};

/// Health check endpoint that verifies database connectivity.
///
/// Returns 200 OK if the database is accessible, 503 Service Unavailable otherwise.
///
/// # Type Parameters
/// * `R` - The repository implementation (static dispatch for zero overhead)
pub async fn health_check<R: GameRepository>(State(state): State<AppState<R>>) -> StatusCode {
    match state.repo.health_check().await {
        Ok(_) => {
            debug!("Health check passed");
            StatusCode::OK
        }
        Err(e) => {
            error!(error = %e, "Health check failed: database unavailable");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}
