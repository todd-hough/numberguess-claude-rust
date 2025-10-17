# Authentication Integration Tests

**Status**: Planning
**Created**: 2025-10-16
**Last Updated**: 2025-10-16

## Overview

Add comprehensive authentication testing that validates the full OAuth2/OIDC flow through oauth2-proxy and Keycloak, ensuring all endpoints work correctly with authenticated users and properly reject unauthenticated requests.

## Background

The application currently uses an authentication proxy pattern with:
- **oauth2-proxy** (port 8080) - Authentication gateway
- **Keycloak** (port 8090) - OIDC identity provider
- **Redis** - Session storage for oauth2-proxy
- **Application** (port 4080) - Internal only, accessed via oauth2-proxy

All web routes require authentication, but current integration tests don't validate the authentication flow or test that unauthenticated requests are properly rejected.

## Objectives

1. ✅ Validate OAuth2/OIDC login flow works correctly
2. ✅ Verify all endpoints work with authenticated users
3. ✅ Verify all endpoints reject unauthenticated requests (401/redirect)
4. ✅ Test session persistence and invalidation
5. ✅ Ensure integration tests use realistic auth stack

## Design Decisions

### Authentication Approach (Hybrid Strategy)
**Decision**: Use different authentication methods based on test type
- **Web UI Tests**: Selenium OAuth2 flow (realistic user experience)
- **API Tests**: Programmatic authentication via Direct Access Grants (fast)

**Rationale**:
- Web UI tests need to validate the full user flow including redirects, oauth2-proxy integration, and browser session cookies
- API tests benefit from speed and can use programmatic tokens without sacrificing coverage
- Hybrid approach provides comprehensive coverage with reasonable test execution time

### Test Profile Strategy
**Decision**: Add authentication stack to ALL integration test profiles
**Rationale**: All integration tests should be realistic and test the actual deployed architecture

### 401 Testing Strategy
**Decision**: Test against oauth2-proxy without valid session (port 8080)
**Rationale**: Tests realistic behavior - users access via oauth2-proxy, not app directly

## Hybrid Authentication Strategy Summary

This implementation uses **two different authentication methods** optimized for different test types:

### Web UI Tests → Selenium OAuth2 Flow
**What**: Full browser-based OAuth2 authorization code flow with PKCE
**Why**: Tests the actual user experience and oauth2-proxy integration
**How**:
- Selenium navigates to protected page
- Detects redirect to Keycloak login
- Fills credentials and submits
- Waits for OAuth2 callback redirect
- Extracts session cookie (`_oauth2_proxy`)
- Uses cookie for subsequent requests

**Target**: http://localhost:8080 (oauth2-proxy)
**Used by**: `tests/web_ui_test.rs`, `tests/web_endpoints_test.rs`, Selenium-based tests in `tests/auth_integration_test.rs`

### API Tests → Programmatic Direct Access Grants
**What**: Direct token acquisition via Keycloak token endpoint (Resource Owner Password Credentials flow)
**Why**: Fast, reliable, no browser needed
**How**:
- POST to Keycloak token endpoint with:
  - `grant_type=password`
  - `client_id=test-client`
  - `client_secret=test-secret-do-not-use-in-production`
  - `username=admin@local.test`
  - `password=password`
- Receive JWT access token
- Use Bearer authentication header
- Bypasses oauth2-proxy, talks to app directly

**Target**: http://localhost:4080 (app directly)
**Used by**: `tests/api_edge_cases_test.rs`, programmatic tests in `tests/auth_integration_test.rs`

### Trade-offs

| Aspect | Selenium OAuth2 | Programmatic |
|--------|----------------|--------------|
| **Speed** | Slow (2-3s per login) | Fast (<100ms per token) |
| **Realism** | ✅ Tests actual user flow | ⚠️ Different from user flow |
| **Coverage** | ✅ Tests oauth2-proxy | ❌ Bypasses oauth2-proxy |
| **Browser Required** | ✅ Yes (Selenium) | ❌ No |
| **Flakiness** | ⚠️ Can have timing issues | ✅ Very reliable |
| **Use Case** | UI/UX validation | Fast API functional tests |

## Implementation Plan

### 1. Configure Keycloak Test Client

**File**: `keycloak/realm-export.json`

Add a test-specific client with programmatic authentication enabled:

