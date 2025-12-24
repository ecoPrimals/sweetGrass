#!/bin/bash
# 🌾 Setup Local Primal Mesh for SweetGrass Integration Demos
# 
# This script starts real primal binaries from ../../bins/ for integration testing.
# Follows Primal Sovereignty principles: runtime discovery, no hardcoding.

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="$(cd "$SCRIPT_DIR/../../bins" && pwd)"
LOGS_DIR="$SCRIPT_DIR/logs"
PIDS_DIR="$SCRIPT_DIR/pids"

# Create directories
mkdir -p "$LOGS_DIR" "$PIDS_DIR"

echo ""
echo -e "${BLUE}🌾 Setting up local primal mesh for SweetGrass integration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if bins exist
if [ ! -d "$BINS_DIR" ]; then
    echo -e "${RED}❌ Error: Bins directory not found at $BINS_DIR${NC}"
    echo "   Please build phase1 primals first."
    exit 1
fi

# Function to check if a port is in use
port_in_use() {
    lsof -i :"$1" >/dev/null 2>&1
}

# Function to wait for service to be ready
wait_for_service() {
    local name=$1
    local port=$2
    local max_attempts=30
    local attempt=0

    echo -e "${YELLOW}   Waiting for $name to be ready on port $port...${NC}"
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "http://localhost:$port/health" >/dev/null 2>&1; then
            echo -e "${GREEN}   ✅ $name is ready!${NC}"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    echo -e "${RED}   ❌ $name failed to start${NC}"
    return 1
}

# Start BearDog (Signing Service)
echo -e "${BLUE}🐻 Starting BearDog (Signing)...${NC}"
if [ -f "$BINS_DIR/beardog" ]; then
    if port_in_use 8091; then
        echo -e "${YELLOW}   ⚠️  Port 8091 already in use, skipping BearDog${NC}"
    else
        BEARDOG_PORT=8091 \
        PRIMAL_NAME="beardog" \
        PRIMAL_CAPABILITIES="signing" \
        "$BINS_DIR/beardog" > "$LOGS_DIR/beardog.log" 2>&1 &
        echo $! > "$PIDS_DIR/beardog.pid"
        echo -e "${GREEN}   ✅ BearDog started (PID: $(cat "$PIDS_DIR/beardog.pid"))${NC}"
        echo -e "${BLUE}      http://localhost:8091${NC}"
        wait_for_service "BearDog" 8091 || true
    fi
else
    echo -e "${YELLOW}   ⚠️  BearDog binary not found, skipping${NC}"
fi
echo ""

# Start NestGate (Storage Service)
echo -e "${BLUE}🏰 Starting NestGate (Storage)...${NC}"
if [ -f "$BINS_DIR/nestgate" ]; then
    if port_in_use 8092; then
        echo -e "${YELLOW}   ⚠️  Port 8092 already in use, skipping NestGate${NC}"
    else
        NESTGATE_PORT=8092 \
        PRIMAL_NAME="nestgate" \
        PRIMAL_CAPABILITIES="storage" \
        STORAGE_PATH="/tmp/sweetgrass-demo-storage" \
        "$BINS_DIR/nestgate" > "$LOGS_DIR/nestgate.log" 2>&1 &
        echo $! > "$PIDS_DIR/nestgate.pid"
        echo -e "${GREEN}   ✅ NestGate started (PID: $(cat "$PIDS_DIR/nestgate.pid"))${NC}"
        echo -e "${BLUE}      http://localhost:8092${NC}"
        wait_for_service "NestGate" 8092 || true
    fi
else
    echo -e "${YELLOW}   ⚠️  NestGate binary not found, skipping${NC}"
fi
echo ""

# Start Songbird (Discovery/Orchestration)
echo -e "${BLUE}🎵 Starting Songbird (Discovery)...${NC}"
if [ -f "$BINS_DIR/songbird-orchestrator" ]; then
    if port_in_use 8093; then
        echo -e "${YELLOW}   ⚠️  Port 8093 already in use, skipping Songbird${NC}"
    else
        SONGBIRD_PORT=8093 \
        PRIMAL_NAME="songbird" \
        PRIMAL_CAPABILITIES="discovery,orchestration" \
        "$BINS_DIR/songbird-orchestrator" > "$LOGS_DIR/songbird.log" 2>&1 &
        echo $! > "$PIDS_DIR/songbird.pid"
        echo -e "${GREEN}   ✅ Songbird started (PID: $(cat "$PIDS_DIR/songbird.pid"))${NC}"
        echo -e "${BLUE}      http://localhost:8093${NC}"
        wait_for_service "Songbird" 8093 || true
    fi
else
    echo -e "${YELLOW}   ⚠️  Songbird binary not found, skipping${NC}"
fi
echo ""

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Local primal mesh setup complete!${NC}"
echo ""
echo "🎯 Running Services:"
echo ""

if [ -f "$PIDS_DIR/beardog.pid" ]; then
    echo -e "   🐻 BearDog (Signing):     ${BLUE}http://localhost:8091${NC}"
fi

if [ -f "$PIDS_DIR/nestgate.pid" ]; then
    echo -e "   🏰 NestGate (Storage):    ${BLUE}http://localhost:8092${NC}"
fi

if [ -f "$PIDS_DIR/songbird.pid" ]; then
    echo -e "   🎵 Songbird (Discovery):  ${BLUE}http://localhost:8093${NC}"
fi

echo ""
echo "📁 Logs: $LOGS_DIR/"
echo "📝 PIDs: $PIDS_DIR/"
echo ""
echo "🛑 To stop all services:"
echo "   ./teardown-local-primals.sh"
echo ""
echo "🌾 Ready for integration demos!"
echo ""

