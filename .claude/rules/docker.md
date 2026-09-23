---
paths:
  - "Dockerfile"
  - ".dockerignore"
  - "docker-compose*.yml"
  - "rust-toolchain.toml"
  - ".devcontainer/**"
---
# Container and toolchain

- Runtime base image `gcr.io/distroless/cc-debian12`: no shell, no package manager,
  ~30MB (vs ~80MB Debian slim), 0 HIGH/CRITICAL vulnerabilities (Trivy). For a debug
  shell, switch to the `:debug` tag (busybox).
- Builder: `rust:<version>-slim` with cargo-chef 0.1.77 (pinned, `--locked`) for
  dependency caching. The Rust version must match `rust-toolchain.toml` and CI — bump
  together (see the Dockerfile comment).
- `BUILD_TYPE` build arg: compose defaults to `debug` (fast dev/test builds);
  `make release` / `make docker-rebuild` build release. Override: `BUILD_TYPE=release make dev`.
- Release builds take 6+ minutes (timeout ≥ 600000 ms); debug ~2-3 minutes.
- All stacks share the image `numberguess-claude-rust:latest`.
- Output is a single binary; binds 0.0.0.0; port set via `--port`.
- Compose files: `docker-compose.yml` (dev; profile `full-stack`),
  `docker-compose.integration.yml` (full test tier; profile `integration`),
  `docker-compose.test-mock-auth.yml` (light test tier; project `numberguess-mock`).
