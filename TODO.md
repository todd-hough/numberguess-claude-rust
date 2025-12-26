# Project Quality TODOs

Remaining tasks from the initial project analysis and refactoring plan.

## Security
- [x] **CSRF Protection**: Implemented application-level CSRF protection for web UI forms using `axum_csrf`.
  - All POST endpoints require valid `authenticity_token` form field
  - Cookie-based tokens with configurable cookie name (`x-csrf-token`)
  - Handlers return token in response tuple to set cookies
  - Integration tests verify enforcement and token reuse within sessions
- [ ] **Rate Limiting**: Add rate limiting to game creation endpoints (`/api/games`, `/game/new`) to prevent resource exhaustion.

## Testing
- [ ] **Property-Based Testing**: Add `proptest` to verify core game logic across all possible ranges and guess limits.
- [ ] **Expanded CI Coverage**: Update GitHub Actions to run the full integration test suite using the existing docker-compose configuration.

## API & CLI Improvements
- [ ] **Structured API Errors**: Enhance `ErrorResponse` with machine-readable error codes (e.g., `ERR_INVALID_RANGE`) for programmatic client handling.
- [ ] **OpenAPI Specification**: Integrate `utoipa` to generate interactive Swagger/OpenAPI documentation.
- [ ] **CLI Robustness**: Replace remaining `.expect()` calls in `src/cli/runner.rs` with graceful error handling.

## Maintenance
- [ ] **Dependency Audit**: Add `cargo-deny` to CI to monitor for security advisories and license compliance.
