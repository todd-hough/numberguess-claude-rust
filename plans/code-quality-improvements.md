# Code Quality Improvements - Idiomatic Rust Review Remediation

**Created**: 2026-07-07
**Status**: Planned (not started)
**Source**: Full-codebase idiomatic Rust review (all modules read, `cargo clippy` default clean, pedantic/nursery produced 157 advisory warnings)

---

## Ground Rules

1. **Exercise ALL test automation BEFORE beginning any changes** (Phase 0). No code changes until we have a green baseline recorded. If the baseline is not green, stop and fix the environment/tests first — otherwise we cannot attribute failures to our changes.
2. **No external API changes without explicit approval.** Several items touch API-adjacent code (response types, error responses). Each such phase includes a wire-format verification step proving the JSON bytes and status codes are unchanged.
3. One phase at a time. Each phase ends with tests proving the change works before moving to the next (per project convention).
4. `cargo fmt` and `cargo clippy` must be clean after every phase.
5. Commit after each completed phase so any regression bisects to a single phase.

---

## Phase 0: Baseline — Run All Test Automation (REQUIRED FIRST)

Run and record results of every automated check before touching code:

- [ ] `make fmt` (should be a no-op; if not, note it)
- [ ] `make lint` — clippy, expect zero warnings
- [ ] `make test-unit` — unit tests, no Docker needed
- [ ] `make test-integration` — full stack (postgres, redis, keycloak, oauth2-proxy, selenium, app). Allow ~2-3 min startup. Environment stays up afterwards.
- [ ] Record pass/fail counts and total durations here as the baseline:

```
Baseline results (2026-07-07):
  fmt:               clean (cargo fmt --check)
  clippy:            clean with -D warnings (--all-targets)
  unit tests:        51 passed / 0 failed, 0.02s
  integration tests: ALL GREEN (exit 0) — ~27 passed, 0 failed, 2 ignored across 8 test binaries.
                     Notable durations: concurrency 32.9s, web_endpoints 20.6s, api/auth 18.7s,
                     csrf 11.0s, web_ui 9.7s, cli 0.3s.
```

Phase 0 gate: **PASSED** (2026-07-07). Baseline is green.

---

## Precursor: Integration Test Optimization (REQUIRED before Phase 1)

**Decided 2026-07-07:** before implementing any quality phases, execute the test-infrastructure
optimization in `plans/integration-test-optimization.md` (see its "2026-07-07 Refresh" section,
phases T1-T6). Rationale:

- This plan requires full integration runs at 5+ checkpoints; on the development laptop the
  current single-tier stack costs ~1.2 GiB (Keycloak + Selenium) and 3-4 minutes per run.
- After the precursor, interim checkpoints use the **light tier** (`make test-func`: postgres +
  app + nginx mock-auth, ≈70 MiB, seconds not minutes); the **full two-tier suite**
  (`make test-integration`) is reserved for the checkpoints marked below.
- The precursor is test-infrastructure-only (no app code changes), so it cannot mask or interact
  with the quality changes — and it must end with both tiers green against the same baseline
  counts recorded above (~27 passed / 2 ignored) before Phase 1 begins.

**Dependency gate: T5 verification green ⇒ quality Phases 1-9 may begin.**

- [ ] `make test-down` when done (or keep running if proceeding straight into Phase 1)

**Gate: do not proceed past this phase unless everything is green.**

---

## Findings Inventory

### Bug (correctness)

| # | Finding | Location |
|---|---------|----------|
| B1 | Integer overflow reachable from difficulty preview endpoint. Handler only checks `min < 0 \|\| max < 0 \|\| max < min` — never enforces `MAX_RANGE` via `validate_range`. `?min=0&max=2147483647` makes `(max - min + 1)` overflow i32: panic (500) in debug, wraparound garbage in release. | `src/web/handlers/difficulty.rs:32`, `src/core/features/difficulty/types.rs:200`, `src/core/features/difficulty/calculator.rs:24` |

### Code quality / idiomatic Rust

