use std::time::Duration;
use thirtyfour::prelude::*;

/// Represents the feedback type shown to the user after a guess
#[derive(Debug, PartialEq, Clone)]
pub enum FeedbackType {
    Correct,
    TooHigh,
    TooLow,
    Error,
    Unknown,
}

/// Page Object Model for the Number Guessing Game web interface
pub struct GamePage<'a> {
    driver: &'a WebDriver,
}

impl<'a> GamePage<'a> {
    pub fn new(driver: &'a WebDriver) -> Self {
        Self { driver }
    }

    /// Navigate to the game URL
    pub async fn goto(&self, url: &str) -> WebDriverResult<()> {
        self.driver.goto(url).await
    }

    /// Fill in the game setup form
    pub async fn fill_game_setup(
        &self,
        min: u32,
        max: u32,
        limit: Option<u32>,
    ) -> WebDriverResult<()> {
        // Fill minimum value
        let min_input = self.driver.find(By::Id("min")).await?;
        min_input.clear().await?;
        min_input.send_keys(&min.to_string()).await?;

        // Fill maximum value
        let max_input = self.driver.find(By::Id("max")).await?;
        max_input.clear().await?;
        max_input.send_keys(&max.to_string()).await?;

        // Fill guess limit if provided
        if let Some(limit_value) = limit {
            let limit_input = self.driver.find(By::Id("max_guesses")).await?;
            limit_input.clear().await?;
            limit_input.send_keys(&limit_value.to_string()).await?;
        }

        Ok(())
    }

    /// Submit the game setup form
    pub async fn submit_game_setup(&self) -> WebDriverResult<()> {
        let submit = self.driver.find(By::Css("button[type='submit']")).await?;
        submit.click().await?;

        // Wait for game form to appear (indicates successful submission)
        self.driver
            .query(By::Css(".guess-form"))
            .wait(Duration::from_secs(30), Duration::from_millis(500))
            .first()
            .await?;

        Ok(())
    }

    /// Fill and submit the game setup form in one step
    pub async fn start_game(&self, min: u32, max: u32, limit: Option<u32>) -> WebDriverResult<()> {
        self.fill_game_setup(min, max, limit).await?;
        self.submit_game_setup().await?;
        Ok(())
    }

    /// Make a guess
    pub async fn make_guess(&self, guess: u32) -> WebDriverResult<FeedbackType> {
        // Find the guess input field
        let guess_input = self.driver.find(By::Css("input[name='guess']")).await?;
        guess_input.clear().await?;
        guess_input.send_keys(&guess.to_string()).await?;

        // Click the submit button
        let submit = self.driver.find(By::Css(".guess-form button")).await?;
        submit.click().await?;

        // Wait for feedback to update by checking for any feedback class
        // Use WebDriver's built-in wait with a selector that matches updated feedback
        self.driver
            .query(By::Css("#feedback.active"))
            .wait(Duration::from_secs(5), Duration::from_millis(100))
            .first()
            .await?;

        // Small delay to let DOM settle after HTMX swap
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check feedback type
        self.get_feedback_type().await
    }

    /// Get the current feedback type
    pub async fn get_feedback_type(&self) -> WebDriverResult<FeedbackType> {
        // Check if feedback element exists and is active
        let feedback_active = self
            .driver
            .query(By::Css("#feedback.active"))
            .nowait()
            .exists()
            .await?;

        if !feedback_active {
            return Ok(FeedbackType::Unknown);
        }

        // Check for correct answer
        if self
            .driver
            .query(By::Css("#feedback.correct"))
            .nowait()
            .exists()
            .await?
        {
            return Ok(FeedbackType::Correct);
        }

        // Check for too high
        if self
            .driver
            .query(By::Css("#feedback.too-high"))
            .nowait()
            .exists()
            .await?
        {
            return Ok(FeedbackType::TooHigh);
        }

        // Check for too low
        if self
            .driver
            .query(By::Css("#feedback.too-low"))
            .nowait()
            .exists()
            .await?
        {
            return Ok(FeedbackType::TooLow);
        }

        // If active but not correct/high/low, it's likely an error
        Ok(FeedbackType::Error)
    }

