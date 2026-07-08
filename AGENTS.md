# Repository Guidelines

## Project Structure & Module Organization
- Core gameplay logic lives in `src/`, with `main.rs` routing between `cli.rs` and `web.rs`. Database orchestration is centered in `db.rs`, `validators.rs`, and SQL helpers.
- Browser templates are in `templates/`, static assets in `static/`, and SQL migrations in `migrations/`. Planning notes sit in `plans/`, while longer-form docs are under `docs/`.
- Integration fixtures reside in `tests/common`; suites such as `web_ui_test.rs` and `api_edge_cases_test.rs` drive end-to-end coverage.

## Build, Test, and Development Commands
- `make devcontainer-up` bootstraps the VS Code/CLI devcontainer (DinD enabled) so everyone shares the same toolchain.
- `make compose-up`/`make compose-down` start or stop the integration stack (Postgres + app) via Docker Compose; `make dev` still launches the full-stack profile for manual UI checks.
- `make test-func` runs functional integration tests on the light mock-auth tier (no Keycloak/Selenium); `make test-auth` runs auth + browser tests on the full stack; `make test-integration` runs both tiers in sequence.
- `make build` performs a debug build; `make run-cli` and `make run-server` remain the fastest ways to run the CLI or HTTP endpoints locally.

## Coding Style & Naming Conventions
- Stick to Rust’s default 4-space indentation, snake_case modules, and descriptive enum/struct names for game state.
- Always run `make fmt` (rustfmt) and `make lint` (clippy with `-D warnings`) before pushing; resolve or justify every diagnostic.
- Tests follow the `_test.rs` suffix; share helpers from `tests/common` using `pub(crate)` modules.

## Testing Guidelines
- `make test-unit` exercises library tests without containers; `make test-func` runs functional suites against the light mock-auth compose tier, and `make test-auth` runs `auth_integration_test.rs`/`web_ui_test.rs` against the full Keycloak + Selenium stack.
- Every `tests/*_test.rs` binary must be assigned to a tier via `FUNC_TESTS`/`AUTH_TESTS` in the Makefile — `make test-tier-check` (run automatically by both tier targets) fails otherwise; prefer the light tier unless the test asserts on the real auth stack or needs a browser.
- Tear down with `make test-func-down` (light) or `make test-down` (both tiers).
- Keep tests deterministic—seed random values, re-run when migrations change, and use the shared `.env` defaults so Compose + CI stay aligned.

## Commit & Pull Request Guidelines
- History favors short, present-tense commits (e.g. “fixed readiness signal”); mirror that voice and keep each change focused.
- Reference related issues in the body, call out database/config changes, and refresh README/docs when behavior shifts.
- PRs should describe user impact, list the commands you ran (e.g. `make test`), and attach screenshots or curl traces when touching the web UI.

## Security & Configuration Tips
- Copy `.env.example` to `.env` for local credentials; never commit populated env files.
- Use `make dev-down` before switching branches to avoid stale containers, and reset local databases with the migrations in `migrations/` when schemas drift.
- Pass secret overrides via environment variables or a local `.env`; avoid hardcoding credentials in source or tests.
