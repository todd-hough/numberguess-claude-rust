use crate::game::GuessingGame;
use rand::Rng;
use sqlx::{PgPool, Row};

#[derive(Debug)]
pub enum DbError {
    NotFound,
    DatabaseError(sqlx::Error),
    ConversionError(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "Game not found"),
            DbError::DatabaseError(e) => write!(f, "Database error: {}", e),
            DbError::ConversionError(e) => write!(f, "Conversion error: {}", e),
        }
    }
}

impl std::error::Error for DbError {}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound,
            e => DbError::DatabaseError(e),
        }
    }
}

/// Create a new game and store it in the database
pub async fn create_game(
    pool: &PgPool,
    min: i32,
    max: i32,
    max_guesses: Option<u32>,
) -> Result<u64, DbError> {
    // Validate game parameters (same as GuessingGame::new_with_limit)
    let game = GuessingGame::new_with_limit(min, max, max_guesses)
        .map_err(|e| DbError::ConversionError(e))?;

    // Generate random game ID
    let game_id = rand::rng().random::<u64>();

    // Get game state
    let (min_val, max_val) = game.get_range();
    let secret = game.secret_number();
    let guess_count = game.get_guess_count() as i32;
    let max_guesses_i32 = max_guesses.map(|g| g as i32);

    // Insert into database
    sqlx::query(
        r#"
        INSERT INTO games (game_id, min_value, max_value, secret_number, guess_count, max_guesses)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#
    )
    .bind(game_id as i64)
    .bind(min_val)
    .bind(max_val)
    .bind(secret)
    .bind(guess_count)
    .bind(max_guesses_i32)
    .execute(pool)
    .await?;

    Ok(game_id)
}

/// Get a game from the database
pub async fn get_game(pool: &PgPool, game_id: u64) -> Result<GuessingGame, DbError> {
    let row = sqlx::query(
        r#"
        SELECT game_id, min_value, max_value, secret_number, guess_count, max_guesses
        FROM games
        WHERE game_id = $1
        "#
    )
    .bind(game_id as i64)
    .fetch_one(pool)
    .await?;

    // Extract values from row
    let min_value: i32 = row.try_get("min_value")?;
    let max_value: i32 = row.try_get("max_value")?;
    let secret_number: i32 = row.try_get("secret_number")?;
    let guess_count: i32 = row.try_get("guess_count")?;
    let max_guesses: Option<i32> = row.try_get("max_guesses")?;

    // Reconstruct GuessingGame from database row
    GuessingGame::from_db(
        min_value,
        max_value,
        secret_number,
        guess_count as u32,
        max_guesses.map(|g| g as u32),
    )
    .map_err(|e| DbError::ConversionError(e))
}

/// Update game state after a guess
pub async fn update_game(pool: &PgPool, game_id: u64, game: &GuessingGame) -> Result<(), DbError> {
    let guess_count = game.get_guess_count() as i32;

    sqlx::query(
        r#"
        UPDATE games
        SET guess_count = $1, updated_at = NOW()
        WHERE game_id = $2
        "#
    )
    .bind(guess_count)
    .bind(game_id as i64)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete a game from the database
pub async fn delete_game(pool: &PgPool, game_id: u64) -> Result<(), DbError> {
    sqlx::query(
        r#"
        DELETE FROM games
        WHERE game_id = $1
        "#
    )
    .bind(game_id as i64)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL database
    // They will be run as integration tests with testcontainers

    #[tokio::test]
    #[ignore] // Only run when DATABASE_URL is available
    async fn test_create_and_get_game() {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Create game
        let game_id = create_game(&pool, 1, 100, Some(10)).await.unwrap();

        // Get game
        let game = get_game(&pool, game_id).await.unwrap();
        assert_eq!(game.get_range(), (1, 100));
        assert_eq!(game.get_max_guesses(), Some(10));
        assert_eq!(game.get_guess_count(), 0);

        // Cleanup
        delete_game(&pool, game_id).await.unwrap();
    }
}
