---
paths:
  - ".github/**"
---
# GitHub Actions

**`ci.yml`** — fast feedback (~1-2 min), no Docker: `cargo fmt` check, clippy, unit tests
(`cargo test --lib`). Runs on every push to every branch (push checks also show on PRs).

**`integration-security.yml`** — full validation (~4-5 min):
- Builds the Docker image once (debug) and shares it with later jobs as an artifact
  (saves ~3-4 min vs separate builds).
- Jobs that don't need the image (cargo-audit, hadolint Dockerfile lint) run in parallel
  with the build.
- Then in parallel: Trivy container scan and integration tests (both tiers, full auth
  stack + Selenium).
- Docker cache is pruned before loading the artifact to avoid stale images in Trivy scans.
- Trivy: a table-format scan enforces failure on HIGH/CRITICAL; a SARIF scan feeds the
  GitHub Security tab (works around a trivy-action SARIF exit-code bug).

Triggers for `integration-security.yml` (deliberate — hobby project):
- PR opened or reopened against main (the ready-to-merge signal)
- Push to main / develop (post-merge safety net)
- Manual dispatch, on any branch
- NOT on every PR push: after pushing more commits to an open PR, re-run via manual
  dispatch (or close/reopen the PR) when ready to merge.
- NOT on a schedule: the old weekly cron produced failing-scan emails and got the workflow
  auto-disabled during repo inactivity.
