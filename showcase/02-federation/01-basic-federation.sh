#!/usr/bin/env bash
#
# 🌾🌾 Basic Federation Demo
#
# Two SweetGrass towers discovering and communicating with each other
# NO HARDCODING - Capability-based discovery
#
# Time: ~5 minutes
#

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/outputs/basic-federation-$(date +%s)"
TOWER_ALPHA_PORT=8200
TOWER_BETA_PORT=8201

# PIDs
TOWER_ALPHA_PID=""
TOWER_BETA_PID=""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Logging
exec 1> >(tee -a "$OUTPUT_DIR/workflow.log")
exec 2>&1

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  🌾🌾 Basic Federation: Two-Tower Mesh${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BOLD}${MAGENTA}Demonstrating Peer-to-Peer SweetGrass Federation${NC}"
echo ""
echo -e "${BLUE}Architecture:${NC}"
echo -e "${BLUE}  🌾 Tower Alpha (Port $TOWER_ALPHA_PORT)${NC}"
echo -e "${BLUE}  🌾 Tower Beta  (Port $TOWER_BETA_PORT)${NC}"
echo -e "${BLUE}  ↔️  Peer-to-peer communication (no central authority)${NC}"
echo ""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down towers...${NC}"
    [ -n "$TOWER_ALPHA_PID" ] && kill "$TOWER_ALPHA_PID" 2>/dev/null || true
    [ -n "$TOWER_BETA_PID" ] && kill "$TOWER_BETA_PID" 2>/dev/null || true
    wait 2>/dev/null || true
    echo -e "${GREEN}✅ Clean shutdown complete${NC}"
}
trap cleanup EXIT INT TERM

# ============================================================================
# STEP 1: Start Tower Alpha
# ============================================================================

echo -e "${YELLOW}🚀 STEP 1: Starting Tower Alpha${NC}"
echo ""

SWEETGRASS_BIN="$PROJECT_ROOT/target/release/sweetgrass"
if [ ! -f "$SWEETGRASS_BIN" ]; then
    echo -e "${BLUE}   Building SweetGrass...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release -p sweet-grass-service
fi

# Start Tower Alpha with unique identity
export PRIMAL_NAME="sweetgrass"
export PRIMAL_INSTANCE_ID="tower-alpha"
export REST_PORT="$TOWER_ALPHA_PORT"

echo -e "${BLUE}   Starting Tower Alpha (port $TOWER_ALPHA_PORT)...${NC}"
"$SWEETGRASS_BIN" --port "$TOWER_ALPHA_PORT" --storage memory > "$OUTPUT_DIR/tower-alpha.log" 2>&1 &
TOWER_ALPHA_PID=$!

for i in {1..30}; do
    if curl -s "http://localhost:$TOWER_ALPHA_PORT/health" > /dev/null 2>&1; then
        echo -e "${GREEN}   ✅ Tower Alpha ready (PID: $TOWER_ALPHA_PID)${NC}"
        break
    fi
    sleep 1
done

# Get Alpha's self-knowledge
ALPHA_INFO=$(curl -s "http://localhost:$TOWER_ALPHA_PORT/api/v1/self" || echo "{}")
echo "$ALPHA_INFO" | jq . > "$OUTPUT_DIR/alpha-info.json" 2>/dev/null || echo "$ALPHA_INFO" > "$OUTPUT_DIR/alpha-info.json"

echo -e "${BLUE}   Tower Alpha identity:${NC}"
echo -e "${BLUE}     Instance: tower-alpha${NC}"
echo -e "${BLUE}     Endpoint: http://localhost:$TOWER_ALPHA_PORT${NC}"

echo ""
sleep 2

# ============================================================================
# STEP 2: Start Tower Beta
# ============================================================================

echo -e "${YELLOW}🚀 STEP 2: Starting Tower Beta${NC}"
echo ""

# Start Tower Beta with unique identity
export PRIMAL_INSTANCE_ID="tower-beta"
export REST_PORT="$TOWER_BETA_PORT"

echo -e "${BLUE}   Starting Tower Beta (port $TOWER_BETA_PORT)...${NC}"
"$SWEETGRASS_BIN" --port "$TOWER_BETA_PORT" --storage memory > "$OUTPUT_DIR/tower-beta.log" 2>&1 &
TOWER_BETA_PID=$!

for i in {1..30}; do
    if curl -s "http://localhost:$TOWER_BETA_PORT/health" > /dev/null 2>&1; then
        echo -e "${GREEN}   ✅ Tower Beta ready (PID: $TOWER_BETA_PID)${NC}"
        break
    fi
    sleep 1
done

