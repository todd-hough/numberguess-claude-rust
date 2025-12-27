# Multi-stage build for the Number Guessing Game using cargo-chef for dependency caching
FROM rust:1.90-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 1: Plan the build
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build dependencies (the cached layer)
FROM chef AS builder
# Build configuration: "release" or "debug"
ARG BUILD_TYPE=release

# Install build dependencies required for compiling crates like openssl or sqlx
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the recipe from the planner stage
COPY --from=planner /app/recipe.json recipe.json

# Cook the dependencies - this layer is cached unless Cargo.toml/Cargo.lock changes
RUN if [ "$BUILD_TYPE" = "release" ]; then \
        cargo chef cook --release --recipe-path recipe.json; \
    else \
        cargo chef cook --recipe-path recipe.json; \
    fi

# Copy the actual source code
COPY . .

# Build the application (conditional: release or debug)
# This step will be very fast if only source code changed
RUN if [ "$BUILD_TYPE" = "release" ]; then \
        cargo build --release; \
    else \
        cargo build; \
    fi

# Stage 3: Runtime - Using distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12

# Build configuration (must match builder stage)
ARG BUILD_TYPE=release

# Set working directory
WORKDIR /app

# Copy the built binary and static assets from builder stage
COPY --from=builder /app/target/${BUILD_TYPE}/number_guessing_game /usr/local/bin/number_guessing_game

# Distroless runs as non-root user "nonroot" (UID 65532) by default
# No need to create user - distroless has this built-in

# Expose the web ports (main app and health check)
EXPOSE 8080 8081

# Set environment variables
ENV RUST_LOG=info

# Default command runs the web server
ENTRYPOINT ["/usr/local/bin/number_guessing_game"]
CMD ["--server", "--port", "8080"]
