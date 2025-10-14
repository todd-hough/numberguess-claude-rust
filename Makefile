.PHONY: help build test test-unit test-integration test-fast test-api test-ui dev dev-db dev-down dev-restart dev-logs dev-status \
        docker-build docker-build-debug docker-rebuild docker-check docker-clean clean logs db-shell fmt lint run-cli run-server \
        devcontainer-up devcontainer-down devcontainer-attach devcontainer-restart devcontainer-status \
        dc-up dc-down dc-attach dc-shell dc-restart dc-status \
        compose-up compose-down test-compose test-compose-ui test-compose-down \
        release release-test status info quick check reset

# Load environment variables from .env file if it exists
-include .env
export

# Database configuration defaults (can be overridden by .env)
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
	@echo "TESTING (Full Suite)"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make test          - Run complete test suite (unit + API + UI)"
	@echo "                       Automatically builds Docker image if needed"
	@echo "  make test-fast     - Quick unit tests only (~10 seconds)"
	@echo "  make test-api      - API integration tests (auto-builds if needed)"
	@echo "  make test-ui       - UI integration tests (auto-builds if needed)"
	@echo "  make test-unit     - Unit tests only (alias for test-fast)"
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
	@echo "RELEASE BUILDS"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make release       - Build production Docker image (optimized)"
	@echo "  make release-test  - Build release + run full test suite"
	@echo "  make docker-build  - Build Docker image (release mode)"
	@echo "  make docker-build-debug - Build Docker image (debug mode)"
	@echo "  make docker-clean  - Clean Docker images and containers"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "UTILITIES"
	@echo "════════════════════════════════════════════════════════════════"
	@echo "  make status        - Show overall system status"
	@echo "  make info          - Display connection strings and URLs"
	@echo "  make quick         - Fast feedback (fmt + unit tests)"
	@echo "  make check         - Verify prerequisites"
	@echo "  make reset         - Stop everything, clean, start fresh"
	@echo "  make build         - Build the application (cargo build)"
	@echo "  make fmt           - Format code with rustfmt"
	@echo "  make lint          - Run clippy linter"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make run-cli       - Run CLI game"
	@echo "  make run-server    - Run web server (requires postgres)"
	@echo ""
	@echo "════════════════════════════════════════════════════════════════"
	@echo "For more details: https://github.com/your-repo/numberguess"
	@echo "════════════════════════════════════════════════════════════════"

## build: Build the application (no database needed for build with runtime-checked SQLx)
build:
	cargo build

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

COMPOSE_STACK = docker compose -f docker-compose.yml -f docker-compose.integration.yml
TEST_DB ?= numberguess_test
HEALTH_TIMEOUT ?= 90

# Internal function: Setup test database
define setup_test_db
	@echo "Setting up test database..."
	@DB_HOST=localhost TEST_DB_NAME=$(TEST_DB) POSTGRES_HOST=localhost ./scripts/reset-db.sh
endef

# Internal function: Wait for service health
define wait_for_health
	@echo "Waiting for $(1) to be healthy..."
	@./scripts/wait-for-http.sh $(1) $(HEALTH_TIMEOUT)
endef

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

## test: Run complete test suite (unit + API + UI integration tests)
test: test-unit test-api test-ui
	@echo ""
	@echo "✓ All tests completed successfully!"

## test-fast: Run unit tests only (fast feedback)
test-fast:
	@echo "Running unit tests (fast feedback)..."
	cargo test --lib

## test-unit: Run unit tests only (no Docker needed)
test-unit:
	@echo "Running unit tests (no Docker required)..."
	cargo test --lib

## test-api: Run API integration tests via docker compose
test-api: test-compose

## test-ui: Run UI integration tests via docker compose
test-ui: test-compose-ui

## test-integration: Run legacy integration tests (uses testcontainers)
test-integration: docker-check
	@echo "Running legacy integration tests (testcontainers will manage containers)..."
	cargo test --test '*'

## test-compose: Run integration tests via docker compose (postgres + app)
test-compose: docker-check
	@echo "Running API integration tests..."
	@bash -c 'set -euo pipefail; \
	COMPOSE_CMD="$(COMPOSE_STACK)"; \
	PROFILE="integration"; \
	cleanup() { \
		echo "Cleaning up test environment..."; \
		$$COMPOSE_CMD --profile $$PROFILE down -v >/dev/null 2>&1 || true; \
	}; \
	trap cleanup EXIT; \
	echo "Starting postgres..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d postgres >/dev/null; \
	$$COMPOSE_CMD --profile $$PROFILE up --wait postgres >/dev/null; \
	echo "Setting up test database..."; \
	POSTGRES_HOST=localhost TEST_DB_NAME=$(TEST_DB) ./scripts/reset-db.sh; \
	echo "Starting app..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d app >/dev/null; \
	echo "Waiting for app to be healthy..."; \
	./scripts/wait-for-http.sh http://localhost:8081/health $(HEALTH_TIMEOUT); \
	echo "Running integration tests..."; \
	GAME_SERVER_BASE_URL=http://localhost:8080 cargo test --tests -- --test-threads=1; \
	'

