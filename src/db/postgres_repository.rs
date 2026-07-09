use crate::core::{GameId, GuessResult, GuessingGame};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use tracing::{debug, error, info, instrument};

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

/// Reconstruct a `GuessingGame` from a `games` table row.
///
/// Shared by `get` and `make_guess` so the column list and i32→u32
/// conversions cannot drift between them.
fn game_from_row(row: &PgRow) -> Result<GuessingGame, DbError> {
    let min_value: i32 = row.try_get("min_value")?;
    let max_value: i32 = row.try_get("max_value")?;
    let secret_number: i32 = row.try_get("secret_number")?;
    let guess_count_i32: i32 = row.try_get("guess_count")?;
    let max_guesses_i32: Option<i32> = row.try_get("max_guesses")?;

    let guess_count: u32 = guess_count_i32
        .try_into()
        .map_err(|_| DbError::ConversionError("Guess count is negative".into()))?;
    let max_guesses: Option<u32> = max_guesses_i32
        .map(|g| g.try_into())
        .transpose()
        .map_err(|_| DbError::ConversionError("Max guesses is negative".into()))?;

    Ok(GuessingGame::from_db(
        min_value,
        max_value,
        secret_number,
        guess_count,
        max_guesses,
    )?)
}

impl GameRepository for PostgresGameRepository {
    #[instrument(skip(self))]
    async fn create(
        &self,
        min: i32,
        max: i32,
        max_guesses: Option<u32>,
    ) -> Result<GameId, DbError> {
        debug!("DB: Creating game");

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
        .bind(i64::from(game_id))
        .bind(min_val)
        .bind(max_val)
        .bind(secret)
        .bind(guess_count)
        .bind(max_guesses_i32)
        .execute(&self.pool)
        .await
        .inspect_err(|e| {
            error!(
                game_id = %game_id,
                error = %e,
                "DB: Failed to insert game into database"
            );
        })?;

        info!(game_id = %game_id, "DB: Game created successfully");

        Ok(game_id)
    }

    #[instrument(skip(self))]
    async fn get(&self, game_id: GameId) -> Result<GuessingGame, DbError> {
        debug!("DB: Fetching game");

        let row = sqlx::query(
            r#"
            SELECT game_id, min_value, max_value, secret_number, guess_count, max_guesses
            FROM games
            WHERE game_id = $1
            "#,
        )
        .bind(i64::from(game_id))
        .fetch_one(&self.pool)
        .await
        .inspect_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                debug!("DB: Game not found");
            } else {
                error!(error = %e, "DB: Failed to fetch game");
            }
        })?;

        let game = game_from_row(&row)?;
        debug!(
            guess_count = game.guess_count(),
            "DB: Game fetched successfully"
        );
        Ok(game)
    }

    #[instrument(skip(self))]
    async fn make_guess(
        &self,
        game_id: GameId,
        guess: i32,
    ) -> Result<(GuessResult, GuessingGame), DbError> {
        debug!("DB: Starting transactional guess");

        // Begin transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .inspect_err(|e| error!(error = %e, "DB: Failed to begin transaction"))?;

        // Lock the row for update to prevent concurrent modifications
        let row = sqlx::query(
            r#"
            SELECT game_id, min_value, max_value, secret_number, guess_count, max_guesses
            FROM games
            WHERE game_id = $1
            FOR UPDATE
            "#,
        )
        .bind(i64::from(game_id))
        .fetch_one(&mut *tx)
        .await
        .inspect_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                debug!("DB: Game not found in transaction");
            } else {
                error!(error = %e, "DB: Failed to lock game row");
            }
        })?;

        // Reconstruct game and make guess
        let mut game = game_from_row(&row)?;
        let result = game.make_guess(guess);

        debug!(result = ?result, "DB: Guess processed, updating database");

        // Update or delete based on result
        match result {
            GuessResult::TooLow | GuessResult::TooHigh => {
                // Game continues - update guess count
                let new_guess_count: i32 = game.guess_count().try_into().map_err(|_| {
                    DbError::ConversionError("Guess count exceeds i32 range".into())
                })?;

                sqlx::query(
                    r#"
                    UPDATE games
                    SET guess_count = $1, updated_at = NOW()
                    WHERE game_id = $2
                    "#,
                )
                .bind(new_guess_count)
                .bind(i64::from(game_id))
                .execute(&mut *tx)
                .await
                .inspect_err(|e| error!(error = %e, "DB: Failed to update game in transaction"))?;
            }
            GuessResult::Correct { .. } | GuessResult::LimitReached { .. } => {
                // Game is over - delete from database
                sqlx::query(
                    r#"
                    DELETE FROM games
                    WHERE game_id = $1
                    "#,
                )
                .bind(i64::from(game_id))
                .execute(&mut *tx)
                .await
                .inspect_err(|e| error!(error = %e, "DB: Failed to delete game in transaction"))?;

                info!(result = ?result, "DB: Game completed and removed from database");
            }
        }

        // Commit transaction
        tx.commit()
            .await
            .inspect_err(|e| error!(error = %e, "DB: Failed to commit transaction"))?;

        debug!(result = ?result, "DB: Transaction committed successfully");

        // Return the post-guess state captured inside the transaction so
        // callers never need a racy follow-up fetch.
        Ok((result, game))
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<(), DbError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .inspect_err(|e| error!(error = %e, "DB: Health check failed"))?;

        debug!("DB: Health check passed");

        Ok(())
    }
}
