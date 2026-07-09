// Allow async fn in trait - this is a deliberate design choice for native async traits (Rust 1.75+)
#![allow(async_fn_in_trait)]

use super::DbError;
use crate::core::{GameId, GuessResult, GuessingGame};

/// Repository trait for game persistence operations.
///
/// This trait provides an abstraction layer over database operations,
/// allowing for different storage implementations (PostgreSQL, in-memory, etc.)
/// while maintaining a consistent interface.
///
/// # Type Parameters
/// - Implementations must be `Send + Sync` for async operations
/// - Implementations must be `Clone` for use with Axum state
///
/// # Native Async Traits
/// This uses Rust's native async trait syntax (available since Rust 1.75),
/// which provides zero-cost abstraction without requiring the `async-trait` crate.
pub trait GameRepository: Send + Sync + Clone {
    /// Create a new game and store it in the repository
    ///
    /// # Arguments
    /// * `min` - Minimum value for the guessing range
    /// * `max` - Maximum value for the guessing range
    /// * `max_guesses` - Optional maximum number of guesses allowed
    ///
    /// # Returns
    /// The unique identifier for the created game
    ///
    /// # Errors
    /// Returns `DbError::GameError` if the game parameters are invalid
    /// Returns `DbError::DatabaseError` if the storage operation fails
    async fn create(&self, min: i32, max: i32, max_guesses: Option<u32>)
    -> Result<GameId, DbError>;

    /// Retrieve a game from the repository
    ///
    /// # Arguments
    /// * `game_id` - The unique identifier of the game to retrieve
    ///
    /// # Returns
    /// The game state
    ///
    /// # Errors
    /// Returns `DbError::NotFound` if the game doesn't exist
    /// Returns `DbError::DatabaseError` if the storage operation fails
    async fn get(&self, game_id: GameId) -> Result<GuessingGame, DbError>;

    /// Process a guess in a transactional, concurrency-safe manner
    ///
    /// This method combines fetching the game, processing the guess,
    /// and updating or deleting the game in a single atomic operation.
    /// The game is automatically deleted if it's completed (correct guess
    /// or limit reached).
    ///
    /// # Arguments
    /// * `game_id` - The unique identifier of the game
    /// * `guess` - The player's guess
    ///
    /// # Returns
    /// The result of the guess (TooLow, TooHigh, Correct, or LimitReached)
    /// together with the post-guess game state, captured inside the same
    /// transaction. Callers that need the state for display (e.g. the web
    /// UI's remaining-guesses counter) must use it instead of issuing a
    /// follow-up `get` — the game may be deleted or changed by a concurrent
    /// request between the two calls.
    ///
    /// # Errors
    /// Returns `DbError::NotFound` if the game doesn't exist
    /// Returns `DbError::DatabaseError` if the storage operation fails
    async fn make_guess(
        &self,
        game_id: GameId,
        guess: i32,
    ) -> Result<(GuessResult, GuessingGame), DbError>;

    /// Perform a health check on the repository
    ///
    /// This verifies that the repository is accessible and operational.
    ///
    /// # Errors
    /// Returns `DbError::DatabaseError` if the health check fails
    async fn health_check(&self) -> Result<(), DbError>;
}
