# Multi-stage build for the Number Guessing Game
FROM rust:1.89-slim AS builder

# Build configuration: "release" or "debug"
ARG BUILD_TYPE=release

# Install build dependencies
RUN apt-get update && apt-get install -y \
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

# Runtime stage
FROM debian:bookworm-slim

# Build configuration (must match builder stage)
ARG BUILD_TYPE=release

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder stage (from release or debug dir)
COPY --from=builder /app/target/${BUILD_TYPE}/number_guessing_game /usr/local/bin/

# Set working directory
WORKDIR /app

# Copy static files to working directory
COPY --from=builder /app/static ./static

# Create a non-root user
RUN useradd -r -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

# Expose the web ports (main app and health check)
EXPOSE 8080 8081

# Set environment variables
ENV RUST_LOG=info

# Default command runs the web server
ENTRYPOINT ["number_guessing_game"]
CMD ["--server", "--port", "8080"]