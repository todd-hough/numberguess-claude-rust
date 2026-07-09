use thirtyfour::prelude::*;

// Helper function to create a new WebDriver client connected to Selenium
pub async fn create_webdriver(selenium_url: &str) -> WebDriverResult<WebDriver> {
    create_webdriver_with_timeout(selenium_url, 30).await
}

// Helper function to create a new WebDriver client with configurable timeout
pub async fn create_webdriver_with_timeout(
    selenium_url: &str,
    timeout_seconds: u64,
) -> WebDriverResult<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;

    println!("Connecting to Selenium WebDriver at {selenium_url} with {timeout_seconds}s timeout");

    // Selenium 4.x: Use the URL directly without /wd/hub suffix
    // (thirtyfour 0.36+ resolves WebDriver API paths relative to server_url)
    WebDriver::new(selenium_url, caps).await
}
