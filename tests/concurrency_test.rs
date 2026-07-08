mod common;

use common::assertions::{GameResponse, GuessResponse};
use common::{auth_helpers, environment};
use serde_json::json;
use std::process::Command;
use std::sync::Arc;

const COMPOSE_FILES: [&str; 4] = [
    "-f",
    "docker-compose.yml",
    "-f",
    "docker-compose.integration.yml",
];

/// Light-tier compose coordinates. The Makefile (`make test-func`) is the
/// owner and exports MOCK_COMPOSE_FILE / MOCK_COMPOSE_PROJECT; the literals
/// here are only fallbacks for running `MOCK_AUTH=1 cargo test` by hand.
fn mock_compose_args() -> [String; 4] {
    let file = std::env::var("MOCK_COMPOSE_FILE")
        .unwrap_or_else(|_| "docker-compose.test-mock-auth.yml".to_string());
    let project =
        std::env::var("MOCK_COMPOSE_PROJECT").unwrap_or_else(|_| "numberguess-mock".to_string());
    ["-f".to_string(), file, "-p".to_string(), project]
}

fn restart_app_via_compose() {
    let mock_args = mock_compose_args();
    let (compose_args, restart_args): (Vec<&str>, [&str; 2]) = if environment::is_mock_auth() {
        (
            mock_args.iter().map(String::as_str).collect(),
            ["restart", "app"],
        )
    } else {
        let mut v = COMPOSE_FILES.to_vec();
        v.extend_from_slice(&["--profile", "integration"]);
        (v, ["restart", "app"])
    };

    // Guard against a silent no-op: `docker compose restart` on a project with
    // no running app container can exit 0 without restarting anything, which
    // would let the persistence test pass while testing nothing.
    let ps = Command::new("docker")
        .arg("compose")
        .args(&compose_args)
        .args(["ps", "-q", "app"])
        .output()
        .expect("Failed to run docker compose ps");
    assert!(
        !String::from_utf8_lossy(&ps.stdout).trim().is_empty(),
        "No running 'app' container found for compose args {:?} — project/file mismatch? \
         (Makefile exports MOCK_COMPOSE_FILE/MOCK_COMPOSE_PROJECT for the light tier.)",
        compose_args
    );

    let status = Command::new("docker")
        .arg("compose")
        .args(&compose_args)
        .args(restart_args)
        .status()
        .expect("Failed to run docker compose restart app");

    assert!(
        status.success(),
        "docker compose restart app failed with status {:?}",
        status
    );

    // Wait for app to come back online using existing helper
    environment::ensure_server_ready();
}

/// Test concurrent guesses on the SAME game to verify transaction isolation
/// This tests the FOR UPDATE row-level locking in the repository implementation
#[tokio::test]
async fn test_concurrent_guesses_on_same_game() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client
    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create authenticated client");

    println!("Created authenticated client");

    // Create one game with a wide range so guesses don't accidentally win
    let create_response = client
        .post("http://localhost:8080/api/games")
        .json(&json!({
            "min": 1,
            "max": 1000,
            "max_guesses": "50"
        }))
        .send()
        .await
        .expect("Should create game");

    if !create_response.status().is_success() {
        let status = create_response.status();
        let body = create_response.text().await.unwrap_or_default();
        panic!("Game creation failed with {}: {}", status, body);
    }
    let game: GameResponse = create_response
        .json()
        .await
        .expect("Should parse game response");
    let game_id = game.game_id;

    println!("Created game {}", game_id);

    // Spawn 10 async tasks to make guesses concurrently on THE SAME game
    let num_tasks = 10;
    let barrier = Arc::new(tokio::sync::Barrier::new(num_tasks));

    let handles: Vec<_> = (0..num_tasks)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            let client = client.clone();

            tokio::task::spawn(async move {
                // Wait for all tasks to be ready
                barrier.wait().await;

                // All tasks guess at the same time
                let guess_value = i * 100 + 1;
                let response = client
                    .post(format!("http://localhost:8080/api/games/{}/guess", game_id))
                    .json(&json!({"guess": guess_value}))
                    .send()
                    .await
                    .expect("Should send guess");

                (response.status().is_success(), guess_value)
            })
        })
        .collect();

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // All requests should succeed
    let success_count = results.iter().filter(|(success, _)| *success).count();
    println!(
        "{}/{} concurrent guesses succeeded",
        success_count, num_tasks
    );
    assert_eq!(
        success_count, num_tasks,
        "All concurrent guesses should succeed"
    );

    println!("Concurrent guesses test passed - transaction isolation verified");
}

