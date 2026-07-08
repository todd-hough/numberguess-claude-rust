.PHONY: help build test test-unit test-up test-tier-check test-func test-func-down test-auth test-integration test-down dev dev-db dev-down dev-restart dev-logs dev-status \
        docker-rebuild docker-check docker-clean clean logs db-shell fmt lint run-cli run-server \
        devcontainer-up devcontainer-down devcontainer-attach devcontainer-restart devcontainer-status \
        dc-up dc-down dc-attach dc-shell dc-restart dc-status \
        compose-up compose-down \
        release status info quick check reset \
        security security-audit security-dockerfile security-scan

# Database configuration defaults (can be overridden by environment variables)
POSTGRES_USER ?= numberguess
POSTGRES_PASSWORD ?= password
POSTGRES_DB ?= numberguess_dev

# Resolve devcontainer CLI path (supports NVM installs)
DEVCONTAINER_BIN ?= $(shell command -v devcontainer 2>/dev/null)
ifeq ($(strip $(DEVCONTAINER_BIN)),)
DEVCONTAINER_BIN := $(shell find $(HOME)/.config/nvm/versions/node -maxdepth 5 \( -type f -o -type l \) -name devcontainer 2>/dev/null | head -n1)
endif
DEVCONTAINER_DIR := $(dir $(DEVCONTAINER_BIN))

# Default target
.DEFAULT_GOAL := help

## help: Show this help message
help:
	@echo "Number Guessing Game - Development Commands"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "DEVCONTAINER WORKFLOW (Primary)"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make dc-up         - Start devcontainer"
	@echo "  make dc-down       - Stop devcontainer"
	@echo "  make dc-attach     - Attach to devcontainer shell"
	@echo "  make dc-shell      - Attach to devcontainer shell (alias)"
	@echo "  make dc-restart    - Restart devcontainer"
	@echo "  make dc-status     - Show devcontainer status"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "BUILD & RELEASE"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make build         - Build Docker image (debug mode, fast)"
	@echo "  make release       - Build production Docker image (optimized)"
	@echo "  make docker-clean  - Clean Docker images and containers"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "TESTING"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make test          - Run all tests (unit + integration)"
	@echo "  make test-unit     - Unit tests only (fast, no Docker)"
	@echo "  make test-up       - Start integration test environment only"
	@echo "  make test-func     - Functional tests, light tier (mock auth, ~70 MiB, fast)"
	@echo "  make test-auth     - Auth + browser tests, full stack (Keycloak + Selenium)"
	@echo "  make test-integration - All integration tests (light tier, then full stack)"
	@echo "  make test-down     - Stop integration test environment"
	@echo ""
	@echo "  Note: test-integration keeps environment running for debugging"
	@echo "        Use test-down when finished to clean up resources"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "SECURITY"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make security           - Run all security checks"
	@echo "  make security-audit     - Audit Rust dependencies (cargo-audit)"
	@echo "  make security-dockerfile - Lint Dockerfile (Hadolint)"
	@echo "  make security-scan      - Scan container image (Trivy)"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "DEVELOPMENT (App + Database)"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make dev           - Start full stack (postgres + app)"
	@echo "  make dev-db        - Start postgres only (run app locally)"
	@echo "  make dev-down      - Stop development services"
	@echo "  make dev-restart   - Restart development services"
	@echo "  make dev-logs      - View development service logs"
	@echo "  make dev-status    - Show development service status"
	@echo "  make db-shell      - Open PostgreSQL shell"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "UTILITIES"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make status        - Show overall system status"
	@echo "  make info          - Display connection strings and URLs"
	@echo "  make quick         - Fast feedback (fmt + unit tests)"
	@echo "  make check         - Verify prerequisites"
	@echo "  make reset         - Stop everything, clean, start fresh"
	@echo "  make fmt           - Format code with rustfmt"
	@echo "  make lint          - Run clippy linter"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make run-cli       - Run CLI game"
	@echo "  make run-server    - Run web server (requires postgres)"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "For more details: https://github.com/your-repo/numberguess"
	@echo "════════════════════════════════════════════════════════════════"

## dev: Start postgres + app server for manual testing (full stack)
dev:
	@echo "Starting full development stack (postgres + app)..."
	docker compose --profile full-stack up -d
	@echo ""
	@echo "✓ Services started!"
	@echo "  Web UI: http://localhost:8080"
	@echo "  Health Check: http://localhost:8081/health"
	@echo "  Database: postgresql://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost:5432/$(POSTGRES_DB)"
	@echo ""
	@echo "View logs: make logs"
	@echo "Stop services: make dev-down"

