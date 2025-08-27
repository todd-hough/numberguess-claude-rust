mod common;

use common::containers::{GameServerInstance, SeleniumInstance};
use std::time::Duration;
use std::process::Command;

// Use thirtyfour library for WebDriver client
use thirtyfour::prelude::*;
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
    
    // Start Game Server with random available port
    let game_server = GameServerInstance::new();
    let game_url = game_server.url();
    let game_port = game_url.split(':').last().unwrap().parse::<u16>().unwrap();
    println!("Game server started at {}", game_url);
    
    // Start a real Selenium instance with Docker container that knows about the game server
    let selenium = SeleniumInstance::new_with_game_server(game_port, 90);
    let selenium_url = selenium.url();
    let container_game_url = selenium.game_server_url();
    println!("Selenium started at {}", selenium_url);
    println!("Game server accessible from container at {}", container_game_url);
    
    // Use thirtyfour WebDriver client with our Selenium instance
    let result = tokio_test::block_on(async move {
        use common::webdriver::*;
        
        // Create new WebDriver session
        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                println!("Failed to create WebDriver: {}", e);
                return false;
            }
        };
        
        // Navigate to game URL (use the container-accessible URL)
        if let Err(e) = driver.goto(&container_game_url).await {
            println!("Failed to navigate to game URL: {}", e);
            let _ = driver.quit().await;
            return false;
        }
        
        println!("Successfully navigated to game URL");
        
        // Fill in the game setup form with small range for deterministic testing
        // Using min=5, max=5 guarantees the answer is 5
        // Find and fill the min field
        match driver.find(By::Id("min")).await {
            Ok(element) => {
                if let Err(e) = element.clear().await {
                    println!("Failed to clear min field: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
                if let Err(e) = element.send_keys("5").await {
                    println!("Failed to fill min field: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
            },
            Err(e) => {
                println!("Failed to find min field: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        }
        
        // Find and fill the max field
        match driver.find(By::Id("max")).await {
            Ok(element) => {
                if let Err(e) = element.clear().await {
                    println!("Failed to clear max field: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
                if let Err(e) = element.send_keys("5").await {
                    println!("Failed to fill max field: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
            },
            Err(e) => {
                println!("Failed to find max field: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        }
        
        // Find and fill the limit field
        match driver.find(By::Id("max_guesses")).await {
            Ok(element) => {
                if let Err(e) = element.clear().await {
                    println!("Failed to clear limit field: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
                if let Err(e) = element.send_keys("10").await {
                    println!("Failed to fill limit field: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
            },
            Err(e) => {
                println!("Failed to find limit field: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        }
        
        println!("Game form filled with min=5, max=5, limit=10");
        
        // Submit the form to create the game
        match driver.find(By::Css("button[type='submit']")).await {
            Ok(element) => {
                if let Err(e) = element.click().await {
                    println!("Failed to click submit button: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
                
                // Wait briefly for game interface to appear
                std::thread::sleep(Duration::from_millis(500));
                println!("Game form submitted successfully");
            },
            Err(e) => {
                println!("Failed to find submit button: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        }
        
        // Since we know the answer is 5, make that guess
        // Find the guess input field
        match driver.find(By::Css("input[name='guess']")).await {
            Ok(element) => {
                if let Err(e) = element.clear().await {
                    println!("Failed to clear guess input: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
                if let Err(e) = element.send_keys("5").await {
                    println!("Failed to enter guess: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
            },
            Err(e) => {
                println!("Failed to find guess input: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        }
        
        // Click the submit button for the guess
        match driver.find(By::Css(".guess-form button")).await {
            Ok(element) => {
                if let Err(e) = element.click().await {
                    println!("Failed to submit guess: {}", e);
                    let _ = driver.quit().await;
                    return false;
                }
                // Wait briefly for feedback
                std::thread::sleep(Duration::from_millis(100));
                println!("Made guess: 5");
            },
            Err(e) => {
                println!("Failed to find guess submit button: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        }
        
        // Check if the guess was correct
        let correct = match driver.query(By::Css("#feedback.correct")).nowait().exists().await {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to check if guess is correct: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        };
        
        let message = match driver.find(By::Css("#feedback")).await {
            Ok(element) => match element.text().await {
                Ok(text) => text,
                Err(_) => String::from("[Could not get feedback text]"),
            },
            Err(_) => String::from("[Could not find feedback element]"),
        };
        
        println!("Feedback message: {}", message);
        
        // Close the browser
        if let Err(e) = driver.quit().await {
            println!("Failed to quit WebDriver: {}", e);
        }
        
        correct
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
    
    // Start Game Server with random available port
    let game_server = GameServerInstance::new();
    let game_url = game_server.url();
    let game_port = game_url.split(':').last().unwrap().parse::<u16>().unwrap();
    println!("Game server started at {}", game_url);
    
    // Start a real Selenium instance with Docker container that knows about the game server
    let selenium = SeleniumInstance::new_with_game_server(game_port, 90);
    let selenium_url = selenium.url();
    let container_game_url = selenium.game_server_url();
    println!("Selenium started at {}", selenium_url);
    println!("Game server accessible from container at {}", container_game_url);
    
    // Run the actual test
    let result = tokio_test::block_on(async {
        use common::webdriver::*;
        
        // Create new WebDriver session
        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                println!("Failed to create WebDriver: {}", e);
                return false;
            }
        };
        
        // Navigate to game URL (use the container-accessible URL)
        if let Err(e) = driver.goto(&container_game_url).await {
            println!("Failed to navigate to game URL: {}", e);
            let _ = driver.quit().await;
            return false;
        }
        
        println!("Successfully navigated to game URL");
        
        // Fill in the game form with invalid parameters (min > max)
        let min_input = match driver.find(By::Id("min")).await {
            Ok(element) => element,
            Err(e) => {
                println!("Failed to find min input: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        };
        
        // Clear the field first to ensure we don't append to existing value
        if let Err(e) = min_input.clear().await {
            println!("Failed to clear min input: {}", e);
            let _ = driver.quit().await;
            return false;
        }
        
        if let Err(e) = min_input.send_keys("100").await {
            println!("Failed to fill min input: {}", e);
            let _ = driver.quit().await;
            return false;
        }
        
        
        let max_input = match driver.find(By::Id("max")).await {
            Ok(element) => element,
            Err(e) => {
                println!("Failed to find max input: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        };
        
        // Clear the field first to ensure we don't append to existing value  
        if let Err(e) = max_input.clear().await {
            println!("Failed to clear max input: {}", e);
            let _ = driver.quit().await;
            return false;
        }
        
        if let Err(e) = max_input.send_keys("10").await {
            println!("Failed to fill max input: {}", e);
            let _ = driver.quit().await;
            return false;
        }
        
        
        // Submit the form
        let submit_button = match driver.find(By::Css("button[type='submit']")).await {
            Ok(element) => element,
            Err(e) => {
                println!("Failed to find submit button: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        };
        if let Err(e) = submit_button.click().await {
            println!("Failed to click submit button: {}", e);
            let _ = driver.quit().await;
            return false;
        }
        
        // Wait a bit for the error to appear
        std::thread::sleep(Duration::from_millis(500));
        
        // Check if error message is displayed - errors use #feedback with active class
        let error_exists = match driver.query(By::Css("#feedback.active")).nowait().exists().await {
            Ok(exists) => exists,
            Err(e) => {
                println!("Failed to check for error message: {}", e);
                let _ = driver.quit().await;
                return false;
            }
        };
        
        // Get error message text if available
        if error_exists {
            if let Ok(error_element) = driver.find(By::Css("#feedback.active")).await {
                if let Ok(error_text) = error_element.text().await {
                    println!("Error message: {}", error_text);
                }
            }
        }
        
        // Close the browser
        if let Err(e) = driver.quit().await {
            println!("Failed to quit WebDriver: {}", e);
        }
        
        error_exists
    });
    
    assert!(result, "Web UI should show error for invalid inputs");
    println!("✅ Web UI invalid inputs test passed with game server at {} and selenium at {}", 
             game_server.url(), selenium.url());
}