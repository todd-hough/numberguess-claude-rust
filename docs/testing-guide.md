# Testing Guide

## Overview

| Kind | Where | Needs Docker? | Run with |
|---|---|---|---|
| Unit tests | `#[cfg(test)]` modules inside `src/` (core logic, validators, difficulty calculator, auth extractor, handlers) | No | `make test-unit` (`cargo test --lib`) |
| CLI tests | `tests/cli_test.rs`, `tests/integration_test.rs` (drive the binary with `assert_cmd`) | No | part of `make test-func` |
| Integration tests | other `tests/*_test.rs`, against a Docker Compose stack | Yes | `make test-func`, `make test-auth` |

## Commands

```bash
make test-unit          # unit tests only, fast
make test-func          # light tier: functional integration tests (mock auth, ~70 MiB)
make test-auth          # full tier: auth + browser tests (Keycloak + Selenium, ~1.2 GiB)
make test-integration   # light tier, teardown, then full tier
make test               # unit + both integration tiers
make test-func-down     # stop the light tier
make test-down          # stop both tiers
```

Stacks are left running after tests so you can inspect them; stop them when done.

## The two integration tiers

Most integration tests don't need a real login, so they run against a small stack where an
nginx proxy injects the identity headers oauth2-proxy would normally add. Only the tests
that exercise real authentication or a real browser use the heavy stack.

| | Light (`make test-func`) | Full (`make test-auth`) |
|---|---|---|
| Services | postgres, app, nginx mock-auth proxy | postgres, app, Keycloak, oauth2-proxy, Redis, Selenium |
| Memory | ~70 MiB | ~1.2 GiB |
| Tests | `api_edge_cases_test`, `web_endpoints_test`, `concurrency_test`, `csrf_test`, `cli_test`, `integration_test` | `auth_integration_test` (OAuth2 flow, redirects, 401s), `web_ui_test` (browser: HTMX swaps, DOM checks) |
| Login | none — headers injected as the test user | real OAuth2 login through Selenium, ~2-3 s each |
| Compose | `docker-compose.test-mock-auth.yml` (project `numberguess-mock`) | `docker-compose.yml` + `docker-compose.integration.yml` (profile `integration`) |

Setting `MOCK_AUTH=1` (done by `make test-func`) switches the shared test helpers to the
light tier; the test code is the same in both. CI runs both tiers in
`integration-security.yml`.

The full stack takes about 60-70 seconds to become healthy, mostly Keycloak importing its
realm. Test login: `admin@local.test` / `password`.

## Writing tests

**Unit tests** live next to the code. Game constructors return `Result<_, GameError>`, so
assert on error variants rather than message text:

```rust
#[test]
fn rejects_inverted_range() {
    assert!(matches!(
        GuessingGame::new(100, 1),
        Err(GameError::InvalidRange { .. })
    ));
}
```

**Integration tests must be async.** Use `#[tokio::test]`, the async `reqwest::Client`, and
`.await`. Never use `tokio_test::block_on()` (nested runtime → panic or deadlock) or
`reqwest::blocking::Client` (DNS failures inside tokio). The readiness checks are blocking,
so wrap them in `spawn_blocking`:

```rust
#[tokio::test]
async fn creates_a_game() {
    tokio::task::spawn_blocking(|| environment::ensure_server_ready())
        .await
        .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create client");

    let response = client
        .post("http://localhost:8080/api/games")
        .json(&serde_json::json!({ "min": 1, "max": 100, "max_guesses": "10" }))
        .send()
        .await
        .expect("Request failed");
    assert!(response.status().is_success());
}
```

Note that the API currently accepts `max_guesses` only as a string (`"10"`), `null`, or
omitted — a JSON number is rejected.

**Checklist for a new test file:**
- Put it in the light tier unless it tests real authentication or needs a browser.
- Add its name to `FUNC_TESTS` or `AUTH_TESTS` in the `Makefile`. `make test-tier-check`
  (run by both tier targets) fails if a `tests/*_test.rs` file is in neither list or both.
- Reuse the helpers in `tests/common/` (`auth_helpers`, `environment`, `assertions`,
  `page_objects`, `webdriver`, `test_helpers`).