    /// Get the feedback message text
    pub async fn get_feedback_message(&self) -> WebDriverResult<String> {
        let feedback = self.driver.find(By::Css("#feedback")).await?;
        feedback.text().await
    }

    /// Check if an error message is displayed
    pub async fn has_error(&self) -> WebDriverResult<bool> {
        self.driver
            .query(By::Css("#feedback.active"))
            .nowait()
            .exists()
            .await
    }

    /// Get the error message text if present
    pub async fn get_error_message(&self) -> WebDriverResult<Option<String>> {
        if self.has_error().await? {
            Ok(Some(self.get_feedback_message().await?))
        } else {
            Ok(None)
        }
    }

    /// Wait for feedback to appear
    pub async fn wait_for_feedback(&self, timeout_ms: u64) -> WebDriverResult<bool> {
        let result = self
            .driver
            .query(By::Css("#feedback.active"))
            .wait(
                Duration::from_millis(timeout_ms),
                Duration::from_millis(100),
            )
            .exists()
            .await;

        Ok(result.unwrap_or(false))
    }

    /// Check if the game interface is visible (game has been started)
    pub async fn is_game_started(&self) -> WebDriverResult<bool> {
        self.driver
            .query(By::Css(".guess-form"))
            .nowait()
            .exists()
            .await
    }

    /// Quit the WebDriver session
    pub async fn quit(&self) -> WebDriverResult<()> {
        self.driver.clone().quit().await
    }

    // =========================================================================
    // Authentication Methods (Keycloak OAuth2 Login)
    // =========================================================================

    /// Perform Keycloak login if on login page.
    ///
    /// This handles the OAuth2 login flow:
    /// 1. Waits for Keycloak login page to load
    /// 2. Fills in username and password
    /// 3. Submits the login form
    /// 4. Waits for OAuth2 redirect back to application
    ///
    /// # Example
    /// ```no_run
    /// # use tests::common::page_objects::GamePage;
    /// # async fn example(page: &GamePage<'_>) -> Result<(), Box<dyn std::error::Error>> {
    /// page.goto("http://localhost:8080").await?;
    /// page.login("admin@local.test", "password").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn login(&self, username: &str, password: &str) -> WebDriverResult<()> {
        // Wait for login page to load
        self.wait_for_login_page().await?;

        // Find and fill username field
        let username_field = self.driver.find(By::Id("username")).await?;
        username_field.send_keys(username).await?;

        // Find and fill password field
        let password_field = self.driver.find(By::Id("password")).await?;
        password_field.send_keys(password).await?;

        // Submit login form
        let submit_button = self.driver.find(By::Id("kc-login")).await?;
        submit_button.click().await?;

        // Wait for redirect back to application
        self.wait_for_app_redirect().await?;

        Ok(())
    }

    /// Check if currently on Keycloak login page.
    pub async fn is_on_login_page(&self) -> WebDriverResult<bool> {
        self.driver
            .query(By::Id("kc-login"))
            .nowait()
            .exists()
            .await
    }

    /// Wait for redirect to Keycloak login page.
    pub async fn wait_for_login_page(&self) -> WebDriverResult<()> {
        self.driver
            .query(By::Id("kc-login"))
            .wait(Duration::from_secs(10), Duration::from_millis(200))
            .first()
            .await?;
        Ok(())
    }

    /// Wait for OAuth2 redirect back to application and page to load.
    pub async fn wait_for_app_redirect(&self) -> WebDriverResult<()> {
        // Wait for the game form to be visible (indicates redirect completed and page loaded)
        // The #min input only exists on the application's index page, not on Keycloak
        self.driver
            .query(By::Id("min"))
            .wait(Duration::from_secs(15), Duration::from_millis(200))
            .first()
            .await?;
        Ok(())
    }
}
