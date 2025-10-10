// Integration test for logging functionality
// Tests that logging is properly configured and emits expected messages
//
// Run with: cargo test logging_test -- --nocapture

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::time::Duration;

#[test]
#[ignore] // Ignore by default since it requires database
fn test_server_startup_logs() {
    // Start database using docker compose
    let db_start = Command::new("make")
        .arg("dev-db")
        .output()
        .expect("Failed to start database");

    assert!(db_start.status.success(), "Database failed to start");

    // Give database time to initialize
    std::thread::sleep(Duration::from_secs(2));

    // Start the server and capture logs
    let mut child = Command::new("cargo")
        .args(&["run", "--quiet", "--", "--server", "--port", "8082"])
        .env("RUST_LOG", "number_guessing_game=info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Capture stderr (where tracing logs go)
    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let reader = BufReader::new(stderr);

    // Collect first few log lines with timeout
    let mut logs = Vec::new();
    let start = std::time::Instant::now();

    for line in reader.lines() {
        if start.elapsed() > Duration::from_secs(5) {
            break;
        }

        if let Ok(line) = line {
            logs.push(line.clone());
            println!("LOG: {}", line);

            // Stop after we see the server is running
            if line.contains("Health Check:") {
                break;
            }
        }
    }

    // Kill the server
    child.kill().expect("Failed to kill server");

    // Stop database
    let _ = Command::new("make")
        .arg("dev-down")
        .output();

    // Assert expected log messages appeared
    let all_logs = logs.join("\n");

    assert!(
        all_logs.contains("Connecting to database"),
        "Missing 'Connecting to database' log"
    );
    assert!(
        all_logs.contains("max_connections"),
        "Missing structured field 'max_connections'"
    );
    assert!(
        all_logs.contains("Running database migrations"),
        "Missing 'Running database migrations' log"
    );
    assert!(
        all_logs.contains("Database initialized successfully"),
        "Missing 'Database initialized successfully' log"
    );
    assert!(
        all_logs.contains("Starting web server"),
        "Missing 'Starting web server' log"
    );
    assert!(
        all_logs.contains("main_addr"),
        "Missing structured field 'main_addr'"
    );
    assert!(
        all_logs.contains("INFO"),
        "Missing INFO log level"
    );
}

#[test]
#[ignore] // Slow test - spawns processes. Run manually with: cargo test logging_test -- --ignored
fn test_cli_mode_no_logs() {
    // First, ensure the binary is built
    let build = Command::new("cargo")
        .args(&["build", "--quiet"])
        .status()
        .expect("Failed to build binary");

    assert!(build.success(), "Build failed");

    // CLI mode should not emit structured logs, only user-facing println
    // Use the built binary directly to avoid compile time
    let binary_path = if cfg!(target_os = "windows") {
        "target/debug/number_guessing_game.exe"
    } else {
        "target/debug/number_guessing_game"
    };

    let mut child = Command::new(binary_path)
        .args(&["--min", "1", "--max", "10", "--limit", "1"])
        .env("RUST_LOG", "number_guessing_game=info")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start CLI");

    // Give it a moment to print welcome message
    std::thread::sleep(Duration::from_millis(200));

    // Kill it before it waits for input
    child.kill().expect("Failed to kill process");

    let output = child.wait_with_output().expect("Failed to get output");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);

    // Stderr should be empty (no tracing logs in CLI mode)
    assert!(
        stderr.is_empty() || !stderr.contains("INFO"),
        "CLI mode should not emit INFO logs to stderr, got: {}",
        stderr
    );

    // Stdout should have user-facing messages
    assert!(
        stdout.contains("Welcome to the Number Guessing Game!"),
        "Missing welcome message, got: {}",
        stdout
    );
}

#[test]
#[ignore] // Slow test - spawns processes. Run manually with: cargo test logging_test -- --ignored
fn test_log_level_filtering() {
    // Test that RUST_LOG=error only shows errors, not info
    let db_start = Command::new("make")
        .arg("dev-db")
        .output()
        .expect("Failed to start database");

    if !db_start.status.success() {
        eprintln!("Skipping test - database not available");
        return;
    }

    std::thread::sleep(Duration::from_secs(2));

    let mut child = Command::new("cargo")
        .args(&["run", "--quiet", "--", "--server", "--port", "8083"])
        .env("RUST_LOG", "error") // Only errors
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    let stderr = child.stderr.take().expect("Failed to capture stderr");
    let reader = BufReader::new(stderr);

    let mut logs = Vec::new();
    let start = std::time::Instant::now();

    for line in reader.lines() {
        if start.elapsed() > Duration::from_secs(3) {
            break;
        }

        if let Ok(line) = line {
            logs.push(line.clone());
            if logs.len() > 20 {
                break;
            }
        }
    }

    child.kill().expect("Failed to kill server");
    let _ = Command::new("make").arg("dev-down").output();

    let all_logs = logs.join("\n");

    // With RUST_LOG=error, we should NOT see INFO logs
    assert!(
        !all_logs.contains("INFO"),
        "Should not see INFO logs with RUST_LOG=error"
    );
}
