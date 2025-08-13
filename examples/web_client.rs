use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3000";

    println!("Testing Number Guessing Game Web API");
    println!("=====================================\n");

    // Create a new game
    println!("Creating a new game (1-10)...");
    let response = client
        .post(format!("{}/api/games", base_url))
        .json(&json!({
            "min": 1,
            "max": 10
        }))
        .send()
        .await?;

    let game_response: serde_json::Value = response.json().await?;
    println!(
        "Response: {}\n",
        serde_json::to_string_pretty(&game_response)?
    );

    let game_id = game_response["game_id"].as_u64().unwrap();
    println!("Game ID: {}\n", game_id);

    // Make some guesses
    let guesses = vec![5, 3, 7, 2, 8, 4, 6, 9, 1, 10];

    for guess in guesses {
        println!("Making guess: {}", guess);
        let response = client
            .post(format!("{}/api/games/{}/guess", base_url, game_id))
            .json(&json!({
                "guess": guess
            }))
            .send()
            .await?;

        let guess_response: serde_json::Value = response.json().await?;
        println!(
            "Response: {}",
            serde_json::to_string_pretty(&guess_response)?
        );

        if guess_response["result"] == "correct" {
            println!("\nGame completed!");
            break;
        }
        println!();
    }

    Ok(())
}
