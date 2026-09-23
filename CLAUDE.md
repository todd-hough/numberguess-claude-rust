# Number Guessing Game

Rust number guessing game with a CLI and an authenticated web UI + JSON API. A random
secret number is drawn from a user-chosen range; each guess gets higher/lower feedback;
an optional guess limit ends the game when exceeded.

## Hard rules
- NEVER change an external API (e.g. a REST endpoint) without explicit approval.
- ALWAYS document external API behavior (`docs/api.md`) so the doc is the reference for behavior.
- New features: determine the dependencies and implement them one at a time, with tests
  proving each works before moving on to the next.
- When moving from planning to implementation, ALWAYS write the plan to `plans/`. When the
  feature is done, ask the user whether to clean up the plan document.
- Integration tests are async only: `#[tokio::test]` + async `reqwest::Client`. Never
  `tokio_test::block_on()` or `reqwest::blocking::Client` (tokio runtime conflicts).
  **Read `tests/CLAUDE.md` before writing, running, or debugging integration tests.**
- Run `cargo fmt` before commits; follow Rust naming; doc-comment public APIs.

## Development environment
- Bash shell, even on Windows.
- Cargo commands can exceed 2 minutes: run them without a timeout.
- Docker release image builds take 6+ minutes: use a timeout of at least 600000 ms.
  Debug builds take ~2-3 minutes.
- No database needed to build (SQLx queries are runtime-checked) or for CLI mode.
- Rust 1.96.1, edition 2024, pinned in `rust-toolchain.toml` — the single source of truth
  for host, CI, and the Dockerfile base image tag; bump all three together.

## Commands
`make help` lists everything. Non-obvious ones:

| Command | What it does |
|---|---|
| `cargo run -- --min 1 --max 100 --limit 10` | CLI game (`make run-cli` for defaults) |
| `make dev` | Full stack in Docker (postgres, keycloak, redis, oauth2-proxy, app) at http://localhost:8080; login `admin@local.test` / `password` |
| `make dev-db` | Postgres only, for running the app locally (below) |
| `make dev-down` | Stop dev services |
| `make build` | Builds the **Docker image** (debug) — not a cargo build |
| `make release` / `make docker-rebuild` | Release Docker image / release with no cache |
| `make test-unit` | `cargo test --lib`, no Docker |
| `make test-func` | Light-tier integration tests — the default choice |
| `make test-auth` | Full-stack auth + browser tests (Keycloak + Selenium, heavy) |
| `make test-integration` | Light tier, teardown, then full tier |
| `make test-func-down` / `make test-down` | Stop light tier / stop both tiers |
| `make fmt` / `make lint` | rustfmt / `cargo clippy -- -D warnings` |
| `make dc-up` / `make dc-attach` / `make dc-down` | Devcontainer |
| `make db-shell` | psql shell |

Run the server locally against `make dev-db` (there is no `.env`; `DATABASE_URL` is required):
`DATABASE_URL=postgresql://numberguess:password@localhost:5432/numberguess_dev cargo run -- --server --port 4080`

Compose builds the app image in debug mode by default; `BUILD_TYPE=release make dev` overrides.

## Architecture
**Auth proxy pattern.** oauth2-proxy (localhost:8080) is the only external entry point;
Keycloak (localhost:8090) is the OIDC provider; Redis holds sessions. The app listens on
4080 (internal only) with a health endpoint on 8081. Every web route requires
authentication; the app trusts the `X-Forwarded-*` identity headers oauth2-proxy adds.
Details: `.claude/rules/auth.md`.

**Layers** (`src/`):
- `core/` — pure logic, no I/O: `GuessingGame` (`game.rs`), `GuessResult` / `GameError`
  (`errors.rs`), `GameId` newtype, `validators.rs` (single source of validation for CLI,
  API, and web), feature modules (`features/difficulty/`)
- `api/` — JSON REST handlers + types · `web/` — HTMX/Askama HTML handlers ·
  `cli/` — clap args, I/O helpers, game loop
- `db/` — `GameRepository` trait (native async fn in traits) + `PostgresGameRepository`
- `server/` — routing, `AppState<R: GameRepository>` · `auth/` — `AuthenticatedUser`
  extractor (`is_in_group()` for authorization)
- `main.rs` — mode selection (CLI vs server), database init

**Conventions:**
- Handlers are generic over `R: GameRepository` (static dispatch, no `dyn`).
- Errors are typed `thiserror` enums (`GameError`, `DbError`). Handlers return `Result` with
  `ApiError` / `WebError` (`src/api/error.rs`, `src/web/error.rs`), whose `IntoResponse`
  impls define the exact, documented HTTP mappings.
- HTML is Askama templates (compile-time checked); `templates/index.html` is the UI shell
  and holds all CSS.
- Logging: `tracing` macros for system events (stderr); `println!` only for user-facing CLI
  output. In server mode stdout carries only `READY`, printed once fully initialized —
  tests and process managers wait on it. Verbosity via `RUST_LOG`
  (e.g. `RUST_LOG=sqlx=warn,number_guessing_game=debug`).

## Constraints
- Range 0 to 1,000,000 inclusive; no negatives; max ≥ min; validation prevents integer overflow.
- Guess limit max 1000 (CLI), 100 (web/API).
- Games persist in PostgreSQL keyed by random u64 IDs and are deleted on completion.
  Abandoned games remain until the cleanup function (see `migrations/`) runs.
- Known gaps: no API rate limiting, no request size limits, HTMX loaded from a CDN —
  see `docs/security-todo.md`.

## Common tasks
**New feature:** core logic (`core/game.rs`) → validation (`core/validators.rs`) →
CLI (`cli/args.rs`, `io.rs`, `runner.rs`) → API (`api/types.rs`, `api/handlers/`) →
web (`web/types.rs`, `web/handlers/`, `web/templates.rs`, `templates/`) → tests →
docs (`README.md`, `docs/api.md`, `docs/requirements.md`).

**Changing game rules:** `GuessingGame::make_guess()` in `core/game.rs`. A new
`GuessResult` variant means updating every match on it in the API, web, and CLI layers.

## Where to look
Area-specific context loads automatically when you read or edit matching files:
- `tests/CLAUDE.md` — test tiers, networking, startup, troubleshooting (`tests/`)
- `.claude/rules/auth.md` — auth flow, headers, security (`src/auth`, `src/server`, `keycloak`, compose, `test-fixtures`)
- `.claude/rules/database.md` — schema, transactions, migrations, credentials (`src/db`, `migrations`, `src/main.rs`)
- `.claude/rules/web-ui.md` — Parlor theme and HTMX swap details (`templates`, `src/web`)
- `.claude/rules/docker.md` — container image, build types, compose files (`Dockerfile`, compose, toolchain, devcontainer)
- `.claude/rules/ci.md` — GitHub Actions triggers and design (`.github`)

Reference docs (read when relevant): `docs/architecture.md`, `docs/api.md` (REST spec),
`docs/testing-guide.md`, `docs/requirements.md`, `docs/security-todo.md`,
`docs/troubleshooting.md`, `docs/contributing.md`, `README.md`.
