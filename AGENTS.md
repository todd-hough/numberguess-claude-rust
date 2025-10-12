# Repository Guidelines

## Project Structure & Module Organization
- Core gameplay logic lives in `src/`, with `main.rs` routing between `cli.rs` and `web.rs`. Database orchestration is centered in `db.rs`, `validators.rs`, and SQL helpers.
- Browser templates are in `templates/`, static assets in `static/`, and SQL migrations in `migrations/`. Planning notes sit in `plans/`, while longer-form docs are under `docs/`.
- Integration fixtures reside in `tests/common`; suites such as `web_ui_test.rs` and `api_edge_cases_test.rs` drive end-to-end coverage.

## Build, Test, and Development Commands
- `make devcontainer-up` bootstraps the VS Code/CLI devcontainer (DinD enabled) so everyone shares the same toolchain.
- `make compose-up`/`make compose-down` start or stop the integration stack (Postgres + app) via Docker Compose; `make dev` still launches the full-stack profile for manual UI checks.
- `make test-compose` runs all API integration tests against the compose stack; `make test-compose-ui` adds Selenium for browser coverage.
- `make build` performs a debug build; `make run-cli` and `make run-server` remain the fastest ways to run the CLI or HTTP endpoints locally.

## Coding Style & Naming Conventions
- Stick to Rust’s default 4-space indentation, snake_case modules, and descriptive enum/struct names for game state.
- Always run `make fmt` (rustfmt) and `make lint` (clippy with `-D warnings`) before pushing; resolve or justify every diagnostic.
- Tests follow the `_test.rs` suffix; share helpers from `tests/common` using `pub(crate)` modules.

## Testing Guidelines
- `make test-unit` exercises library/CLI tests without containers; `make test-compose` resets the DB via `scripts/reset-db.sh`, then runs integration suites with Compose orchestration.
- `make test-compose-ui` extends the stack with Selenium (`selenium_remote_url` exposed) so `web_ui_test.rs` can drive the browser; prefer this over ad-hoc containers.
- Keep tests deterministic—seed random values, re-run when migrations change, and use the shared `.env` defaults so Compose + CI stay aligned.

## Commit & Pull Request Guidelines
- History favors short, present-tense commits (e.g. “fixed readiness signal”); mirror that voice and keep each change focused.
- Reference related issues in the body, call out database/config changes, and refresh README/docs when behavior shifts.
- PRs should describe user impact, list the commands you ran (e.g. `make test`), and attach screenshots or curl traces when touching the web UI.

## Security & Configuration Tips
- Copy `.env.example` to `.env` for local credentials; never commit populated env files.
- Use `make dev-down` before switching branches to avoid stale containers, and reset local databases with the migrations in `migrations/` when schemas drift.
- Pass secret overrides via environment variables or a local `.env`; avoid hardcoding credentials in source or tests.