| # | Finding | Location |
|---|---------|----------|
| Q1 | Handlers hand-assemble `(StatusCode, Json<ErrorResponse>)` / template error responses at every failure point; 6 functions exceed cognitive-complexity 25 (worst 83). Should be an `ApiError` enum implementing `IntoResponse`, handlers use `?`. | `src/api/handlers/game.rs`, `src/api/handlers/guess.rs`, `src/web/handlers/game.rs`, `src/web/handlers/guess.rs` |
| Q2 | Non-idiomatic `get_` getter prefixes; struct internally inconsistent (`secret_number()` follows convention, `get_range()`/`get_guess_count()`/`get_max_guesses()` don't). | `src/core/game.rs:35-45` |
| Q3 | Stringly-typed `MakeGuessResponse.result: String` ("too_low" etc.). Should be enum with `#[serde(rename_all = "snake_case")]` — wire format stays byte-identical. | `src/api/types.rs:49-53` |
| Q4 | Double-match + two `unreachable!()` arms in web guess handler. | `src/web/handlers/guess.rs:70-122` |
| Q5 | Copy-paste duplication: (a) `deserialize_option_u32` identical in two files; (b) DB row extraction duplicated between `get` and `make_guess`; (c) `prompt_min_value` retry loop duplicated in both match arms. | `src/api/types.rs:10-20` + `src/web/types.rs:9-19`; `src/db/postgres_repository.rs:119-149` + `199-221`; `src/cli/io.rs:33-66` |
| Q6 | Log-and-return `.map_err(|e| { error!(...); e })` scaffolding throughout repository; idiomatic form is `.inspect_err(...)`, and `#[tracing::instrument]` can replace manual breadcrumb `debug!`s. | `src/db/postgres_repository.rs` (all methods) |
| Q7 | `DbError::DatabaseError(sqlx::Error)` missing `#[source]` (custom `From` prevents `#[from]`, but `#[source]` still works) — breaks `Error::source()` chains. `validate_guess_limit` uses catch-all `ValidationError(String)` instead of a typed variant. | `src/db/mod.rs:18`, `src/core/validators.rs:72` |
| Q8 | `Default for GameId` returns a random ID — violates convention that `Default` is canonical/predictable. `from_i64`/`as_i64` redundant with existing `From` impls. | `src/core/game_id.rs:22-36` |
| Q9 | `main` does `error!` then `panic!` with same message (duplicate output + backtrace for config errors). `run_server` panics internally on bind failure. Prefer `main() -> anyhow::Result<()>` / `run_server` returning `Result`. `Key::from` panics on short `CSRF_SECRET` without a clear message. | `src/main.rs:54-72`, `src/server/mod.rs:45-50, 103-109` |
| Q10 | 32 instances of `format!("{}", x)` instead of inline `format!("{x}")`; `Html("".to_string())` instead of `Html(String::new())`; a few redundant closures. All auto-fixable. | throughout |
| Q11 | Duplicated back-to-back comment lines (edit leftover). | `src/auth/mod.rs:101-104` |
| Q12 | **Pool passed by clone unnecessarily**: `run_server` receives `pool: PgPool`, calls `PostgresGameRepository::new(pool.clone())`, and never uses `pool` again. Should pass by move. **⚠ Needs specific verification — see Phase 2 test notes.** | `src/server/mod.rs:56` |
| Q13 | `difficulty_preview` swallows template render errors: `render().unwrap_or_default()`. Log before returning empty. | `src/web/handlers/difficulty.rs:58` |
| Q14 | Web guess handler makes a second `repo.get()` round-trip after `make_guess` just to show remaining guesses — extra query + small race window (game deleted by concurrent request between calls). Fix by having `make_guess` return post-guess state. | `src/web/handlers/guess.rs:84` |
| Q15 | `GameRepository::update` and `delete` have no callers outside the trait/impl (verified by grep over `src/` and `tests/`) — dead surface superseded by transactional `make_guess`. | `src/db/repository.rs:69-87` |
| Q16 | CLAUDE.md drift: still describes `Result<T, String>` error handling (code uses `thiserror`); lists "games remain in memory / no persistent storage" as known issues despite the PostgreSQL layer. | `CLAUDE.md` |

---

## Implementation Phases

Ordered lowest-risk-first within each tier; bug fix first overall.

### Phase 1: Bug fix — difficulty preview overflow (B1)

- [ ] In `difficulty_preview`, replace the ad-hoc range check with `core::validators::validate_range(min, max)` (returns empty `Html` on any validation error, preserving the "silent while typing" behavior).
- [ ] Defense in depth: make `calculate_optimal_guesses` and `calculate_difficulty` overflow-safe (`i64` widening or checked arithmetic for `max - min + 1`).
- [ ] Add unit tests: `calculate_difficulty(0, i32::MAX, ...)` does not panic; handler-level test that out-of-range params yield empty response.
- [ ] Run: `make test-unit`. Not an external API change: endpoint already returns empty body for invalid input; we only widen what counts as invalid to match documented range limits (0..=1,000,000).

### Phase 2: Mechanical cleanups (Q10, Q11, Q13, Q12, Q5a)

Small, independent, low-risk edits batched together:

- [ ] Q10: `cargo clippy --fix -- -W clippy::uninlined_format_args` + manual pass for `Html(String::new())` and redundant closures.
- [ ] Q11: delete duplicated comment lines in `src/auth/mod.rs`.
- [ ] Q13: log template render errors in `difficulty_preview` before returning empty.
- [ ] Q5a: move `deserialize_option_u32` to one shared location (e.g., a small `src/serde_helpers.rs` or into `web/types.rs` re-exported to api) — pick whichever avoids a web→api dependency.
- [ ] Q12: **pool ownership change** — remove `pool.clone()` in `run_server`; pass `pool` by move into `PostgresGameRepository::new`.

**⚠ Q12 specific test requirement (per review discussion):** the pool change must be explicitly proven to work, not just compile:
- [ ] `make dev-db` + `cargo run -- --server --port 4080`: confirm startup logs show DB connection, migrations run, and `READY` is emitted on stdout.
- [ ] Hit `http://localhost:8081/health` → expect 200 (exercises the pool through the repository).
- [ ] Run the **full integration suite** (`make test-integration`) — game create + guess flows exercise pooled connections under concurrency (`--test-threads=1` but multiple sequential connections); confirm counts match the Phase 0 baseline.
- [ ] Confirm no behavior change: `PgPool` is internally `Arc`-based, so move vs. clone is semantically identical — the tests above prove that assumption in this codebase rather than trusting it.

### Phase 3: Getter renames (Q2)

- [ ] Rename `get_range()` → `range()`, `get_guess_count()` → `guess_count()`, `get_max_guesses()` → `max_guesses()` on `GuessingGame`; update all call sites (core tests, db, web, cli).
- [ ] Internal-only change (library API, not REST). Run `make test-unit`; grep for stragglers.

### Phase 4: Error-type polish (Q7)

- [ ] Add `#[source]` to `DbError::DatabaseError`.
- [ ] Add `GameError::GuessLimitExceedsMax { value: u32, limit: u32 }`; use it in `validate_guess_limit`; remove `ValidationError(String)` if nothing else uses it.
- [ ] **Wire-format check**: error message text appears in API error responses. Keep the `Display` text identical (`"Guess limit ({}) exceeds maximum allowed ({})"`) so REST responses are unchanged. Verify with a curl against the running dev stack and by integration tests.

### Phase 5: Typed guess result in API response (Q3)

- [ ] Replace `result: String` with `result: GuessOutcome` enum, `#[serde(rename_all = "snake_case")]` → serializes to the exact same `"too_low"` / `"too_high"` / `"correct"` / `"limit_reached"` strings.
- [ ] **Wire-format check (required — this is external-API-adjacent)**: capture JSON responses for all four outcomes before and after (curl or integration test assertions) and diff — must be byte-identical. `docs/api.md` remains the reference; no doc change needed if identical.
- [ ] Run full integration suite.

### Phase 6: `ApiError` / handler error refactor (Q1) — largest change

- [ ] Define `ApiError` enum in `src/api/` implementing `IntoResponse`, with `From<DbError>` and `From<GameError>`. Must reproduce today's exact mapping: validation → 400 + `{"error": "<Display text>"}`, `NotFound` → 404 + `Game with ID {} not found`, other DB errors → 500 + `e.to_string()`.
- [ ] Rewrite `create_game_api` / `make_guess_api` to `Result<Json<...>, ApiError>` with `?`. Keep all existing `tracing` events (warn on validation failure, error on DB failure) — move them into `ApiError` constructors or keep at call sites.
- [ ] Analogous `WebError` (renders `ErrorTemplate` / `GameNotFoundTemplate` / `UpdateErrorTemplate`) for the web handlers.
- [ ] Extract the shared validate-range → validate-limit → `repo.create` sequence used by both API and web creation handlers into one helper.
- [ ] **External API constraint**: status codes and JSON error body shape must be unchanged. Verify against `docs/api.md` and with before/after curl captures for: invalid range (400), invalid limit (400), unknown game id (404).
- [ ] Run full test suite (`make test`).

### Phase 7: Web guess handler cleanup (Q4, Q14)

- [ ] Q4: restructure the match so each `GuessResult` variant is handled once; delete both `unreachable!()` arms.
- [ ] Q14: change `GameRepository::make_guess` to return post-guess state alongside the result (e.g., `(GuessResult, GameSnapshot)` or result struct carrying guess_count/max_guesses), computed inside the existing transaction. Remove the follow-up `repo.get()` in the web handler. This is an internal trait change (not REST); API handler ignores the extra data.
- [ ] Verify the "Guesses remaining: X" counter still renders correctly in the web UI (integration test `web_ui_test.rs` covers this; also manual check via `make dev`).

### Phase 8: Remaining polish (Q8, Q9, Q15, Q5b, Q5c, Q6)

- [ ] Q8: remove `Default for GameId` and `from_i64`/`as_i64` (keep `From` impls + `new()`); fix call sites.
- [ ] Q9: `main() -> anyhow::Result<()>` (new `anyhow` dep — acceptable for the binary), `run_server` returns `Result`, drop error!+panic duplication, add explicit length check + clear message for `CSRF_SECRET`.
- [ ] Q15: delete unused `update`/`delete` from `GameRepository` trait + impl (re-verify no callers first).
- [ ] Q5b: extract `game_from_row` helper (or `sqlx::FromRow` row struct) in postgres repository.
- [ ] Q5c: restructure `prompt_min_value` with a helper like `prompt_valid_max`.
- [ ] Q6: replace `.map_err(log; e)` with `.inspect_err(...)`; adopt `#[tracing::instrument(skip(self))]` on repository methods and prune redundant `debug!`s.
- [ ] Run full test suite.

### Phase 9: Documentation sync (Q16)

- [ ] Update CLAUDE.md: error-handling description (`thiserror` enums, not `Result<T, String>`), remove stale "in-memory / no persistence" known issues, fix "Performance Considerations" section (HashMap/no-DB claims predate the PostgreSQL layer).
- [ ] Confirm `docs/api.md` accurately documents current REST behavior (it is the behavioral reference for Phases 5-6).
- [ ] Ask user whether to clean up this plan document (per project convention).

---

## Test Strategy Summary

| Checkpoint | What runs |
|---|---|
| Phase 0 (gate) | fmt, clippy, unit, **full integration** — recorded baseline ✅ DONE |
| Precursor T1-T6 (gate) | test-tier split implemented; **both tiers green** vs baseline counts |
| After each quality phase | `cargo fmt`, `make lint`, `make test-unit`, `make test-func` (light tier — seconds, ~70 MiB) |
| After Phases 2 (pool change), 6, and final Phase 9 sign-off | **full two-tier suite** (`make test-integration`), compared against baseline counts |
| Phase 2 extra (Q12 pool) | manual server start + health endpoint + full suite, explicitly verifying DB connectivity through the moved pool |
| Phases 4-6 (API-adjacent) | before/after wire-format capture (curl via light tier) proving byte-identical responses |
| Phases 5, 7 | light tier + `test-auth` tier if web UI rendering affected (Phase 7 touches the guesses-remaining counter → run `web_ui_test` via full tier) |

## Risk Notes

- **Highest risk**: Phase 6 (error refactor) — mitigated by exact-mapping requirement and wire captures.
- **Explicitly flagged**: Phase 2 pool move — semantically a no-op (`PgPool` is `Arc`-based) but verified empirically per plan, not assumed.
- **Trait signature change**: Phase 7 `make_guess` return type — touches trait, impl, and both handlers; done in one phase with full integration run.
- All phases are independent enough to pause between them; committing per phase keeps bisection clean.
