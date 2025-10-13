.PHONY: help build test test-unit test-integration dev dev-db dev-down \
        docker-build docker-build-debug docker-rebuild docker-check clean logs db-shell fmt lint run-cli run-server \
        devcontainer-up devcontainer-down devcontainer-attach dc-up dc-down dc-attach \
        compose-up compose-down test-compose test-compose-ui test-compose-down

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
	@echo "Development:"
	@echo "  make dc-up         - Launch devcontainer (alias for devcontainer-up)"
	@echo "  make dc-down       - Stop devcontainer (alias for devcontainer-down)"
	@echo "  make dc-attach     - Attach terminal to devcontainer (alias for devcontainer-attach)"
	@echo "  make dev           - Start postgres + app server for manual testing"
	@echo "  make dev-db        - Start only postgres (run app locally with cargo run)"
	@echo "  make dev-down      - Stop development services"
	@echo "  make compose-up    - Start docker-compose integration stack (postgres + app)"
	@echo "  make compose-down  - Stop docker-compose integration stack"
	@echo "  make logs          - View docker-compose logs"
	@echo "  make db-shell      - Open PostgreSQL shell"
	@echo ""
	@echo "Building:"
	@echo "  make build         - Build the application (no database needed)"
	@echo "  make docker-build  - Build Docker image (release mode, optimized)"
	@echo "  make docker-build-debug - Build Docker image (debug mode, fast builds)"
	@echo "  make docker-rebuild - Force rebuild Docker image (no cache, release mode)"
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run all tests (builds Docker if needed)"
	@echo "  make test-unit     - Run unit tests only (fast, no Docker)"
	@echo "  make test-integration - Legacy integration tests (testcontainers)"
	@echo "  make test-compose  - Run integration tests against docker-compose stack"
	@echo "  make test-compose-ui - Run UI integration tests against docker-compose stack"
	@echo "  make test-compose-down - Stop and remove integration test services"
	@echo ""
	@echo "Running:"
	@echo "  make run-cli       - Run CLI game (interactive)"
	@echo "  make run-server    - Run web server (requires postgres)"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt           - Format code with rustfmt"
	@echo "  make lint          - Run clippy linter"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean         - Clean build artifacts and Docker resources"

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

COMPOSE_STACK = docker compose -f docker-compose.yml -f docker-compose.integration.yml

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

## test: Run all tests (builds Docker image first if needed)
test: docker-check
	cargo test

## test-unit: Run unit tests only (no Docker needed)
test-unit:
	@echo "Running unit tests (no Docker required)..."
	cargo test --lib

## test-integration: Run legacy integration tests (uses testcontainers)
test-integration: docker-check
	@echo "Running legacy integration tests (testcontainers will manage containers)..."
	cargo test --test '*'

## test-compose: Run integration tests via docker compose (postgres + app)
test-compose:
	@bash -c 'set -euo pipefail; \
	COMPOSE_CMD="docker compose -f docker-compose.yml -f docker-compose.integration.yml"; \
	cleanup(){ $$COMPOSE_CMD --profile integration down -v >/dev/null 2>&1 || true; }; \
	trap cleanup EXIT; \
	$$COMPOSE_CMD --profile integration up -d postgres >/dev/null; \
	$$COMPOSE_CMD --profile integration up --wait postgres >/dev/null; \
	DB_HOST=localhost; \
	TEST_DB=$${TEST_DB_NAME:-numberguess_test}; \
	POSTGRES_HOST=$$DB_HOST TEST_DB_NAME=$$TEST_DB ./scripts/reset-db.sh; \
	$$COMPOSE_CMD --profile integration up -d app >/dev/null; \
	HEALTH_URL=http://localhost:8081/health; \
	./scripts/wait-for-http.sh $$HEALTH_URL 90; \
	BASE_URL=http://localhost:8080; \
	GAME_SERVER_BASE_URL=$$BASE_URL cargo test --tests -- --test-threads=1; \
	'

## test-compose-ui: Run web UI integration tests via docker compose (app + selenium)
test-compose-ui:
	@bash -c 'set -euo pipefail; \
	COMPOSE_CMD="docker compose -f docker-compose.yml -f docker-compose.integration.yml"; \
	cleanup(){ $$COMPOSE_CMD --profile integration-ui down -v >/dev/null 2>&1 || true; }; \
	trap cleanup EXIT; \
	$$COMPOSE_CMD --profile integration-ui up -d postgres >/dev/null; \
	$$COMPOSE_CMD --profile integration-ui up --wait postgres >/dev/null; \
	DB_HOST=localhost; \
	TEST_DB=$${TEST_DB_NAME:-numberguess_test}; \
	POSTGRES_HOST=$$DB_HOST TEST_DB_NAME=$$TEST_DB ./scripts/reset-db.sh; \
	$$COMPOSE_CMD --profile integration-ui up -d app selenium >/dev/null; \
	APP_HEALTH=http://localhost:8081/health; \
	SEL_HEALTH=http://localhost:4444/status; \
	./scripts/wait-for-http.sh $$APP_HEALTH 90; \
	./scripts/wait-for-http.sh $$SEL_HEALTH 90; \
	BASE_URL=http://localhost:8080; \
	BROWSER_URL=http://app:8080; \
	SEL_URL=http://localhost:4444; \
	GAME_SERVER_BASE_URL=$$BASE_URL GAME_SERVER_BROWSER_URL=$$BROWSER_URL SELENIUM_REMOTE_URL=$$SEL_URL cargo test --test web_ui_test -- --test-threads=1; \
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

## clean: Clean up build artifacts and Docker resources
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Removing Docker image..."
	-docker rmi numberguess-claude-rust:latest
	@echo "Stopping and removing containers..."
	-docker compose --profile full-stack down -v
	@echo "✓ Cleanup complete"
