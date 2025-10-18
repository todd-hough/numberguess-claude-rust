mod common;

use common::environment;
use tokio_test;

#[test]
fn test_web_ui_game_flow() {
    let base_url = environment::ensure_server_ready();
    let browser_url = environment::browser_base_url();
    let selenium_url = match environment::ensure_selenium_ready() {
        Some(url) => url,
        None => {
            println!(
                "Skipping web UI test - Selenium not available. Run via `make test-compose-ui`."
            );
            return;
        }
    };
    let base_url_for_log = base_url.clone();
    let browser_url_for_log = browser_url.clone();
    let selenium_for_log = selenium_url.clone();

    let result = tokio_test::block_on(async move {
        use common::page_objects::{FeedbackType, GamePage};
        use common::webdriver::*;

        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                println!("Failed to create WebDriver: {}", e);
                return false;
            }
        };

        let page = GamePage::new(&driver);

        if let Err(e) = page.goto(browser_url.as_str()).await {
            println!("Failed to navigate to game URL: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Successfully navigated to game URL {}", browser_url);

        // Perform OAuth2 login via Keycloak
        if let Err(e) = page.login("admin@local.test", "password").await {
            println!("Failed to login: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Successfully logged in via Keycloak");

        if let Err(e) = page.start_game(5, 5, Some(10)).await {
            println!("Failed to start game: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Game started with min=5, max=5, limit=10");

        let feedback = match page.make_guess(5).await {
            Ok(fb) => fb,
            Err(e) => {
                println!("Failed to make guess: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        println!("Made guess: 5");

        let message = page
            .get_feedback_message()
            .await
            .unwrap_or_else(|_| String::from("[Could not get feedback]"));
        println!("Feedback message: {}", message);

        if let Err(e) = page.quit().await {
            println!("Failed to quit WebDriver: {}", e);
        }

        feedback == FeedbackType::Correct
    });

    assert!(result, "Web UI test should find the correct answer");
    println!(
        "✅ Web UI test passed with API at {}, browser URL {}, selenium at {}",
        base_url_for_log, browser_url_for_log, selenium_for_log
    );
}

#[test]
fn test_web_ui_invalid_inputs() {
    let base_url = environment::ensure_server_ready();
    let browser_url = environment::browser_base_url();
    let selenium_url = match environment::ensure_selenium_ready() {
        Some(url) => url,
        None => {
            println!(
                "Skipping web UI test - Selenium not available. Run via `make test-compose-ui`."
            );
            return;
        }
    };
    let base_url_for_log = base_url.clone();
    let browser_url_for_log = browser_url.clone();
    let selenium_for_log = selenium_url.clone();

    let result = tokio_test::block_on(async move {
        use common::page_objects::GamePage;
        use common::webdriver::*;

        let driver = match create_webdriver(&selenium_url).await {
            Ok(driver) => driver,
            Err(e) => {
                println!("Failed to create WebDriver: {}", e);
                return false;
            }
        };

        let page = GamePage::new(&driver);

        if let Err(e) = page.goto(browser_url.as_str()).await {
            println!("Failed to navigate to game URL: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Successfully navigated to game URL {}", browser_url);

        // Perform OAuth2 login via Keycloak
        if let Err(e) = page.login("admin@local.test", "password").await {
            println!("Failed to login: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Successfully logged in via Keycloak");

        if let Err(e) = page.fill_game_setup(100, 10, None).await {
            println!("Failed to fill game setup: {}", e);
            let _ = page.quit().await;
            return false;
        }

        if let Err(e) = page.submit_game_setup().await {
            println!("Failed to submit game setup: {}", e);
            let _ = page.quit().await;
            return false;
        }

        println!("Game form filled with min=100, max=10");

        let has_error = match page.has_error().await {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to check for error: {}", e);
                let _ = page.quit().await;
                return false;
            }
        };

        println!("Error displayed? {}", has_error);

        if let Err(e) = page.quit().await {
            println!("Failed to quit WebDriver: {}", e);
        }

        has_error
    });

    assert!(
        result,
        "Web UI invalid input test should detect validation errors"
    );
    println!(
        "✅ Web UI invalid input test passed with API at {}, browser URL {}, selenium at {}",
        base_url_for_log, browser_url_for_log, selenium_for_log
    );
}