## dev-db: Start only postgres (run app locally with cargo run)
dev-db:
	@echo "Starting postgres only..."
	docker compose up -d postgres
	@echo ""
	@echo "✓ PostgreSQL started!"
	@echo "  Connection: postgresql://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost:5432/$(POSTGRES_DB)"
	@echo ""
	@echo "Now run: cargo run -- --server"

## dev-down: Stop development services
dev-down:
	@echo "Stopping development services..."
	docker compose --profile full-stack down

## dev-restart: Restart development services
dev-restart:
	@echo "Restarting development services..."
	docker compose --profile full-stack restart
	@echo ""
	@echo "✓ Services restarted!"

## dev-logs: View development service logs
dev-logs:
	docker compose --profile full-stack logs -f

## dev-status: Show development service status
dev-status:
	@echo "Development service status:"
	@docker compose --profile full-stack ps 2>/dev/null || echo "No development services running"

## devcontainer-up: Launch devcontainer environment using devcontainer CLI
devcontainer-up:
	@if [ -z "$(strip $(DEVCONTAINER_BIN))" ]; then \
		echo "devcontainer CLI not found. Install from https://github.com/devcontainers/cli or set DEVCONTAINER_BIN"; \
		exit 1; \
	fi
	PATH="$(DEVCONTAINER_DIR):$$PATH" "$(DEVCONTAINER_BIN)" up --workspace-folder .

## devcontainer-down: Stop devcontainer (devcontainer CLI has no down command, uses docker directly)
devcontainer-down:
	@echo "Stopping devcontainer..."
	@docker stop $$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)") 2>/dev/null || echo "No running devcontainer found"
	@docker rm $$(docker ps -aq --filter "label=devcontainer.local_folder=$$(pwd)") 2>/dev/null || true

## devcontainer-attach: Attach terminal to running devcontainer
devcontainer-attach:
	@if [ -z "$(strip $(DEVCONTAINER_BIN))" ]; then \
		echo "devcontainer CLI not found. Install from https://github.com/devcontainers/cli or set DEVCONTAINER_BIN"; \
		exit 1; \
	fi
	@echo "Attaching to devcontainer..."
	PATH="$(DEVCONTAINER_DIR):$$PATH" "$(DEVCONTAINER_BIN)" exec --workspace-folder . bash

## Devcontainer shortcuts
dc-up: devcontainer-up
dc-down: devcontainer-down
dc-attach: devcontainer-attach
dc-shell: devcontainer-attach

## devcontainer-restart: Restart devcontainer
devcontainer-restart: devcontainer-down devcontainer-up

## dc-restart: Restart devcontainer (shortcut)
dc-restart: devcontainer-restart

## devcontainer-status: Show devcontainer status
devcontainer-status:
	@CONTAINER=$$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)"); \
	if [ -n "$$CONTAINER" ]; then \
		echo "✓ Devcontainer is running (container: $$CONTAINER)"; \
		docker ps --filter "label=devcontainer.local_folder=$$(pwd)" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"; \
	else \
		echo "✗ Devcontainer is not running"; \
	fi

## dc-status: Show devcontainer status (shortcut)
dc-status: devcontainer-status

# Test targets orchestrate stateful docker compose stacks that share host
# ports; running them in parallel (make -j) would race teardown against tests
# and collide both tiers on port 8080.
.NOTPARALLEL:

COMPOSE_STACK = docker compose -f docker-compose.yml -f docker-compose.integration.yml
# Light test tier: postgres + app + nginx mock-auth proxy (~70 MiB, no Keycloak/Selenium).
# The project name isolates it from the full-tier compose project. These two
# variables are the single owner of the light-tier coordinates: test-func
# exports them to the tests (tests/concurrency_test.rs reads them for its
# app-restart; the literals there are fallbacks only).
MOCK_COMPOSE_FILE = docker-compose.test-mock-auth.yml
MOCK_COMPOSE_PROJECT = numberguess-mock
COMPOSE_MOCK = docker compose -f $(MOCK_COMPOSE_FILE) -p $(MOCK_COMPOSE_PROJECT)
TEST_DB ?= numberguess_test

