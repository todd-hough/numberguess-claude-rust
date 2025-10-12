# Number Guessing Game - Development Commands (just)
# Usage: just <recipe>
# List all recipes: just --list

# Load environment variables from .env
set dotenv-load

# Database configuration defaults (can be overridden by .env)
POSTGRES_USER := env_var_or_default('POSTGRES_USER', 'numberguess')
POSTGRES_PASSWORD := env_var_or_default('POSTGRES_PASSWORD', 'password')
POSTGRES_DB := env_var_or_default('POSTGRES_DB', 'numberguess_dev')

# Default recipe shows help
default:
    @just --list

# Build the application (no database needed)
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Development: Start postgres + app server for manual testing (full stack)
dev:
    @echo "Starting full development stack (postgres + app)..."
    docker compose --profile full-stack up -d
    @echo ""
    @echo "✓ Services started!"
    @echo "  Web UI: http://localhost:8080"
    @echo "  Health Check: http://localhost:8081/health"
    @echo "  Database: postgresql://{{POSTGRES_USER}}:{{POSTGRES_PASSWORD}}@localhost:5432/{{POSTGRES_DB}}"
    @echo ""
    @echo "View logs: just logs"
    @echo "Stop services: just dev-down"

# Development: Start only postgres (run app locally)
dev-db:
    @echo "Starting postgres only..."
    docker compose up -d postgres
    @echo ""
    @echo "✓ PostgreSQL started!"
    @echo "  Connection: postgresql://{{POSTGRES_USER}}:{{POSTGRES_PASSWORD}}@localhost:5432/{{POSTGRES_DB}}"
    @echo ""
    @echo "Now run: just run-server"

# Stop development services
dev-down:
    @echo "Stopping development services..."
    docker compose --profile full-stack down

# Launch devcontainer environment using devcontainer CLI
devcontainer-up:
    #!/usr/bin/env bash
    DEVCONTAINER_BIN=$(command -v devcontainer 2>/dev/null || find ~/.config/nvm/versions/node -maxdepth 5 \( -type f -o -type l \) -name devcontainer 2>/dev/null | head -n1)
    if [ -z "$DEVCONTAINER_BIN" ]; then
        echo "devcontainer CLI not found. Install from https://github.com/devcontainers/cli"
        exit 1
    fi
    DEVCONTAINER_DIR=$(dirname "$DEVCONTAINER_BIN")
    PATH="$DEVCONTAINER_DIR:$PATH" "$DEVCONTAINER_BIN" up --workspace-folder .

# Stop devcontainer services and clean up using devcontainer CLI
devcontainer-down:
    #!/usr/bin/env bash
    DEVCONTAINER_BIN=$(command -v devcontainer 2>/dev/null || find ~/.config/nvm/versions/node -maxdepth 5 \( -type f -o -type l \) -name devcontainer 2>/dev/null | head -n1)
    if [ -z "$DEVCONTAINER_BIN" ]; then
        echo "devcontainer CLI not found. Install from https://github.com/devcontainers/cli"
        exit 1
    fi
    echo "Stopping devcontainer..."
    DEVCONTAINER_DIR=$(dirname "$DEVCONTAINER_BIN")
    PATH="$DEVCONTAINER_DIR:$PATH" "$DEVCONTAINER_BIN" down --workspace-folder . || true

# Attach terminal to running devcontainer
devcontainer-attach:
    #!/usr/bin/env bash
    DEVCONTAINER_BIN=$(command -v devcontainer 2>/dev/null || find ~/.config/nvm/versions/node -maxdepth 5 \( -type f -o -type l \) -name devcontainer 2>/dev/null | head -n1)
    if [ -z "$DEVCONTAINER_BIN" ]; then
        echo "devcontainer CLI not found. Install from https://github.com/devcontainers/cli"
        exit 1
    fi
    echo "Attaching to devcontainer..."
    DEVCONTAINER_DIR=$(dirname "$DEVCONTAINER_BIN")
    PATH="$DEVCONTAINER_DIR:$PATH" "$DEVCONTAINER_BIN" exec --workspace-folder . bash

# View docker-compose logs
logs:
    docker compose --profile full-stack logs -f

# Open PostgreSQL shell
db-shell:
    docker compose exec postgres psql -U {{POSTGRES_USER}} -d {{POSTGRES_DB}}

# Run all tests
test: docker-check
    cargo test

# Run unit tests only (fast, no Docker)
test-unit:
    @echo "Running unit tests (no Docker required)..."
    cargo test --lib

# Run integration tests via docker compose
test-integration:
    @echo "Running integration tests against docker compose stack..."
    make test-compose

# Run a specific test
test-one TEST:
    cargo test {{TEST}}

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Stop and remove integration test services (auto-cleanup on exit, rarely needed)
test-compose-down:
    @echo "Stopping integration test services..."
    @make test-compose-down

# Run CLI game with custom range
run-cli MIN="1" MAX="100" LIMIT="":
    #!/usr/bin/env bash
    if [ -n "{{LIMIT}}" ]; then
        cargo run -- --min {{MIN}} --max {{MAX}} --limit {{LIMIT}}
    else
        cargo run -- --min {{MIN}} --max {{MAX}}
    fi

# Run web server (requires postgres)
run-server PORT="8080":
    @echo "Starting web server on port {{PORT}}..."
    @echo "Make sure postgres is running: just dev-db"
    cargo run -- --server --port {{PORT}}

# Format code with rustfmt
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Fix clippy warnings automatically (when possible)
lint-fix:
    cargo clippy --fix --allow-dirty -- -D warnings

# Run formatter and linter
check: fmt lint

# Build Docker image
docker-build:
    @echo "Building Docker image..."
    docker build -t numberguess-claude-rust:latest .

# Force rebuild Docker image (no cache)
docker-rebuild:
    @echo "Rebuilding Docker image (no cache)..."
    docker build --no-cache -t numberguess-claude-rust:latest .

# Check if Docker image exists, build if not
docker-check:
    #!/usr/bin/env bash
    if ! docker images numberguess-claude-rust:latest | grep -q latest; then
        echo "Docker image not found, building..."
        just docker-build
    else
        echo "✓ Docker image exists"
    fi

# Clean build artifacts and Docker resources
clean:
    @echo "Cleaning build artifacts..."
    cargo clean
    @echo "Removing Docker image..."
    -docker rmi numberguess-claude-rust:latest
    @echo "Stopping and removing containers..."
    -docker compose --profile full-stack down -v
    @echo "✓ Cleanup complete"

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Run security audit
audit:
    cargo audit

# Generate documentation
doc:
    cargo doc --open

# Watch and re-run tests on file changes (requires cargo-watch)
watch-test:
    cargo watch -x test

# Watch and re-run the server on file changes (requires cargo-watch)
watch-server:
    cargo watch -x 'run -- --server'

# Install development tools
install-tools:
    @echo "Installing development tools..."
    cargo install cargo-watch cargo-outdated cargo-audit
    @echo "✓ Tools installed!"

# Show project statistics
stats:
    @echo "Lines of code:"
    @find src -name '*.rs' -exec wc -l {} + | tail -1
    @echo ""
    @echo "Test coverage would require: cargo install cargo-tarpaulin"
    @echo "Run with: cargo tarpaulin --out Html"
