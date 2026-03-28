FROM rust:latest AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    npm \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy frontend files
COPY package.json package-lock.json tsconfig.json tsconfig.node.json vite.config.ts ./
COPY src ./src
COPY public ./public

# Copy backend files  
COPY src-tauri ./src-tauri

# Build frontend
RUN npm install && npm run build

# Build backend
WORKDIR /build/src-tauri
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built backend binary
COPY --from=builder /build/src-tauri/target/release/bittorrent-client /app/
COPY --from=builder /build/dist ./www

# Create necessary directories
RUN mkdir -p /app/downloads /app/logs /app/config

# Expose ports
EXPOSE 6881 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["./bittorrent-client"]
