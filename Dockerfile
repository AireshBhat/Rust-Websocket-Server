FROM rust:1.73-slim as builder

WORKDIR /app

# Create a new empty shell project
RUN USER=root cargo new --bin temp-rust-websocket
WORKDIR /app/temp-rust-websocket

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies - this is the caching Docker layer
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/temp-rust-websocket/target \
    cargo build --release

# Copy local code to the container
COPY . .

# Build for release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/temp-rust-websocket/target \
    touch src/main.rs && cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/temp-rust-websocket/target/release/temp-rust-websocket /app/

# Set environment variables
ENV RUST_LOG=info

# Expose ports
EXPOSE 8080

# Run the application
CMD ["/app/temp-rust-websocket"] 