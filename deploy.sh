#!/bin/bash
# SweetGrass Production Deployment Script
# Version: v0.7.13
# Date: March 15, 2026
# Status: PRODUCTION READY

set -e  # Exit on error

echo "🌾 SweetGrass Production Deployment"
echo "===================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration — all from env vars (capability-based, no hardcoding)
PORT="${SWEETGRASS_HTTP_PORT:-${1:-8080}}"
BACKEND="${STORAGE_BACKEND:-${2:-redb}}"

echo -e "${BLUE}Configuration:${NC}"
echo "  Port: $PORT"
echo "  Storage Backend: $BACKEND"
echo ""

# Verify binary exists
if [ ! -f "target/release/sweetgrass" ]; then
    echo -e "${YELLOW}Binary not found. Building release...${NC}"
    cargo build --release
    echo ""
fi

# Verify environment
echo -e "${BLUE}Verifying environment...${NC}"

if [ "$BACKEND" = "postgres" ]; then
    if [ -z "$DATABASE_URL" ]; then
        echo -e "${YELLOW}⚠️  DATABASE_URL not set. Using default.${NC}"
        export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/sweetgrass"
        echo "  Set: $DATABASE_URL"
    else
        echo "  ✅ DATABASE_URL configured"
    fi
fi

if [ -z "$PRIMAL_NAME" ]; then
    export PRIMAL_NAME="sweetgrass"
    echo "  Set PRIMAL_NAME: $PRIMAL_NAME"
else
    echo "  ✅ PRIMAL_NAME: $PRIMAL_NAME"
fi

echo ""

# Health check function
check_health() {
    local retries=5
    local delay=2
    
    for i in $(seq 1 $retries); do
        if curl -s "http://localhost:$PORT/health" > /dev/null 2>&1; then
            return 0
        fi
        if [ $i -lt $retries ]; then
            sleep $delay
        fi
    done
    return 1
}

# Start service
echo -e "${GREEN}🚀 Starting SweetGrass service...${NC}"
echo ""

STORAGE_BACKEND="$BACKEND" \
SWEETGRASS_HTTP_ADDRESS="0.0.0.0:$PORT" \
./target/release/sweetgrass server &

SERVICE_PID=$!

echo "  PID: $SERVICE_PID"
echo "  Port: $PORT"
echo ""

# Wait for service to be ready
echo -e "${BLUE}Waiting for service to be ready...${NC}"
if check_health; then
    echo -e "${GREEN}✅ Service is healthy!${NC}"
    echo ""
    
    # Display service info
    echo -e "${BLUE}Service Information:${NC}"
    curl -s "http://localhost:$PORT/health/detailed" | jq '.' 2>/dev/null || \
        curl -s "http://localhost:$PORT/health"
    echo ""
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}SweetGrass is running!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Endpoints:"
    echo "  Health:      http://localhost:$PORT/health"
    echo "  API:         http://localhost:$PORT/api/v1"
    echo "  Braids:      http://localhost:$PORT/api/v1/braids"
    echo "  Provenance:  http://localhost:$PORT/api/v1/provenance"
    echo ""
    echo "To stop: kill $SERVICE_PID"
    echo "To monitor: journalctl -u sweetgrass -f  (systemd) or check stderr"
    echo ""
    
    # Keep running
    wait $SERVICE_PID
else
    echo -e "${YELLOW}⚠️  Service failed health check${NC}"
    kill $SERVICE_PID 2>/dev/null || true
    exit 1
fi

