# Toolchain Version Upgrades (July 2026)

**Branch**: version-upgrades-july-2026 · **Status**: COMPLETE 2026-07-09 — full local validation green (58 unit + 27 integration / 2 ignored on both tiers, fresh image build); smoke test confirmed cargo-chef 0.1.77 installs on rust:1.96.1-slim

## Context

Follow-up to the fable-improvements branch, which pinned rotted tools (cargo-chef, trivy)
against the existing Rust 1.90 base. This branch does the deliberate, coordinated version
bump those pins anticipated (the Dockerfile comment says: "Bump the version together with
the base image, not independently"). (Correction discovered during implementation: a repo-root `rust-toolchain.toml` pins
1.90.0 and overrides everything — host rustup AND CI's dtolnay@stable — so CI was never
actually testing latest stable. The toolchain file is the true single source of truth.)

Targets (checked 2026-07-09): Rust **1.96.1** (from 1.90.0), cargo-chef **0.1.77**
(from pinned 0.1.72; 0.1.77 requires rustc ≥1.91 — the original rot trigger).

## Changes

1. **Dockerfile**: `FROM rust:1.90-slim` → `rust:1.96-slim` (verify tag exists first);
   cargo-chef pin `0.1.72` → `0.1.77 --locked` (smoke-test install on the new base
   before the full build). Keep the bump-together comment.
2. **Host toolchain**: `rustup update stable` → 1.96.x so local and container builds
   match. Then fmt/clippy/unit must stay clean (new clippy lints possible).
3. **CI toolchain parity**: no ci.yml change needed — `rust-toolchain.toml` (bumped to
   1.96.1) already governs CI via rustup override; quick checks were and remain
   deterministic. Base image tag now exact (`rust:1.96.1-slim`) to match it, avoiding a
   mid-build rustup download of a second toolchain.
4. **Lint fallout**: one new clippy lint on 1.96 (`useless_vec` in
   tests/integration_test.rs) — fixed (vec! → array).
5. **CLAUDE.md**: Version Information section updated; `rust-toolchain.toml` added to
   the file-structure inventory (it was undocumented).
6. **Dependency refresh (added by decision on the PR)**: `cargo update` — 240 packages
   bumped within semver. Clears ALL RUSTSEC advisories (cargo-audit exits 0, zero
   findings — was ~8 advisories incl. 2 high in quinn-proto). Full validation repeated
   on the updated tree: clippy clean, 58 unit, 27 integration / 2 ignored on both tiers,
   fresh image build.

## Verification

- Smoke: `docker run rust:1.96-slim cargo install cargo-chef --version 0.1.77 --locked`.
- Host: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --lib`
  on the updated toolchain.
- `make build` (image on new base), then full `make test` (unit + both tiers).
- Push; open PR (fires the full suite via the new PR-creation trigger); after this
  branch's merge, workflow_dispatch is available on main for re-runs.