# Test tier membership — the single source of truth for which integration
# test binary runs in which tier. `make test-tier-check` fails if a
# tests/*_test.rs binary is not assigned to exactly one tier, so a new test
# file cannot silently run in no tier.
FUNC_TESTS = api_edge_cases_test web_endpoints_test concurrency_test csrf_test cli_test integration_test
AUTH_TESTS = auth_integration_test web_ui_test

## test-tier-check: Verify every tests/*_test.rs is assigned to a test tier
test-tier-check:
	@status=0; \
	for f in tests/*_test.rs; do \
		t=$$(basename $$f .rs); \
		func_hit=0; auth_hit=0; \
		for x in $(FUNC_TESTS); do [ "$$x" = "$$t" ] && func_hit=1; done; \
		for x in $(AUTH_TESTS); do [ "$$x" = "$$t" ] && auth_hit=1; done; \
		if [ $$((func_hit + auth_hit)) -eq 0 ]; then \
			echo "ERROR: tests/$$t.rs is not assigned to any test tier."; \
			echo "       Add it to FUNC_TESTS or AUTH_TESTS in the Makefile."; \
			status=1; \
		elif [ $$((func_hit + auth_hit)) -gt 1 ]; then \
			echo "ERROR: tests/$$t.rs is assigned to BOTH tiers (FUNC_TESTS and AUTH_TESTS)."; \
			status=1; \
		fi; \
	done; \
	for x in $(FUNC_TESTS) $(AUTH_TESTS); do \
		[ -f "tests/$$x.rs" ] || { echo "ERROR: tier lists reference missing tests/$$x.rs"; status=1; }; \
	done; \
	[ $$status -eq 0 ] && echo "✓ All test binaries assigned to exactly one tier" || exit 1

## compose-up: Start integration stack with postgres + app
compose-up:
	$(COMPOSE_STACK) --profile integration up -d postgres app

## compose-down: Stop integration stack and remove volumes
compose-down:
	$(COMPOSE_STACK) --profile integration down -v

## logs: View docker-compose logs
logs:
	docker compose --profile full-stack logs -f

## db-shell: Open PostgreSQL shell
db-shell:
	docker compose exec postgres psql -U $(POSTGRES_USER) -d $(POSTGRES_DB)

## test: Run complete test suite (unit + integration tests)
test: test-unit test-integration
	@echo ""
	@echo "✓ All tests completed successfully!"

## test-unit: Run unit tests only (no Docker needed)
test-unit:
	@echo "Running unit tests (no Docker required)..."
	cargo test --lib

## test-up: Start integration test environment without running tests
test-up: docker-check
	@echo "Starting integration test environment..."
	@echo "Services will remain running for debugging/inspection"
	@echo "Use 'make test-down' to stop when done"
	@echo ""
	$(COMPOSE_STACK) --profile integration up -d --wait --wait-timeout 240
	@echo ""
	@echo "✓ Integration test environment started!"
	@echo ""
	@echo "Services running:"
	@echo "  Web UI: http://localhost:8080 (via oauth2-proxy)"
	@echo "  Keycloak: http://localhost:8090"
	@echo "  Health Check: http://localhost:8081/health"
	@echo "  Selenium: http://localhost:4444"
	@echo ""
	@echo "View logs: make logs"
	@echo "Stop services: make test-down"

## test-func: Run functional integration tests on the LIGHT tier (mock auth, no Keycloak/Selenium)
test-func: docker-check test-tier-check
	@echo "Starting light test tier (postgres + app + mock-auth proxy)..."
	$(COMPOSE_MOCK) up -d --wait --wait-timeout 120
	@echo ""
	@echo "Running functional tests (mock auth)..."
	MOCK_AUTH=1 \
	MOCK_COMPOSE_FILE=$(MOCK_COMPOSE_FILE) \
	MOCK_COMPOSE_PROJECT=$(MOCK_COMPOSE_PROJECT) \
	GAME_SERVER_BASE_URL=http://localhost:8080 \
	cargo test $(addprefix --test ,$(FUNC_TESTS)) -- --test-threads=1
	@echo ""
	@echo "✓ Functional tests passed. Light tier still running (make test-func-down to stop)."

## test-func-down: Stop the light test tier
test-func-down:
	$(COMPOSE_MOCK) down -v
	@echo "✓ Light test tier stopped"

## test-auth: Run auth + browser UI tests on the FULL stack (Keycloak, oauth2-proxy, Redis, Selenium)
test-auth: docker-check test-tier-check
	@echo "Starting full auth stack (Keycloak takes ~60s)..."
	@echo "Note: Environment will remain running after tests for debugging"
	@echo "Use 'make test-down' to stop when done"
	@echo ""
	$(COMPOSE_STACK) --profile integration up -d --wait --wait-timeout 240
	@echo ""
	@echo "Running auth + browser UI tests..."
	GAME_SERVER_BASE_URL=http://localhost:8080 \
	GAME_SERVER_BROWSER_URL=http://oauth2-proxy:4180 \
	SELENIUM_REMOTE_URL=http://localhost:4444 \
	TEST_DB_NAME=$(TEST_DB) \
	cargo test $(addprefix --test ,$(AUTH_TESTS)) -- --test-threads=1

## test-integration: Run all integration tests (light tier first, then full auth stack)
## Tiers run sequentially with teardown between them so peak memory never
## includes both (light ~70 MiB; full ~1.2 GiB with Keycloak + Selenium).
test-integration: test-func test-func-down test-auth
	@echo ""
	@echo "✓ Both integration test tiers passed."

## test-down: Stop and remove all integration test services (both tiers)
test-down:
	@echo "Stopping integration test services..."
	$(COMPOSE_STACK) --profile integration down -v
	$(COMPOSE_MOCK) down -v
	@echo "✓ Integration test services stopped (both tiers)"

## run-cli: Run CLI game
run-cli:
	cargo run -- --min 1 --max 100

## run-server: Run web server (requires postgres running)
run-server:
	@echo "Starting web server (make sure postgres is running: make dev-db)..."
	cargo run -- --server --port 8080

## fmt: Format code with rustfmt
fmt:
	cargo fmt

## lint: Run clippy linter
lint:
	cargo clippy -- -D warnings

## build: Build Docker image (debug mode - faster for dev/test)
build:
	@echo "Building Docker image (debug mode)..."
	docker build --build-arg BUILD_TYPE=debug -t numberguess-claude-rust:latest .

## docker-rebuild: Force rebuild Docker image (no cache, release mode)
docker-rebuild:
	@echo "Rebuilding Docker image (no cache, release mode)..."
	docker build --no-cache -t numberguess-claude-rust:latest .

## docker-check: Check if Docker image exists, build if not
docker-check:
	@if ! docker images numberguess-claude-rust:latest | grep -q latest; then \
		echo "Docker image not found, building..."; \
		$(MAKE) build; \
	else \
		echo "✓ Docker image exists"; \
	fi

## release: Build optimized production Docker image
release:
	@echo "Building release Docker image (optimized, this will take several minutes)..."
	docker build -t numberguess-claude-rust:latest -t numberguess-claude-rust:release .
	@echo ""
	@echo "✓ Release image built successfully!"
	@echo "  Tags: numberguess-claude-rust:latest, numberguess-claude-rust:release"

## docker-clean: Clean Docker images and containers
docker-clean:
	@echo "Cleaning Docker resources..."
	@echo "Stopping all compose services..."
	-docker compose --profile full-stack down -v 2>/dev/null
	-$(COMPOSE_STACK) --profile integration down -v 2>/dev/null
	@echo "Removing numberguess images..."
	-docker rmi numberguess-claude-rust:latest numberguess-claude-rust:release 2>/dev/null
	@echo "Pruning unused Docker resources..."
	docker system prune -f
	@echo "✓ Docker cleanup complete"

## status: Show overall system status
status:
	@echo "=== System Status ==="
	@echo ""
	@echo "Devcontainer:"
	@CONTAINER=$$(docker ps -q --filter "label=devcontainer.local_folder=$$(pwd)"); \
	if [ -n "$$CONTAINER" ]; then \
		echo "  ✓ Running ($$CONTAINER)"; \
	else \
		echo "  ✗ Not running"; \
	fi
	@echo ""
	@echo "Development Services:"
	@if docker compose --profile full-stack ps | grep -q "Up"; then \
		docker compose --profile full-stack ps; \
	else \
		echo "  ✗ Not running"; \
	fi
	@echo ""
	@echo "Docker Images:"
	@docker images numberguess-claude-rust --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" 2>/dev/null || echo "  No images found"

## info: Display connection strings and URLs
info:
	@echo "=== Connection Information ==="
	@echo ""
	@echo "Database:"
	@echo "  Host: localhost:5432"
	@echo "  User: $(POSTGRES_USER)"
	@echo "  Password: $(POSTGRES_PASSWORD)"
	@echo "  Database: $(POSTGRES_DB)"
	@echo "  Connection String: postgresql://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost:5432/$(POSTGRES_DB)"
	@echo ""
	@echo "Application URLs:"
	@echo "  Web UI: http://localhost:8080"
	@echo "  Health Check: http://localhost:8081/health"
	@echo ""
	@echo "Commands:"
	@echo "  Database Shell: make db-shell"
	@echo "  View Logs: make dev-logs"

## quick: Fast feedback loop (format + unit tests)
quick:
	@echo "Running quick checks..."
	@echo "1/2 Formatting code..."
	@cargo fmt
	@echo "2/2 Running unit tests..."
	@cargo test --lib
	@echo ""
	@echo "✓ Quick checks passed!"

## check: Verify prerequisites
check:
	@echo "Checking prerequisites..."
	@command -v docker >/dev/null 2>&1 || (echo "✗ Docker not found" && exit 1)
	@echo "✓ Docker installed: $$(docker --version)"
	@docker info >/dev/null 2>&1 || (echo "✗ Docker daemon not running" && exit 1)
	@echo "✓ Docker daemon running"
	@command -v cargo >/dev/null 2>&1 || (echo "✗ Cargo not found" && exit 1)
	@echo "✓ Cargo installed: $$(cargo --version)"
	@if [ -n "$(strip $(DEVCONTAINER_BIN))" ]; then \
		echo "✓ Devcontainer CLI found: $(DEVCONTAINER_BIN)"; \
	else \
		echo "⚠ Devcontainer CLI not found (optional)"; \
	fi
	@echo ""
	@echo "✓ All prerequisites satisfied!"

## reset: Stop everything, clean, and start fresh
reset:
	@echo "Resetting everything..."
	@echo "1/4 Stopping all services..."
	@$(MAKE) dev-down 2>/dev/null || true
	@$(MAKE) dc-down 2>/dev/null || true
	@$(MAKE) test-down 2>/dev/null || true
	@echo "2/4 Cleaning Docker resources..."
	@$(MAKE) docker-clean 2>/dev/null || true
	@echo "3/4 Cleaning build artifacts..."
	@cargo clean
	@echo "4/4 Starting fresh..."
	@$(MAKE) dev
	@echo ""
	@echo "✓ Reset complete!"

## security-audit: Audit Rust dependencies for vulnerabilities
security-audit:
	@echo "Checking for cargo-audit..."
	@command -v cargo-audit >/dev/null 2>&1 || { \
		echo "Installing cargo-audit..."; \
		cargo install cargo-audit; \
	}
	@echo "Running cargo-audit..."
	cargo audit

## security-dockerfile: Lint Dockerfile with Hadolint
security-dockerfile:
	@echo "Running Hadolint on Dockerfile..."
	docker run --rm -i -v $(PWD)/.hadolint.yaml:/.config/hadolint.yaml hadolint/hadolint < Dockerfile

## security-scan: Build and scan Docker image with Trivy
security-scan:
	@echo "Building Docker image (debug mode)..."
	docker build --build-arg BUILD_TYPE=debug -t numberguess-security-scan:latest .
	@echo ""
	@echo "Running Trivy security scan..."
	docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
		aquasec/trivy image --quiet --format json --severity HIGH,CRITICAL numberguess-security-scan:latest
	@echo ""
	@echo "Cleaning up scan image..."
	-docker rmi numberguess-security-scan:latest

## security: Run all security checks (audit + dockerfile + scan)
security:
	@echo "════════════════════════════════════════════════════════════════"
	@echo "Running Security Checks"
	@echo "════════════════════════════════════════════════════════════════"
	@echo ""
	@echo "1/3 Dependency Audit (cargo-audit)"
	@echo "────────────────────────────────────────────────────────────────"
	@$(MAKE) security-audit
	@echo ""
	@echo "2/3 Dockerfile Lint (Hadolint)"
	@echo "────────────────────────────────────────────────────────────────"
	@$(MAKE) security-dockerfile
	@echo ""
	@echo "3/3 Container Scan (Trivy)"
	@echo "────────────────────────────────────────────────────────────────"
	@$(MAKE) security-scan
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "✓ All security checks completed!"
	@echo "════════════════════════════════════════════════════════════════"

## clean: Clean up build artifacts and Docker resources
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Removing Docker image..."
	-docker rmi numberguess-claude-rust:latest
	@echo "Stopping and removing containers..."
	-docker compose --profile full-stack down -v
	@echo "✓ Cleanup complete"
