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

✅ **Gate PASSED 2026-07-08**: precursor implemented and verified — `make test-integration`
(both tiers) green with 27 passed / 0 failed / 2 ignored, exactly matching the baseline.
Quality Phases 1-9 are cleared to begin, using `make test-func` for interim checkpoints.

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

### Phase 1: Bug fix — difficulty preview overflow (B1) ✅ DONE 2026-07-08

- [x] In `difficulty_preview`, replaced the ad-hoc range check with `core::validators::validate_range(min, max)` (returns empty `Html` on any validation error, preserving the "silent while typing" behavior).
- [x] Defense in depth: `calculate_optimal_guesses` and `calculate_difficulty` now widen to `i64` for `max - min + 1` (types.rs clamps range_size to u32::MAX for degenerate input).
- [x] Unit tests added: extreme-input no-overflow tests in calculator.rs and types.rs; handler-level tests (new `#[cfg(test)]` mod in `src/web/handlers/difficulty.rs`) assert empty response for `max=i32::MAX`, `max=1_000_001`, negative min, and inverted range, plus non-empty for valid input.
- [x] Verified: `cargo test --lib` 56 passed (was 51); clippy clean; light tier green on rebuilt image (20 passed / 2 ignored); end-to-end curl through the mock proxy confirms `?min=0&max=2147483647` → 200 empty (previously debug-panic/500) and valid input still renders.
- [x] Bonus: Q13 (difficulty template render errors silently swallowed) fixed in the same edit — render failures now log via `error!` before returning empty. Strike Q13 from Phase 2.

### Phase 2: Mechanical cleanups (Q10, Q11, Q13, Q12, Q5a) ✅ DONE 2026-07-08

- [x] Q10: `cargo clippy --fix -- -W clippy::uninlined_format_args -W clippy::redundant_closure` applied across 18 files (106 insertions / 133 deletions).
- [x] Q11: duplicated comment lines deleted in `src/auth/mod.rs`.
- [x] Q13: done early in Phase 1 (render errors logged in `difficulty_preview`).
- [x] Q5a: `deserialize_option_u32` moved to new `src/serde_helpers.rs`; `api/types.rs` and `web/types.rs` import it (no api↔web dependency).
- [x] Q12: pool passed by move into `PostgresGameRepository::new` in `run_server`.

**Q12 verification (all passed 2026-07-08):**
- [x] `make dev-db` + local `cargo run -- --server --port 4080`: DB connection established, migrations completed, `READY` emitted on stdout.
- [x] `http://localhost:8081/health` → 200 (SELECT 1 through the moved pool); game creation INSERT via direct request with auth headers also verified.
- [x] Full two-tier suite on rebuilt image: light 20 passed / 2 ignored, full 5 + 2 passed — **27 passed / 0 failed / 2 ignored, matches baseline**.
- [x] Move-vs-clone equivalence proven empirically, not assumed.

