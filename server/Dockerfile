# Game Server Dockerfile
# Rust + Tokio for authoritative game simulation

# Build stage
FROM rust:1.92 as builder

WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build optimized binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /app/target/release/game-server /app/game-server

# Expose WebSocket port
EXPOSE 9001

# Health check (simple process check for now)
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD pgrep game-server || exit 1

# Run the game server
CMD ["./game-server"]

# Builder target (for docker-compose game-server-builder)
FROM builder as builder
