# Multi-stage build for WASM frontend
FROM rust:1.88-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js for TailwindCSS
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install trunk and wasm target
RUN cargo install trunk \
    && rustup target add wasm32-unknown-unknown

# Install TailwindCSS globally and ensure npx works
RUN npm install -g @tailwindcss/cli
RUN npm config set prefix /usr/local
ENV PATH="/usr/local/bin:$PATH"

# Create app directory
WORKDIR /app

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/kiko/Cargo.toml ./crates/kiko/
COPY crates/kiko-backend/Cargo.toml ./crates/kiko-backend/
COPY crates/kiko-frontend/Cargo.toml ./crates/kiko-frontend/

# Create dummy backend source to satisfy workspace
RUN mkdir -p crates/kiko-backend/src && echo "fn main() {}" > crates/kiko-backend/src/main.rs
RUN mkdir -p crates/kiko-backend/benches && echo "fn main() {}" > crates/kiko-backend/benches/pubsub.rs
RUN echo "fn main() {}" > crates/kiko-backend/benches/session_service.rs

# Copy source code
COPY crates/kiko ./crates/kiko
COPY crates/kiko-frontend ./crates/kiko-frontend

# Build frontend for release
WORKDIR /app/crates/kiko-frontend

# Ensure TailwindCSS is available for npx
RUN npm init -y && npm install @tailwindcss/cli --save-dev

RUN trunk build --release

# Runtime stage with nginx and brotli
FROM alpine:latest as runtime

# Install nginx and brotli from same Alpine repository to ensure compatibility
RUN apk add --no-cache nginx nginx-mod-http-brotli

# Copy custom nginx config
COPY config/nginx.conf /etc/nginx/nginx.conf

# Copy built assets from builder stage
COPY --from=builder /app/crates/kiko-frontend/dist /usr/share/nginx/html

# Create nginx user directories
RUN mkdir -p /var/cache/nginx/client_temp \
    && mkdir -p /var/cache/nginx/proxy_temp \
    && mkdir -p /var/cache/nginx/fastcgi_temp \
    && mkdir -p /var/cache/nginx/uwsgi_temp \
    && mkdir -p /var/cache/nginx/scgi_temp

# Change ownership
RUN chown -R nginx:nginx /var/cache/nginx \
    && chown -R nginx:nginx /usr/share/nginx/html

# Switch to non-root user
USER nginx

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/ || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]