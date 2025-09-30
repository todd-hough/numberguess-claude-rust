mod common;

use common::containers::{GameServerInstance, SeleniumInstance};
use std::process::Command;

// Use tokio_test for blocking async operations
use tokio_test;

#[test]
fn test_web_ui_game_flow() {
    // Skip this test if Docker is not available or not running
    let docker_available = match Command::new("docker").args(["info"]).output() {
        Ok(output) => output.status.success(),
        Err(e) => {
            println!("Docker command failed: {}", e);
            false
        }
    };

    if !docker_available {
        println!("Skipping web UI test - Docker not available or not running");
        return;
    }

    // Print Docker version information for debugging
    if let Ok(output) = Command::new("docker").args(["version"]).output() {
        if let Ok(version) = std::str::from_utf8(&output.stdout) {
            println!("Docker version: {}", version.trim());
        }
    }

    // Start Game Server container
    let game_server = GameServerInstance::new();
    let game_url = game_server.url();
    let container_game_url = game_server.internal_url();
    println!("Game server started at {} (host)", game_url);
    println!("Game server internal URL: {}", container_game_url);

    // Start Selenium container (will be on same bridge network)
    let selenium = SeleniumInstance::new_with_timeout(90);
    let selenium_url = selenium.url();
    println!("Selenium started at {}", selenium_url);

    // Use thirtyfour WebDriver client with our Selenium instance
    let result = tokio_test::block_on(async move {
        use common::webdriver::*;
        use common::page_objects::{GamePage, FeedbackType};

        // Create new WebDriver session
        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                println!("Failed to create WebDriver: {}", e);
                return false;
            }
        };

        // Create page object
        let page = GamePage::new(&driver);

        // Navigate to game URL
        if let Err(e) = page.goto(container_game_url.as_str()).await {
            println!("Failed to navigate to game URL: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Successfully navigated to game URL");

        // Start game with min=5, max=5, limit=10
        // Using min=5, max=5 guarantees the answer is 5
        if let Err(e) = page.start_game(5, 5, Some(10)).await {
            println!("Failed to start game: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Game started with min=5, max=5, limit=10");

        // Make a guess (we know the answer is 5)
        let feedback = match page.make_guess(5).await {
            Ok(fb) => fb,
            Err(e) => {
                println!("Failed to make guess: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        println!("Made guess: 5");

        // Get feedback message
        let message = page.get_feedback_message().await.unwrap_or_else(|_| String::from("[Could not get feedback]"));
        println!("Feedback message: {}", message);

        // Close the browser
        if let Err(e) = page.quit().await {
            println!("Failed to quit WebDriver: {}", e);
        }

        feedback == FeedbackType::Correct
    });
    
    assert!(result, "Web UI test should find the correct answer");
    println!("✅ Web UI test passed with game server at {} and selenium at {}", 
             game_server.url(), selenium.url());
}

// Test for invalid inputs
#[test]
fn test_web_ui_invalid_inputs() {
    // Skip this test if Docker is not available or not running
    let docker_available = match Command::new("docker").args(["info"]).output() {
        Ok(output) => output.status.success(),
        Err(e) => {
            println!("Docker command failed: {}", e);
            false
        }
    };

    if !docker_available {
        println!("Skipping web UI test - Docker not available or not running");
        return;
    }

    // Print Docker version information for debugging
    if let Ok(output) = Command::new("docker").args(["version"]).output() {
        if let Ok(version) = std::str::from_utf8(&output.stdout) {
            println!("Docker version: {}", version.trim());
        }
    }

    // Start Game Server container
    let game_server = GameServerInstance::new();
    let game_url = game_server.url();
    let container_game_url = game_server.internal_url();
    println!("Game server started at {} (host)", game_url);
    println!("Game server internal URL: {}", container_game_url);

    // Start Selenium container (will be on same bridge network)
    let selenium = SeleniumInstance::new_with_timeout(90);
    let selenium_url = selenium.url();
    println!("Selenium started at {}", selenium_url);

    // Run the actual test
    let result = tokio_test::block_on(async {
        use common::webdriver::*;
        use common::page_objects::GamePage;

        // Create new WebDriver session
        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                println!("Failed to create WebDriver: {}", e);
                return false;
            }
        };

        // Create page object
        let page = GamePage::new(&driver);

        // Navigate to game URL
        if let Err(e) = page.goto(container_game_url.as_str()).await {
            println!("Failed to navigate to game URL: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Successfully navigated to game URL");

        // Fill form with invalid parameters (min > max)
        if let Err(e) = page.fill_game_setup(100, 10, None).await {
            println!("Failed to fill game setup: {}", e);
            let _ = page.quit().await;
            return false;
        }

        // Submit the form
        if let Err(e) = page.submit_game_setup().await {
            println!("Failed to submit game setup: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Game form filled with min=100, max=10");

        // Check if error message is displayed
        let has_error = match page.has_error().await {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to check for error: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        // Get error message if available
        if has_error {
            if let Ok(Some(error_text)) = page.get_error_message().await {
                println!("Error message: {}", error_text);
            }
        }

        // Close the browser
        if let Err(e) = page.quit().await {
            println!("Failed to quit WebDriver: {}", e);
        }

        has_error
    });
    
    assert!(result, "Web UI should show error for invalid inputs");
    println!("✅ Web UI invalid inputs test passed with game server at {} and selenium at {}", 
             game_server.url(), selenium.url());
}