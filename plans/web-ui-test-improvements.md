# Web UI Integration Test Improvements

## Analysis Summary

**Date:** 2025-09-29

### Current State
- Tests in `tests/web_ui_test.rs` use process-based game server + Docker Selenium containers
- Recent git diff shows attempted migration to Docker networks (reverted/incomplete)
- Tests marked with `#[ignore]` across multiple files
- Uses `thirtyfour` WebDriver client with blocking `tokio_test`
- Docker image exists: `numberguess-claude-rust:latest`

### Issues Identified
1. **Network isolation problems** - Selenium container can't reach game server on host
2. **Verbose error handling** - excessive boilerplate in test code
3. **Serial test execution** - due to port conflicts and container lifecycle management
4. **Mixed dependencies** - two WebDriver libraries (`fantoccini` and `thirtyfour`)

---

## Option 1: Complete Docker Network Migration ⭐ Recommended

Convert game server to containerized deployment with shared Docker network.

### Benefits
- Solves network isolation issues permanently
- Both containers on same network, reliable connectivity
- Faster startup (no cargo compilation per test)
- Mirrors production deployment better
- Enables parallel test execution

### Implementation Details
- Use `numberguess-claude-rust:latest` image in containers
- Create shared Docker network for game-server + selenium
- Update `GameServerInstance` to use `Container<GameServerImage>`
- Use container names for DNS resolution (e.g., `http://game-server:3000`)

### Code Changes Required

**`tests/common/containers.rs`:**
```rust
#[derive(Debug, Default, Clone)]
pub struct GameServerImage;

impl Image for GameServerImage {
    fn name(&self) -> &str {
        "numberguess-claude-rust"
    }

    fn tag(&self) -> &str {
        "latest"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Starting web server on")]
    }
}

pub struct GameServerInstance<'d, D: Docker> {
    pub container: Container<'d, D, GameServerImage>,
    port: u16,
}

impl<'d, D: Docker> GameServerInstance<'d, D> {
    pub fn new(client: &'d D, network: &str) -> Self {
        let image = GameServerImage::default()
            .with_network(network);
        let container = client.run(image);
        let port = container.get_host_port(3000);
        Self { container, port }
    }
}
```

**Test setup:**
```rust
#[test]
fn test_web_ui_game_flow() {
    let docker = clients::Cli::default();
    let network = "test-network";

    let server = GameServerInstance::new(&docker, network);
    let selenium = SeleniumInstance::new_with_game_server(&docker, network, 90);

    // Use http://game-server:3000 from within selenium container
    let game_url = "http://game-server:3000";
    // ...
}
```

### Effort
Medium (2-3 hours)

---

## Option 2: Add Page Object Model + Test Helpers

Refactor tests to reduce duplication and improve maintainability.

### Benefits
- DRY principle - reusable UI interactions
- More readable test code (business logic visible)
- Easier to maintain when UI changes
- Better error messages and debugging

### Implementation Details

**`tests/common/page_objects.rs`:**
```rust
pub struct GamePage<'a> {
    driver: &'a WebDriver,
}

impl<'a> GamePage<'a> {
    pub fn new(driver: &'a WebDriver) -> Self {
        Self { driver }
    }

    pub async fn fill_game_setup(
        &self,
        min: u32,
        max: u32,
        limit: Option<u32>
    ) -> WebDriverResult<()> {
        let min_input = self.driver.find(By::Id("min")).await?;
        min_input.clear().await?;
        min_input.send_keys(&min.to_string()).await?;

        let max_input = self.driver.find(By::Id("max")).await?;
        max_input.clear().await?;
        max_input.send_keys(&max.to_string()).await?;

        if let Some(limit_value) = limit {
            let limit_input = self.driver.find(By::Id("max_guesses")).await?;
            limit_input.clear().await?;
            limit_input.send_keys(&limit_value.to_string()).await?;
        }

        let submit = self.driver.find(By::Css("button[type='submit']")).await?;
        submit.click().await?;

        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    pub async fn submit_guess(&self, guess: u32) -> WebDriverResult<FeedbackType> {
        let guess_input = self.driver.find(By::Css("input[name='guess']")).await?;
        guess_input.clear().await?;
        guess_input.send_keys(&guess.to_string()).await?;

        let submit = self.driver.find(By::Css(".guess-form button")).await?;
        submit.click().await?;

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check feedback type
        if self.driver.query(By::Css("#feedback.correct"))
            .nowait().exists().await? {
            Ok(FeedbackType::Correct)
        } else if self.driver.query(By::Css("#feedback.too-high"))
            .nowait().exists().await? {
            Ok(FeedbackType::TooHigh)
        } else if self.driver.query(By::Css("#feedback.too-low"))
            .nowait().exists().await? {
            Ok(FeedbackType::TooLow)
        } else {
            Ok(FeedbackType::Unknown)
        }
    }

    pub async fn get_feedback_message(&self) -> WebDriverResult<String> {
        let feedback = self.driver.find(By::Css("#feedback")).await?;
        feedback.text().await
    }
}

#[derive(Debug, PartialEq)]
pub enum FeedbackType {
    Correct,
    TooHigh,
    TooLow,
    Unknown,
}
```

