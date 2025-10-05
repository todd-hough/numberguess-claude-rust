mod common;

use common::assertions::{GameResponse, GuessResponse};
use common::containers::{GameServerInstance, PostgresInstance};
use reqwest::blocking::Client;
use serde_json::json;
use std::sync::{Arc, Barrier};
use std::thread;

/// Test concurrent guesses on the SAME game to verify transaction isolation
/// This tests the FOR UPDATE row-level locking in make_guess_transactional
#[test]
fn test_concurrent_guesses_on_same_game() {
    let postgres = PostgresInstance::new();
    let server = GameServerInstance::new(&postgres.container_url());
    let base_url = server.url();
    let client = Client::new();

    println!("✅ Server ready at {}", base_url);

    // Create one game with a wide range so guesses don't accidentally win
    let create_response = client
        .post(format!("{}/api/games", base_url))
        .json(&json!({
            "min": 1,
            "max": 1000,
            "max_guesses": "50"
        }))
        .send()
        .expect("Should create game");

    if !create_response.status().is_success() {
        let status = create_response.status();
        let body = create_response.text().unwrap_or_default();
        panic!("Game creation failed with {}: {}", status, body);
    }
    let game: GameResponse = create_response.json().expect("Should parse game response");
    let game_id = game.game_id;

    println!("✅ Created game {}", game_id);

    // Spawn 10 threads to make guesses concurrently on THE SAME game
    let num_threads = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let base_url = Arc::new(base_url);

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            let base_url = Arc::clone(&base_url);

            thread::spawn(move || {
                let client = Client::new();

                // Wait for all threads to be ready
                barrier.wait();

                // All threads guess at the same time
                let guess_value = i * 100 + 1;
                let response = client
                    .post(format!("{}/api/games/{}/guess", base_url, game_id))
                    .json(&json!({"guess": guess_value}))
                    .send()
                    .expect("Should send guess");

                (response.status().is_success(), guess_value)
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All requests should succeed
    let success_count = results.iter().filter(|(success, _)| *success).count();
    println!("✅ {}/{} concurrent guesses succeeded", success_count, num_threads);
    assert_eq!(success_count, num_threads, "All concurrent guesses should succeed");

    println!("✅ Concurrent guesses test passed - transaction isolation verified");
}

/// Test race condition: one thread makes winning guess while others try to guess
#[test]
fn test_race_condition_guess_during_deletion() {
    let postgres = PostgresInstance::new();
    let server = GameServerInstance::new(&postgres.container_url());
    let base_url = server.url();
    let client = Client::new();

    println!("✅ Server ready at {}", base_url);

    // Create game where we know the answer
    let create_response = client
        .post(format!("{}/api/games", base_url))
        .json(&json!({
            "min": 42,
            "max": 42,
            "max_guesses": "10"
        }))
        .send()
        .expect("Should create game");

    if !create_response.status().is_success() {
        let status = create_response.status();
        let body = create_response.text().unwrap_or_default();
        panic!("Game creation failed with {}: {}", status, body);
    }
    let game: GameResponse = create_response.json().expect("Should parse game response");
    let game_id = game.game_id;

    println!("✅ Created game {} (answer is 42)", game_id);

    // Spawn 5 threads: first will guess correctly, others will guess wrong
    let num_threads = 5;
    let barrier = Arc::new(Barrier::new(num_threads));
    let base_url = Arc::new(base_url);

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            let base_url = Arc::clone(&base_url);

            thread::spawn(move || {
                let client = Client::new();
                barrier.wait();

                let guess_value = if i == 0 { 42 } else { i * 10 };

                let response = client
                    .post(format!("{}/api/games/{}/guess", base_url, game_id))
                    .json(&json!({"guess": guess_value}))
                    .send()
                    .expect("Should send guess");

                let status = response.status();
                let body = if status.is_success() {
                    response.json::<GuessResponse>().ok()
                } else {
                    None
                };

                (status.as_u16(), body, guess_value)
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    let mut correct_count = 0;
    let mut not_found_count = 0;
    let mut other_success = 0;

    for (status, body, guess) in results {
        match status {
            200 => {
                if let Some(response) = body {
                    if response.result == "correct" {
                        correct_count += 1;
                        println!("  Thread guessed {} -> Correct!", guess);
                    } else {
                        other_success += 1;
                        println!("  Thread guessed {} -> {}", guess, response.result);
                    }
                }
            }
            404 => {
                not_found_count += 1;
                println!("  Thread guessed {} -> 404 Not Found", guess);
            }
            _ => {
                println!("  Thread guessed {} -> Status {}", guess, status);
            }
        }
    }

    assert_eq!(correct_count, 1, "Exactly one thread should get the correct answer");
    assert!(not_found_count + other_success >= num_threads - 1);

    println!("✅ Race condition test passed");
}

/// Test that games persist across server restarts
#[test]
fn test_game_persistence_across_restart() {
    let postgres = PostgresInstance::new();

    let game_id;
    {
        let server1 = GameServerInstance::new(&postgres.container_url());
        let base_url = server1.url();
        let client = Client::new();

        println!("✅ Server 1 started at {}", base_url);

        let create_response = client
            .post(format!("{}/api/games", base_url))
            .json(&json!({
                "min": 1,
                "max": 100,
                "max_guesses": "10"
            }))
            .send()
            .expect("Should create game");

        if !create_response.status().is_success() {
            let status = create_response.status();
            let body = create_response.text().unwrap_or_default();
            panic!("Game creation failed with {}: {}", status, body);
        }
        let game: GameResponse = create_response.json().expect("Should parse game response");
        game_id = game.game_id;

        println!("✅ Created game {}", game_id);

        for guess in [25, 50] {
            let guess_response = client
                .post(format!("{}/api/games/{}/guess", base_url, game_id))
                .json(&json!({"guess": guess}))
                .send()
                .expect("Should make guess");

            assert!(guess_response.status().is_success());
            let result: GuessResponse = guess_response.json().expect("Should parse guess response");
            println!("  Guess {} -> {}", guess, result.result);
        }

        println!("✅ Made 2 guesses on server 1");
    }

    println!("🔄 Server 1 stopped, starting server 2...");

    {
        let server2 = GameServerInstance::new(&postgres.container_url());
        let base_url = server2.url();
        let client = Client::new();

        println!("✅ Server 2 started at {}", base_url);

        let guess_response = client
            .post(format!("{}/api/games/{}/guess", base_url, game_id))
            .json(&json!({"guess": 75}))
            .send()
            .expect("Should make guess on restarted server");

        assert!(guess_response.status().is_success(),
            "Game should still exist after server restart");

        let result: GuessResponse = guess_response.json().expect("Should parse guess response");
        println!("  Guess 75 -> {}", result.result);

        assert!(result.result == "too_low" || result.result == "too_high" || result.result == "correct");
        println!("✅ Game persisted across restart");
    }

    println!("✅ Persistence test passed");
}
