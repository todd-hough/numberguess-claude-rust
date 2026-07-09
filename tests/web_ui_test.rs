mod common;

use common::environment;

#[tokio::test]
async fn test_web_ui_game_flow() {
    use common::page_objects::{FeedbackType, GamePage};
    use common::webdriver::*;

    // Environment checks in blocking context
    let (base_url, browser_url, selenium_url) = tokio::task::spawn_blocking(|| {
        let base_url = environment::ensure_server_ready();
        let browser_url = environment::browser_base_url();
        let selenium_url = environment::ensure_selenium_ready();
        (base_url, browser_url, selenium_url)
    })
    .await
    .expect("Environment checks failed");

    let base_url_for_log = base_url.clone();
    let browser_url_for_log = browser_url.clone();
    let selenium_for_log = selenium_url.clone();

    // Create WebDriver with direct await
    let driver = create_webdriver(&selenium_url)
        .await
        .expect("Failed to create WebDriver");

    let page = GamePage::new(&driver);

    // Navigate to game URL
    page.goto(browser_url.as_str())
        .await
        .expect("Failed to navigate to game URL");

    println!("Successfully navigated to game URL {browser_url}");

    // Perform OAuth2 login via Keycloak
    page.login("admin@local.test", "password")
        .await
        .expect("Failed to login");

    println!("Successfully logged in via Keycloak");

    // Start game
    page.start_game(5, 5, Some(10))
        .await
        .expect("Failed to start game");

    println!("Game started with min=5, max=5, limit=10");

    // Make guess
    let feedback = page.make_guess(5).await.expect("Failed to make guess");

    println!("Made guess: 5");

    let message = page
        .get_feedback_message()
        .await
        .unwrap_or_else(|_| String::from("[Could not get feedback]"));
    println!("Feedback message: {message}");

    // Cleanup
    page.quit().await.ok();

    assert_eq!(
        feedback,
        FeedbackType::Correct,
        "Web UI test should find the correct answer"
    );
    println!(
        "Web UI test passed with API at {base_url_for_log}, browser URL {browser_url_for_log}, selenium at {selenium_for_log}"
    );
}

#[tokio::test]
async fn test_web_ui_invalid_inputs() {
    use common::page_objects::GamePage;
    use common::webdriver::*;

    // Environment checks in blocking context
    let (base_url, browser_url, selenium_url) = tokio::task::spawn_blocking(|| {
        let base_url = environment::ensure_server_ready();
        let browser_url = environment::browser_base_url();
        let selenium_url = environment::ensure_selenium_ready();
        (base_url, browser_url, selenium_url)
    })
    .await
    .expect("Environment checks failed");

    let base_url_for_log = base_url.clone();
    let browser_url_for_log = browser_url.clone();
    let selenium_for_log = selenium_url.clone();

    // Create WebDriver with direct await
    let driver = create_webdriver(&selenium_url)
        .await
        .expect("Failed to create WebDriver");

    let page = GamePage::new(&driver);

    // Navigate to game URL
    page.goto(browser_url.as_str())
        .await
        .expect("Failed to navigate to game URL");

    println!("Successfully navigated to game URL {browser_url}");

    // Perform OAuth2 login via Keycloak
    page.login("admin@local.test", "password")
        .await
        .expect("Failed to login");

    println!("Successfully logged in via Keycloak");

    // Fill game setup with invalid inputs (min > max)
    page.fill_game_setup(100, 10, None)
        .await
        .expect("Failed to fill game setup");

    // Click submit (don't use submit_game_setup which expects success)
    let submit = driver
        .find(thirtyfour::By::Css("button[type='submit']"))
        .await
        .expect("Should find submit button");
    submit.click().await.expect("Should click submit");

    println!("Game form submitted with invalid inputs: min=100, max=10");

    // Wait for error feedback to appear
    let has_error = page
        .wait_for_feedback(5000)
        .await
        .expect("Failed to wait for feedback");

    println!("Error displayed? {has_error}");

    // Cleanup
    page.quit().await.ok();

    assert!(
        has_error,
        "Web UI invalid input test should detect validation errors"
    );
    println!(
        "Web UI invalid input test passed with API at {base_url_for_log}, browser URL {browser_url_for_log}, selenium at {selenium_for_log}"
    );
}
