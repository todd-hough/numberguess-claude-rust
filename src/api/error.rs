//! Error type for JSON API handlers.
//!
//! Converts handler failures into the exact HTTP responses the API has
//! always produced — status codes and `{"error": "..."}` bodies are part of
//! the external API contract (see docs/api.md) and must not change:
//!
//! | Variant        | Status | Body                                   |
//! |----------------|--------|----------------------------------------|
//! | `Validation`   | 400    | the `GameError`'s Display text         |
//! | `GameNotFound` | 404    | `Game with ID {id} not found`          |
//! | `Internal`     | 500    | the underlying error's Display text    |

use crate::api::types::ErrorResponse;
use crate::core::{GameError, GameId};
use crate::db::DbError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum ApiError {
    /// Invalid game parameters (range, guess limit) → 400.
    Validation(GameError),
    /// Unknown or already-completed game → 404.
    GameNotFound(GameId),
    /// Database or other unexpected failure → 500.
    Internal(String),
}

impl ApiError {
    /// Map a repository error for an operation on a specific game:
    /// `NotFound` becomes 404 for that game id, anything else is a 500.
    pub fn from_db_for_game(game_id: GameId) -> impl FnOnce(DbError) -> Self {
        move |e| match e {
            DbError::NotFound => Self::GameNotFound(game_id),
            e => Self::Internal(e.to_string()),
        }
    }
}

impl From<GameError> for ApiError {
    fn from(e: GameError) -> Self {
        Self::Validation(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            Self::GameNotFound(id) => (
                StatusCode::NOT_FOUND,
                format!("Game with ID {id} not found"),
            ),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(ErrorResponse { error })).into_response()
    }
}
