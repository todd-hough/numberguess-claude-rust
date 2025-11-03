# Multi-stage build for the Number Guessing Game
FROM rust:1.91-slim AS builder

# Build configuration: "release" or "debug"
ARG BUILD_TYPE=release

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source files
COPY . .

# Build the application (conditional: release or debug)
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