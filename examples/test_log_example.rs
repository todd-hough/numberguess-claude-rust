// Example showing how to use test-log for automatic log initialization
//
// Add to Cargo.toml dev-dependencies:
// test-log = { version = "0.2", features = ["trace"] }
// tracing = "0.1" (already in dependencies)
// tracing-subscriber = { version = "0.3", features = ["env-filter"] } (already in dependencies)
//
// Usage:
// RUST_LOG=debug cargo test --example test_log_example
// cargo test --example test_log_example -- --nocapture

#[cfg(test)]
mod tests {
    use tracing::{info, debug, error};

    // Replace #[test] with #[test_log::test]
    #[test_log::test]
    fn test_with_automatic_logging() {
        info!("Test started");
        debug!("Debug information");

        // Your test logic here
        let result = 2 + 2;
        assert_eq!(result, 4);

        info!("Test completed");
    }

    #[test_log::test]
    fn test_error_scenarios() {
        error!(error = "something failed", "Error occurred");

        // Logs automatically shown on failure
        // Or with --nocapture flag
        assert!(true);
    }

    // Works with async tests too
    #[test_log::test(tokio::test)]
    async fn test_async_with_logging() {
        info!("Async test started");

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        info!("Async test completed");
        assert!(true);
    }
}

fn main() {
    println!("This is an example file. Run: cargo test --example test_log_example");
}
