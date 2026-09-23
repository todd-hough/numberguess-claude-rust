# Integration tests

Loaded automatically when working in `tests/`. Human-facing version with a network
diagram and manual test checklist: `docs/testing-guide.md`.

API quirk to remember when writing requests: `max_guesses` must be sent as a string
(`"10"`), `null`, or omitted — a JSON number is rejected.

## Async only (mandatory)
- `#[tokio::test] async fn`, async `reqwest::Client`, `.await` everything.
- Never `tokio_test::block_on()` (nested runtime → panic or deadlock), never
  `reqwest::blocking::Client` (DNS issues inside tokio), never `#[test]` with async code.
- Environment readiness checks use a blocking client, so wrap them in `spawn_blocking`.
- Tests run with `--test-threads=1` (prevents session conflicts).

```rust
#[tokio::test]
async fn test_something() {
    tokio::task::spawn_blocking(|| {
        environment::ensure_server_ready();
        environment::ensure_selenium_ready(); // full tier only
    })
    .await
    .expect("Environment checks failed");

    let client = auth_helpers::create_authenticated_client()
        .await
        .expect("Failed to create client");
    let response = client.get("http://localhost:8080").send().await.expect("Request failed");
}
```

## Two tiers
The tier is selected by `MOCK_AUTH=1` (set by `make test-func`); test files and assertions
are identical in both tiers.

| | LIGHT — `make test-func` | FULL — `make test-auth` |
|---|---|---|
| Stack | postgres + app + nginx mock-auth proxy (~70 MiB) | adds Keycloak, oauth2-proxy, Redis, Selenium (~1.2 GiB) |
| Test binaries | `api_edge_cases_test`, `web_endpoints_test`, `concurrency_test`, `csrf_test`, plus the non-Docker `cli_test` and `integration_test` | `auth_integration_test` (OAuth2 flow, redirects, 401s), `web_ui_test` (real browser: HTMX swaps, DOM assertions) |
| Auth | nginx injects the `X-Forwarded-*` headers oauth2-proxy would add, as the real test user (admin / admin@local.test / group `/admin`); no login overhead | Selenium runs a real OAuth2 authorization code flow with PKCE (~2-3s per login) |
| Compose | `docker-compose.test-mock-auth.yml` + `test-fixtures/mock-auth-nginx.conf`, project `numberguess-mock` (`concurrency_test.rs` uses it for the app-restart test) | `docker-compose.yml` + `docker-compose.integration.yml`, `--profile integration` |
| Stop | `make test-func-down` | `make test-down` (stops both tiers) |

- Default to the light tier; put a new test there unless it asserts on the real auth stack.
- **Every `tests/*_test.rs` must be listed in exactly one of `FUNC_TESTS` / `AUTH_TESTS` in
  the Makefile.** `make test-tier-check` (run by both tier targets) fails otherwise.
- `make test-integration` runs light → teardown → full, so peak memory never includes both.
  CI runs both tiers in `integration-security.yml` (PR opened to main, pushes to
  main/develop, manual dispatch) — full OAuth2/browser coverage is never skipped there.
- **Stale image:** the tier targets build the app image only if it is missing. After
  changing `src/` or `templates/` (templates compile into the binary), rebuild first or tests
  assert against old HTML. Both tiers run `numberguess-claude-rust:latest`, so `make build`
  (or `docker compose -f docker-compose.test-mock-auth.yml -p numberguess-mock build app`)
  refreshes it for either tier.
- **No session caching in the full tier (deliberate):** each `create_authenticated_client()`
  does a fresh browser login. A per-binary cookie cache was removed because each full-tier
  binary makes at most one such call, so it never got a second hit. If you add more
  authenticated-client tests to a full-tier binary, accept ~2-3s each, reintroduce caching
  (design note in `tests/common/auth_helpers.rs`), or — usually better — use the light tier.
- Full-tier resource caps (`docker-compose.integration.yml`): Keycloak heap `-Xmx256m` +
  768m mem_limit; Selenium shm 512mb + 1g mem_limit + single session.

## Networking
Tests run on the host; Selenium runs inside Docker, so the same service has two addresses:

| Service | From tests (host) | Inside Docker network |
|---|---|---|
| oauth2-proxy (entry point) | localhost:8080 | oauth2-proxy:4180 |
| Keycloak | localhost:8090 | keycloak:8090 |
| app | not exposed | app:4080 |
| app health | localhost:8081 | app:8081 |
| Redis | localhost:6379 | redis:6379 |
| Selenium | localhost:4444 | selenium:4444 |

Environment variables (the make targets set them):
- `GAME_SERVER_BASE_URL=http://localhost:8080` — where tests send requests
- `SELENIUM_REMOTE_URL=http://localhost:4444` — where tests reach Selenium
- `GAME_SERVER_BROWSER_URL=http://oauth2-proxy:4180` — where the browser navigates; it runs
  in Docker, where `localhost:4180` does not exist
- Light tier also: `MOCK_AUTH=1`, `MOCK_COMPOSE_FILE`, `MOCK_COMPOSE_PROJECT`

Re-run one binary with the stack already up:
```bash
# light tier
MOCK_AUTH=1 GAME_SERVER_BASE_URL=http://localhost:8080 \
  cargo test --test web_endpoints_test -- --test-threads=1
# full tier
GAME_SERVER_BASE_URL=http://localhost:8080 GAME_SERVER_BROWSER_URL=http://oauth2-proxy:4180 \
SELENIUM_REMOTE_URL=http://localhost:4444 \
  cargo test --test auth_integration_test -- --test-threads=1
```

## Startup and debugging
- Full stack takes ~60-70s to become healthy; Keycloak (H2 in-memory DB: migrations + realm
  import, ~60s) is the bottleneck. Order: postgres + redis → keycloak (independent) → app
  (after postgres) → oauth2-proxy (after keycloak, redis, app) → selenium (after
  oauth2-proxy, app). The make targets wait on health checks.
- Stacks stay up after tests for debugging; tear down with `make test-func-down` /
  `make test-down` when done.
- Test credentials: `admin@local.test` / `password`.
- Health: Keycloak http://localhost:8090/health/ready · app http://localhost:8081/health ·
  oauth2-proxy http://localhost:8080 (should redirect to Keycloak).
- Logs: `docker compose logs keycloak` (or `app`, `oauth2-proxy`). Status:
  `docker compose -f docker-compose.yml -f docker-compose.integration.yml --profile integration ps`.

| Symptom | Check |
|---|---|
| Keycloak not responding | Still starting (~60s); `docker compose logs keycloak \| grep -i realm` for realm import progress |
| Login fails / session cookie not found | Realm import (`keycloak/realm-export.json`), `docker compose logs oauth2-proxy`; the session cookie is `_oauth2_proxy` |
| Selenium connection refused | `curl http://localhost:4444/status`; is `SELENIUM_REMOTE_URL` set? |
| Session issues | `docker compose exec redis redis-cli ping` |
| 401 errors | Auth headers sent? See `tests/common/auth_helpers.rs` |
| Tests see old HTML | Stale app image — rebuild (above) |
| Port conflicts | 4080, 5432, 6379, 8080, 8081, 8090 must be free |
