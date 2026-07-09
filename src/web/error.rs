//! Error type for Web UI (HTMX) handlers.
//!
//! Converts handler failures into the exact HTML fragments the web UI has
//! always returned (all render with status 200 — HTMX swaps the fragment
//! into the page — except the CSRF rejection, which is a plain 400):
//!
//! | Variant        | Response                                        |
//! |----------------|-------------------------------------------------|
//! | `ErrorMessage` | `ErrorTemplate` with the message                |
//! | `GameNotFound` | `GameNotFoundTemplate`                          |
//! | `UpdateFailed` | `UpdateErrorTemplate`                           |
//! | `InvalidCsrf`  | 400 "Invalid CSRF token"                        |

use crate::core::GameError;
use crate::web::templates::{ErrorTemplate, GameNotFoundTemplate, UpdateErrorTemplate};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum WebError {
    /// Validation or creation failure shown to the user via `ErrorTemplate`.
    ErrorMessage(String),
    /// Unknown or already-completed game.
    GameNotFound,
    /// Failure while processing a guess (non-NotFound database error).
    UpdateFailed,
    /// CSRF token missing or invalid.
    InvalidCsrf,
}

impl From<GameError> for WebError {
    fn from(e: GameError) -> Self {
        Self::ErrorMessage(e.to_string())
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        match self {
            Self::ErrorMessage(msg) => ErrorTemplate {
                error_message: &msg,
            }
            .into_response(),
            Self::GameNotFound => GameNotFoundTemplate.into_response(),
            Self::UpdateFailed => UpdateErrorTemplate.into_response(),
            Self::InvalidCsrf => (StatusCode::BAD_REQUEST, "Invalid CSRF token").into_response(),
        }
    }
}
