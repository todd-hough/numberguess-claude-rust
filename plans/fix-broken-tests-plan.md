# Enhanced Diagnostic Plan for Selenium Container Timeout Issues

## Problem Analysis

The Selenium container tests are failing with WaitContainer(StartupTimeout) error. The container starts but never produces the expected "Selenium Grid ready" message on stdout.

## Diagnostic Steps (Prioritized)

### 1. Parallel Initial Investigation

- Create a minimal test case that only starts the Selenium container to isolate the issue
- Run the Selenium container manually and observe its behavior:
  ```bash
  docker run --rm -p 4444:4444 seleniarm/standalone-chromium:latest
  ```
- Simultaneously investigate readiness signals:
  - Capture actual stdout messages
  - Check if "Selenium Grid ready" message format has changed
  - Test HTTP endpoint at http://localhost:4444/status
  - Observe container health check status

### 2. Container Configuration Review

- Check compatibility between testcontainers-rs and Selenium image version
- Review container resource allocation (memory, CPU)
- Check if timeout settings need adjustment
- Verify environment variables and port mappings

### 3. Alternative Ready Condition Implementation

- Implement HTTP-based readiness check as primary method:
  - Primary: HTTP /status endpoint returns 200
  - Fallback: Log message detection with updated pattern
  - Ultimate fallback: Port binding verification

### 4. Test Environment Verification

- Check Docker daemon configuration
- Verify network connectivity within containers
- Test with different Selenium image versions
- Review dependency versions (testcontainers-rs, etc.)

### 5. Enhanced Logging (Only if Previous Steps Fail)

- Add container log capture to SeleniumInstance::new()
- Print container logs when startup fails
- Capture both stdout and stderr
- Add verbose logging throughout container startup process
- Log Docker API responses
- Capture container inspect output on failure

## Implementation Order

1. Quick manual test + minimal isolated test case creation
2. HTTP endpoint readiness implementation (most likely solution)
3. Container configuration tuning (timeout, resources)
4. Test with alternative Selenium image versions if needed
5. Only add extensive logging if previous steps fail
6. Document findings and create permanent fix
