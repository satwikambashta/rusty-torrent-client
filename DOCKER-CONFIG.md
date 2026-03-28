# Docker Deployment Checklist & Configuration

## Pre-Deployment Checklist

### Security
- [ ] Set `RUST_LOG=warn` (reduce verbosity in production)
- [ ] Generate SSL/TLS certificates for HTTPS
- [ ] Configure firewall rules (allow only necessary ports)
- [ ] Set up rate limiting in Nginx
- [ ] Configure CORS appropriately
- [ ] Review security headers in nginx.conf

### Performance
- [ ] Set resource limits (CPU/Memory) in docker-compose.yml
- [ ] Configure log rotation to prevent disk space issues
- [ ] Set up health check monitoring
- [ ] Plan backup strategy
- [ ] Test failover procedures

### Operations
- [ ] Set automatic restart policies
- [ ] Configure centralized logging
- [ ] Set up monitoring/alerting
- [ ] Document recovery procedures
- [ ] Test container restarts
- [ ] Verify volume persistence

### Testing
- [ ] Test API endpoints manually
- [ ] Run load tests
- [ ] Verify health checks
- [ ] Test SSL/TLS connectivity
- [ ] Test volume backups
- [ ] Verify file permissions

---

## Environment Configuration Examples

### Development Environment (.env.dev)
```bash
# Logging - verbose for debugging
RUST_LOG=debug

# Downloads location
DOWNLOADS_DIR=/app/downloads

# API Configuration
API_HOST=localhost
API_PORT=8080

# Frontend
FRONTEND_URL=http://localhost:3000
```

### Staging Environment (.env.staging)
```bash
# Logging - normal verbosity
RUST_LOG=info

# Downloads location
DOWNLOADS_DIR=/data/torrents

# API Configuration
API_HOST=staging.example.com
API_PORT=8080

# Frontend
FRONTEND_URL=https://staging.example.com

# Security
ENABLE_HTTPS=true
CERTIFICATE_PATH=/etc/nginx/cert/cert.pem
```

### Production Environment (.env.prod)
```bash
# Logging - errors only
RUST_LOG=warn

# Downloads location - external volume
DOWNLOADS_DIR=/mnt/nfs/torrents

# API Configuration
API_HOST=api.example.com
API_PORT=8080

# Frontend
FRONTEND_URL=https://example.com

# Security
ENABLE_HTTPS=true
CERTIFICATE_PATH=/etc/letsencrypt/live/example.com/fullchain.pem

# Performance
MAX_MEMORY=2G
MAX_CPU=2

# Monitoring
SENTRY_DSN=https://xxxxx@sentry.io/xxxx
```

---

## Docker Compose Override Files

### Development Override (docker-compose.dev.yml)
```yaml
version: '3.8'

services:
  bittorrent-client:
    build:
      context: .
      dockerfile: Dockerfile
      target: backend-builder  # Optional: build specific target
    
    environment:
      RUST_LOG: debug
      DEBUG: "true"
    
    volumes:
      - ./src-tauri:/build/src-tauri  # Live code reload
      - ./web:/build/web
    
    ports:
      - "8080:8080"
      - "6881:6881"
    
    healthcheck:
      disable: true  # Disable for development
```

Usage:
```bash
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up
```

### Production Override (docker-compose.prod.yml)
```yaml
version: '3.8'

services:
  bittorrent-client:
    restart: always
    
    environment:
      RUST_LOG: warn
    
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 512M
    
    logging:
      driver: json-file
      options:
        max-size: 10m
        max-file: 3
    
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
      interval: 1m
      timeout: 10s
      retries: 3
      start_period: 30s

  nginx:
    restart: always
    
    environment:
      ENABLE_GZIP: "true"
      GZIP_LEVEL: 6
```

Usage:
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

---

## Resource Configuration

### Minimal (1GB RAM)
```yaml
deploy:
  resources:
    limits:
      cpus: '0.5'
      memory: 512M
```

### Standard (2GB RAM, 2 cores)
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

### High Performance (4GB+ RAM, 4+ cores)
```yaml
deploy:
  resources:
    limits:
      cpus: '4'
      memory: 4G
    reservations:
      cpus: '2'
      memory: 2G
```

---

## SSL/TLS Certificate Setup

### Self-Signed (Development)
```bash
# Create cert directory
mkdir -p cert

# Generate certificate (valid 365 days)
openssl req -x509 -newkey rsa:4096 \
  -keyout cert/key.pem \
  -out cert/cert.pem \
  -days 365 \
  -nodes \
  -subj "/C=US/ST=State/L=City/O=Org/CN=localhost"

# Set permissions
chmod 600 cert/key.pem
chmod 644 cert/cert.pem
```

