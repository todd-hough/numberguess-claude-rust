-- Create games table
CREATE TABLE games (
    game_id BIGINT PRIMARY KEY,
    min_value INTEGER NOT NULL,
    max_value INTEGER NOT NULL,
    secret_number INTEGER NOT NULL,
    guess_count INTEGER NOT NULL DEFAULT 0,
    max_guesses INTEGER NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add index on created_at for cleanup queries
CREATE INDEX idx_games_created_at ON games(created_at);
