#!/bin/bash
# ============================================================================
# Quick Start Script for Docker Deployment
# ============================================================================
# This script helps you quickly set up Rusty Torrents in Docker
# Usage: bash docker-quickstart.sh [start|stop|logs|clean]
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${BLUE}================================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Check prerequisites
check_requirements() {
    print_header "Checking Prerequisites"
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        echo "Install from: https://docs.docker.com/get-docker/"
        exit 1
    fi
    print_success "Docker installed: $(docker --version)"
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed"
        echo "Install from: https://docs.docker.com/compose/install/"
        exit 1
    fi
    print_success "Docker Compose installed: $(docker-compose --version)"
}

# Start services
start_services() {
    print_header "Starting Services"
    
    # Check if volumes exist
    if [ ! -d "./downloads" ]; then
        print_info "Creating downloads directory..."
        mkdir -p ./downloads
    fi
    
    if [ ! -d "./logs" ]; then
        print_info "Creating logs directory..."
        mkdir -p ./logs
    fi
    
    if [ ! -d "./config" ]; then
        print_info "Creating config directory..."
        mkdir -p ./config
    fi
    
    print_info "Building Docker images... (first time may take 5-10 minutes)"
    docker-compose up -d
    
    # Wait for services to be ready
    print_info "Waiting for services to start..."
    sleep 10
    
    # Check health
    if docker-compose ps | grep -q healthy; then
        print_success "Services started successfully!"
    else
        print_error "Some services may not be ready yet"
    fi
    
    print_header "Access Points"
    echo -e "  ${BLUE}API Backend:${NC}        http://localhost:8080"
    echo -e "  ${BLUE}Health Check:${NC}       http://localhost:8080/api/health"
    echo -e "  ${BLUE}Web UI:${NC}             http://localhost (when Nginx is configured)"
    echo ""
    echo -e "  ${YELLOW}Container Status:${NC}"
    docker-compose ps
}

# Stop services
stop_services() {
    print_header "Stopping Services"
    docker-compose stop
    print_success "Services stopped"
}

# View logs
view_logs() {
    print_header "Service Logs (Press Ctrl+C to exit)"
    docker-compose logs -f "$1"
}

# Clean up everything
cleanup() {
    print_header "Cleanup"
    read -p "Are you sure? This will delete all containers and volumes. (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        docker-compose down --volumes
        print_success "Cleanup complete"
    else
        print_info "Cleanup cancelled"
    fi
}

# Rebuild images
rebuild() {
    print_header "Rebuilding Docker Images"
    docker-compose build --no-cache
    print_success "Rebuild complete"
}

# Main menu
show_menu() {
    echo ""
    echo "Rusty Torrents Docker Manager"
    echo "============================="
    echo "1) Start services"
    echo "2) Stop services"
    echo "3) View logs (backend)"
    echo "4) View logs (nginx)"
    echo "5) Rebuild images"
    echo "6) Clean up (remove containers + volumes)"
    echo "7) Status"
    echo "8) Exit"
    echo ""
    read -p "Select option [1-8]: " choice
    
    case $choice in
        1)
            check_requirements
            start_services
            ;;
        2)
            stop_services
            ;;
        3)
            view_logs bittorrent-client
            ;;
        4)
            view_logs nginx
            ;;
        5)
            check_requirements
            rebuild
            ;;
        6)
            cleanup
            ;;
        7)
            echo ""
            docker-compose ps
            echo ""
            docker stats --no-stream
            ;;
        8)
            print_info "Goodbye!"
            exit 0
            ;;
        *)
            print_error "Invalid option"
            ;;
    esac
}

# Command line interface
if [ $# -eq 0 ]; then
    # Interactive mode
    check_requirements
    while true; do
        show_menu
    done
else
    # Non-interactive mode
    case "$1" in
        start)
            check_requirements
            start_services
            ;;
        stop)
            stop_services
            ;;
        logs)
            if [ -z "$2" ]; then
                view_logs bittorrent-client
            else
                view_logs "$2"
            fi
            ;;
        rebuild)
            check_requirements
            rebuild
            ;;
        clean)
            cleanup
            ;;
        status)
            docker-compose ps
            echo ""
            docker stats --no-stream
            ;;
        *)
            echo "Usage: $0 [start|stop|logs|rebuild|clean|status]"
            echo ""
            echo "Commands:"
            echo "  start       Start all services"
            echo "  stop        Stop all services"
            echo "  logs        View service logs"
            echo "  rebuild     Rebuild Docker images"
            echo "  clean       Remove containers and volumes"
            echo "  status      Show service status and resource usage"
            echo ""
            echo "Run with no arguments for interactive menu"
            exit 1
            ;;
    esac
fi
