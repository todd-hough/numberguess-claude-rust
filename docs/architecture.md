# System Architecture

## Overview

One Rust binary provides three interfaces over the same core game logic: an interactive
CLI, a JSON REST API, and an HTMX web UI. In server mode every request passes through an
authentication proxy; the app itself is never exposed directly.

```
Terminal ──► CLI mode (same binary; no server, no database)

Browser / API client
   │
   ▼
oauth2-proxy (localhost:8080) ◄──► Keycloak (localhost:8090, OIDC login)
   │   sessions in Redis
   ▼  adds X-Forwarded-* identity headers
app (port 4080, internal only)          health server (port 8081)
   ├─ /api/*            JSON API
   └─ /, /game/*, /difficulty-preview   HTML fragments for HTMX
   │
   ▼
PostgreSQL (games table)
```

## Interfaces

| Interface | Entry points | Notes |
|---|---|---|
| CLI | `cargo run -- [-m MIN] [-x MAX] [-l LIMIT]` | Prompts for any value not given as a flag and re-prompts on invalid input. Guess limit up to 1000. No database. |
| JSON API | `POST /api/games`, `POST /api/games/{game_id}/guess` | Full contract in [api.md](api.md). Guess limit up to 100. |
| Web UI | `GET /`, `POST /game/new`, `POST /game/{game_id}/guess`, `GET /difficulty-preview` | Server-rendered Askama templates; HTMX swaps the returned fragments into the page. The difficulty preview updates live while the setup form is edited (empty response for invalid input). |
| Health | `GET /health` on port 8081 | 200 if the database is reachable, 503 otherwise. |

## Code layout

| Module | Responsibility |
|---|---|
| `src/core/` | Pure game logic with no I/O: `GuessingGame` (`game.rs`), `GuessResult` and `GameError` (`errors.rs`), `GameId` newtype, shared validation (`validators.rs`), difficulty calculator (`features/difficulty/`) |
| `src/api/` | JSON handlers, request/response types, `ApiError` |
| `src/web/` | HTML handlers, Askama template structs, form types, `WebError` |
| `src/cli/` | Argument parsing (clap), input prompts, game loop |
| `src/db/` | `GameRepository` trait and its PostgreSQL implementation |
| `src/server/` | Router, middleware (CSRF, tracing), `AppState`, startup |
| `src/auth/` | `AuthenticatedUser` extractor built from proxy headers |
| `src/main.rs` | Chooses CLI or server mode; database setup |
| `templates/` | Askama templates; `index.html` is the page shell and holds all CSS |
| `migrations/` | SQLx migrations, applied automatically at startup |

Design choices worth knowing:
- **One source of validation.** CLI, API, and web all call `core::validators`, so limits
  can't drift between interfaces.
- **Repository pattern with static dispatch.** Handlers are generic over
  `R: GameRepository` (native async trait methods), so storage can be swapped without
  dynamic dispatch.
- **Typed errors.** `thiserror` enums in every layer; each interface converts them into
  its own response format (see Error handling).

## Server startup and request flow

1. `main.rs` parses arguments; `--server` selects server mode.
2. Connects to PostgreSQL (`DATABASE_URL`, pool of `DB_MAX_CONNECTIONS`, default 5,
   clamped to 1–100) and runs pending migrations.
3. Starts the main server on `--port` (default 4080) and the health server on 8081, then
   prints `READY` to stdout. Logs go to stderr.

For each request:
1. oauth2-proxy checks the session; without one it redirects to Keycloak.
2. Keycloak authenticates the user and redirects back with an authorization code (PKCE
   S256); oauth2-proxy exchanges it for tokens and stores the session in Redis (cookie
   `_oauth2_proxy`).
3. oauth2-proxy forwards the request with `X-Forwarded-User`, `X-Forwarded-Email`,
   `X-Forwarded-Preferred-Username`, and `X-Forwarded-Groups`.
4. The handler receives an `AuthenticatedUser`, validates input, and calls the repository.
5. A guess runs in one database transaction (`SELECT ... FOR UPDATE`), so concurrent
   guesses on the same game are serialized. A game's row is deleted when it ends.