**Updated test:**
```rust
#[test]
fn test_web_ui_game_flow() {
    // ... setup ...

    let result = tokio_test::block_on(async {
        let driver = create_webdriver(&selenium_url).await?;
        driver.goto(&game_url).await?;

        let page = GamePage::new(&driver);

        // Much cleaner test code!
        page.fill_game_setup(5, 5, Some(10)).await?;
        let feedback = page.submit_guess(5).await?;

        assert_eq!(feedback, FeedbackType::Correct);

        driver.quit().await?;
        Ok::<(), WebDriverError>(())
    });

    assert!(result.is_ok());
}
```

### Effort
Low (1-2 hours)

---

## Option 3: Add Screenshot Capture on Failure + Artifacts

Enhance debugging capabilities for failed tests.

### Benefits
- Visual debugging for CI/CD failures
- Captures browser state at failure point
- Can save HTML source for analysis
- Helps diagnose timing/rendering issues

### Implementation Details

**`tests/common/test_utils.rs`:**
```rust
use std::fs;
use std::path::PathBuf;
use chrono::Utc;

pub struct TestArtifacts {
    test_name: String,
    artifact_dir: PathBuf,
}

impl TestArtifacts {
    pub fn new(test_name: &str) -> Self {
        let artifact_dir = PathBuf::from("target/test-artifacts")
            .join(test_name)
            .join(Utc::now().format("%Y%m%d_%H%M%S").to_string());

        fs::create_dir_all(&artifact_dir)
            .expect("Failed to create artifact directory");

        Self {
            test_name: test_name.to_string(),
            artifact_dir,
        }
    }

    pub async fn capture_screenshot(
        &self,
        driver: &WebDriver,
        name: &str
    ) -> WebDriverResult<PathBuf> {
        let screenshot = driver.screenshot_as_png().await?;
        let path = self.artifact_dir.join(format!("{}.png", name));
        fs::write(&path, screenshot)?;
        println!("📸 Screenshot saved: {}", path.display());
        Ok(path)
    }

    pub async fn capture_page_source(
        &self,
        driver: &WebDriver,
        name: &str
    ) -> WebDriverResult<PathBuf> {
        let source = driver.source().await?;
        let path = self.artifact_dir.join(format!("{}.html", name));
        fs::write(&path, source)?;
        println!("📄 Page source saved: {}", path.display());
        Ok(path)
    }

    pub async fn capture_browser_logs(
        &self,
        driver: &WebDriver,
    ) -> WebDriverResult<PathBuf> {
        // Note: Browser logs require specific capabilities
        let logs = driver.logs("browser").await.unwrap_or_default();
        let path = self.artifact_dir.join("browser.log");
        let log_text = logs.iter()
            .map(|e| format!("[{}] {}", e.level, e.message))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, log_text)?;
        println!("📋 Browser logs saved: {}", path.display());
        Ok(path)
    }
}
```

**Updated test with artifact capture:**
```rust
#[test]
fn test_web_ui_game_flow() {
    let artifacts = TestArtifacts::new("test_web_ui_game_flow");

    let result = tokio_test::block_on(async {
        let driver = create_webdriver(&selenium_url).await?;

        match run_test(&driver, &artifacts).await {
            Ok(()) => {
                driver.quit().await?;
                Ok(())
            }
            Err(e) => {
                // Capture debugging artifacts on failure
                let _ = artifacts.capture_screenshot(&driver, "failure").await;
                let _ = artifacts.capture_page_source(&driver, "failure").await;
                let _ = artifacts.capture_browser_logs(&driver).await;

                driver.quit().await?;
                Err(e)
            }
        }
    });

    assert!(result.is_ok(), "Test failed - check artifacts at target/test-artifacts/");
}
```

### Additional Enhancements
- Add `.gitignore` entry for `target/test-artifacts/`
- CI integration: upload artifacts as build artifacts
- Video recording using Selenium's video capture
- Performance metrics (page load time, interaction latency)

### Effort
Low (1 hour)

---

## Recommended Implementation Order

1. **Phase 1: Fix Core Issues (Option 1)**
   - Implement Docker network migration
   - Removes `#[ignore]` from tests
   - Gets tests passing reliably
   - **Priority:** HIGH

2. **Phase 2: Improve Code Quality (Option 2)**
   - Add Page Object Model
   - Refactor existing tests to use POM
   - Remove duplicate WebDriver library
   - **Priority:** MEDIUM

3. **Phase 3: Enhance Debugging (Option 3)**
   - Add screenshot capture
   - Configure artifact collection
   - Set up CI artifact upload
   - **Priority:** LOW (but very useful)

## Success Metrics

- ✅ All web UI tests pass without `#[ignore]`
- ✅ Tests run in parallel (< 30 seconds total)
- ✅ Test code reduced by 40-50% (via POM)
- ✅ Zero network connectivity issues
- ✅ Failed tests produce actionable artifacts

## Dependencies to Add/Remove

**Add:**
```toml
chrono = "0.4"  # For timestamped artifacts
```

**Remove:**
```toml
fantoccini = "0.19.3"  # Consolidate on thirtyfour only
```

## Related Files
- `tests/web_ui_test.rs` - Main test file
- `tests/common/containers.rs` - Container infrastructure
- `tests/common/webdriver.rs` - WebDriver helpers
- `Dockerfile` - Game server image definition
- `Cargo.toml` - Dependencies