**Location**: In the `clients` array, add:
```json
{
  "clientId": "test-client",
  "name": "Test Client for Integration Tests",
  "description": "OAuth2 client for programmatic authentication in tests",
  "enabled": true,
  "clientAuthenticatorType": "client-secret",
  "secret": "test-secret-do-not-use-in-production",
  "publicClient": false,
  "bearerOnly": false,
  "standardFlowEnabled": true,
  "implicitFlowEnabled": false,
  "directAccessGrantsEnabled": true,
  "serviceAccountsEnabled": true,
  "authorizationServicesEnabled": false,
  "redirectUris": [
    "http://localhost:8080/*",
    "http://localhost:4080/*"
  ],
  "webOrigins": ["+"],
  "protocol": "openid-connect",
  "attributes": {
    "access.token.lifespan": "300"
  },
  "defaultClientScopes": [
    "web-origins",
    "profile",
    "roles",
    "email"
  ]
}
```

**Key settings**:
- `directAccessGrantsEnabled: true` - Enables Resource Owner Password Credentials (ROPC) flow for programmatic login
- `serviceAccountsEnabled: true` - Enables client credentials flow (optional, for service-to-service)
- `secret` - Hardcoded for tests (OK since this is dev/test only)

**Note**: The existing `numberguess-client` is used by oauth2-proxy for browser-based login. This new `test-client` is exclusively for programmatic test authentication.

### 2. Update Docker Compose Profiles

**File**: `docker-compose.yml`

Add `integration` and `integration-ui` profiles to:
- `redis` service
- `keycloak` service
- `oauth2-proxy` service

This ensures the full auth stack is available during integration tests.

**Changes**:
```yaml
redis:
  profiles:
    - dev
    - full-stack
    - integration        # ADD
    - integration-ui     # ADD

keycloak:
  profiles:
    - dev
    - full-stack
    - integration        # ADD
    - integration-ui     # ADD

oauth2-proxy:
  profiles:
    - dev
    - full-stack
    - integration        # ADD
    - integration-ui     # ADD
```

### 3. Create Authentication Helper Module

**New File**: `tests/common/auth_helpers.rs`

Provides authentication utilities for tests with two strategies:

#### Selenium-Based Authentication (for Web UI tests)

