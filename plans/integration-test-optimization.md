# Integration Test Optimization Analysis

**Date:** 2025-11-09 · **Refreshed:** 2026-07-07
**Current Test Runtime:** 160-220 seconds (2.7-3.7 minutes)
**Optimized Target:** 15-20 seconds
**Potential Improvement:** 89% faster (8.9x speedup)
**Status:** APPROVED — implement as precursor to `plans/code-quality-improvements.md`

---

## 2026-07-07 Refresh (READ THIS FIRST — supersedes stale details below)

The original analysis below remains architecturally correct, but was written 2025-11-09 and never
implemented (Makefile still has only the monolithic `test-integration` target). This refresh
updates it against the current codebase, adds **memory/resource reduction** as a co-equal goal
(development happens on a resource-constrained laptop), and is now approved for implementation
**before** the code-quality plan, whose per-phase test checkpoints will depend on the light tier.

### Measured resource usage (2026-07-07, live during a green baseline run)

| Service | Memory | Needed by |
|---|---|---|
| selenium (seleniarm/standalone-chromium, shm_size 2gb) | 610 MiB | 5 tests |
| keycloak (quay.io/keycloak 26.0, H2 in-mem) | 567 MiB | 5 tests |
| postgres:16 | 52 MiB | all |
| oauth2-proxy | 25 MiB | 5 tests |
| redis:7-alpine | 8 MiB | 5 tests |
| app | 5 MiB | all |

~93% of stack memory (≈1.2 GiB) serves the 5 tests that genuinely need the auth/browser stack.
The light tier (postgres + app + nginx mock-auth) needs **≈70 MiB** total.

### Refreshed test inventory (was 19 tests; now 21 Docker-dependent tests)

Changes since the original analysis: `csrf_test.rs` added (2 tests); `cli_test.rs` has 6 non-Docker
tests; `integration_test.rs` has 2 `#[ignore]`d tests. Baseline 2026-07-07: all green.

| File | Tests | Tier | Notes |
|---|---|---|---|
| `api_edge_cases_test.rs` | 5 | **Light** | Business logic via HTTP; headers via nginx |
| `web_endpoints_test.rs` | 4 | **Light** | HTML fragment assertions; no DOM needed |
| `concurrency_test.rs` | 3 | **Light** | Transaction/race tests; DB + app only |
| `csrf_test.rs` | 2 | **Light** | NEW since original plan. Verified 2026-07-07: uses Selenium only to obtain a session cookie; CSRF mechanics (axum-csrf cookie + `authenticity_token` form field) are app-level. Needs a cookie-jar `reqwest::Client` in the light tier — mock nginx must pass cookies through untouched. |
| `auth_integration_test.rs` (2 of 5: `*_work_when_authenticated`) | 2 | **Light** | Test that endpoints accept an authenticated user, not the OAuth2 flow itself |
| `auth_integration_test.rs` (3 of 5: login flow, redirect, 401) | 3 | **Full** | Validate oauth2-proxy/Keycloak behavior itself |
| `web_ui_test.rs` | 2 | **Full** | Real browser: HTMX swaps, DOM, client-side validation |
| **Totals** | **21** | 16 light / 5 full | 76% of tests escape the heavy stack |

### Additions to the original plan (resource focus)

These apply to the FULL tier so that even when it runs, it fits a small laptop:

