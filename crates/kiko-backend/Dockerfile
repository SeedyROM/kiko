# Multi-stage build for Rust backend
FROM rust:1.88-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/kiko/Cargo.toml ./crates/kiko/
COPY crates/kiko-backend/Cargo.toml ./crates/kiko-backend/
COPY crates/kiko-frontend/Cargo.toml ./crates/kiko-frontend/

# Create dummy frontend source to satisfy workspace
RUN mkdir -p crates/kiko-frontend/src && echo "fn main() {}" > crates/kiko-frontend/src/main.rs

# Copy source code
COPY crates/kiko ./crates/kiko
COPY crates/kiko-backend ./crates/kiko-backend

# Build for release
RUN cargo build --release -p kiko-backend

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/kiko-backend /usr/local/bin/kiko-backend

# Change ownership
RUN chown appuser:appuser /usr/local/bin/kiko-backend

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3030

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3030/health || exit 1

# Run the binary
CMD ["kiko-backend"]