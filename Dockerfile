# ============================================================================
# RUSTY TORRENTS - MULTI-STAGE DOCKER BUILD
# ============================================================================
# This Dockerfile builds a complete containerized BitTorrent application with:
#   - Web Client (React/TypeScript) - served via Nginx or as static files
#   - REST Backend API (Rust/Actix-web) - runs on port 8080
#   - BitTorrent Protocol Support - listens on port 6881
#
# USAGE:
#   docker build -t rusty-torrents:latest .
#   docker run -p 8080:8080 -p 6881:6881 -v ./downloads:/app/downloads rusty-torrents
#
# OR use docker-compose:
#   docker-compose up
#
# ============================================================================

# Stage 1: Build Web Client (React/TypeScript)
# ============================================================================
FROM node:20-alpine AS web-builder

WORKDIR /build/web

# Copy web client files
COPY web/package*.json ./
COPY web/tsconfig*.json ./
COPY web/vite.config.ts ./
COPY web/index.html ./
COPY web/public ./public
COPY web/src ./src

# Install dependencies and build web client
RUN npm ci && npm run build

# Stage 2: Build Rust Backend
# ============================================================================
FROM rust:latest AS backend-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Cargo.toml and cargo.lock files
COPY src-tauri/Cargo.* ./

# Copy source code
COPY src-tauri/src ./src
COPY src-tauri/build.rs ./

# Build backend in release mode
RUN cargo build --release

# Stage 3: Runtime Environment
# ============================================================================
FROM debian:bookworm-slim

LABEL maintainer="Rusty Torrents Team"
LABEL description="A powerful BitTorrent client with remote web monitoring"
LABEL version="0.1.0"

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create necessary directories for persistent data
RUN mkdir -p /app/downloads \
    && mkdir -p /app/logs \
    && mkdir -p /app/config \
    && mkdir -p /app/www

# Copy built backend binary from builder
COPY --from=backend-builder /build/target/release/bittorrent-client /app/bittorrent-client

# Copy built web client from web-builder
COPY --from=web-builder /build/web/dist /app/www

# Make backend binary executable
RUN chmod +x /app/bittorrent-client

# ============================================================================
# PORTS & NETWORKING
# ============================================================================
# Port 6881  : BitTorrent DHT/Protocol (UDP)
# Port 8080  : REST API Backend
# Port 3000  : Web Client (development only - not exposed in production)
EXPOSE 6881 8080

# ============================================================================
# HEALTH CHECK
# ============================================================================
# Verifies the backend API is responding correctly
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# ============================================================================
# ENVIRONMENT VARIABLES
# ============================================================================
# RUST_LOG: Controls logging verbosity (trace, debug, info, warn, error)
# DOWNLOADS_DIR: Where torrents are saved (default: /app/downloads)
ENV RUST_LOG=info
ENV DOWNLOADS_DIR=/app/downloads

# ============================================================================
# STARTUP
# ============================================================================
# The backend server initializes:
#   1. Loads configuration
#   2. Starts REST API on port 8080
#   3. Initializes BitTorrent protocol listener on port 6881
#   4. Serves web client static files
#
# Access points:
#   - API: http://localhost:8080/api/*
#   - Health: http://localhost:8080/api/health
#   - Web UI: Served via Nginx on port 80/443 (see docker-compose.yml)
#
CMD ["./bittorrent-client"]
