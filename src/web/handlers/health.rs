//! Health check handler.
//!
//! Provides a health endpoint that verifies database connectivity.

use axum::{extract::State, http::StatusCode};
use sqlx::PgPool;
use tracing::{debug, error};

type SharedState = PgPool;

/// Health check endpoint that verifies database connectivity.
///
/// Returns 200 OK if the database is accessible, 503 Service Unavailable otherwise.
pub async fn health_check(State(pool): State<SharedState>) -> StatusCode {
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
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
