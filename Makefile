.PHONY: help build test test-unit test-integration dev dev-db dev-down \
        docker-build docker-rebuild docker-check clean logs db-shell fmt lint run-cli run-server

# Load environment variables from .env file if it exists
-include .env
export

# Database configuration defaults (can be overridden by .env)
POSTGRES_USER ?= numberguess
POSTGRES_PASSWORD ?= password
POSTGRES_DB ?= numberguess_dev

# Default target
.DEFAULT_GOAL := help

## help: Show this help message
help:
	@echo "Number Guessing Game - Development Commands"
	@echo ""
	@echo "Development:"
	@echo "  make dev           - Start postgres + app server for manual testing"
	@echo "  make dev-db        - Start only postgres (run app locally with cargo run)"
	@echo "  make dev-down      - Stop development services"
	@echo "  make logs          - View docker-compose logs"
	@echo "  make db-shell      - Open PostgreSQL shell"
	@echo ""
	@echo "Building:"
	@echo "  make build         - Build the application (no database needed)"
	@echo "  make docker-build  - Build Docker image"
	@echo "  make docker-rebuild - Force rebuild Docker image (no cache)"
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run all tests (builds Docker if needed)"
	@echo "  make test-unit     - Run unit tests only (fast, no Docker)"
	@echo "  make test-integration - Run integration tests (with testcontainers)"
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

## test-integration: Run integration tests (uses testcontainers)
test-integration: docker-check
	@echo "Running integration tests (testcontainers will manage databases)..."
	cargo test --test '*'

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

## docker-build: Build Docker image
docker-build:
	@echo "Building Docker image..."
	docker build -t numberguess-claude-rust:latest .

## docker-rebuild: Force rebuild Docker image (no cache)
docker-rebuild:
	@echo "Rebuilding Docker image (no cache)..."
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