/// Test race condition: one task makes winning guess while others try to guess
#[tokio::test]
async fn test_race_condition_guess_during_deletion() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client
    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create authenticated client");

    println!("Created authenticated client");

    // Create game where we know the answer
    let create_response = client
        .post("http://localhost:8080/api/games")
        .json(&json!({
            "min": 42,
            "max": 42,
            "max_guesses": "10"
        }))
        .send()
        .await
        .expect("Should create game");

    if !create_response.status().is_success() {
        let status = create_response.status();
        let body = create_response.text().await.unwrap_or_default();
        panic!("Game creation failed with {}: {}", status, body);
    }
    let game: GameResponse = create_response
        .json()
        .await
        .expect("Should parse game response");
    let game_id = game.game_id;

    println!("Created game {} (answer is 42)", game_id);

    // Spawn 5 async tasks: first will guess correctly, others will guess wrong
    let num_tasks = 5;
    let barrier = Arc::new(tokio::sync::Barrier::new(num_tasks));

    let handles: Vec<_> = (0..num_tasks)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            let client = client.clone();

            tokio::task::spawn(async move {
                barrier.wait().await;

                let guess_value = if i == 0 { 42 } else { i * 10 };

                let response = client
                    .post(format!("http://localhost:8080/api/games/{}/guess", game_id))
                    .json(&json!({"guess": guess_value}))
                    .send()
                    .await
                    .expect("Should send guess");

                let status = response.status();
                let body = if status.is_success() {
                    response.json::<GuessResponse>().await.ok()
                } else {
                    None
                };

                (status.as_u16(), body, guess_value)
            })
        })
        .collect();

    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    let mut correct_count = 0;
    let mut not_found_count = 0;
    let mut other_success = 0;

    for (status, body, guess) in results {
        match status {
            200 => {
                if let Some(response) = body {
                    if response.result == "correct" {
                        correct_count += 1;
                        println!("  Task guessed {} -> Correct!", guess);
                    } else {
                        other_success += 1;
                        println!("  Task guessed {} -> {}", guess, response.result);
                    }
                }
            }
            404 => {
                not_found_count += 1;
                println!("  Task guessed {} -> 404 Not Found", guess);
            }
            _ => {
                println!("  Task guessed {} -> Status {}", guess, status);
            }
        }
    }

    assert_eq!(
        correct_count, 1,
        "Exactly one task should get the correct answer"
    );
    assert!(not_found_count + other_success >= num_tasks - 1);

    println!("Race condition test passed");
}

/// Test that games persist across server restarts
#[tokio::test]
async fn test_game_persistence_across_restart() {
    // Run environment checks in blocking context
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready();
    })
    .await
    .expect("Environment checks failed");

    // Create authenticated client
    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create authenticated client");

    println!("Created authenticated client");

    let create_response = client
        .post("http://localhost:8080/api/games")
        .json(&json!({
            "min": 1,
            "max": 100,
            "max_guesses": "10"
        }))
        .send()
        .await
        .expect("Should create game");

    if !create_response.status().is_success() {
        let status = create_response.status();
        let body = create_response.text().await.unwrap_or_default();
        panic!("Game creation failed with {}: {}", status, body);
    }
    let game: GameResponse = create_response
        .json()
        .await
        .expect("Should parse game response");
    let game_id = game.game_id;

    println!("Created game {}; restarting app container...", game_id);

    // Restart app in blocking context
    tokio::task::spawn_blocking(move || {
        restart_app_via_compose();
        environment::ensure_server_ready();
    })
    .await
    .expect("Restart failed");

    // After restart, we need a new authenticated client (session may have been lost)
    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create authenticated client after restart");

    let guess_response = client
        .post(format!("http://localhost:8080/api/games/{}/guess", game_id))
        .json(&json!({"guess": 75}))
        .send()
        .await
        .expect("Should make guess on restarted server");

    assert!(
        guess_response.status().is_success(),
        "Game should still exist after server restart"
    );

    let result: GuessResponse = guess_response
        .json()
        .await
        .expect("Should parse guess response");
    println!("  Guess 75 -> {}", result.result);

    assert!(matches!(
        result.result.as_str(),
        "too_low" | "too_high" | "correct"
    ));
    println!("Game persisted across restart");
}