**Function**: `login_with_keycloak_selenium(driver: &WebDriver) -> WebDriverResult<Cookie>`
- Navigates to protected endpoint (http://localhost:8080)
- Detects redirect to Keycloak login page
- Fills in credentials (admin@local.test / password)
- Submits login form
- Waits for OAuth2 redirect back to app
- Extracts and returns `_oauth2_proxy` session cookie

**Function**: `create_authenticated_client_selenium() -> Result<Client, Error>`
- Creates temporary WebDriver
- Performs login via Selenium
- Extracts session cookie
- Creates reqwest::Client with session cookie
- Closes WebDriver
- Returns authenticated client ready for use

#### Programmatic Authentication (for API tests)

**Function**: `get_access_token(username: &str, password: &str) -> Result<String, Error>`
```rust
// Uses Direct Access Grants (ROPC) flow
// POST to Keycloak token endpoint with:
//   client_id: test-client
//   client_secret: test-secret-do-not-use-in-production
//   grant_type: password
//   username: admin@local.test
//   password: password
// Returns: access_token (JWT)
```

**Function**: `create_authenticated_client_programmatic() -> Result<Client, Error>`
```rust
// 1. Get access token via Direct Access Grants
let token = get_access_token("admin@local.test", "password")?;

// 2. Create reqwest client that adds Authorization header to all requests
let client = Client::builder()
    .default_headers({
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?
        );
        headers
    })
    .build()?;

// Note: This bypasses oauth2-proxy and sends bearer token directly to app
// For API tests, we access app on port 4080 (internal) not 8080 (oauth2-proxy)
```

**Important**: API tests using programmatic auth should target:
- `http://localhost:4080` (app directly with bearer token)

Web UI tests using Selenium should target:
- `http://localhost:8080` (oauth2-proxy with session cookie)

#### Shared Functions

**Function**: `create_unauthenticated_client() -> Client`
- Returns reqwest client without any authentication
- For testing 401 responses

**Function**: `extract_session_cookie(driver: &WebDriver) -> WebDriverResult<Cookie>`
- Gets cookie named `_oauth2_proxy`
- Converts to reqwest cookie format

**Constants**:
```rust
const KEYCLOAK_TOKEN_URL: &str = "http://localhost:8090/realms/numberguess/protocol/openid-connect/token";
const TEST_CLIENT_ID: &str = "test-client";
const TEST_CLIENT_SECRET: &str = "test-secret-do-not-use-in-production";
const TEST_USERNAME: &str = "admin@local.test";
const TEST_PASSWORD: &str = "password";
```

**Update**: `tests/common/mod.rs`
- Add `pub mod auth_helpers;`

### 4. Create Authentication-Specific Tests

**New File**: `tests/auth_integration_test.rs`

This file tests the authentication mechanisms themselves.

#### Test: `test_oauth2_login_flow` (Selenium)
Validates complete OAuth2 login flow via browser:
1. Navigate to protected page (http://localhost:8080)
2. Assert redirected to Keycloak login page
3. Fill in username: admin@local.test
4. Fill in password: password
5. Submit login form
6. Assert redirected back to application
7. Assert session cookie (`_oauth2_proxy`) set
8. Assert can access protected page

**Strategy**: Selenium (tests oauth2-proxy integration)

#### Test: `test_programmatic_authentication_works`
Validates Direct Access Grants flow:
1. Request token from Keycloak with username/password
2. Assert token received
3. Assert token is valid JWT
4. Assert token contains expected claims (sub, email, preferred_username)
5. Make API request with bearer token
6. Assert request succeeds

**Strategy**: Programmatic (tests Keycloak token endpoint)

#### Test: `test_web_ui_authenticated_endpoints` (Selenium)
Tests Web UI endpoints WITH valid session cookie:
- `GET /` - Index page should load
- `POST /game/new` - Game creation should work
- `POST /game/{id}/guess` - Guessing should work
- `GET /difficulty-preview` - Difficulty preview should work

For each endpoint:
- Login via Selenium to get session cookie
- Create reqwest client with session cookie
- Make request to http://localhost:8080 (oauth2-proxy)
- Assert 200 or appropriate success status
- Assert response contains expected content

**Strategy**: Selenium login, then reqwest with cookie

#### Test: `test_api_authenticated_endpoints` (Programmatic)
Tests API endpoints WITH valid bearer token:
- `POST /api/games` - API game creation should work
- `POST /api/games/{id}/guess` - API guessing should work

For each endpoint:
- Get token via Direct Access Grants
- Create reqwest client with Authorization header
- Make request to http://localhost:4080 (app directly)
- Assert 200 or appropriate success status
- Assert response contains expected JSON

**Strategy**: Programmatic auth with bearer token

#### Test: `test_unauthenticated_endpoints_return_401`
Tests all endpoints WITHOUT authentication:

**Endpoints to test**:
- `POST /game/new`
- `POST /game/{id}/guess` (with dummy ID)
- `GET /difficulty-preview`
- `POST /api/games`
- `POST /api/games/{id}/guess` (with dummy ID)

For each endpoint:
- Create unauthenticated client (no session cookie)
- Make request to port 8080 (oauth2-proxy)
- Assert response is 302 redirect to Keycloak login OR 401/403
- If redirect, assert redirect URL contains Keycloak domain

#### Test: `test_session_persistence`
Validates session persists across requests:
1. Login once to get session cookie
2. Make multiple API calls with same cookie:
   - Create game
   - Make guess
   - Get difficulty preview
3. Assert no re-authentication required
4. Assert all requests succeed

#### Test: `test_invalid_session_rejected`
Validates invalid sessions are rejected:
1. Create client with invalid cookie value
2. Make request to protected endpoint
3. Assert redirect to login page
4. Assert cannot access protected resource

#### Test: `test_expired_session_rejected`
Validates expired sessions trigger re-authentication:
1. Get valid session cookie
2. Wait for session to expire (or invalidate in Redis)
3. Make request with expired cookie
4. Assert redirect to login page

### 5. Update Existing Tests

#### Web UI Tests (Use Selenium Authentication)

**File**: `tests/web_ui_test.rs`

These tests use browser automation, so they use Selenium-based OAuth2 login:

**Changes**:
- `test_web_ui_game_flow()`:
  1. After navigating to app, detect redirect to Keycloak
  2. Call `page.login("admin@local.test", "password")` (new method)
  3. Wait for redirect back to app
  4. Continue with existing test flow

- `test_web_ui_invalid_inputs()`:
  1. Same login flow before interacting with form
  2. Continue with existing test

**Strategy**: Full OAuth2 flow via Selenium, access via oauth2-proxy (port 8080)

#### Web Endpoints Tests (Use Programmatic Authentication)

**File**: `tests/web_endpoints_test.rs`

These tests use reqwest HTTP client to test web endpoints, use programmatic auth for speed:

**Changes to ALL tests**:
```rust
use common::auth_helpers;

#[test]
fn test_static_file_serving() {
    let base_url = environment::ensure_server_ready();
    // Change: Use authenticated client
    let client = auth_helpers::create_authenticated_client_selenium()
        .expect("Failed to create authenticated client");

    // Rest of test unchanged, but requests go through oauth2-proxy with session cookie
}
```

**Tests to update**:
- `test_static_file_serving()` - Add Selenium auth (tests oauth2-proxy HTML serving)
- `test_web_form_endpoints()` - Add Selenium auth (tests web forms through proxy)
- `test_remaining_guesses_display()` - Add Selenium auth
- `test_no_remaining_guesses_display_without_limit()` - Add Selenium auth

**Strategy**: Selenium login once per test to get cookie, then use reqwest with cookie
**Target**: http://localhost:8080 (oauth2-proxy)

#### API Tests (Use Programmatic Authentication)

**File**: `tests/api_edge_cases_test.rs` and similar API test files

These test JSON API endpoints, use programmatic auth for speed:

**Changes**:
```rust
use common::auth_helpers;

#[test]
fn test_api_endpoint() {
    let base_url = "http://localhost:4080"; // App directly, not oauth2-proxy
    // Use programmatic auth
    let client = auth_helpers::create_authenticated_client_programmatic()
        .expect("Failed to get auth token");

    // Make API request with Bearer token
    let resp = client.post(format!("{}/api/games", base_url))
        .json(&request_body)
        .send()
        .expect("Should send request");
}
```

**Strategy**: Direct Access Grants for fast token acquisition, bearer auth
**Target**: http://localhost:4080 (app directly, bypasses oauth2-proxy)

### 6. Update Page Objects for Authentication

**File**: `tests/common/page_objects.rs`

Add authentication-related methods to `GamePage`:

```rust
/// Perform Keycloak login if on login page
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

/// Check if currently on Keycloak login page
pub async fn is_on_login_page(&self) -> WebDriverResult<bool> {
    self.driver
        .query(By::Id("kc-login"))
        .nowait()
        .exists()
        .await
}

/// Wait for redirect to Keycloak login page
pub async fn wait_for_login_page(&self) -> WebDriverResult<()> {
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if self.is_on_login_page().await? {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(WebDriverError::Timeout(
        "Timeout waiting for Keycloak login page".to_string()
    ))
}

/// Wait for OAuth2 redirect back to application
pub async fn wait_for_app_redirect(&self) -> WebDriverResult<()> {
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let url = self.driver.current_url().await?;
        if !url.contains("keycloak") && !url.contains("oauth2/callback") {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
```

### 7. Update Test Infrastructure

**File**: `tests/common/environment.rs`

Add service readiness checks:

```rust
/// Return Keycloak base URL
pub fn keycloak_url() -> String {
    env::var("KEYCLOAK_URL").unwrap_or_else(|_| "http://localhost:8090".to_string())
}

/// Ensure Keycloak is reachable and ready
pub fn ensure_keycloak_ready() -> String {
    let url = keycloak_url();
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 60; // Keycloak can take up to 60s to start
    let health_endpoint = format!("{}/health/ready", url);

    while attempts < max_attempts {
        if let Ok(resp) = client.get(&health_endpoint).send() {
            if resp.status().is_success() {
                return url;
            }
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!("Keycloak at {} is not ready", url);
}

/// Ensure Redis is reachable
pub fn ensure_redis_ready() {
    // Redis health check via TCP connection
    let mut attempts = 0;
    let max_attempts = 30;

    while attempts < max_attempts {
        if std::net::TcpStream::connect("localhost:6379").is_ok() {
            return;
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!("Redis is not ready");
}

/// Ensure oauth2-proxy is reachable
pub fn ensure_oauth2_proxy_ready() {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects
        .build()
        .expect("Failed to build HTTP client");

    let mut attempts = 0;
    let max_attempts = 30;

    while attempts < max_attempts {
        // oauth2-proxy will redirect to Keycloak, which is fine
        if let Ok(resp) = client.get("http://localhost:8080").send() {
            let status = resp.status();
            if status.is_success() || status.is_redirection() {
                return;
            }
        }
        attempts += 1;
        thread::sleep(Duration::from_secs(1));
    }

    panic!("oauth2-proxy is not ready");
}
```

Update `ensure_server_ready()`:
```rust
pub fn ensure_server_ready() -> String {
    // Wait for all services in order
    ensure_redis_ready();
    ensure_keycloak_ready();
    ensure_oauth2_proxy_ready();

    // Then wait for app
    let base = base_url();
    // ... existing implementation ...
}
```

### 8. Update Makefile Test Targets

**File**: `Makefile`

Update `test-compose` target:
```makefile
test-compose: docker-check
	@echo "Running API integration tests..."
	@bash -c 'set -euo pipefail; \
	COMPOSE_CMD="$(COMPOSE_STACK)"; \
	PROFILE="integration"; \
	cleanup() { \
		echo "Cleaning up test environment..."; \
		$$COMPOSE_CMD --profile $$PROFILE down -v >/dev/null 2>&1 || true; \
	}; \
	trap cleanup EXIT; \
	echo "Starting postgres, redis, keycloak..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d postgres redis keycloak >/dev/null; \
	$$COMPOSE_CMD --profile $$PROFILE up --wait postgres redis keycloak >/dev/null; \
	echo "Setting up test database..."; \
	POSTGRES_HOST=localhost TEST_DB_NAME=$(TEST_DB) ./scripts/reset-db.sh; \
	echo "Starting app and oauth2-proxy..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d app oauth2-proxy >/dev/null; \
	echo "Waiting for services to be healthy..."; \
	./scripts/wait-for-http.sh http://localhost:8081/health $(HEALTH_TIMEOUT); \
	./scripts/wait-for-http.sh http://localhost:8090/health/ready 60; \
	./scripts/wait-for-http.sh http://localhost:8080 30; \
	echo "Running integration tests..."; \
	GAME_SERVER_BASE_URL=http://localhost:8080 cargo test --tests -- --test-threads=1; \
	'
```

Update `test-compose-ui` target:
```makefile
test-compose-ui: docker-check
	@echo "Running UI integration tests..."
	@bash -c 'set -euo pipefail; \
	COMPOSE_CMD="$(COMPOSE_STACK)"; \
	PROFILE="integration-ui"; \
	cleanup() { \
		echo "Cleaning up test environment..."; \
		$$COMPOSE_CMD --profile $$PROFILE down -v >/dev/null 2>&1 || true; \
	}; \
	trap cleanup EXIT; \
	echo "Starting postgres, redis, keycloak..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d postgres redis keycloak >/dev/null; \
	$$COMPOSE_CMD --profile $$PROFILE up --wait postgres redis keycloak >/dev/null; \
	echo "Setting up test database..."; \
	POSTGRES_HOST=localhost TEST_DB_NAME=$(TEST_DB) ./scripts/reset-db.sh; \
	echo "Starting app, oauth2-proxy, and selenium..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d app oauth2-proxy selenium >/dev/null; \
	echo "Waiting for services to be healthy..."; \
	./scripts/wait-for-http.sh http://localhost:8081/health $(HEALTH_TIMEOUT); \
	./scripts/wait-for-http.sh http://localhost:8090/health/ready 60; \
	./scripts/wait-for-http.sh http://localhost:8080 30; \
	./scripts/wait-for-http.sh http://localhost:4444/status $(HEALTH_TIMEOUT); \
	echo "Running UI tests..."; \
	GAME_SERVER_BASE_URL=http://localhost:8080 \
	GAME_SERVER_BROWSER_URL=http://oauth2-proxy:8080 \
	SELENIUM_REMOTE_URL=http://localhost:4444 \
	cargo test --test web_ui_test --test auth_integration_test -- --test-threads=1; \
	'
```

### 9. Update Documentation

**File**: `CLAUDE.md`

Add new section under "Testing Strategy":
```markdown
## Testing Strategy

### Authentication in Tests

All integration tests run through the full authentication stack:
- **Keycloak** (OIDC provider) - http://localhost:8090
- **oauth2-proxy** (auth gateway) - http://localhost:8080
- **Redis** (session storage)

**Test Credentials**:
- Username: admin@local.test
- Password: password

**Test Flow**:
1. Tests start full stack via docker compose
2. Selenium automates OAuth2 login flow
3. Tests use authenticated session for requests
4. Unauthenticated tests verify 401/redirect behavior

**Running Tests**:
```bash
# Unit tests (no authentication needed)
cargo test --lib

# Integration tests (includes authentication)
make test-compose

# Web UI tests (includes authentication)
make test-compose-ui

# Full suite
make test
```

**Troubleshooting**:
- If tests fail with "Keycloak not ready", increase timeout
- If login fails, check Keycloak realm import succeeded
- If session issues, verify Redis is running
- View logs: `docker compose --profile integration-ui logs`
```

Update "Common Tasks" section:
```markdown
### Debugging Test Failures

**Authentication Issues**:
- Check Keycloak: http://localhost:8090 (admin/admin)
- Verify realm "numberguess" exists
- Check user admin@local.test exists
- Verify oauth2-proxy can reach Keycloak
- Check Redis: `docker compose exec redis redis-cli ping`

**Service Startup Order**:
1. postgres, redis, keycloak (parallel)
2. app, oauth2-proxy (after keycloak ready)
3. selenium (for UI tests)

**Timing Issues**:
- Keycloak can take 30-60s to start
- OAuth2 flow adds 2-3s per test
- Use `--test-threads=1` to avoid race conditions
```

## Implementation Order

1. ✅ Configure Keycloak test client (enables programmatic auth)
2. ✅ Update docker-compose.yml profiles (enables auth stack in tests)
3. ✅ Create auth_helpers.rs (both Selenium and programmatic auth utilities)
4. ✅ Update environment.rs (service readiness checks for all auth services)
5. ✅ Update page_objects.rs (Selenium login support for Keycloak)
6. ✅ Create auth_integration_test.rs (comprehensive auth tests, both strategies)
7. ✅ Update existing web UI tests (add Selenium authentication)
8. ✅ Update existing API tests (add programmatic authentication)
9. ✅ Update Makefile (orchestrate all auth services in test profiles)
10. ✅ Update documentation (testing guide with hybrid approach explanation)
11. ✅ Test and validate (run full test suite with both auth strategies)

## Testing & Validation

After implementation, verify:

```bash
# 1. Unit tests still work (no auth needed)
cargo test --lib

# 2. Integration tests work with auth
make test-compose

# 3. UI tests work with auth
make test-compose-ui

# 4. Full test suite passes
make test

# 5. Verify auth services start correctly
docker compose --profile integration up -d
docker compose ps
docker compose logs keycloak oauth2-proxy

# 6. Manual verification
# Visit http://localhost:8080
# Should redirect to Keycloak login
# Login with admin@local.test / password
# Should access application successfully
```

## Success Criteria

- ✅ All integration tests use full authentication stack
- ✅ Tests validate OAuth2 login flow works
- ✅ Tests verify authenticated endpoints work correctly
- ✅ Tests verify unauthenticated requests are rejected (401/302)
- ✅ Tests verify session persistence across requests
- ✅ Tests verify invalid/expired sessions are rejected
- ✅ No test failures due to authentication
- ✅ Test execution time is reasonable (<5 minutes total)
- ✅ Documentation updated with authentication testing guide

## Known Limitations

### Startup Overhead
- **Keycloak**: Takes 30-60s to start (imports realm, starts services)
- **Full Stack**: ~90s total for all services (postgres, redis, keycloak, oauth2-proxy, app, selenium)

### Test Speed
- **Selenium tests**: 2-3s per OAuth2 login flow (web UI tests)
- **Programmatic tests**: <100ms per token (API tests) - much faster
- **Overall**: Hybrid approach balances realism with speed

### Parallelization
- **Must use** `--test-threads=1` to avoid:
  - Session cookie conflicts (Selenium tests)
  - Database state conflicts
  - OAuth2 redirect race conditions

### Dependencies
- **Selenium**: Required for web UI tests (ChromeDriver in Docker)
- **Keycloak**: Required for all integration tests
- **Redis**: Required for session storage

### Security Notes
- **Test client secret**: Hardcoded in realm export (acceptable for dev/test only)
- **Direct Access Grants**: ROPC flow is deprecated in OAuth 2.1 (but fine for testing)
- **Network isolation**: Tests expose ports on localhost (dev/test only)

## Future Enhancements

- [ ] Cache authenticated sessions across tests (reduce login overhead)
- [ ] Add test for group-based authorization
- [ ] Test token refresh flow
- [ ] Test concurrent sessions from different users
- [ ] Add performance tests for auth flow
- [ ] Test auth error conditions (invalid credentials, network failures)

## References

- OAuth2-proxy docs: https://oauth2-proxy.github.io/oauth2-proxy/
- Keycloak docs: https://www.keycloak.org/documentation
- OIDC spec: https://openid.net/connect/
- PKCE spec: https://tools.ietf.org/html/rfc7636
