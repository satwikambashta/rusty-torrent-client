# 🐳 Docker Deployment Guide - Rusty Torrents

This guide covers containerizing and deploying Rusty Torrents using Docker and Docker Compose.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [System Requirements](#system-requirements)
- [Building](#building)
- [Running](#running)
- [Configuration](#configuration)
- [SSL/TLS Setup](#ssltls-setup)
- [Troubleshooting](#troubleshooting)
- [Production Deployment](#production-deployment)

---

## Quick Start

### Prerequisites
- **Docker**: v20.10+
- **Docker Compose**: v2.0+
- **System**: Linux, macOS, or Windows (with WSL2)

### Run in 2 Commands

```bash
# Clone/navigate to project directory
cd rusty-torrents

# Start all services (backend + nginx proxy)
docker-compose up

# Access in browser:
# - API: http://localhost:8080/api/health
# - Web UI: http://localhost (via Nginx)
```

---

## Architecture

### Services

```
┌─────────────────────────────────────────────┐
│         Docker Compose Network              │
│                                             │
│  ┌──────────────┐         ┌──────────────┐ │
│  │   Nginx      │◄────────┤   Backend    │ │
│  │  (Reverse    │         │  (REST API)  │ │
│  │   Proxy)     │         │              │ │
│  │              │         │ Port: 8080   │ │
│  │ Port: 80/443 │         │ Port: 6881   │ │
│  └──────────────┘         └──────────────┘ │
│       │                                    │
│       ▼                                    │
│  ┌──────────────────────────────────┐     │
│  │     Persistent Volumes           │     │
│  │  ├─ downloads/  (torrents)       │     │
│  │  ├─ logs/       (server logs)    │     │
│  │  └─ config/     (settings)       │     │
│  └──────────────────────────────────┘     │
└─────────────────────────────────────────────┘
```

### Build Stages

The **Dockerfile** uses multi-stage builds for optimized image size:

1. **Stage 1: Web Builder** (node:20-alpine)
   - Compiles React/TypeScript web client
   - Output: `/app/www` (static HTML/CSS/JS)

2. **Stage 2: Backend Builder** (rust:latest)
   - Compiles Rust backend binary
   - Output: `/app/bittorrent-client`

3. **Stage 3: Runtime** (debian:bookworm-slim)
   - Minimal base image
   - Includes only built artifacts + runtime dependencies
   - ~150MB final image size (optimized)

---

## System Requirements

### Minimum
- **CPU**: 2 cores
- **RAM**: 1 GB
- **Disk**: 2 GB (base) + storage for torrents
- **Bandwidth**: Unlimited (internal network)

### Recommended
- **CPU**: 4 cores
- **RAM**: 4 GB
- **Disk**: 10 GB free
- **Network**: Gigabit Ethernet recommended

### Port Requirements
- `6881` (TCP/UDP): BitTorrent protocol (can be remapped)
- `8080` (TCP): Backend API (internal to Docker network)
- `80` (TCP): HTTP → HTTPS redirect
- `443` (TCP): HTTPS (optional, requires certificates)

---

## Building

### Build Options

#### Option 1: Standard Build
```bash
docker build -t rusty-torrents:latest .
```

#### Option 2: With Docker Compose
```bash
docker-compose build
```

#### Option 3: Build-Time Arguments
```bash
docker build \
  --build-arg RUST_LOG=debug \
  -t rusty-torrents:custom .
```

### View Image Details
```bash
docker images rusty-torrents
docker inspect rusty-torrents:latest
```

### Clean Up Build Cache
```bash
docker builder prune
docker system prune -a
```

---

## Running

### Using Docker Compose (Recommended)

#### Start Services
```bash
# Start in foreground (see logs)
docker-compose up

# Start in background (daemon mode)
docker-compose up -d
```

#### View Status
```bash
# List running services
docker-compose ps

# View logs from all services
docker-compose logs -f

# View logs from specific service
docker-compose logs -f bittorrent-client
docker-compose logs -f nginx
```

#### Stop Services
```bash
# Stop without removing containers
docker-compose stop

# Stop and remove containers + volumes
docker-compose down
docker-compose down --volumes
```

### Using Docker CLI (Manual)

#### Build
```bash
docker build -t rusty-torrents:latest .
```

#### Run
```bash
docker run \
  --name rusty-torrents \
  -p 6881:6881 \
  -p 8080:8080 \
  -v downloads:/app/downloads \
  -v logs:/app/logs \
  -v config:/app/config \
  -e RUST_LOG=info \
  --restart unless-stopped \
  rusty-torrents:latest
```

---

## Configuration

### Environment Variables

Set in `docker-compose.yml` or via `--env` flag:

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `DOWNLOADS_DIR` | `/app/downloads` | Where torrents are saved |

### Modify for Production

```yaml
# docker-compose.yml
services:
  bittorrent-client:
    environment:
      RUST_LOG: warn  # Reduce log verbosity
      DOWNLOADS_DIR: /app/downloads
```

### Volume Mounts

#### Mount Local Directory (for downloads backup)
```bash
docker-compose down

# Update docker-compose.yml:
# volumes:
#   downloads:
#     driver: local
#     driver_opts:
#       type: none
#       o: bind
#       device: /home/user/torrents

docker-compose up
```

#### View Volumes
```bash
docker volume ls
docker volume inspect rusty-torrents_downloads
```

#### Backup Downloads
```bash
docker cp rusty-torrents-backend:/app/downloads ./downloads-backup
```

---

## SSL/TLS Setup

### Generate Self-Signed Certificates (Development)

```bash
# Create cert directory
mkdir -p cert

# Generate certificate (valid for 365 days)
openssl req -x509 -newkey rsa:4096 \
  -keyout cert/key.pem \
  -out cert/cert.pem \
  -days 365 \
  -nodes \
  -subj "/C=US/ST=State/L=City/O=Org/CN=localhost"
```

### Production Certificates (Let's Encrypt)

```bash
# Install Certbot
sudo apt-get install certbot

# Generate Let's Encrypt certificate
sudo certbot certonly --standalone -d yourdomain.com

# Copy to project
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./cert/cert.pem
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ./cert/key.pem
```

### Enable HTTPS in Nginx
```nginx
# nginx.conf - Already configured!
# - HTTP redirects to HTTPS
# - TLS 1.2+ enforced
# - Security headers added
# - CORS configured
```

### Restart Services
```bash
docker-compose down
docker-compose up -d
```

---

## Health Checks

### Monitor Service Health
```bash
# View health status
docker-compose ps

# Detailed health info
docker inspect rusty-torrents-backend --format='{{.State.Health}}'
```

### Manual Health Check
```bash
# Test API endpoint
curl http://localhost:8080/api/health

# Expected response:
# {"status": "ok", "version": "0.1.0"}
```

### Troubleshooting Unhealthy Containers
```bash
# View container logs
docker logs rusty-torrents-backend

# Check resource usage
docker stats rusty-torrents-backend

# Restart container
docker-compose restart bittorrent-client
```

---

## Troubleshooting

### Container Won't Start

**Symptom**: Container exits immediately

```bash
# View error logs
docker logs rusty-torrents-backend

# Common causes:
# 1. Port already in use
sudo lsof -i :8080
sudo lsof -i :6881

# 2. Insufficient permissions
docker logs --details rusty-torrents-backend

# 3. Resource constraints
docker stats
```

### API Not Responding

```bash
# Test connectivity
curl -v http://localhost:8080/api/health

# Check from inside container
docker-compose exec bittorrent-client curl http://localhost:8080/api/health

# View container IP
docker inspect rusty-torrents-backend | grep -A 2 "Networks"
```

### Volume Issues

```bash
# List volumes
docker volume ls

# Check volume contents
docker run -v rusty-torrents_downloads:/data --rm \
  -it debian:bookworm-slim ls -la /data

# Remove orphaned volumes
docker volume prune
```

### Memory Issues

```bash
# Monitor resource usage
docker stats

# Set memory limits in docker-compose.yml:
# deploy:
#   resources:
#     limits:
#       memory: 2G
```

---

## Production Deployment

### Pre-Production Checklist

- [ ] Set `RUST_LOG=warn` (reduce log verbosity)
- [ ] Configure SSL certificates (Let's Encrypt)
- [ ] Set up automated backups for `downloads` volume
- [ ] Configure firewall rules (see "Firewall Rules")
- [ ] Test health checks
- [ ] Review resource limits
- [ ] Set up monitoring/logging

### Firewall Rules

```bash
# Allow only necessary ports
sudo ufw allow ssh
sudo ufw allow 80/tcp      # HTTP
sudo ufw allow 443/tcp     # HTTPS
sudo ufw allow 6881/tcp    # BitTorrent (TCP)
sudo ufw allow 6881/udp    # BitTorrent (UDP)
sudo ufw default deny incoming
```

### Automatic Restarts

```yaml
# docker-compose.yml
restart: unless-stopped

# Docker daemon restart flag
# sudo dockerd --restart-policy=unless-stopped
```

### Monitoring

```bash
# View real-time stats
watch docker stats

# Export metrics (Prometheus format)
curl http://localhost:9090/metrics

# Set up log aggregation:
# docker logs rusty-torrents-backend | tee /var/log/rusty-torrents.log
```

### Backup Strategy

```bash
#!/bin/bash
# backup.sh - Automated backup script

# Backup downloads
tar -czf /backup/downloads-$(date +%Y%m%d).tar.gz \
  /var/lib/docker/volumes/rusty-torrents_downloads/_data

# Backup config
tar -czf /backup/config-$(date +%Y%m%d).tar.gz \
  /var/lib/docker/volumes/rusty-torrents_config/_data

# Cleanup old backups (keep 7 days)
find /backup -mtime +7 -delete
```

### Scaling & Clustering

For handling multiple instances:

```yaml
# docker-compose.yml with multiple instances
services:
  backend-1:
    # ... config
    environment:
      INSTANCE_ID: "1"

  backend-2:
    # ... config
    environment:
      INSTANCE_ID: "2"

  # Load balancer (Nginx/HAProxy)
  loadbalancer:
    image: nginx:alpine
    volumes:
      - ./lb-config.conf:/etc/nginx/nginx.conf
    ports:
      - "80:80"
    depends_on:
      - backend-1
      - backend-2
```

---

## Advanced Topics

### Custom Base Image

```dockerfile
# Use slimmer base for production
FROM debian:bookworm-slim
# vs
FROM debian:bookworm  # Includes development tools
```

### Build Optimizations

```dockerfile
# Use BuildKit for faster builds
DOCKER_BUILDKIT=1 docker build .

# Parallel multi-stage builds
# Cache optimization with layer ordering
RUN apt-get update && apt-get install -y curl  # Cached together
```

### Development vs Production

```yaml
# docker-compose.dev.yml
services:
  bittorrent-client:
    environment:
      RUST_LOG: debug

# docker-compose.prod.yml
services:
  bittorrent-client:
    environment:
      RUST_LOG: warn
    deploy:
      resources:
        limits:
          memory: 2G
```

---

## Useful Commands

```bash
# Build and run
docker-compose up --build

# View all containers
docker ps -a

# Execute command in container
docker exec rusty-torrents-backend ps aux

# Interactive shell
docker-compose exec bittorrent-client /bin/bash

# Remove everything
docker system prune -a --volumes

# Export image
docker save rusty-torrents:latest | gzip > rusty-torrents.tar.gz

# Import image
gunzip -c rusty-torrents.tar.gz | docker load
```

---

## Support & Resources

- **Docker Docs**: https://docs.docker.com
- **Docker Compose Docs**: https://docs.docker.com/compose
- **Rust Docker**: https://hub.docker.com/_/rust
- **Node Docker**: https://hub.docker.com/_/node

---

## License

This Docker configuration is part of Rusty Torrents and follows the same license terms.

---

**Last Updated**: March 28, 2026
**Compatible With**: Rusty Torrents v0.1.0+, Docker v20.10+, Docker Compose v2.0+
