// Module declarations
pub mod postgres_repository;
pub mod repository;

// Re-exports
pub use postgres_repository::PostgresGameRepository;
pub use repository::GameRepository;

use crate::core::GameError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Game not found")]
    NotFound,

    #[error("Database error: {0}")]
    DatabaseError(sqlx::Error),

    #[error("Game validation error: {0}")]
    GameError(#[from] GameError),

    #[error("Type conversion error: {0}")]
    ConversionError(String),
}

// Custom From implementation to handle RowNotFound specially
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound,
            e => DbError::DatabaseError(e),
        }
    }
}
