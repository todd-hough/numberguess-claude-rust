-- Function to clean up old games (optional, for future use)
CREATE OR REPLACE FUNCTION cleanup_old_games(hours_old INTEGER DEFAULT 24)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM games
    WHERE created_at < NOW() - (hours_old || ' hours')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