### Let's Encrypt (Production)
```bash
# Install Certbot
sudo apt-get install certbot

# Generate certificate
sudo certbot certonly --standalone \
  -d yourdomain.com \
  -d www.yourdomain.com

# Copy to project
cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./cert/cert.pem
cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ./cert/key.pem

# Auto-renewal (Certbot handles this automatically)
# Verify: sudo systemctl status certbot.timer
```

### Certificate Renewal
```bash
# Manual renewal
sudo certbot renew --force-renewal

# Copy updated certificates
cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./cert/cert.pem
cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ./cert/key.pem

# Restart Nginx
docker-compose restart nginx
```

---

## Backup & Recovery

### Automated Backup Script (backup.sh)
```bash
#!/bin/bash

BACKUP_DIR="/backup/rusty-torrents"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup downloads
echo "Backing up downloads..."
docker run --rm \
  -v rusty-torrents_downloads:/data \
  -v "$BACKUP_DIR":/backup \
  debian:bookworm-slim \
  bash -c "tar -czf /backup/downloads_$TIMESTAMP.tar.gz /data"

# Backup config
echo "Backing up configuration..."
docker run --rm \
  -v rusty-torrents_config:/data \
  -v "$BACKUP_DIR":/backup \
  debian:bookworm-slim \
  bash -c "tar -czf /backup/config_$TIMESTAMP.tar.gz /data"

# Cleanup old backups (keep 7 days)
find "$BACKUP_DIR" -type f -mtime +7 -delete

echo "Backup complete: $BACKUP_DIR/downloads_$TIMESTAMP.tar.gz"
echo "Backup complete: $BACKUP_DIR/config_$TIMESTAMP.tar.gz"
```

Schedule with cron:
```bash
# Edit crontab
crontab -e

# Add (daily backup at 2 AM)
0 2 * * * /path/to/backup.sh
```

### Restore Procedure
```bash
# Stop services
docker-compose down

# Restore downloads
docker run --rm \
  -v rusty-torrents_downloads:/data \
  -v "$BACKUP_DIR":/backup \
  debian:bookworm-slim \
  bash -c "cd /data && tar -xzf /backup/downloads_*.tar.gz --strip-components=1"

# Restore config
docker run --rm \
  -v rusty-torrents_config:/data \
  -v "$BACKUP_DIR":/backup \
  debian:bookworm-slim \
  bash -c "cd /data && tar -xzf /backup/config_*.tar.gz --strip-components=1"

# Start services
docker-compose up -d
```

---

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Build and Push Docker Image

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2
      
      - name: Login to Docker Hub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}
      
      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: |
            ${{ secrets.DOCKER_USERNAME }}/rusty-torrents:latest
            ${{ secrets.DOCKER_USERNAME }}/rusty-torrents:${{ github.ref_name }}
          cache-from: type=registry,ref=${{ secrets.DOCKER_USERNAME }}/rusty-torrents:buildcache
          cache-to: type=registry,ref=${{ secrets.DOCKER_USERNAME }}/rusty-torrents:buildcache,mode=max
```

---

## Monitoring & Logging

### Prometheus Metrics
```yaml
# docker-compose.yml - Add metrics collector
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
```

### Log Aggregation
```yaml
# docker-compose.yml - Add ELK stack
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.0.0
    environment:
      - discovery.type=single-node
    ports:
      - "9200:9200"

  kibana:
    image: docker.elastic.co/kibana/kibana:8.0.0
    ports:
      - "5601:5601"
```

---

## Troubleshooting Guide

### Container Won't Start
```bash
# Check logs
docker-compose logs bittorrent-client

# Check disk space
docker system df

# Check port conflicts
sudo lsof -i :8080
sudo lsof -i :6881
```

### High Memory Usage
```bash
# Monitor resource usage
docker stats

# Check what's using memory
docker exec rusty-torrents-backend top -b -n1 | head -20

# Reduce RUST_LOG verbosity
# Update docker-compose.yml: RUST_LOG=warn
```

### API Not Responding
```bash
# Test API endpoint
curl -v http://localhost:8080/api/health

# Check from container
docker-compose exec bittorrent-client curl http://localhost:8080/api/health

# Check network
docker network inspect rusty-torrents_default
```

---

## Additional Resources

- **Docker Documentation**: https://docs.docker.com
- **Docker Security Best Practices**: https://docs.docker.com/engine/security
- **Nginx Documentation**: https://nginx.org/en/docs
- **SSL/TLS Best Practices**: https://mozilla.github.io/serverless-best-practices

---

**Last Updated**: March 28, 2026
