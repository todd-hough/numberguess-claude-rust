.PHONY: test test-unit test-integration docker-build docker-check clean

# Main test target - builds Docker image first
test: docker-check
	cargo test

# Run only unit tests (no Docker needed)
test-unit:
	cargo test --lib
	cargo test --test integration_test
	cargo test --test web_endpoints_test
	cargo test --test api_edge_cases_test

# Run web UI tests (requires Docker)
test-web-ui: docker-check
	cargo test --test web_ui_test

# Build Docker image
docker-build:
	@echo "Building Docker image..."
	docker build -t numberguess-claude-rust:latest .

# Check if Docker image exists, build if not
docker-check:
	@if ! docker images numberguess-claude-rust:latest | grep -q latest; then \
		echo "Docker image not found, building..."; \
		$(MAKE) docker-build; \
	else \
		echo "✓ Docker image exists"; \
	fi

# Force rebuild Docker image
docker-rebuild:
	@echo "Rebuilding Docker image..."
	docker build --no-cache -t numberguess-claude-rust:latest .

# Clean up Docker images and build artifacts
clean:
	cargo clean
	-docker rmi numberguess-claude-rust:latest

# Help target
help:
	@echo "Available targets:"
	@echo "  make test          - Run all tests (builds Docker if needed)"
	@echo "  make test-unit     - Run unit tests only"
	@echo "  make test-web-ui   - Run web UI tests only"
	@echo "  make docker-build  - Build Docker image"
	@echo "  make docker-rebuild - Force rebuild Docker image"
	@echo "  make clean         - Clean build artifacts and Docker image"