- Full-tier tests don't cache logins: every `create_authenticated_client()` call does a
  fresh browser login.

**Rebuild the app image after changing `src/` or `templates/`.** The tier targets only
build the image when it doesn't exist, and templates are compiled into the binary, so
tests will otherwise run against old code. `make build` rebuilds the image both tiers use.

## Networking

Tests run on the host; Selenium runs inside Docker. The same service therefore has two
addresses:

```
Host (cargo test)                     Docker network
  localhost:8080  ─── port map ───►  oauth2-proxy:4180 ──► app:4080
  localhost:4444  ─── port map ───►  selenium:4444
                                          │
                                          └─ browser opens http://oauth2-proxy:4180
```

| Variable | Value | Used by |
|---|---|---|
| `GAME_SERVER_BASE_URL` | `http://localhost:8080` | tests sending HTTP requests |
| `SELENIUM_REMOTE_URL` | `http://localhost:4444` | tests driving Selenium |
| `GAME_SERVER_BROWSER_URL` | `http://oauth2-proxy:4180` | the browser inside Docker (`localhost` would point at the Selenium container itself) |

The make targets set these. To re-run one test binary with the stack already up:

```bash
# light tier
MOCK_AUTH=1 GAME_SERVER_BASE_URL=http://localhost:8080 \
  cargo test --test web_endpoints_test -- --test-threads=1

# full tier
GAME_SERVER_BASE_URL=http://localhost:8080 GAME_SERVER_BROWSER_URL=http://oauth2-proxy:4180 \
SELENIUM_REMOTE_URL=http://localhost:4444 \
  cargo test --test auth_integration_test -- --test-threads=1
```

Integration tests always run with `--test-threads=1` to avoid session conflicts.

## Troubleshooting

| Symptom | What to check |
|---|---|
| Keycloak not responding | Still starting (~60 s): `docker compose logs keycloak \| grep -i realm` |
| Login fails, no session cookie | `docker compose logs oauth2-proxy`; the session cookie is `_oauth2_proxy` |
| Session problems | `docker compose exec redis redis-cli ping` |
| Selenium connection refused | `curl http://localhost:4444/status`; is `SELENIUM_REMOTE_URL` set? |
| 401 responses | Is the client authenticated? See `tests/common/auth_helpers.rs` |
| Tests see old HTML or behavior | Stale app image — rebuild it |
| Port conflicts | Ports 4080, 5432, 6379, 8080, 8081, 8090 must be free |
| Service status | `docker compose -f docker-compose.yml -f docker-compose.integration.yml --profile integration ps` |

Health endpoints: Keycloak `http://localhost:8090/health/ready`, app
`http://localhost:8081/health` (full stack), oauth2-proxy `http://localhost:8080` (should
redirect to Keycloak).

Useful `cargo test` options: `cargo test NAME -- --nocapture` shows `println!` output;
`cargo test NAME -- --exact` runs only the test with exactly that name.

## Manual testing checklist

**CLI**
- [ ] Start without arguments; all prompts appear
- [ ] Give some flags; only the missing values are prompted
- [ ] Invalid input (letters, symbols) re-prompts
- [ ] Boundary values (0, 1,000,000)
- [ ] Guess limit ends the game and reveals the number

**Web UI**
- [ ] Setup form validation (empty fields, inverted range)
- [ ] Difficulty preview updates while editing the form
- [ ] Guesses show feedback; history capsules and the range tracker update
- [ ] Remaining-guesses counter changes state at ≤3 and on the last guess
- [ ] Win and limit-reached endings
- [ ] Light and dark themes

**API**
- [ ] Create games with and without a guess limit
- [ ] Too low / too high / correct / limit reached responses
- [ ] 400 for invalid parameters, 404 for unknown or finished games
- [ ] Several games active at once
- [ ] Finished games are removed (guessing again returns 404)

## Continuous integration

- `ci.yml` (every push): format check, clippy, unit tests.
- `integration-security.yml` (PR opened to main, pushes to main/develop, manual dispatch):
  Docker build, both integration tiers, cargo-audit, hadolint, Trivy.
