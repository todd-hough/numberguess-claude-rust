---
paths:
  - "templates/**"
  - "src/web/**"
---
# Web UI ("Parlor" theme)

Design decisions: `plans/ui-refresh-parlor.md`.

- HTMX-driven updates without page reloads. `templates/index.html` is the shell and holds
  ALL CSS; light + dark themes via `prefers-color-scheme`.
- Askama templates are compile-time checked; structs in `src/web/templates.rs`, form types
  in `src/web/types.rs`. Templates compile into the binary — rebuild the app image before
  Docker-based tests.
- Remaining-guesses counter pill (`#counter`, `.counter-pill`): calm → `warn` at ≤3 →
  pulsing `crit` on the last guess. Shows "N left" plus a visually-hidden
  "Guesses remaining:" label; updated per guess via OOB swap.
- Guess history capsules (`#history`): each guess prepends a number+chevron capsule via
  `hx-swap-oob`; the newest renders filled. There is no server-side history.
- Range tracker (`#track-live`, `#track-marks`): display-only bounds round-trip through
  hidden `low` / `high` form fields and are sanitized server-side.
- Every form POST must carry the hidden `authenticity_token` field (`{{ csrf_token }}`);
  handlers reject missing/invalid tokens with 400. CSRF details: `.claude/rules/auth.md`.
- HTMX is loaded from a CDN (consider bundling for production).
- Debugging: check the browser console for HTMX errors; `curl` the API endpoints directly;
  server logs go to stderr.