**Unplanned fix discovered during Q12 verification — dev/test postgres volume collision:**
The full test tier shared dev's `postgres_data` volume; `POSTGRES_DB` only takes effect when
postgres initializes an EMPTY data dir, so a `make dev-db` session left a volume without
`numberguess_test` and the next full-tier run crashed the app ("database numberguess_test
does not exist"). Fixed by giving the integration overlay a dedicated `postgres_test_data`
volume (compose merges service volumes by container target path, so it cleanly replaces the
dev mount). Regression-tested by deliberately reproducing the contamination sequence
(`make dev-db` → `make dev-down` → `make test-auth`): previously crashed, now 7/7 pass.
The light tier was already immune (anonymous volume, separate project).

### Phase 3: Getter renames (Q2) ✅ DONE 2026-07-08

- [x] Renamed `get_range()` → `range()`, `get_guess_count()` → `guess_count()`, `get_max_guesses()` → `max_guesses()` on `GuessingGame`; 27 replacements across core, db, web, cli, and docs/testing-guide.md.
- [x] Verified: zero stragglers by grep, clippy clean (-D warnings), 56/56 unit tests.

### Phase 4: Error-type polish (Q7) ✅ DONE 2026-07-08

- [x] Added `#[source]` to `DbError::DatabaseError` (not `#[from]` — the custom `From` maps `RowNotFound` specially), restoring `Error::source()` chains.
- [x] Added `GameError::GuessLimitExceedsMax { value, limit }`; `validate_guess_limit` uses it; `ValidationError(String)` removed (it had exactly one constructor site).
- [x] **Wire-format check passed**: the new variant's Display includes the old wrapper's `"Validation error: "` prefix, so responses are byte-identical — verified by live curl (`400` + `{"error":"Validation error: Guess limit (101) exceeds maximum allowed (100)"}`), a unit test pinning the exact Display text, and a green light-tier run (20 passed / 2 ignored). Invalid-range path sanity-checked unchanged.

### Phase 5: Typed guess result in API response (Q3) ✅ DONE 2026-07-08

- [x] Replaced `result: String` with `result: GuessOutcome` enum (`#[serde(rename_all = "snake_case")]`); handler match arms set variants; tracing log fields keep plain strings.
- [x] **Wire-format check passed**: captured live JSON for all four outcomes (too_low, too_high, correct, limit_reached) before and after against the mock-proxy stack — `diff` empty, byte-identical. Serialization also pinned by a unit test asserting each variant's exact legacy string. No docs/api.md change needed.
- [x] Light-tier suite green (20 passed / 2 ignored; API-only change, browser tier not affected per test strategy table). 58/58 unit tests, clippy clean.

### Phase 6: `ApiError` / handler error refactor (Q1) ✅ DONE 2026-07-08

- [x] `src/api/error.rs`: `ApiError { Validation, GameNotFound, Internal }` implements `IntoResponse` with the exact legacy mapping (400/404/500 + `{"error": ...}`), documented in a table in the module docs. `From<GameError>` provided; `ApiError::from_db_for_game(game_id)` maps `DbError::NotFound` → 404 for that game (a plain `From<DbError>` can't know the id).
- [x] `create_game_api` / `make_guess_api` rewritten to `Result<Json<...>, ApiError>` with `?`; all tracing events kept at call sites (they carry request-context fields the error type can't know).
- [x] `src/web/error.rs`: `WebError { ErrorMessage, GameNotFound, UpdateFailed, InvalidCsrf }` renders the same templates/status as before; both web handlers now return `Result<_, WebError>`.
- [x] Shared `validators::validate_new_game_params` (range + guess limit in one call) used by both creation handlers — the validation sequence can no longer drift between API and web.
- [x] **Wire-format check passed**: before/after captures of SIX error paths (API: invalid range 400, invalid limit 400, unknown game 404; WEB: invalid-range ErrorTemplate, unknown-game GameNotFoundTemplate, bad-CSRF 400) — `diff` empty, byte-identical.
- [x] Full two-tier suite green: **27 passed / 0 failed / 2 ignored, matches baseline**. 58/58 unit tests, clippy clean.

### Phase 7: Web guess handler cleanup (Q4, Q14) ✅ DONE 2026-07-08

- [x] Q4: four-arm match with a `render_guess_form` helper for the ongoing-game arms — both `unreachable!()`s deleted; each `GuessResult` variant handled exactly once.
- [x] Q14: `GameRepository::make_guess` now returns `(GuessResult, GuessingGame)` captured inside the transaction; the racy follow-up `repo.get()` in the web handler is gone (one fewer query per ongoing guess, and the trait docs now state the no-follow-up-fetch contract). API handler destructures and ignores the state.
- [x] Verified: too_low/too_high HTML fragments (with remaining-guesses counter) byte-identical before/after (normalized for game id + CSRF token); full two-tier suite green including browser `web_ui_test` — **27 passed / 0 failed / 2 ignored, matches baseline**; 58/58 unit tests; clippy clean.

### Phase 8: Remaining polish (Q8, Q9, Q15, Q5b, Q5c, Q6) ✅ DONE 2026-07-08

- [x] Q8: `Default for GameId`, `from_i64`, `as_i64` removed (call sites use `From`/`i64::from`); `#[allow(clippy::new_without_default)]` with a comment explaining the deliberate absence of a randomized Default.
- [x] Q9: `main() -> anyhow::Result<()>` with `.context(...)` on env/connect/migrations; `run_server` returns `anyhow::Result<()>` (binds use `with_context`); `CSRF_SECRET` shorter than 64 bytes now fails with a clear message via `anyhow::ensure!` instead of a library panic. Verified: missing `DATABASE_URL` prints `Error: DATABASE_URL must be set in environment or .env file` (no panic/backtrace).
- [x] Q15: dead `update`/`delete` deleted from `GameRepository` trait + impl (no callers, re-verified by grep).
- [x] Q5b: `game_from_row` helper deduplicates the row-parsing between `get` and `make_guess`.
- [x] Q5c: `prompt_min_value` restructured with a `prompt_valid_min` helper (mirrors `prompt_valid_max`); CLI output strings unchanged.
- [x] Q6: all `.map_err(|e| { error!(...); e })` scaffolding replaced with `.inspect_err(...)`; `#[tracing::instrument(skip(self))]` on the four repository methods (arg fields now carried by spans; redundant per-call fields pruned from log lines).
- [x] Verified: clippy clean (-D warnings), 58/58 unit tests, light tier green on rebuilt image (20 passed / 2 ignored; includes the 6 CLI tests covering the prompt refactor).

### Phase 9: Documentation sync (Q16) ✅ DONE 2026-07-08

- [x] CLAUDE.md updated: error-handling pattern now describes `thiserror` enums + `ApiError`/`WebError`; stale "games remain in memory / no persistent storage" known issues replaced with the accurate cleanup-function note; Performance Considerations rewritten for the PostgreSQL layer (primary-key lookup, FOR UPDATE transaction, PgPool); db/mod.rs description no longer mentions deprecated standalone functions.
- [x] `docs/api.md` confirmed accurate against the live wire captures from Phases 4-6 (result strings, 400/404 error bodies) — no changes needed.
- [x] **FINAL SIGN-OFF: `make test` fully green 2026-07-08** — 58 unit + light tier (20 passed / 2 ignored) + full tier (7 passed) = 27 integration passed / 0 failed / 2 ignored, matching the Phase 0 baseline exactly.
- [x] Asked user about plan cleanup (2026-07-08): **keep both plan documents as-is** as project history.

---

## PLAN COMPLETE (2026-07-08)

All phases 0-9 implemented and verified. Every finding from the original review is resolved
except items intentionally deferred to their own future work: none. External API verified
unchanged at every step (byte-identical wire captures for guess outcomes and all error paths).

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
