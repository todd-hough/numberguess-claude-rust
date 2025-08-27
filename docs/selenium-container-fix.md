# Selenium Container Timeout Fix

## Problem Summary

The Selenium container in our integration tests was failing with `WaitContainer(StartupTimeout)` errors. The container was starting successfully but the testcontainers library was not detecting the expected "Selenium Grid ready" message in the container logs.

## Solution Implemented

We've implemented a robust solution with multiple layers of readiness detection:

1. **Multi-condition Ready Check**: 
   - Added HTTP readiness check as primary mechanism
   - Kept log message detection as backup
   - Added port binding verification as last-resort fallback

2. **Configurable Timeouts**:
   - Added timeout parameter to `SeleniumInstance::new_with_timeout()`
   - Increased default timeout from 30s to 60s
   - Propagated timeout to container startup configuration

3. **Enhanced Error Detection**:
   - Added container log capture and display when startup fails
   - Improved logging for all detection methods
   - Added clearer error messages

4. **Minimal Test Case**:
   - Created `tests/selenium_startup_test.rs` to isolate and diagnose container issues
   - Test runs container both manually and via testcontainers-rs

## Technical Details

### HTTP Readiness Check

The primary detection mechanism now uses the HTTP endpoint at `http://localhost:{port}/status`, which is more reliable than parsing log messages.

```rust
// Simple helper function to check if an HTTP endpoint is ready
pub fn check_http_endpoint(url: &str) -> bool {
    let client = Client::new();
    
    match client.get(url).send() {
        Ok(response) => response.status().is_success(),
        Err(_) => false
    }
}
```

### Port Binding Verification

As a last-resort fallback, we check if the port is accepting TCP connections:

```rust
// Port binding verification as a last-resort fallback
pub fn verify_port_binding(host: &str, port: u16, timeout_seconds: u64) -> Result<(), String> {
    println!("Checking port binding for {}:{}", host, port);
    let start = Instant::now();
    let max_duration = Duration::from_secs(timeout_seconds);
    
    while start.elapsed() < max_duration {
        match TcpStream::connect(format!("{}:{}", host, port)) {
            Ok(_) => {
                println!("Successfully connected to {}:{}", host, port);
                return Ok(());
            },
            Err(_) => {}
        }
        
        thread::sleep(Duration::from_millis(500));
    }
    
    Err(format!("Port {}:{} not available after {} seconds", host, port, timeout_seconds))
}
```

### Multi-condition Container Readiness

The container setup now uses multiple readiness conditions:

```rust
fn ready_conditions(&self) -> Vec<WaitFor> {
    // Use multi-condition readiness check with both message and HTTP check
    // Only one condition needs to pass for container to be considered ready
    vec![
        // Primary: HTTP readiness check via custom wait function
        WaitFor::custom(|container| {
            // HTTP endpoint check logic
        }),
        
        // Fallback: Standard message-based check
        WaitFor::message_on_stdout("Selenium Grid ready")
    ]
}
```

### WebDriver Client Improvements

The WebDriver client now includes configurable timeout:

```rust
pub async fn create_webdriver_with_timeout(
    selenium_url: &str,
    timeout_seconds: u64
) -> WebDriverResult<WebDriver> {
    // ...timeout configuration...
}
```

## Usage

When using the Selenium container in tests, you can now:

1. Use the default configuration:
   ```rust
   let selenium = SeleniumInstance::new(); // Uses 60s timeout
   ```

2. Specify a custom timeout:
   ```rust
   let selenium = SeleniumInstance::new_with_timeout(90); // 90s timeout
   ```

3. Configure WebDriver timeout:
   ```rust
   let driver = create_webdriver_with_timeout(&selenium_url, 30).await?;
   ```

## Test Changes

We've removed the `#[ignore]` attributes from the web UI tests since they should now pass reliably with the improved container handling.