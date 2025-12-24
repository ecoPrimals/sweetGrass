#!/bin/bash
# 🌾 Teardown Local Primal Mesh
# 
# Cleanly stops all primal services started by setup-local-primals.sh

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PIDS_DIR="$SCRIPT_DIR/pids"

echo ""
echo -e "${BLUE}🌾 Tearing down local primal mesh${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Function to stop a service
stop_service() {
    local name=$1
    local pid_file="$PIDS_DIR/$2.pid"

    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            echo -e "${YELLOW}🛑 Stopping $name (PID: $pid)...${NC}"
            kill "$pid" 2>/dev/null || true
            sleep 1
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                echo -e "${YELLOW}   Force killing $name...${NC}"
                kill -9 "$pid" 2>/dev/null || true
            fi
            
            echo -e "${GREEN}   ✅ $name stopped${NC}"
        else
            echo -e "${YELLOW}   ⚠️  $name not running (PID $pid not found)${NC}"
        fi
        rm "$pid_file"
    else
        echo -e "${YELLOW}   ⚠️  $name PID file not found${NC}"
    fi
}

# Stop all services
stop_service "BearDog" "beardog"
stop_service "NestGate" "nestgate"
stop_service "Songbird" "songbird"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All services stopped${NC}"
echo ""
echo "🧹 Cleanup:"
echo "   • PIDs removed from $PIDS_DIR/"
echo "   • Logs preserved in logs/"
echo ""
echo "🌾 Primal mesh teardown complete!"
echo ""