## Network and ports

| Service | Inside Docker | From the host | Purpose |
|---|---|---|---|
| oauth2-proxy | `oauth2-proxy:4180` | `localhost:8080` | Only external entry point |
| Keycloak | `keycloak:8090` | `localhost:8090` | OIDC provider |
| app | `app:4080` | not exposed | Application |
| app health | `app:8081` | `localhost:8081` (full test stack only) | Health check |
| Redis | `redis:6379` | `localhost:6379` | oauth2-proxy sessions |
| PostgreSQL | `postgres:5432` | `localhost:5432` | Game storage |

Containers reach each other by service name; the host reaches them through the mapped
ports. This matters for browser tests, where Selenium runs inside Docker — see
[testing-guide.md](testing-guide.md).

## Persistence

- One `games` row per active game: `game_id` (random u64 stored as BIGINT, primary key),
  `min_value`, `max_value`, `secret_number`, `guess_count`, `max_guesses` (NULL for
  unlimited), and timestamps.
- Queries are checked at runtime, so building the project needs no database.
- Games persist across server restarts. Abandoned games stay until
  `cleanup_old_games(hours_old)` (default 24 hours) is run; nothing schedules it yet.

## Error handling

| Layer | Behavior |
|---|---|
| Core | `GameError` describes each validation failure (range, negatives, limits). |
| JSON API | `ApiError`: validation → **400**, unknown or finished game → **404** `Game with ID {id} not found`, database failure → **500**; body `{"error": "..."}`. Malformed JSON is rejected by Axum before reaching a handler. |
| Web UI | `WebError` renders an error fragment with status **200** so HTMX swaps it into the page (validation message, game not found, update failed). A bad CSRF token returns **400**. |
| CLI | Prints the problem and prompts again. |

## Security

- **Authentication:** every route sits behind oauth2-proxy + Keycloak; the app port is
  never published. Group membership is available via `AuthenticatedUser::is_in_group()`.
- **CSRF:** web form posts must include a valid `authenticity_token` (from `axum_csrf`).
  `CSRF_SECRET` (at least 64 bytes) keeps tokens valid across restarts and instances; if
  unset, a random key is generated at startup.
- **Input validation:** range 0–1,000,000, no negative values, guess limits (100 web/API,
  1000 CLI), overflow-safe arithmetic.
- **Database:** parameterized SQLx queries; pooled connections.
- **Container:** distroless runtime image (no shell or package manager).
- **Development secrets** in the compose files are marked `DEV-LOCAL-TESTING-ONLY-*` and
  must be replaced in production.
- **Known gaps** (rate limiting, request size limits, CORS policy, HTMX from a CDN) are
  tracked in [security-todo.md](security-todo.md).

## Configuration

| Setting | Where | Default |
|---|---|---|
| `--server` / `-s` | CLI flag | CLI mode |
| `--port` / `-p` | CLI flag | 4080 |
| `--min` / `-m`, `--max` / `-x`, `--limit` / `-l` | CLI flags (CLI mode) | prompted |
| `DATABASE_URL` | environment or `.env` | required in server mode |
| `DB_MAX_CONNECTIONS` | environment | 5 (range 1–100) |
| `CSRF_SECRET` | environment | random per process |
| `RUST_LOG` | environment | e.g. `number_guessing_game=info` |

## Technology choices

- **Rust:** memory safety without garbage collection, a strong type system for
  validation, single-binary deployment.
- **Axum + Tokio:** modern async web framework, type-safe routing and extractors, Tower
  middleware (CSRF, tracing).
- **HTMX + Askama:** server-rendered HTML with partial page updates and no front-end build
  step; Askama checks templates at compile time.
- **SQLx + PostgreSQL:** async driver with connection pooling and parameterized queries;
  runtime-checked queries keep builds database-free.
- **oauth2-proxy + Keycloak:** keeps OAuth/OIDC code out of the app; the app only reads
  trusted headers, and network isolation keeps it unreachable except through the proxy.
- **Clap:** derive-based argument parsing with generated help.

Ideas for future work are collected in `plans/feature-ideas.md`.