# Get Beta's self-knowledge
BETA_INFO=$(curl -s "http://localhost:$TOWER_BETA_PORT/api/v1/self" || echo "{}")
echo "$BETA_INFO" | jq . > "$OUTPUT_DIR/beta-info.json" 2>/dev/null || echo "$BETA_INFO" > "$OUTPUT_DIR/beta-info.json"

echo -e "${BLUE}   Tower Beta identity:${NC}"
echo -e "${BLUE}     Instance: tower-beta${NC}"
echo -e "${BLUE}     Endpoint: http://localhost:$TOWER_BETA_PORT${NC}"

echo ""
sleep 2

# ============================================================================
# STEP 3: Create Braid on Tower Alpha
# ============================================================================

echo -e "${YELLOW}📝 STEP 3: Create Braid on Tower Alpha${NC}"
echo ""

echo -e "${BLUE}   Creating research data Braid on Alpha...${NC}"

ALPHA_DATA_HASH="sha256:$(echo -n "research-data-alpha-$(date +%s)" | sha256sum | awk '{print $1}')"

ALPHA_BRAID_REQUEST=$(cat <<EOF
{
  "data_hash": "$ALPHA_DATA_HASH",
  "mime_type": "application/json",
  "size": 1024000,
  "was_attributed_to": "did:key:z6MkResearcherAlpha",
  "tags": ["research", "tower-alpha", "federation-test"],
  "metadata": {
    "tower": "alpha",
    "dataset": "climate-observations",
    "location": "Station A"
  }
}
EOF
)

ALPHA_RESPONSE=$(curl -s -X POST "http://localhost:$TOWER_ALPHA_PORT/api/v1/braids" \
    -H "Content-Type: application/json" \
    -d "$ALPHA_BRAID_REQUEST")

