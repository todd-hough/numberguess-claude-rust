# Web UI Refresh — "Parlor" Theme

Approved through mockup review (July 2026). Mockups:
- Full screen set: https://claude.ai/code/artifact/8b5003a9-0297-4dcd-8d6d-7bc36f639e2a
- Theme options: https://claude.ai/code/artifact/7d91d1e4-1826-4b36-8730-550943e21073
- Final Parlor light/dark + chevron capsules: https://claude.ai/code/artifact/8ac8a97a-4316-4b1d-9dee-724ce0b14388

## Scope

Presentation-layer only. **No REST API changes, no DB schema changes.** One new
optional pair of form fields on the web guess form (display-state round-trip,
see below) — the web form is not a documented external API and the fields are
optional, so old clients keep working.

## Approved design (locked decisions)

- **Theme "Parlor"**: 1960s game-box print. One accent carries the UI; tomato is
  the second ink for direction; green/amber/red reserved for win/urgency/error.
- **Light tokens**: ground `#F3EBDB`, surface `#FFFBF2`, ink `#2E2A22`,
  muted `#86795F`, line `#E4D8BE`, accent teal `#0F8A7B` (ink `#FFFDF6`,
  soft `#DFEEE7`), tomato `#D04F32` (ink `#FFF6EF`), win `#588F2E`
  (ink `#F6FBEE`), warn `#C8930F`, danger = tomato family.
- **Dark tokens** (via `prefers-color-scheme: dark`): ground `#211A10`,
  surface `#2E2517`, ink `#F1E8D6`, muted `#B3A184`, line `#463A24`,
  accent **delft blue `#7B9DCB`** (ink `#121D2C`, soft `#2B3240`);
  tomato and win keep daylight values `#D04F32` / `#588F2E` with cream digits.
- **Play card**: header (range + guesses pill), range tracker (eliminated
  ranges shaded, live window in accent, one mark per past guess), spacing,
  input + Guess button, hint line under the input, then ONE capsule row —
  current guess as a filled tomato capsule (number + drawn SVG chevron),
  previous guesses as outlined capsules to its right, newest first.
- **Chevrons**: inline SVG (stroke 2.6, round caps); hidden "too high/too low"
  text kept for screen readers.
- **Counter pill**: "N left" with spent/remaining dots (dots only when limit
  ≤ 10); escalates calm → amber (≤3) → pulsing red (last guess);
  pulse respects `prefers-reduced-motion`.
- **Win screen**: number in green halo, "found in X — optimal is Y" using
  `calculate_optimal_guesses`, Start New Game button.
- **Errors**: own `error` class (red tint, ✕) — real errors no longer reuse
  the `too-high` gameplay class.
- **Difficulty ramp recolor** (`DifficultyLevel::color()`): Unlimited `#8A7F6A`,
  VeryEasy `#3F9464`, Easy `#7FA344`, Medium `#C8930F`, Hard `#D9772E`,
  Expert `#C64545`, Impossible `#4A4238` (charcoal — no longer collides with
  any accent).
- **Setup screen**: same two-column layout; solid accent button; small-caps
  labels with range hints under the fields; CSS bullseye mark instead of 🎯;
  visible focus rings; difficulty preview `load` trigger fires once (on #min
  only) instead of three times.
- **Timer: out of scope** (parked; never implemented — plans/feature-ideas.md:91).

## Mechanism: guess history + tracker without server-side history

The server stores no per-guess history and this plan does not add any.

1. **Capsule row** — each guess response includes an out-of-band fragment
   `hx-swap-oob="afterbegin:#history"` that prepends the just-made guess as a
   capsule. CSS styles `#history .cap:first-child` as the filled "current"
   capsule; older siblings render outlined. History therefore accumulates in
   the DOM across HTMX swaps with zero storage.
2. **Tracker** — the current display window `[low, high]` rides in hidden form
   fields and round-trips through the guess form. The handler sanitizes them
   (clamp to the game's real range; reset to full range when missing/invalid —
   tampering only affects cosmetics, real validation is untouched), narrows by
   the guess result, and re-renders. Per guess the response OOB-replaces
   `#track-live` (the accent window segment) and OOB-prepends one positioned
   mark into `#track-marks`.
