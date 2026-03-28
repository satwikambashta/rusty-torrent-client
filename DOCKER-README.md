# 🐳 Docker Setup - Quick Reference

This directory contains everything needed to run Rusty Torrents in Docker containers.

## Files Overview

| File | Purpose |
|------|---------|
| **Dockerfile** | Multi-stage build for web client + Rust backend |
| **docker-compose.yml** | Orchestrates backend + Nginx services |
| **.dockerignore** | Excludes unnecessary files from build context |
| **docker-quickstart.sh** | Interactive CLI for managing services |
| **DOCKER.md** | Comprehensive deployment & operations guide |
| **DOCKER-CONFIG.md** | Configuration examples & deployment checklists |

## Quick Start

### Option 1: Interactive Menu (Recommended)
```bash
bash docker-quickstart.sh
```
This launches an interactive menu to start/stop services, view logs, etc.

### Option 2: Docker Compose Direct
```bash
docker-compose up        # Start services (foreground)
docker-compose up -d     # Start services (background)
docker-compose down      # Stop services
```

### Option 3: Docker CLI Manual
```bash
docker build -t rusty-torrents .
docker run -p 8080:8080 -v downloads:/app/downloads rusty-torrents
```

## Architecture

### Services & Ports

```
┌─────────────────────────────────────────────┐
│         Docker Compose Network              │
│                                             │
│  ┌──────────────┐      ┌──────────────┐   │
│  │    Nginx     │◄─────┤   Backend    │   │
│  │ (Reverse     │      │   (API)      │   │
│  │  Proxy)      │      │              │   │
│  │              │      │ :8080        │   │
│  │ :80 :443     │      │ :6881 (BT)   │   │
│  └──────────────┘      └──────────────┘   │
│                                             │
│    ┌────────────────────────────────┐     │
│    │   Persistent Volumes           │     │
│    │  - downloads/                  │     │
│    │  - logs/                       │     │
│    │  - config/                     │     │
│    └────────────────────────────────┘     │
└─────────────────────────────────────────────┘
```

### Ports

- **6881** (TCP/UDP): BitTorrent DHT protocol
- **8080** (TCP): REST API backend
- **80** (TCP): HTTP (redirects to HTTPS)
- **443** (TCP): HTTPS (requires SSL certificates)

## Configuration

### Environment Variables

Edit `docker-compose.yml` to change:

```yaml
environment:
  RUST_LOG: info              # Change to: trace, debug, info, warn, error
  DOWNLOADS_DIR: /app/downloads
```

### Resource Limits

Add to `docker-compose.yml` under `bittorrent-client`:

```yaml
deploy:
  resources:
    limits:
      cpus: '2'
      memory: 2G
    reservations:
      cpus: '1'
      memory: 1G
```

### Bind Volumes to Local Directory

To persist downloads locally:

```bash
# Create local directory
mkdir -p /home/user/torrents

# Update docker-compose.yml volumes:
# volumes:
#   downloads:
#     driver: local
#     driver_opts:
#       type: none
#       o: bind
#       device: /home/user/torrents
```

## Common Tasks

### View Service Status
```bash
docker-compose ps
```

### Check Service Health
```bash
curl http://localhost:8080/api/health
```

### View Logs
```bash
docker-compose logs -f                    # All services
docker-compose logs -f bittorrent-client  # Backend only
docker-compose logs -f nginx              # Nginx only
```

### Monitor Resources
```bash
docker stats rusty-torrents-backend
```

### Execute Command in Container
```bash
docker-compose exec bittorrent-client /bin/bash
```

### Restart Services
```bash
docker-compose restart
docker-compose restart bittorrent-client  # Specific service
```

## Docker Build Details

### Multi-Stage Build Process

**Stage 1: Web Builder** (node:20-alpine)
- Compiles React/TypeScript web client
- Output: `/app/www` (static HTML/CSS/JS)

**Stage 2: Backend Builder** (rust:latest)
- Compiles Rust backend binary
- Output: `/app/bittorrent-client`

**Stage 3: Runtime** (debian:bookworm-slim)
- Combines both artifacts
- Minimal ~150MB final image

### Build Command
```bash
# Standard build
docker-compose build

# No cache (clean rebuild)
docker-compose build --no-cache

# Build specific service
docker-compose build bittorrent-client
```

## SSL/TLS Setup

### Development (Self-Signed)
```bash
mkdir -p cert
openssl req -x509 -newkey rsa:4096 \
  -keyout cert/key.pem -out cert/cert.pem \
  -days 365 -nodes \
  -subj "/C=US/ST=State/L=City/O=Org/CN=localhost"

docker-compose up
```

### Production (Let's Encrypt)
```bash
# Install Certbot
sudo apt-get install certbot

# Generate certificate
sudo certbot certonly --standalone -d yourdomain.com

# Copy to project
cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./cert/cert.pem
cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ./cert/key.pem

docker-compose up -d
```

## Troubleshooting

### Container Won't Start
```bash
docker-compose logs bittorrent-client
```

### Port Already in Use
```bash
# Find process using port 8080
sudo lsof -i :8080

# Kill process
sudo kill -9 <PID>

# Or change port in docker-compose.yml:
# ports:
#   - "9090:8080"
```

### API Not Responding
```bash
# Test from host
curl http://localhost:8080/api/health

# Test from container
docker-compose exec bittorrent-client curl http://localhost:8080/api/health
```

### High Memory Usage
```bash
# Monitor memory
docker stats

# Reduce logging verbosity in docker-compose.yml:
# RUST_LOG: warn
```

## Documentation Links

- **[DOCKER.md](DOCKER.md)** - Comprehensive deployment guide
- **[DOCKER-CONFIG.md](DOCKER-CONFIG.md)** - Configuration examples & checklists
- **[docker-compose.yml](docker-compose.yml)** - Service configuration
- **[Dockerfile](Dockerfile)** - Build instructions

## Common Commands Cheat Sheet

```bash
# Start/Stop
docker-compose up                    # Start (foreground)
docker-compose up -d                 # Start (background)
docker-compose down                  # Stop & remove containers
docker-compose down --volumes        # Stop & remove everything

# Build
docker-compose build                 # Build images
docker-compose build --no-cache      # Clean rebuild

# Management
docker-compose ps                    # List containers
docker-compose logs -f               # View logs
docker-compose exec service bash     # Shell access
docker-compose restart               # Restart services
docker-compose pause                 # Pause services
docker-compose unpause               # Resume services

# Cleanup
docker system df                     # Disk usage
docker system prune -a               # Remove unused images/containers
docker volume prune                  # Remove unused volumes

# Stats
docker stats                         # Real-time resource usage
docker exec api curl http://localhost:8080/api/health  # Health check
```

## Resource Requirements

### Minimum
- CPU: 2 cores
- RAM: 1 GB
- Disk: 2 GB (base) + storage for torrents

### Recommended
- CPU: 4 cores
- RAM: 4 GB
- Disk: 10 GB free

## Support

For detailed documentation, see:
- [DOCKER.md](DOCKER.md) - Full deployment guide
- [DOCKER-CONFIG.md](DOCKER-CONFIG.md) - Configuration & troubleshooting
- [Docker Docs](https://docs.docker.com) - Official Docker documentation

---

**Last Updated**: March 28, 2026  
**Compatible With**: Docker v20.10+, Docker Compose v2.0+
