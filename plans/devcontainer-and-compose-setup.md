# Devcontainer and Docker Compose Alignment Plan

1. **Inspect Current Tooling**  
   Review the existing `docker-compose.yml`, `Makefile`, integration tests, and database helpers to document service definitions, environment variables, and any existing DB reset logic.

2. **Design Devcontainer Stack**  
   Create `.devcontainer/devcontainer.json` referencing the base compose file plus a new `docker-compose.devcontainer.yml`. Configure Docker-in-Docker (privileged with socket mount) and ensure Rust tooling, cargo helpers, and recommended extensions install via `postCreateCommand`.

3. **Database Reset Mechanism**  
   Add a reusable database reset script or helper (e.g., `scripts/reset-db.sh`) that drops/recreates the test database and reapplies migrations based on shared environment variables.

4. **Compose Profiles for Tests**  
   Introduce compose overrides/profiles for integration tests: one for standard tests (database only) and another that starts Selenium for web UI tests. Ensure shared `.env` configuration so CI and local runs use identical settings.

5. **Makefile Enhancements**  
   Add targets to launch the devcontainer, handle compose up/down workflows, and execute integration suites (`test-compose`, `test-compose-ui`) with automatic DB resets and robust cleanup on failure.

6. **Validation**  
   Inside the devcontainer, verify `cargo build`, `cargo test --lib`, `docker compose build` (or equivalent image build), and both compose-backed integration test targets complete successfully.

7. **Documentation Updates**  
   Update `AGENTS.md` (and related docs) with instructions for launching the devcontainer, running the new make targets, and understanding the compose profiles and database reset workflow.
