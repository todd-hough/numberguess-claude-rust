use reqwest::blocking::Client;
use std::env;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://localhost:8080";
const DEFAULT_BROWSER_URL: &str = "http://oauth2-proxy:4180";
const DEFAULT_KEYCLOAK_URL: &str = "http://localhost:8090";
const DEFAULT_SELENIUM_URL: &str = "http://localhost:4444";

/// Return the base URL for the running game server, falling back to localhost.
pub fn base_url() -> String {
    env::var("GAME_SERVER_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

/// Return the base URL used by browsers (e.g., Selenium).
/// Defaults to oauth2-proxy's internal Docker address since Selenium runs in Docker.
pub fn browser_base_url() -> String {
    env::var("GAME_SERVER_BROWSER_URL").unwrap_or_else(|_| DEFAULT_BROWSER_URL.to_string())
}

/// Return the Selenium remote URL.
pub fn selenium_url() -> String {
    env::var("SELENIUM_REMOTE_URL").unwrap_or_else(|_| DEFAULT_SELENIUM_URL.to_string())
}

/// Return the Keycloak base URL.
pub fn keycloak_url() -> String {
    env::var("KEYCLOAK_URL").unwrap_or_else(|_| DEFAULT_KEYCLOAK_URL.to_string())
}

/// Ensure the game server is reachable before running tests.
/// Panics with guidance if the server is not responding within the timeout.
///
/// This function waits for all auth services (redis, keycloak, oauth2-proxy)
/// to be ready before checking the application server.
pub fn ensure_server_ready() -> String {
    // Wait for all auth services in order
    eprintln!("=== Checking authentication services ===");
    ensure_redis_ready();
    ensure_keycloak_ready();
    ensure_oauth2_proxy_ready();
    eprintln!("=== All auth services ready ===\n");

    let base = base_url();
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none()) // Don't follow oauth2-proxy redirects
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 30;

    eprintln!("Waiting for application server at {}...", base);

    while attempts < max_attempts {
        match client.get(&base).send() {
            Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                eprintln!("Application server is ready");
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

/// Ensure Selenium is reachable, returning the URL.
/// Panics if Selenium is not responding within the timeout.
pub fn ensure_selenium_ready() -> String {
    let url = selenium_url();
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 20;
    let status_endpoint = format!("{}/status", url.trim_end_matches('/'));

    eprintln!("Waiting for Selenium to be ready at {}...", url);

    while attempts < max_attempts {
        if let Ok(resp) = client.get(&status_endpoint).send()
            && resp.status().is_success()
        {
            eprintln!("Selenium is ready");
            return url;
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!(
        "Selenium at {} is not responding after {} attempts. Check `docker compose logs selenium`",
        url, max_attempts
    );
}

/// Ensure Keycloak is reachable and ready.
pub fn ensure_keycloak_ready() -> String {
    let url = keycloak_url();
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 60; // Keycloak can take up to 60s to start
    let health_endpoint = format!(
        "{}/realms/numberguess/.well-known/openid-configuration",
        url
    );

    eprintln!("Waiting for Keycloak to be ready at {}...", url);

    while attempts < max_attempts {
        if let Ok(resp) = client.get(&health_endpoint).send()
            && resp.status().is_success()
        {
            eprintln!("Keycloak is ready");
            return url;
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!(
        "Keycloak at {} is not responding after {} attempts. Check `docker compose logs keycloak`",
        url, max_attempts
    );
}

/// Ensure Redis is reachable.
pub fn ensure_redis_ready() {
    let mut attempts = 0;
    let max_attempts = 30;

    eprintln!("Waiting for Redis to be ready...");

    while attempts < max_attempts {
        if TcpStream::connect("localhost:6379").is_ok() {
            eprintln!("Redis is ready");
            return;
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!(
        "Redis is not responding after {} attempts. Check `docker compose logs redis`",
        max_attempts
    );
}

/// Ensure oauth2-proxy is reachable.
pub fn ensure_oauth2_proxy_ready() {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 30;

    eprintln!("Waiting for oauth2-proxy to be ready...");

    while attempts < max_attempts {
        // oauth2-proxy will redirect to Keycloak (302), which is fine
        if let Ok(resp) = client.get("http://localhost:8080").send() {
            let status = resp.status();
            if status.is_success() || status.is_redirection() {
                eprintln!("oauth2-proxy is ready");
                return;
            }
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!(
        "oauth2-proxy is not responding after {} attempts. Check `docker compose logs oauth2-proxy`",
        max_attempts
    );
}
