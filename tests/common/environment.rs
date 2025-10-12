use reqwest::blocking::Client;
use std::env;
use std::thread;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://localhost:8080";

/// Return the base URL for the running game server, falling back to localhost.
pub fn base_url() -> String {
    env::var("GAME_SERVER_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

/// Return the base URL used by browsers (e.g., Selenium), defaulting to the API base.
pub fn browser_base_url() -> String {
    env::var("GAME_SERVER_BROWSER_URL").unwrap_or_else(|_| base_url())
}

/// Return the Selenium remote URL if configured.
pub fn selenium_url() -> Option<String> {
    env::var("SELENIUM_REMOTE_URL").ok()
}

/// Ensure the game server is reachable before running tests.
/// Panics with guidance if the server is not responding within the timeout.
pub fn ensure_server_ready() -> String {
    let base = base_url();
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 30;

    while attempts < max_attempts {
        match client.get(&base).send() {
            Ok(resp) if resp.status().is_success() => {
                return base;
            }
            Ok(resp) => {
                eprintln!(
                    "Server responded with status {} while waiting for readiness (attempt {}/{})",
                    resp.status(),
                    attempts + 1,
                    max_attempts
                );
            }
            Err(err) => {
                eprintln!(
                    "Server not ready yet (attempt {}/{}): {}",
                    attempts + 1,
                    max_attempts,
                    err
                );
            }
        }

        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!(
        "Game server at {} is not responding. Start it with `make compose-up` or run tests via `make test-compose`.",
        base
    );
}

/// Ensure Selenium is reachable, returning the configured URL.
pub fn ensure_selenium_ready() -> Option<String> {
    let url = selenium_url()?;
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 20;
    let status_endpoint = format!("{}/status", url.trim_end_matches('/'));

    while attempts < max_attempts {
        if let Ok(resp) = client.get(&status_endpoint).send() {
            if resp.status().is_success() {
                return Some(url);
            }
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    None
}
