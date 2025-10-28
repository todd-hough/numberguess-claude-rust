# Multi-stage build for the Number Guessing Game
FROM rust:1.90-slim AS builder

# Build configuration: "release" or "debug"
ARG BUILD_TYPE=release

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency manifests first (for caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy src directory to satisfy cargo build
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn lib() {}" > src/lib.rs

# Build dependencies only (this layer will be cached)
RUN if [ "$BUILD_TYPE" = "release" ]; then \
        cargo build --release; \
    else \
        cargo build; \
    fi

# Remove dummy source files and target directory artifacts
RUN rm -rf src

# Copy actual source files
COPY . .

# Build the actual application (only source code compilation, deps are cached)
RUN if [ "$BUILD_TYPE" = "release" ]; then \
        cargo build --release; \
    else \
        cargo build; \
    fi

# Runtime stage - Using distroless for minimal attack surface
# Mitigates: CVE linux-pam (directory traversal), CVE zlib/MiniZip (heap overflow)
FROM gcr.io/distroless/cc-debian12

# Build configuration (must match builder stage)
ARG BUILD_TYPE=release

# Copy the built binary from builder stage (from release or debug dir)
COPY --from=builder /app/target/${BUILD_TYPE}/number_guessing_game /usr/local/bin/number_guessing_game

# Set working directory
WORKDIR /app

# Copy static files to working directory
COPY --from=builder /app/static ./static

# Distroless runs as non-root user "nonroot" (UID 65532) by default
# No need to create user - distroless has this built-in

# Expose the web ports (main app and health check)
EXPOSE 8080 8081

# Set environment variables
ENV RUST_LOG=info

# Default command runs the web server
ENTRYPOINT ["/usr/local/bin/number_guessing_game"]
CMD ["--server", "--port", "8080"]