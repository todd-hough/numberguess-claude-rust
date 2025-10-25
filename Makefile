.PHONY: help build test test-unit test-integration test-down dev dev-db dev-down dev-restart dev-logs dev-status \
        docker-rebuild docker-check docker-clean clean logs db-shell fmt lint run-cli run-server \
        devcontainer-up devcontainer-down devcontainer-attach devcontainer-restart devcontainer-status \
        dc-up dc-down dc-attach dc-shell dc-restart dc-status \
        compose-up compose-down \
        release status info quick check reset

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
	@echo "  make test-integration - Integration tests (starts Docker Compose)"
	@echo "  make test-down     - Stop integration test environment"
	@echo ""
	@echo "  Note: test-integration keeps environment running for debugging"
	@echo "        Use test-down when finished to clean up resources"
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

COMPOSE_STACK = docker compose -f docker-compose.yml -f docker-compose.integration.yml
TEST_DB ?= numberguess_test

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

## test-integration: Run integration tests via docker compose
test-integration: docker-check
	@echo "Starting integration test environment..."
	@echo "Note: Environment will remain running after tests for debugging"
	@echo "Use 'make test-down' to stop when done"
	@echo ""
	$(COMPOSE_STACK) --profile integration up -d --wait
	@echo ""
	@echo "Running integration tests..."
	GAME_SERVER_BASE_URL=http://localhost:8080 \
	GAME_SERVER_BROWSER_URL=http://oauth2-proxy:4180 \
	SELENIUM_REMOTE_URL=http://localhost:4444 \
	TEST_DB_NAME=$(TEST_DB) \
	cargo test --tests -- --test-threads=1

## test-down: Stop and remove integration test services
test-down:
	@echo "Stopping integration test services..."
	$(COMPOSE_STACK) --profile integration down -v
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

## clean: Clean up build artifacts and Docker resources
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Removing Docker image..."
	-docker rmi numberguess-claude-rust:latest
	@echo "Stopping and removing containers..."
	-docker compose --profile full-stack down -v
	@echo "✓ Cleanup complete"