- [ ] Cap Keycloak JVM heap in compose: `JAVA_OPTS_KC_HEAP="-Xms64m -Xmx256m"` (measured 567 MiB unbounded; near-empty dev realm doesn't need it)
- [ ] Reduce selenium `shm_size: 2gb` → `512mb` and set `SE_NODE_MAX_SESSIONS=1` (tests run `--test-threads=1`; never more than one browser session)
- [ ] Add `mem_limit` to keycloak (768m) and selenium (1g) so a spike OOMs the container instead of swap-thrashing the host
- [ ] `make test-integration` runs tiers **sequentially with teardown between them** — peak memory never includes both tiers
- [ ] Session reuse for the full tier (1 Selenium login per test binary via `tokio::sync::OnceCell`, not per test) — cuts Chrome CPU churn

### Corrections to the original plan text

- Test counts/categorization in the body below are stale — use the table above.
- `once_cell` crate examples below: prefer `std::sync::OnceLock` / `tokio::sync::OnceCell` (no new dependency).
- Original savings table omitted csrf tests; light tier is 16 tests, not 14-16.
- CI note: `integration-security.yml` must run BOTH tiers on every push — that is what preserves
  full-fidelity coverage while local runs default to the light tier.

### Implementation phases (supersedes "Implementation Plan" section below in ordering; details there still apply)

- [x] **T1 — Light-tier infrastructure** (2026-07-07): `docker-compose.test-mock-auth.yml` (postgres + app + nginx header-injecting proxy on localhost:8080, project `numberguess-mock`), `test-fixtures/mock-auth-nginx.conf` (mirrors real test user identity; forwards cookies + `Host`). Verified manually: game create + guess + CSRF cookie round-trip via curl through the proxy. Gotcha found: healthcheck must use 127.0.0.1 (in-container localhost resolves to ::1; nginx listens IPv4 only).
- [x] **T2 — Test helper split** (2026-07-07): implemented as `MOCK_AUTH=1` env-var branch inside the existing helpers instead of a separate module — zero structural changes to test files. `create_authenticated_client_selenium` renamed to tier-aware `create_authenticated_client`; `ensure_server_ready`/`ensure_selenium_ready` skip auth-stack checks in mock mode; `concurrency_test.rs` app-restart is tier-aware. **Deviation from the triage table:** the 2 `*_work_when_authenticated` tests stay in the full tier with the rest of `auth_integration_test.rs` (avoids splitting the file; costs ~2 logins ≈ 5s in the full tier). Actual split: 14 light + 7 full Docker tests. No assertion changes.
- [x] **T3 — Makefile + tier wiring** (2026-07-07): `test-func` (light, also runs non-Docker `cli_test` + `integration_test`), `test-func-down`, `test-auth` (full), `test-integration` = func → teardown → auth (keep-running-for-debug on last tier only), `test-down` tears down both.
- [x] **T4 — Full-tier resource caps** (2026-07-07): Keycloak `JAVA_OPTS_KC_HEAP=-Xms64m -Xmx256m` + mem_limit 768m; selenium shm 2gb→512mb, `SE_NODE_MAX_SESSIONS=1`, mem_limit 1g. ~~Session cookie cached per test binary via `tokio::sync::OnceCell`~~ **Reverted 2026-07-08 after independent review**: the cache provided zero deduplication — each full-tier binary makes at most one `create_authenticated_client()` call (web_ui_test drives the browser directly), so the cell was written once and never re-read. Removed as dead complexity; design choice documented in `tests/common/auth_helpers.rs` and CLAUDE.md so future full-tier tests know each call costs a ~2-3s browser login.
- [x] **T5 — Verification gate** (2026-07-08): `make test-integration` GREEN, exit 0.
  **27 passed / 0 failed / 2 ignored — exactly matches the 2026-07-07 baseline.**
  - Light tier: 20 passed / 2 ignored (api_edge_cases 5, cli 6, concurrency 3, csrf 2,
    integration 0+2 ignored, web_endpoints 4). Per-suite speedups vs baseline:
    api_edge_cases 18.7s→2.2s, csrf 11.0s→0.6s, web_endpoints 20.6s→1.0s,
    concurrency 32.9s→~13-22s (dominated by the app-restart readiness wait).
  - Full tier: auth_integration 5 passed (36.0s), web_ui 2 passed (11.4s).
  - Resource caps measured live: keycloak 485 MiB (limit 768 MiB; was 567 MiB unbounded),
    selenium 479 MiB (limit 1 GiB, shm 512 MB; was 610 MiB + 2 GB shm). Full-tier stack
    ≈1.07 GiB peak — and now runs only for 7 tests instead of gating everything.
  - Light-tier stack ≈70-100 MiB total (nginx + postgres + app).
  - CI (`integration-security.yml`) switched to `make test-func` + `make test-auth`
    (both tiers on every push); also fixed a latent bug where compose silently rebuilt
    the image because the loaded artifact tag didn't match — added a retag step.
  - CLAUDE.md and README testing docs updated (README's stale `test-compose*` targets
    replaced with the real ones). fmt/clippy(-D warnings)/unit all clean after changes.
- [x] **T6 — Handoff** (2026-07-08): **PLAN IMPLEMENTED.** The code-quality plan's interim
  checkpoints now use `make test-func`; full `make test-integration` at its Phases 2/6
  and final verification. Note for CI: not yet exercised on GitHub Actions — verify the
  workflow on the first push of this branch.
- [x] **Post-review fixes** (2026-07-08, from independent 8-angle review — 9 findings, all resolved):
  1. Tier membership guard: `FUNC_TESTS`/`AUTH_TESTS` Makefile vars are now the single
     source of truth; `make test-tier-check` (prereq of both tier targets) fails if any
     `tests/*_test.rs` is unassigned or double-assigned (restores the safety the removed
     `cargo test --tests` catch-all provided).
  2. Light-tier compose coordinates owned by the Makefile (`MOCK_COMPOSE_FILE/PROJECT`
     exported to tests); concurrency test asserts a running app container before
     restarting (kills the silent no-op false-green mode).
  3. Session-reuse OnceCell deleted (see T4 note above).
  4. Full-stack `--wait-timeout` 120→240 (Keycloak worst-case-healthy ≈115s left ~5s margin).
  5. `.NOTPARALLEL:` added — `make -j` no longer races tier teardown/port 8080.
  6. Light-tier healthcheck budgets extended to cover the 120s wait window (proxy retries
     10→40, postgres 5→24).
  7. `MOCK_AUTH` accepts truthy/falsy variants and panics on unrecognized values;
     `create_webdriver` asserts non-mock mode with a clear run-via-`make test-auth` message.
  8. Doc drift fixed: docs/testing-guide.md, AGENTS.md, CLAUDE.md (Key Points + async
     examples + session claims), environment.rs panic message.
  9. Compose duplication (postgres/app vs docker-compose.yml) documented as deliberate
     with keep-in-sync notes (extends/override judged not worth the coupling).
  Refuted by verification (no action): nginx stale-upstream-IP (compose `restart`
  preserves the container), Selenium shm crash (`--disable-dev-shm-usage` already set in
  tests/common/webdriver.rs), hardcoded cookie URL (all request sites use the same
  literal), mock-identity divergence (user identity is logging-only).

**Verification principle: no test is deleted, no assertion changes, both tiers stay green.**
Test-infrastructure-only change; application code and REST API untouched.

---

## Executive Summary

The integration test suite currently takes **3-4 minutes** to run, with the majority of time spent on unnecessary authentication overhead. Analysis reveals that:

- **Only 3 out of 19 tests (15.8%)** actually need the full OAuth2/Keycloak stack
- **16 tests (84.2%)** are testing business logic and waste ~2-3 seconds each on OAuth2 flows
- **Keycloak startup** is the critical bottleneck (55-60 seconds)
- **Authentication overhead** accounts for 38-57 seconds of wasted time

By implementing a **hybrid testing architecture** that separates auth tests from functional tests, we can achieve an **89% reduction** in test runtime with minimal risk.

---

## Table of Contents

1. [Current Performance Analysis](#current-performance-analysis)
2. [Application Authentication Architecture](#application-authentication-architecture)
3. [Test Categorization](#test-categorization)
4. [Optimization Opportunities](#optimization-opportunities)
5. [Recommended Solutions](#recommended-solutions)
6. [Implementation Plan](#implementation-plan)
7. [Risk Assessment](#risk-assessment)

---

## Current Performance Analysis

### Timing Breakdown

| Phase | Current Time | Bottleneck |
|-------|-------------|------------|
| **Service Startup** | 60-70s | Keycloak (55-60s) |
| - postgres, redis | 10s | - |
| - keycloak | 55-60s | H2 migrations + realm import |
| - app | 5s | - |
| - oauth2-proxy | 5s | - |
| - selenium | 10s | - |
| **Authentication Overhead** | 38-57s | 19 tests × 2-3s each |
| **Test Execution** | 60-90s | Sequential (`--test-threads=1`) |
| **Total Runtime** | **160-220s** | **2.7-3.7 minutes** |

### Service Dependencies

```
Startup Sequence (health check dependencies):
1. postgres (5s) + redis (10s) → parallel start
2. keycloak (55-60s) → CRITICAL PATH BOTTLENECK
3. app (depends on postgres)
4. oauth2-proxy (depends on keycloak + redis + app)
5. selenium (20s start_period, depends on oauth2-proxy)
```

### Test Inventory

- **Total integration tests:** 19 tests across 7 files
  - `api_edge_cases_test.rs`: 5 tests
  - `auth_integration_test.rs`: 5 tests
  - `web_endpoints_test.rs`: 4 tests
  - `concurrency_test.rs`: 3 tests
  - `web_ui_test.rs`: 2 tests
  - `cli_test.rs`: 0 tokio tests
  - `integration_test.rs`: 0 tokio tests

---

## Application Authentication Architecture

### How Authentication Works

**Critical Discovery:** The application uses **trust-based header authentication**.

```rust
// src/auth/mod.rs - AuthenticatedUser extractor
// Application ONLY checks for these headers:
- X-Forwarded-User (required)
- X-Forwarded-Email (required)
- X-Forwarded-Preferred-Username (optional)
- X-Forwarded-Groups (optional)
```

**Security Model:**
- Application does NOT validate tokens, sessions, or perform cryptographic checks
- Simply reads headers and returns 401 if required headers are missing
- Security relies on network isolation (app port 4080 not exposed externally)
- oauth2-proxy (port 8080) is the only way to reach the app
- Headers are trusted because they can ONLY come from oauth2-proxy

### Network Topology

**Integration Test Environment:**

```
┌─────────────────────────────────────────────┐
│ Host Machine (Tests)                        │
│   localhost:8080  → oauth2-proxy            │
│   localhost:4080  → NOT EXPOSED ⚠️          │
│   localhost:8081  → app health (EXPOSED)    │
└─────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────┐
│ Docker Network                              │
│  ┌──────────────┐      ┌──────────────┐    │
│  │oauth2-proxy  │─────▶│     app      │    │
│  │   :4180      │      │   :4080      │    │
│  └──────────────┘      │   :8081      │    │
│         │              └──────────────┘    │
│         ▼                                   │
│  ┌──────────────┐      ┌──────────────┐    │
│  │  Keycloak    │      │    Redis     │    │
│  │   :8090      │      │   :6379      │    │
│  └──────────────┘      └──────────────┘    │
└─────────────────────────────────────────────┘
```

**Key Constraint:** App port 4080 is NOT exposed in `docker-compose.integration.yml`. Only port 8081 (health check) is exposed. This prevents direct header injection and requires tests to go through oauth2-proxy.

---

## Test Categorization

### Tests That MUST Use Full OAuth2 Stack (3 tests - 15.8%)

| Test File | Test Name | Reason |
|-----------|-----------|--------|
| `auth_integration_test.rs` | `test_oauth2_login_flow` | Tests the OAuth2 flow itself (redirect, login, callback, session) |
| `auth_integration_test.rs` | `test_unauthenticated_web_ui_redirects_to_login` | Tests oauth2-proxy redirect behavior |
| `auth_integration_test.rs` | `test_unauthenticated_api_returns_401` | Tests oauth2-proxy rejection of unauthenticated requests |

**These tests validate the authentication infrastructure and CANNOT be optimized.**

### Tests That DON'T Need Full OAuth2 Stack (16 tests - 84.2%)

#### api_edge_cases_test.rs (5 tests) ✅ CAN OPTIMIZE

1. `test_guess_nonexistent_game` - Tests 404 on nonexistent game
2. `test_concurrent_games` - Tests multiple games at once
3. `test_guess_after_limit_reached` - Tests guess limit enforcement
4. `test_zero_limit_means_unlimited` - Tests unlimited guesses
5. `test_web_rejects_excessive_guess_limit` - Tests validation boundaries

**Current:** Full Selenium OAuth2 (~2-3s each)
**Could be:** Direct authentication with headers (<0.1s each)
**Savings:** 10-15 seconds for this file

#### web_endpoints_test.rs (4 tests) ✅ CAN OPTIMIZE

1. `test_static_file_serving` - Tests index.html serving
2. `test_web_form_endpoints` - Tests form submission
3. `test_remaining_guesses_display` - Tests counter updates
4. `test_no_remaining_guesses_display_without_limit` - Tests counter absence

**Current:** Full Selenium OAuth2 (~2-3s each)
**Could be:** Direct authentication with headers (<0.1s each)
**Savings:** 8-12 seconds for this file

#### concurrency_test.rs (3 tests) ✅ CAN OPTIMIZE

1. `test_concurrent_guesses_on_same_game` - Tests transaction isolation
2. `test_race_condition_guess_during_deletion` - Tests race conditions
3. `test_game_persistence_across_restart` - Tests database persistence

**Current:** Full Selenium OAuth2 (~2-3s each)
**Could be:** Direct authentication with headers (<0.1s each)
**Savings:** 6-9 seconds for this file

#### auth_integration_test.rs (2 additional tests) ✅ CAN OPTIMIZE

4. `test_web_ui_endpoints_work_when_authenticated` - Tests POST /game/new works
5. `test_api_endpoints_work_when_authenticated` - Tests API endpoints accept auth

**Current:** Full Selenium OAuth2 (~2-3s each)
**Could be:** Direct authentication with headers (<0.1s each)
**Savings:** 4-6 seconds for this file

#### web_ui_test.rs (2 tests) ⚠️ PARTIAL OPTIMIZATION

1. `test_web_ui_game_flow` - Tests browser-based game flow
2. `test_web_ui_invalid_inputs` - Tests browser form validation

**Current:** Selenium OAuth2 + browser automation
**Could be:** Selenium browser with pre-injected session cookie (skip OAuth2 login)
**Reason:** Still need actual browser for HTMX, DOM updates, and client-side validation
**Savings:** 2-4 seconds for this file (skip login, inject cookie directly)

### Summary

| Category | Count | Current Time | Optimized Time | Savings |
|----------|-------|--------------|----------------|---------|
| Must use full OAuth2 | 3 | 7-9s | 7-9s | 0s |
| Can use mock auth | 14 | 28-42s | 1-2s | 26-40s |
| Can inject session cookie | 2 | 5-6s | 2-3s | 2-3s |
| **Total** | **19** | **40-57s** | **10-14s** | **28-43s** |

---

## Optimization Opportunities

### 1. Session Reuse (Quick Win)

**Current Problem:** Every test performs a fresh Selenium OAuth2 login

**Solution:** Create authenticated session once per test file, reuse across tests

**Implementation:**
```rust
// tests/common/auth_helpers.rs
use once_cell::sync::OnceCell;

static SHARED_SESSION: OnceCell<Cookie> = OnceCell::new();

pub async fn get_or_create_session() -> Result<Cookie, Box<dyn Error>> {
    SHARED_SESSION.get_or_try_init(|| async {
        let driver = create_webdriver(selenium_url()).await?;
        let cookie = login_with_keycloak_selenium(&driver).await?;
        driver.quit().await?;
        Ok(cookie)
    }).await.cloned()
}
```

**Benefits:**
- 1 OAuth2 login per test file instead of per test
- Reduces 19 logins to ~7 logins
- Saves 30 seconds (63% reduction in auth overhead)
- Minimal code changes

**Drawbacks:**
- Still requires full stack for the initial login
- Session expiration needs handling (1h default)
- Doesn't work well with `--test-threads=1` (current setting)

**Time Savings:** 30 seconds
**Effort:** 2-4 hours
**Risk:** Low

---

### 2. Service Health Check Tuning (Quick Win)

**Current Problem:** Conservative health check timings

**Solution:** Reduce start periods and increase check frequencies

**Changes:**
```yaml
selenium:
  healthcheck:
    start_period: 10s  # Down from 20s
    interval: 5s       # More frequent checks

oauth2-proxy:
  healthcheck:
    interval: 5s       # Down from 10s
```

**Benefits:**
- Saves 10-15 seconds on startup
- No code changes needed
- Low risk (health checks still robust)

**Time Savings:** 10-15 seconds
**Effort:** 1 hour
**Risk:** Low

---

### 3. Mock Authentication Proxy (Best Long-Term Solution)

**Current Problem:** 84% of tests don't need OAuth2 but must go through full stack

**Solution:** Create nginx-based mock authentication proxy that adds headers

**Implementation:**

**docker-compose.test-mock-auth.yml:**
```yaml
version: '3.8'

services:
  postgres:
    # ... same as integration

  app:
    # ... same as integration

  mock-auth-proxy:
    image: nginx:alpine
    ports:
      - "8080:8080"
    volumes:
      - ./test-fixtures/mock-auth-nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      app:
        condition: service_healthy
```

**test-fixtures/mock-auth-nginx.conf:**
```nginx
events {
    worker_connections 1024;
}

http {
    server {
        listen 8080;

        location / {
            proxy_pass http://app:4080;
            proxy_set_header X-Forwarded-User "test-user-id";
            proxy_set_header X-Forwarded-Email "test@example.com";
            proxy_set_header X-Forwarded-Preferred-Username "testuser";
            proxy_set_header X-Forwarded-Groups "users";
            proxy_set_header Host $host;
        }
    }
}
```

**Benefits:**
- No Keycloak needed (saves 55-60s startup)
- No oauth2-proxy needed
- No Redis needed
- No Selenium needed
- Instant authentication (<0.1s)
- 30x faster than current approach
- Clear separation: auth tests vs functional tests

**Drawbacks:**
- Requires maintaining two docker-compose configurations
- Need to update Makefile and CI
- More complex test infrastructure

**Time Savings:** 38-40 seconds (81% reduction in auth overhead)
**Effort:** 4-6 hours
**Risk:** Low (test-only changes)

---

### 4. Keycloak Pre-Import (Medium Effort)

**Current Problem:** Keycloak imports realm on every startup (adds 10-15s)

**Solution:** Create custom Keycloak image with realm baked in

**Implementation:**
```dockerfile
# Dockerfile.keycloak-test
FROM quay.io/keycloak/keycloak:26.0.7

COPY keycloak/realm-export.json /opt/keycloak/data/import/realm.json

# Pre-import realm during image build
RUN /opt/keycloak/bin/kc.sh import --dir /opt/keycloak/data/import
```

**Benefits:**
- Consistent 10-15s savings
- No state management issues

**Drawbacks:**
- Maintenance overhead (custom image)
- CI/CD changes needed
- Image needs rebuilding when realm changes

**Time Savings:** 10-15 seconds
**Effort:** 2-3 days
**Risk:** Medium

---

### 5. Hybrid Testing Architecture (RECOMMENDED)

**Solution:** Combine optimizations into a comprehensive strategy

**Architecture:**

```
┌─────────────────────────────────────────────────────┐
│ Integration Test Suite (19 tests)                   │
└─────────────────────────────────────────────────────┘
                    │
        ┌───────────┴────────────┐
        │                        │
        ▼                        ▼
┌──────────────────┐    ┌─────────────────────┐
│ Auth Tests (3)   │    │ Functional Tests    │
│                  │    │ (16)                │
│ Full Stack:      │    │                     │
│ - Keycloak       │    │ Mock Auth:          │
│ - oauth2-proxy   │    │ - nginx proxy       │
│ - Redis          │    │ - Static headers    │
│ - Selenium       │    │ - No Keycloak       │
│ - postgres       │    │ - No Selenium       │
│ - app            │    │ - postgres          │
│                  │    │ - app               │
│ Runtime: 10-15s  │    │                     │
│                  │    │ Runtime: 5s         │
└──────────────────┘    └─────────────────────┘
```

**Makefile Targets:**
```makefile
# Run only auth tests (full stack)
test-auth:
    docker compose -f docker-compose.integration.yml up -d --wait
    cargo test --test auth_integration_test -- --nocapture

# Run only functional tests (mock auth)
test-func:
    docker compose -f docker-compose.test-mock-auth.yml up -d --wait
    cargo test --test api_edge_cases_test --test web_endpoints_test \
               --test concurrency_test --test web_ui_test \
               -- --nocapture

# Run all integration tests
test-integration: test-auth test-func
```

**Benefits:**
- 90% of tests run in 5 seconds
- Clear separation of concerns
- Faster CI feedback (run functional tests first)
- Auth tests still validate full flow
- No changes to application code
- No security model violations

**Drawbacks:**
- More complex Makefile
- Two docker-compose configurations
- Need to maintain mock auth setup

**Time Savings:**
- Current: ~180s total runtime
- Auth tests: 10-15s
- Functional tests: 5s
- **New total: 15-20s**
- **Savings: 160-165 seconds (89% reduction)**

**Effort:** 6-8 hours
**Risk:** Low

---

## Recommended Solutions

### Priority Ranking

| Solution | Time Savings | Effort | Risk | Priority | Recommendation |
|----------|-------------|--------|------|----------|----------------|
| Health Check Tuning | 10-15s | 1h | Low | 1 | ✅ Do First |
| Session Reuse | 30s | 2-4h | Low | 2 | ✅ Quick Win |
| Mock Auth Proxy | 38-40s | 4-6h | Low | 3 | ⭐ Best ROI |
| Keycloak Pre-Import | 10-15s | 2-3d | Medium | 4 | Optional |
| Hybrid Architecture | 160s | 6-8h | Low | - | ⭐⭐ Recommended |

### Recommended Path

**Option A: Quick Wins First (Incremental Approach)**

**Week 1:**
1. Health check tuning (1 hour) → 10-15s savings
2. Session reuse (2-4 hours) → 30s savings
3. **Total improvement: 40-45 seconds (20-25% faster)**

**Week 2:**
4. Mock auth proxy (4-6 hours) → Additional 50-60s savings
5. **Cumulative improvement: 90-105 seconds (50-55% faster)**

**Week 3:**
6. Full hybrid architecture (integrate above) → Final optimization
7. **Final improvement: 160-165 seconds (89% faster)**

**Option B: Hybrid Architecture (All-In Approach)**

**Single Implementation (6-8 hours):**
- Implement complete hybrid architecture
- Mock auth proxy for functional tests
- Session reuse for auth tests
- Tune health checks
- **Immediate improvement: 89% faster**

### Our Recommendation: **Option B (Hybrid Architecture)**

**Rationale:**
- Most comprehensive solution
- Clean architectural separation
- Best long-term maintainability
- Similar effort to incremental approach (6-8 hours vs 7-11 hours)
- Avoids throwaway code from incremental steps

---

## Implementation Plan

### Phase 1: Infrastructure Setup (2-3 hours)

**1. Create Mock Auth Proxy**

Create `docker-compose.test-mock-auth.yml`:
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: numberguess_test
      POSTGRES_USER: numberguess
      POSTGRES_PASSWORD: password
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U numberguess"]
      interval: 5s
      timeout: 5s
      retries: 5

  app:
    build:
      context: .
      dockerfile: Dockerfile
      target: debug
    environment:
      DATABASE_URL: postgresql://numberguess:password@postgres:5432/numberguess_test
      RUST_LOG: info
    depends_on:
      postgres:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
      interval: 5s
      timeout: 5s
      retries: 5

  mock-auth-proxy:
    image: nginx:alpine
    ports:
      - "8080:8080"
    volumes:
      - ./test-fixtures/mock-auth-nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      app:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:8080/health"]
      interval: 5s
      timeout: 5s
      retries: 5
```

Create `test-fixtures/mock-auth-nginx.conf`:
```nginx
events {
    worker_connections 1024;
}

http {
    server {
        listen 8080;

        location / {
            proxy_pass http://app:4080;
            proxy_set_header X-Forwarded-User "test-user-123";
            proxy_set_header X-Forwarded-Email "test@example.com";
            proxy_set_header X-Forwarded-Preferred-Username "testuser";
            proxy_set_header X-Forwarded-Groups "users";
            proxy_set_header Host $host;
        }

        location /health {
            proxy_pass http://app:8081/health;
        }
    }
}
```

**2. Tune Health Checks**

Update `docker-compose.integration.yml`:
```yaml
selenium:
  healthcheck:
    start_period: 10s  # Reduced from 20s
    interval: 5s

oauth2-proxy:
  healthcheck:
    interval: 5s       # Reduced from 10s
```

### Phase 2: Test Refactoring (2-3 hours)

**1. Update Test Helpers**

Create `tests/common/mock_auth_helpers.rs`:
```rust
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub async fn create_mock_auth_client() -> Result<Client, Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()?;

    Ok(client)
}
```

Update `tests/common/auth_helpers.rs`:
```rust
use once_cell::sync::OnceCell;
use reqwest::cookie::Cookie;

static SHARED_SESSION: OnceCell<Cookie> = OnceCell::new();

pub async fn get_or_create_selenium_session() -> Result<Cookie, Box<dyn std::error::Error>> {
    SHARED_SESSION.get_or_try_init(|| async {
        let driver = create_webdriver(selenium_url()).await?;
        let cookie = login_with_keycloak_selenium(&driver).await?;
        driver.quit().await?;
        Ok(cookie)
    }).await.cloned()
}
```

**2. Refactor Test Files**

Update functional test files to use mock auth:
- `tests/api_edge_cases_test.rs`
- `tests/web_endpoints_test.rs`
- `tests/concurrency_test.rs`
- Parts of `tests/auth_integration_test.rs`

Pattern:
```rust
#[tokio::test]
async fn test_example() {
    // Use mock auth for functional tests
    let client = mock_auth_helpers::create_mock_auth_client()
        .await
        .expect("Failed to create client");

    // Test logic...
}
```

Keep auth tests using Selenium:
- `tests/auth_integration_test.rs` (3 tests)
- `tests/web_ui_test.rs` (2 tests, with session reuse)

### Phase 3: Build System Updates (1-2 hours)

**1. Update Makefile**

Add new targets:
```makefile
# Functional tests with mock auth (fast)
.PHONY: test-func
test-func:
	@echo "Starting mock auth environment..."
	docker compose -f docker-compose.test-mock-auth.yml up -d --wait
	@echo "Running functional tests..."
	cargo test --test api_edge_cases_test \
	           --test web_endpoints_test \
	           --test concurrency_test \
	           -- --nocapture
	@echo "Functional tests complete. Use 'make test-func-down' to stop services."

.PHONY: test-func-down
test-func-down:
	docker compose -f docker-compose.test-mock-auth.yml down -v

# Auth tests with full stack (slower)
.PHONY: test-auth
test-auth:
	@echo "Starting full auth stack..."
	docker compose -f docker-compose.yml -f docker-compose.integration.yml \
		--profile integration up -d --wait
	@echo "Running auth tests..."
	cargo test --test auth_integration_test --test web_ui_test -- --nocapture
	@echo "Auth tests complete. Use 'make test-down' to stop services."

# Full integration test suite
.PHONY: test-integration
test-integration:
	@echo "Running functional tests first..."
	$(MAKE) test-func
	@echo "Running auth tests..."
	$(MAKE) test-auth
	@echo "All integration tests complete."

.PHONY: test-integration-cleanup
test-integration-cleanup:
	$(MAKE) test-func-down
	$(MAKE) test-down
```

**2. Update CI Configuration**

Update `.github/workflows/integration-security.yml`:
```yaml
integration-tests:
  runs-on: ubuntu-latest
  needs: build-docker
  steps:
    # ... existing setup ...

    - name: Run functional tests (fast)
      run: make test-func
      timeout-minutes: 5

    - name: Run auth tests (slower)
      run: make test-auth
      timeout-minutes: 10

    - name: Cleanup
      if: always()
      run: make test-integration-cleanup
```

### Phase 4: Testing & Validation (1 hour)

**1. Verify Test Execution**
```bash
# Test mock auth setup
make test-func

# Test auth stack
make test-auth

# Test full suite
make test-integration

# Measure performance
time make test-integration
```

**2. Verify Test Coverage**
```bash
# Ensure all tests still pass
cargo test --all-features

# Check for any missed tests
cargo test --list | grep "#[tokio::test]"
```

### Phase 5: Documentation (1 hour)

**1. Update CLAUDE.md**
- Document new test architecture
- Update "Testing Strategy" section
- Add "Quick Commands" for new Makefile targets

**2. Update README.md**
- Add section on test performance
- Document when to use each test target

**3. Create Migration Guide**
- Document changes for developers
- Explain when to use functional vs auth tests

---

## Risk Assessment

### Low Risk (Test-Only Changes)

**All proposed changes are confined to the test infrastructure:**
- No application code changes
- No production configuration changes
- No database schema changes
- No API contract changes

**Failure modes are isolated:**
- If mock auth fails → Fall back to full stack
- If session reuse fails → Fall back to per-test auth
- If tests fail → Easy to debug (same assertions as before)

### Mitigation Strategies

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Mock auth misconfiguration | Low | Medium | Validate headers match production exactly |
| Session expiration during tests | Low | Low | Implement refresh logic or use shorter test suites |
| Health check timeouts in slow environments | Medium | Low | Keep conservative defaults, tune per-environment |
| Docker compose complexity | Low | Low | Document clearly, provide Makefile helpers |
| CI/CD pipeline failures | Low | Medium | Test locally first, gradual rollout |

### Rollback Plan

If issues arise:
1. Comment out new Makefile targets
2. Revert to `make test-integration` (original)
3. Keep new infrastructure for future refinement
4. No code changes to roll back (test-only)

---

## Success Metrics

### Performance Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| **Total test runtime** | 180s | 20s | 89% faster |
| Service startup (functional) | 65s | 10s | 85% faster |
| Service startup (auth) | 65s | 55s | 15% faster |
| Auth overhead per test | 2.5s | 0.1s | 96% faster |
| Functional test suite | 120s | 5s | 96% faster |
| Auth test suite | 60s | 15s | 75% faster |

### Validation Criteria

**Phase 1 (Infrastructure):**
- ✅ Mock auth compose starts in <10 seconds
- ✅ nginx correctly forwards headers to app
- ✅ App accepts requests through mock auth proxy
- ✅ Health checks pass consistently

**Phase 2 (Test Refactoring):**
- ✅ All 16 functional tests pass with mock auth
- ✅ 3 auth tests pass with full stack
- ✅ 2 web UI tests pass with session injection
- ✅ No test behavior changes (same assertions)

**Phase 3 (Build System):**
- ✅ `make test-func` completes in <10 seconds
- ✅ `make test-auth` completes in <20 seconds
- ✅ `make test-integration` completes in <25 seconds
- ✅ CI pipeline passes

**Phase 4 (Performance):**
- ✅ Total runtime <25 seconds (target: 15-20s)
- ✅ No flaky tests
- ✅ No test coverage loss

---

## Alternative Approaches Considered

### 1. Expose app:4080 Directly (Rejected)

**Approach:** Expose app port 4080 to host for direct header injection

**Pros:**
- Simplest implementation
- Fastest possible (<0.1s per test)

**Cons:**
- ❌ Violates security model (app should never be externally exposed)
- ❌ Requires app code changes (skip auth flag)
- ❌ Different behavior in tests vs production
- ❌ Creates security risk if flag accidentally enabled in production

**Decision:** Rejected - Architectural anti-pattern

### 2. Keycloak with Persistent Database (Rejected)

**Approach:** Use persistent H2 or PostgreSQL for Keycloak

**Pros:**
- Faster on subsequent runs (15-20s vs 60s)

**Cons:**
- ❌ First run still slow (60s)
- ❌ State management between test runs
- ❌ Doesn't help in CI (clean environment each time)
- ❌ Adds complexity

**Decision:** Rejected - Doesn't solve CI problem

### 3. Session Pooling (Deferred)

**Approach:** Maintain pool of authenticated sessions for parallel tests

**Pros:**
- Enables parallel test execution
- Better resource utilization

**Cons:**
- ⚠️ High complexity
- ⚠️ Requires multiple Keycloak users
- ⚠️ Session lifecycle management
- ⚠️ Race condition risks

**Decision:** Deferred - Implement after hybrid architecture stabilizes

### 4. Testcontainers (Evaluated, Not Pursued)

**Approach:** Use Testcontainers instead of docker-compose

**Pros:**
- More flexible container management
- Better isolation per test

**Cons:**
- ⚠️ Slower startup (no shared services)
- ⚠️ More resource intensive
- ⚠️ Complex setup for multi-container dependencies
- ⚠️ Current docker-compose setup works well

**Decision:** Not pursued - Current approach sufficient

---

## Next Steps

### Immediate Actions

1. **Review and approve this plan** with team
2. **Allocate 6-8 hours** for implementation
3. **Schedule implementation** for low-risk period (not before critical release)

### Implementation Checklist

- [ ] Phase 1: Infrastructure Setup (2-3 hours)
  - [ ] Create `docker-compose.test-mock-auth.yml`
  - [ ] Create `test-fixtures/mock-auth-nginx.conf`
  - [ ] Tune health checks in `docker-compose.integration.yml`
  - [ ] Test mock auth setup manually

- [ ] Phase 2: Test Refactoring (2-3 hours)
  - [ ] Create `tests/common/mock_auth_helpers.rs`
  - [ ] Update `tests/common/auth_helpers.rs` with session reuse
  - [ ] Refactor `tests/api_edge_cases_test.rs`
  - [ ] Refactor `tests/web_endpoints_test.rs`
  - [ ] Refactor `tests/concurrency_test.rs`
  - [ ] Refactor applicable tests in `tests/auth_integration_test.rs`
  - [ ] Update `tests/web_ui_test.rs` with session injection

- [ ] Phase 3: Build System Updates (1-2 hours)
  - [ ] Add `test-func` target to Makefile
  - [ ] Add `test-auth` target to Makefile
  - [ ] Update `test-integration` target to run both
  - [ ] Add cleanup targets
  - [ ] Update `.github/workflows/integration-security.yml`

- [ ] Phase 4: Testing & Validation (1 hour)
  - [ ] Run `make test-func` locally
  - [ ] Run `make test-auth` locally
  - [ ] Run `make test-integration` locally
  - [ ] Measure performance improvements
  - [ ] Verify all tests pass
  - [ ] Test in CI pipeline

- [ ] Phase 5: Documentation (1 hour)
  - [ ] Update `CLAUDE.md` with new architecture
  - [ ] Update `README.md` with new test commands
  - [ ] Create migration guide for developers
  - [ ] Update this plan document with results

### Post-Implementation

- [ ] Monitor test stability over 1 week
- [ ] Collect performance metrics
- [ ] Gather developer feedback
- [ ] Consider session pooling for further optimization (if needed)

---

## Conclusion

The integration test suite can be optimized from **3-4 minutes to 15-20 seconds** (89% faster) by implementing a hybrid testing architecture. This approach:

- ✅ Separates auth validation tests (3 tests) from functional tests (16 tests)
- ✅ Uses mock auth proxy for functional tests (no Keycloak needed)
- ✅ Maintains full OAuth2 validation for auth-specific tests
- ✅ Requires only test infrastructure changes (low risk)
- ✅ Improves developer productivity and CI feedback time
- ✅ Maintains test coverage and reliability

**Recommendation: Proceed with hybrid architecture implementation (6-8 hours effort).**

---

## Appendix

### A. Current Test List

```
tests/api_edge_cases_test.rs:
  - test_guess_nonexistent_game
  - test_concurrent_games
  - test_guess_after_limit_reached
  - test_zero_limit_means_unlimited
  - test_web_rejects_excessive_guess_limit

tests/auth_integration_test.rs:
  - test_oauth2_login_flow (MUST use full stack)
  - test_unauthenticated_web_ui_redirects_to_login (MUST use full stack)
  - test_unauthenticated_api_returns_401 (MUST use full stack)
  - test_web_ui_endpoints_work_when_authenticated
  - test_api_endpoints_work_when_authenticated

tests/web_endpoints_test.rs:
  - test_static_file_serving
  - test_web_form_endpoints
  - test_remaining_guesses_display
  - test_no_remaining_guesses_display_without_limit

tests/concurrency_test.rs:
  - test_concurrent_guesses_on_same_game
  - test_race_condition_guess_during_deletion
  - test_game_persistence_across_restart

tests/web_ui_test.rs:
  - test_web_ui_game_flow (needs browser)
  - test_web_ui_invalid_inputs (needs browser)
```

### B. Docker Compose Files

**Current Files:**
- `docker-compose.yml` - Development environment
- `docker-compose.integration.yml` - Integration test overrides

**New Files:**
- `docker-compose.test-mock-auth.yml` - Mock auth for functional tests

### C. Environment Variables

**Mock Auth Environment:**
```bash
# No Keycloak variables needed
DATABASE_URL=postgresql://numberguess:password@postgres:5432/numberguess_test
RUST_LOG=info
```

**Full Auth Environment:**
```bash
# Existing variables (no changes)
OAUTH2_PROXY_*
KEYCLOAK_*
DATABASE_URL
RUST_LOG
```

### D. Test Execution Times

**Baseline (Current):**
```
Service startup: 60-70s
test-auth tests:  60-80s (includes startup)
test-func tests: 100-120s (includes startup)
Total: 160-220s
```

**Optimized (Target):**
```
Mock auth startup: 8-10s
test-func: 5s (16 tests)
Auth startup: 55-60s (Keycloak still needed)
test-auth: 10-15s (3 tests with session reuse)
Total: 15-25s
```

---

**Document Version:** 1.0
**Last Updated:** 2025-11-09
**Status:** Proposed
**Next Review:** After implementation
