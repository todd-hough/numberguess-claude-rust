---
paths:
  - "src/auth/**"
  - "src/server/**"
  - "keycloak/**"
  - "docker-compose*.yml"
  - "test-fixtures/**"
---
# Authentication architecture

Components:
- **oauth2-proxy** — localhost:8080 → `oauth2-proxy:4180`; the only external entry point
- **Keycloak** — localhost:8090; OIDC provider; realm, users, and groups in `keycloak/realm-export.json`
- **Redis** — oauth2-proxy session store
- **app** — port 4080, internal only; health endpoint on 8081 (internal in dev; the full
  test tier publishes it as localhost:8081)

Flow:
1. User opens http://localhost:8080; oauth2-proxy checks for a valid session.
2. No session → redirect to Keycloak (localhost:8090) to log in.
3. Keycloak redirects back to oauth2-proxy with an authorization code (PKCE, S256).
4. oauth2-proxy exchanges the code for tokens and stores the session in Redis.
5. oauth2-proxy forwards the request to the app with identity headers (proxy mode sends
   `X-Forwarded-*`, not `X-Auth-Request-*`):
   - `X-Forwarded-User` — user ID (OIDC subject)
   - `X-Forwarded-Email`
   - `X-Forwarded-Preferred-Username`
   - `X-Forwarded-Groups` — comma-separated
6. `AuthenticatedUser` (`src/auth/mod.rs`) is the Axum extractor for these headers;
   `is_in_group()` supports authorization.

Security properties:
- App is network-isolated; no route allows anonymous access.
- Redis-backed sessions (horizontally scalable) with secure cookies (httponly, samesite=lax).
- Dev secrets are hardcoded in compose and marked `DEV-LOCAL-TESTING-ONLY-*`; production
  must override them via proper secret management.
- Dev/test login: `admin@local.test` / `password` (username `admin`, group `/admin`).

CSRF (web forms only; configured in `src/server/mod.rs`):
- `axum_csrf` layer with cookie `x-csrf-token`. Pages embed the token as a hidden
  `authenticity_token` form field; both web POST handlers (`/game/new`,
  `/game/{id}/guess`) verify it and return `WebError::InvalidCsrf` (400) on failure.
  The JSON API does not use CSRF tokens.
- `CSRF_SECRET` is optional. If set it must be ≥ 64 bytes or startup fails; if unset, a
  random key is generated per process, so tokens don't survive a restart and aren't shared
  across instances.
- Covered by `tests/csrf_test.rs`.

The light test tier replaces oauth2-proxy with nginx (`test-fixtures/mock-auth-nginx.conf`)
injecting the same `X-Forwarded-*` headers for the test user — keep it in sync when header
handling changes.
