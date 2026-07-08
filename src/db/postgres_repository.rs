use crate::core::{GameId, GuessResult, GuessingGame};
use sqlx::{PgPool, Row};
use tracing::{debug, error, info};

use super::{DbError, repository::GameRepository};

/// PostgreSQL implementation of the GameRepository trait.
///
/// This repository uses SQLx for database operations and provides
/// transactional guarantees for concurrent operations.
///
/// # Cloning
/// The struct derives Clone because PgPool already contains an Arc internally,
/// making it cheap to clone and safe to use across multiple handlers.
#[derive(Clone)]
pub struct PostgresGameRepository {
    pool: PgPool,
}

impl PostgresGameRepository {
    /// Create a new PostgreSQL repository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl GameRepository for PostgresGameRepository {
    async fn create(
        &self,
        min: i32,
        max: i32,
        max_guesses: Option<u32>,
    ) -> Result<GameId, DbError> {
        debug!(
            min = min,
            max = max,
            max_guesses = ?max_guesses,
            "DB: Creating game"
        );

        // Validate game parameters (same as GuessingGame::new_with_limit)
        let game = GuessingGame::new_with_limit(min, max, max_guesses)?;

        // Generate random game ID
        let game_id = GameId::new();

        // Get game state
        let (min_val, max_val) = game.range();
        let secret = game.secret_number();
        let guess_count: i32 = game
            .guess_count()
            .try_into()
            .map_err(|_| DbError::ConversionError("Guess count exceeds i32 range".into()))?;
        let max_guesses_i32: Option<i32> = max_guesses
            .map(|g| g.try_into())
            .transpose()
            .map_err(|_| DbError::ConversionError("Max guesses exceeds i32 range".into()))?;

        // Insert into database
        sqlx::query(
            r#"
            INSERT INTO games (game_id, min_value, max_value, secret_number, guess_count, max_guesses)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(game_id.as_i64())
        .bind(min_val)
        .bind(max_val)
        .bind(secret)
        .bind(guess_count)
        .bind(max_guesses_i32)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                game_id = %game_id,
                min = min,
                max = max,
                error = %e,
                "DB: Failed to insert game into database"
            );
            e
        })?;

        info!(
            game_id = %game_id,
            min = min,
            max = max,
            max_guesses = ?max_guesses,
            "DB: Game created successfully"
        );

        Ok(game_id)
    }

    async fn get(&self, game_id: GameId) -> Result<GuessingGame, DbError> {
        debug!(game_id = %game_id, "DB: Fetching game");

        let row = sqlx::query(
            r#"
            SELECT game_id, min_value, max_value, secret_number, guess_count, max_guesses
            FROM games
            WHERE game_id = $1
            "#,
        )
        .bind(game_id.as_i64())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                debug!(game_id = %game_id, "DB: Game not found");
            } else {
                error!(game_id = %game_id, error = %e, "DB: Failed to fetch game");
            }
            e
        })?;

        // Extract values from row
        let min_value: i32 = row.try_get("min_value")?;
        let max_value: i32 = row.try_get("max_value")?;
        let secret_number: i32 = row.try_get("secret_number")?;
        let guess_count_i32: i32 = row.try_get("guess_count")?;
        let max_guesses_i32: Option<i32> = row.try_get("max_guesses")?;

        // Convert to u32 with proper error handling
        let guess_count: u32 = guess_count_i32
            .try_into()
            .map_err(|_| DbError::ConversionError("Guess count is negative".into()))?;
        let max_guesses: Option<u32> = max_guesses_i32
            .map(|g| g.try_into())
            .transpose()
            .map_err(|_| DbError::ConversionError("Max guesses is negative".into()))?;

        debug!(
            game_id = %game_id,
            min = min_value,
            max = max_value,
            guess_count = guess_count,
            "DB: Game fetched successfully"
        );

        // Reconstruct GuessingGame from database row
        Ok(GuessingGame::from_db(
            min_value,
            max_value,
            secret_number,
            guess_count,
            max_guesses,
        )?)
    }

    async fn make_guess(&self, game_id: GameId, guess: i32) -> Result<GuessResult, DbError> {
        debug!(
            game_id = %game_id,
            guess = guess,
            "DB: Starting transactional guess"
        );

        // Begin transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!(
                game_id = %game_id,
                error = %e,
                "DB: Failed to begin transaction"
            );
            e
        })?;

        debug!(game_id = %game_id, "DB: Transaction started, acquiring row lock");

        // Lock the row for update to prevent concurrent modifications
        let row = sqlx::query(
            r#"
            SELECT game_id, min_value, max_value, secret_number, guess_count, max_guesses
            FROM games
            WHERE game_id = $1
            FOR UPDATE
            "#,
        )
        .bind(game_id.as_i64())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                debug!(game_id = %game_id, "DB: Game not found in transaction");
            } else {
                error!(
                    game_id = %game_id,
                    error = %e,
                    "DB: Failed to lock game row"
                );
            }
            e
        })?;

        debug!(game_id = %game_id, "DB: Row lock acquired");

        // Extract values from row
        let min_value: i32 = row.try_get("min_value")?;
        let max_value: i32 = row.try_get("max_value")?;
        let secret_number: i32 = row.try_get("secret_number")?;
        let guess_count_i32: i32 = row.try_get("guess_count")?;
        let max_guesses_i32: Option<i32> = row.try_get("max_guesses")?;

        // Convert to u32 with proper error handling
        let guess_count: u32 = guess_count_i32
            .try_into()
            .map_err(|_| DbError::ConversionError("Guess count is negative".into()))?;
        let max_guesses: Option<u32> = max_guesses_i32
            .map(|g| g.try_into())
            .transpose()
            .map_err(|_| DbError::ConversionError("Max guesses is negative".into()))?;

        // Reconstruct game and make guess
        let mut game = GuessingGame::from_db(
            min_value,
            max_value,
            secret_number,
            guess_count,
            max_guesses,
        )?;

        let result = game.make_guess(guess);

        debug!(
            game_id = %game_id,
            result = ?result,
            "DB: Guess processed, updating database"
        );

        // Update or delete based on result
        match result {
            GuessResult::TooLow | GuessResult::TooHigh => {
                // Game continues - update guess count
                let new_guess_count: i32 = game.guess_count().try_into().map_err(|_| {
                    DbError::ConversionError("Guess count exceeds i32 range".into())
                })?;

                debug!(
                    game_id = %game_id,
                    new_guess_count = new_guess_count,
                    "DB: Updating guess count"
                );

                sqlx::query(
                    r#"
                    UPDATE games
                    SET guess_count = $1, updated_at = NOW()
                    WHERE game_id = $2
                    "#,
                )
                .bind(new_guess_count)
                .bind(game_id.as_i64())
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!(
                        game_id = %game_id,
                        error = %e,
                        "DB: Failed to update game in transaction"
                    );
                    e
                })?;
            }
            GuessResult::Correct { .. } | GuessResult::LimitReached { .. } => {
                // Game is over - delete from database
                debug!(game_id = %game_id, "DB: Game complete, deleting from database");

                sqlx::query(
                    r#"
                    DELETE FROM games
                    WHERE game_id = $1
                    "#,
                )
                .bind(game_id.as_i64())
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!(
                        game_id = %game_id,
                        error = %e,
                        "DB: Failed to delete game in transaction"
                    );
                    e
                })?;

                info!(
                    game_id = %game_id,
                    result = ?result,
                    "DB: Game completed and removed from database"
                );
            }
        }

        // Commit transaction
        debug!(game_id = %game_id, "DB: Committing transaction");
        tx.commit().await.map_err(|e| {
            error!(
                game_id = %game_id,
                error = %e,
                "DB: Failed to commit transaction"
            );
            e
        })?;

        debug!(
            game_id = %game_id,
            result = ?result,
            "DB: Transaction committed successfully"
        );

        Ok(result)
    }

    async fn delete(&self, game_id: GameId) -> Result<(), DbError> {
        debug!(game_id = %game_id, "DB: Deleting game");

        sqlx::query(
            r#"
            DELETE FROM games
            WHERE game_id = $1
            "#,
        )
        .bind(game_id.as_i64())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                game_id = %game_id,
                error = %e,
                "DB: Failed to delete game"
            );
            e
        })?;

        info!(game_id = %game_id, "DB: Game deleted successfully");

        Ok(())
    }

    async fn update(&self, game_id: GameId, game: &GuessingGame) -> Result<(), DbError> {
        let guess_count: i32 = game
            .guess_count()
            .try_into()
            .map_err(|_| DbError::ConversionError("Guess count exceeds i32 range".into()))?;

        debug!(
            game_id = %game_id,
            guess_count = guess_count,
            "DB: Updating game"
        );

        sqlx::query(
            r#"
            UPDATE games
            SET guess_count = $1, updated_at = NOW()
            WHERE game_id = $2
            "#,
        )
        .bind(guess_count)
        .bind(game_id.as_i64())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                game_id = %game_id,
                error = %e,
                "DB: Failed to update game"
            );
            e
        })?;

        debug!(game_id = %game_id, "DB: Game updated successfully");

        Ok(())
    }

    async fn health_check(&self) -> Result<(), DbError> {
        debug!("DB: Performing health check");

        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                error!(error = %e, "DB: Health check failed");
                e
            })?;

        debug!("DB: Health check passed");

        Ok(())
    }
}