3. **Counter pill** — OOB-replaced per guess (emitted only when a limit exists).

## Test-compat contract (hooks preserved)

- `#feedback.active` plus one of `.too-low` / `.too-high` / `.correct`
  (hint line and win block carry these) — used by `tests/common/page_objects.rs`.
- Error templates keep `#feedback.active` but switch modifier to `.error`
  (page objects map "active but not correct/high/low" to `FeedbackType::Error`;
  `wait_for_feedback` only needs `.active`).
- `.guess-form`, `input[name='guess']`, `.guess-form button`,
  `button[type='submit']`, `#min` / `#max` / `#max_guesses`, `.new-game-btn`.
- Literal text `Guesses remaining:` stays in the DOM as visually-hidden text
  inside the counter pill (`tests/web_endpoints_test.rs` asserts presence with
  a limit and absence without).

## File-by-file

| File | Change |
|------|--------|
| `templates/index.html` | New token CSS (light + dark), setup restyle, single `load` trigger |
| `templates/game_started.html` | New play card: header/pill, tracker, form w/ hidden bounds, empty `#history` |
| `templates/guess_form.html` | Swap fragment (form + hint) + OOB capsule/track-live/mark/counter |
| `templates/game_complete.html` | Win halo / loss block + OOB win capsule + counter cleanup |
| `templates/error.html`, `game_not_found.html`, `update_error.html` | `.error` styling, actionable copy |
| `templates/difficulty_indicator.html` | Unchanged structure; restyled via CSS |
| `src/web/templates.rs` | New fields on `GuessFormTemplate` / `GameCompleteTemplate` |
| `src/web/types.rs` | `MakeGuessRequest`: optional `low` / `high` |
| `src/web/handlers/guess.rs` | Bounds sanitize+narrow, percentages, counter class, optimal count, new messages |
| `src/core/features/difficulty/types.rs` | `color()` ramp recolor + update pinned color unit test |

## Dependency order (each step compiles + unit tests pass)

1. Difficulty ramp recolor + its unit test (isolated, proves harness).
2. Template structs + types + handler logic (server side complete, old
   templates still render — fields unused until templates change).
3. Templates: `game_started` → `guess_form` → `game_complete` → errors.
4. `index.html` CSS/token system + setup restyle (pure presentation).
5. Test updates if any assertion drifted; `cargo test --lib`,
   `make fmt`, `make lint`, `make test-func` (light tier).
   Full tier (`make test-auth`) left for CI per resource constraints; the
   Selenium hooks above are preserved by design.

## Verification

- `cargo test --lib` — unit tests including recolor.
- `make test-func` — light-tier integration: web endpoints, CSRF, API edge
  cases against the new templates.
- Manual smoke via `make dev-db` + `cargo run -- --server` if needed.

## Play-testing revision (2026-07-11)

After play testing, the guess-history capsules were judged unhelpful and the
tracker promoted to the primary feedback surface:

- **Capsule row removed** (`#history`, `.cap`, chevron SVGs, OOB prepends).
- **Bound labels on the tracker** (`#track-labels`, OOB-replaced per guess):
  min/max render emphasized at the bar ends at game start; once a side
  narrows, the closest high/low guess renders emphasized at its position on
  the bar and that side's end label de-emphasizes. Sides switch independently.
  Labels show the numbers ACTUALLY GUESSED (inclusive), not the ±1
  mathematical window — deliberately, players recognize their own guesses.
  When both labels exist and sit closer than 14 percentage points they are
  spread around their midpoint (and kept ≥4pts off the bar ends) so the
  numbers never overlap; spread applies only when both bounds exist.
- **Win magnifier**: the halo is width-proportional (30%, flanks at 35%/65%)
  and pulled up by its own radius (`margin-top: -15%`, % resolves against
  container width) so the `viewBox 0 0 100 24` SVG cone's bottom edge sits at
  circle-center height — the dashed lines land exactly on the circle's SIDES.
  The halo's fill ends opaque (`var(--surface)`) and it carries
  `position: relative`, so line segments crossing the circle paint behind it.
  A green `.mark-win` lands on the bar via OOB.
- `range_pct` centers (50%) on a degenerate single-number range.

## Cleanup

Ask the user about deleting this document when the work ships.