## test-compose-ui: Run web UI integration tests via docker compose (app + selenium)
test-compose-ui: docker-check
	@echo "Running UI integration tests..."
	@bash -c 'set -euo pipefail; \
	COMPOSE_CMD="$(COMPOSE_STACK)"; \
	PROFILE="integration-ui"; \
	cleanup() { \
		echo "Cleaning up test environment..."; \
		$$COMPOSE_CMD --profile $$PROFILE down -v >/dev/null 2>&1 || true; \
	}; \
	trap cleanup EXIT; \
	echo "Starting postgres..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d postgres >/dev/null; \
	$$COMPOSE_CMD --profile $$PROFILE up --wait postgres >/dev/null; \
	echo "Setting up test database..."; \
	POSTGRES_HOST=localhost TEST_DB_NAME=$(TEST_DB) ./scripts/reset-db.sh; \
	echo "Starting app and selenium..."; \
	$$COMPOSE_CMD --profile $$PROFILE up -d app selenium >/dev/null; \
	echo "Waiting for app to be healthy..."; \
	./scripts/wait-for-http.sh http://localhost:8081/health $(HEALTH_TIMEOUT); \
	echo "Waiting for selenium to be healthy..."; \
	./scripts/wait-for-http.sh http://localhost:4444/status $(HEALTH_TIMEOUT); \
	echo "Running UI tests..."; \
	GAME_SERVER_BASE_URL=http://localhost:8080 \
	GAME_SERVER_BROWSER_URL=http://app:8080 \
	SELENIUM_REMOTE_URL=http://localhost:4444 \
	cargo test --test web_ui_test -- --test-threads=1; \
	'

## test-compose-down: Stop and remove integration test services
test-compose-down:
	@echo "Stopping integration test services..."
	@docker compose -f docker-compose.yml -f docker-compose.integration.yml --profile integration down -v 2>/dev/null || true
	@docker compose -f docker-compose.yml -f docker-compose.integration.yml --profile integration-ui down -v 2>/dev/null || true
	@echo "✓ Integration test services stopped"

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

## docker-build: Build Docker image (release mode by default)
docker-build:
	@echo "Building Docker image (release mode)..."
	docker build -t numberguess-claude-rust:latest .

## docker-build-debug: Build Docker image in debug mode (faster builds for dev/test)
docker-build-debug:
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
		$(MAKE) docker-build; \
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

## release-test: Build release image and run full test suite
release-test: release test
	@echo ""
	@echo "✓ Release build passed all tests!"

## docker-clean: Clean Docker images and containers
docker-clean:
	@echo "Cleaning Docker resources..."
	@echo "Stopping all compose services..."
	-docker compose --profile full-stack down -v 2>/dev/null
	-docker compose -f docker-compose.yml -f docker-compose.integration.yml --profile integration down -v 2>/dev/null
	-docker compose -f docker-compose.yml -f docker-compose.integration.yml --profile integration-ui down -v 2>/dev/null
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
	@if [ -f .env ]; then \
		echo "✓ .env file exists"; \
	else \
		echo "⚠ .env file not found (will use defaults)"; \
	fi
	@echo ""
	@echo "✓ All prerequisites satisfied!"

## reset: Stop everything, clean, and start fresh
reset:
	@echo "Resetting everything..."
	@echo "1/4 Stopping all services..."
	@$(MAKE) dev-down 2>/dev/null || true
	@$(MAKE) dc-down 2>/dev/null || true
	@$(MAKE) test-compose-down 2>/dev/null || true
	@echo "2/4 Cleaning Docker resources..."
	@$(MAKE) docker-clean 2>/dev/null || true
	@echo "3/4 Cleaning build artifacts..."
	@cargo clean
	@echo "4/4 Starting fresh..."
	@$(MAKE) dev
	@echo ""
	@echo "✓ Reset complete!"

## clean: Clean up build artifacts and Docker resources
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Removing Docker image..."
	-docker rmi numberguess-claude-rust:latest
	@echo "Stopping and removing containers..."
	-docker compose --profile full-stack down -v
	@echo "✓ Cleanup complete"
