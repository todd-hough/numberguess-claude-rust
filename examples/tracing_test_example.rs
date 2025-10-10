// Example showing how to use tracing-test for automated log testing
//
// Add to Cargo.toml dev-dependencies:
// tracing-test = "0.2"
//
// Usage:
// cargo test --example tracing_test_example -- --nocapture

#[cfg(test)]
mod tests {
    use tracing::{info, error};

    // Annotate tests with #[traced_test] to capture logs
    #[test]
    #[tracing_test::traced_test]
    fn test_info_logging() {
        info!(max_connections = 5, "Connecting to database");

        // Assert that the log contains expected text
        assert!(logs_contain("Connecting to database"));
        assert!(logs_contain("max_connections"));
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_error_logging() {
        error!(game_id = 12345, error = "not found", "Failed to make guess");

        // Assert error log was emitted
        assert!(logs_contain("Failed to make guess"));
        assert!(logs_contain("game_id"));
        assert!(logs_contain("12345"));
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_log_levels() {
        info!("This is info level");
        tracing::debug!("This is debug level");
        tracing::warn!("This is warn level");

        // All levels are captured by default
        assert!(logs_contain("This is info level"));
        assert!(logs_contain("This is debug level"));
        assert!(logs_contain("This is warn level"));
    }
}

fn main() {
    println!("This is an example file. Run: cargo test --example tracing_test_example");
}
