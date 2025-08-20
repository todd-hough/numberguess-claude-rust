# Multi-stage build for the Number Guessing Game
FROM rust:1.89-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source files
COPY . .

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/number_guessing_game /usr/local/bin/

# Copy static files
COPY --from=builder /app/static /usr/local/share/number_guessing_game/static

# Create a non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

# Expose the web port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info

# Default command runs the web server
ENTRYPOINT ["number_guessing_game"]
CMD ["--server", "--port", "3000"]