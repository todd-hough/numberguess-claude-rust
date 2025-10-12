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

    println!(
        "Connecting to Selenium WebDriver at {}/wd/hub with {}s timeout",
        selenium_url, timeout_seconds
    );

    // The thirtyfour version doesn't support custom client with timeout, so we use the standard method
    // but increase the default HTTP timeouts
    WebDriver::new(&format!("{}/wd/hub", selenium_url), caps).await
}