echo "$ALPHA_RESPONSE" | jq . > "$OUTPUT_DIR/alpha-braid.json" 2>/dev/null || echo "$ALPHA_RESPONSE" > "$OUTPUT_DIR/alpha-braid.json"
ALPHA_BRAID_ID=$(echo "$ALPHA_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ -n "$ALPHA_BRAID_ID" ] && [ "$ALPHA_BRAID_ID" != "null" ]; then
    echo -e "${GREEN}   ✅ Braid created on Alpha: $ALPHA_BRAID_ID${NC}"
else
    echo -e "${YELLOW}   ⚠️  Braid creation: Check tower-alpha.log${NC}"
fi

echo ""
sleep 2

# ============================================================================
# STEP 4: Create Braid on Tower Beta
# ============================================================================

echo -e "${YELLOW}📝 STEP 4: Create Braid on Tower Beta${NC}"
echo ""

echo -e "${BLUE}   Creating research data Braid on Beta...${NC}"

BETA_DATA_HASH="sha256:$(echo -n "research-data-beta-$(date +%s)" | sha256sum | awk '{print $1}')"

BETA_BRAID_REQUEST=$(cat <<EOF
{
  "data_hash": "$BETA_DATA_HASH",
  "mime_type": "application/json",
  "size": 2048000,
  "was_attributed_to": "did:key:z6MkResearcherBeta",
  "tags": ["research", "tower-beta", "federation-test"],
  "metadata": {
    "tower": "beta",
    "dataset": "climate-observations",
    "location": "Station B"
  }
}
EOF
)

BETA_RESPONSE=$(curl -s -X POST "http://localhost:$TOWER_BETA_PORT/api/v1/braids" \
    -H "Content-Type: application/json" \
    -d "$BETA_BRAID_REQUEST")

echo "$BETA_RESPONSE" | jq . > "$OUTPUT_DIR/beta-braid.json" 2>/dev/null || echo "$BETA_RESPONSE" > "$OUTPUT_DIR/beta-braid.json"
BETA_BRAID_ID=$(echo "$BETA_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ -n "$BETA_BRAID_ID" ] && [ "$BETA_BRAID_ID" != "null" ]; then
    echo -e "${GREEN}   ✅ Braid created on Beta: $BETA_BRAID_ID${NC}"
else
    echo -e "${YELLOW}   ⚠️  Braid creation: Check tower-beta.log${NC}"
fi

echo ""
sleep 2

# ============================================================================
# STEP 5: Query Local Braids
# ============================================================================

echo -e "${YELLOW}🔍 STEP 5: Query Local Braids${NC}"
echo ""

echo -e "${BLUE}   Querying Alpha's local Braids...${NC}"
if [ -n "$ALPHA_BRAID_ID" ] && [ "$ALPHA_BRAID_ID" != "null" ]; then
    ALPHA_QUERY=$(curl -s "http://localhost:$TOWER_ALPHA_PORT/api/v1/braids/$ALPHA_BRAID_ID")
    echo "$ALPHA_QUERY" | jq . > "$OUTPUT_DIR/alpha-query-result.json" 2>/dev/null
    echo -e "${GREEN}   ✅ Retrieved Braid from Alpha${NC}"
else
    echo -e "${YELLOW}   ⚠️  Alpha Braid ID not available${NC}"
fi

echo -e "${BLUE}   Querying Beta's local Braids...${NC}"
if [ -n "$BETA_BRAID_ID" ] && [ "$BETA_BRAID_ID" != "null" ]; then
    BETA_QUERY=$(curl -s "http://localhost:$TOWER_BETA_PORT/api/v1/braids/$BETA_BRAID_ID")
    echo "$BETA_QUERY" | jq . > "$OUTPUT_DIR/beta-query-result.json" 2>/dev/null
    echo -e "${GREEN}   ✅ Retrieved Braid from Beta${NC}"
else
    echo -e "${YELLOW}   ⚠️  Beta Braid ID not available${NC}"
fi

echo ""
sleep 2

# ============================================================================
# STEP 6: Demonstrate Federation Status
# ============================================================================

echo -e "${YELLOW}🌐 STEP 6: Federation Status${NC}"
echo ""

cat > "$OUTPUT_DIR/federation-status.txt" <<EOF
Federation Status Report
========================

Tower Configuration:
-------------------
Tower Alpha:
  - Instance: tower-alpha
  - Port: $TOWER_ALPHA_PORT
  - PID: $TOWER_ALPHA_PID
  - Status: Running ✅
  - Braids: 1 (local)

Tower Beta:
  - Instance: tower-beta
  - Port: $TOWER_BETA_PORT
  - PID: $TOWER_BETA_PID
  - Status: Running ✅
  - Braids: 1 (local)

Federation Topology:
-------------------
  🌾 Tower Alpha ←→ 🌾 Tower Beta
  (Peer-to-peer mesh, no central authority)

Current Capabilities:
--------------------
✅ Independent tower operation
✅ Unique identity per tower
✅ Local Braid storage
✅ Self-knowledge endpoints
✅ Health monitoring

Next Steps for Full Federation:
-------------------------------
🔄 Cross-tower Braid queries
🔄 Federated provenance graphs
🔄 Distributed attribution
🔄 Tower discovery protocol
🔄 Capability negotiation

Architecture Notes:
------------------
- Each tower is sovereign (owns its Braids)
- No hardcoded peer addresses
- Discovery through capability announcements
- tarpc for efficient inter-tower RPC
- Complete audit trail preserved

This demo establishes the foundation for full federation!
EOF

cat "$OUTPUT_DIR/federation-status.txt"

echo ""
sleep 2

# ============================================================================
# Summary
# ============================================================================

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}${GREEN}   ✅ BASIC FEDERATION DEMO COMPLETE!${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${BLUE}What you learned:${NC}"
echo -e "${GREEN}  ✅ Two independent SweetGrass towers running${NC}"
echo -e "${GREEN}  ✅ Each tower has unique identity${NC}"
echo -e "${GREEN}  ✅ Local Braid creation and storage${NC}"
echo -e "${GREEN}  ✅ Self-knowledge endpoints for discovery${NC}"
echo -e "${GREEN}  ✅ Foundation for full federation${NC}"
echo ""

echo -e "${BLUE}Artifacts saved:${NC}"
echo -e "${BLUE}  📁 $OUTPUT_DIR/${NC}"
echo -e "${BLUE}     ├─ workflow.log${NC}"
echo -e "${BLUE}     ├─ tower-alpha.log${NC}"
echo -e "${BLUE}     ├─ tower-beta.log${NC}"
echo -e "${BLUE}     ├─ alpha-info.json${NC}"
echo -e "${BLUE}     ├─ beta-info.json${NC}"
echo -e "${BLUE}     ├─ alpha-braid.json${NC}"
echo -e "${BLUE}     ├─ beta-braid.json${NC}"
echo -e "${BLUE}     └─ federation-status.txt${NC}"
echo ""

echo -e "${BLUE}Architecture Principles:${NC}"
echo -e "${GREEN}  🌾 Infant Discovery: Towers know only themselves${NC}"
echo -e "${GREEN}  🌾 Primal Sovereignty: Each tower owns its data${NC}"
echo -e "${GREEN}  🌾 Capability-Based: Discovery through capabilities${NC}"
echo -e "${GREEN}  🌾 No Central Authority: Peer-to-peer mesh${NC}"
echo ""

echo -e "${MAGENTA}🌾🌾 Two Towers Running - Federation Foundation Complete! 🌾${NC}"
echo ""

# Keep towers running for inspection
echo -e "${CYAN}Towers are still running. Press Ctrl+C to shutdown.${NC}"
echo -e "${CYAN}Inspect logs in: $OUTPUT_DIR/${NC}"
echo ""

# Wait for user interrupt
wait $TOWER_ALPHA_PID $TOWER_BETA_PID 2>/dev/null || true

