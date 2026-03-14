#!/usr/bin/env bash
#
# 🌾 SweetGrass + BearDog: REAL Integration Demo
# 
# This demo starts BOTH services and makes REAL API calls
# No mocks - this will expose actual integration gaps!
#

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="$(cd "$PROJECT_ROOT/../bins" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/outputs/real-integration-$(date +%s)"
GAPS_FILE="$OUTPUT_DIR/INTEGRATION_GAPS_DISCOVERED.md"

mkdir -p "$OUTPUT_DIR"

echo ""
echo "🌾 SweetGrass + BearDog: REAL Integration Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This demo will:"
echo "  1. Start real SweetGrass service"
echo "  2. Start real BearDog service"
echo "  3. Make real API calls between them"
echo "  4. Document any integration gaps"
echo ""
echo "⚠️  This may expose bugs - that's the point!"
echo ""
sleep 2

# Initialize gaps document
cat > "$GAPS_FILE" <<EOF
# Integration Gaps Discovered

**Date**: $(date)
**Test**: SweetGrass + BearDog Real Integration

## Summary

This document records actual integration issues discovered during real service testing.

## Gaps Discovered

EOF

GAP_COUNT=0

# Function to record a gap
record_gap() {
    local title="$1"
    local description="$2"
    local severity="$3"
    
    GAP_COUNT=$((GAP_COUNT + 1))
    
    cat >> "$GAPS_FILE" <<EOF

### Gap $GAP_COUNT: $title

**Severity**: $severity  
**Description**: $description  
**Discovered**: $(date)

EOF
    
    echo -e "${RED}🐛 INTEGRATION GAP #$GAP_COUNT: $title${NC}"
    echo "   Severity: $severity"
    echo "   (Documented in $GAPS_FILE)"
    echo ""
}

# Step 1: Check binaries
echo -e "${BLUE}Step 1: Checking binaries...${NC}"

if [ ! -x "$BINS_DIR/beardog" ]; then
    echo -e "${RED}❌ BearDog binary not found at $BINS_DIR/beardog${NC}"
    record_gap "Missing BearDog Binary" \
        "BearDog binary not available for testing" \
        "CRITICAL"
    exit 1
fi

echo "   ✅ BearDog: $BINS_DIR/beardog"
echo "   Version: $($BINS_DIR/beardog --version 2>&1 | head -1 || echo 'unknown')"

if [ ! -f "$PROJECT_ROOT/target/release/sweetgrass" ]; then
    echo "   Building SweetGrass..."
    cd "$PROJECT_ROOT"
    cargo build --release -p sweet-grass-service
fi

echo "   ✅ SweetGrass: $PROJECT_ROOT/target/release/sweetgrass"
echo ""
sleep 1

# Step 2: Start BearDog service
echo -e "${BLUE}Step 2: Starting BearDog service...${NC}"

# Try to start BearDog as a service
# NOTE: This may fail if BearDog doesn't have a 'server' or 'service' command
if $BINS_DIR/beardog --help 2>&1 | grep -q "server\|service"; then
    echo "   Starting BearDog server..."
    
    # Try different port options
    if $BINS_DIR/beardog --help 2>&1 | grep -q "\-\-port"; then
        $BINS_DIR/beardog server --port 8091 > "$OUTPUT_DIR/beardog.log" 2>&1 &
        BEARDOG_PID=$!
    elif $BINS_DIR/beardog --help 2>&1 | grep -q "service"; then
        $BINS_DIR/beardog service start --port 8091 > "$OUTPUT_DIR/beardog.log" 2>&1 &
        BEARDOG_PID=$!
    else
        record_gap "BearDog Server Port Configuration" \
            "BearDog server command exists but port configuration unclear" \
            "HIGH"
        BEARDOG_PID=""
    fi
    
    if [ -n "$BEARDOG_PID" ]; then
        sleep 2
        
        # Check if BearDog is running
        if kill -0 $BEARDOG_PID 2>/dev/null; then
            echo "   ✅ BearDog service running (PID: $BEARDOG_PID)"
            
            # Try to check health
            if curl -s http://localhost:8091/health > /dev/null 2>&1; then
                BEARDOG_HEALTH=$(curl -s http://localhost:8091/health)
                echo "   ✅ BearDog health check: $(echo $BEARDOG_HEALTH | jq -r '.status' 2>/dev/null || echo 'ok')"
            elif curl -s http://localhost:8091/api/health > /dev/null 2>&1; then
                echo "   ✅ BearDog responding (alternate health endpoint)"
                record_gap "BearDog Health Endpoint Path" \
                    "Health endpoint at /api/health instead of /health" \
                    "LOW"
            else
                record_gap "BearDog Health Endpoint Missing" \
                    "BearDog service running but no health endpoint accessible" \
                    "MEDIUM"
            fi
        else
            record_gap "BearDog Service Failed to Start" \
                "BearDog service command executed but process died immediately. Check logs: $OUTPUT_DIR/beardog.log" \
                "CRITICAL"
            BEARDOG_PID=""
        fi
    fi
else
    record_gap "BearDog Server Mode Not Available" \
        "BearDog binary doesn't have 'server' or 'service' subcommand. Current version may be CLI-only." \
        "CRITICAL"
    echo -e "${YELLOW}⚠️  BearDog doesn't have server mode${NC}"
    echo "   Current BearDog capabilities:"
    $BINS_DIR/beardog --help 2>&1 | grep -E "^\s+[a-z]" | head -10
    echo ""
    BEARDOG_PID=""
fi
echo ""
sleep 1

# Step 3: Start SweetGrass service
echo -e "${BLUE}Step 3: Starting SweetGrass service...${NC}"

cd "$PROJECT_ROOT"
RUST_LOG=info "$PROJECT_ROOT/target/release/sweetgrass" \
    --port 8080 \
    --storage memory \
    > "$OUTPUT_DIR/sweetgrass.log" 2>&1 &
SWEETGRASS_PID=$!

sleep 2

# Check if SweetGrass is running
if ! kill -0 $SWEETGRASS_PID 2>/dev/null; then
    echo -e "${RED}❌ SweetGrass failed to start${NC}"
    echo "   Check logs: $OUTPUT_DIR/sweetgrass.log"
    [ -n "$BEARDOG_PID" ] && kill $BEARDOG_PID 2>/dev/null || true
    exit 1
fi

# Wait for health
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        break
    fi
    sleep 0.5
done

SWEETGRASS_HEALTH=$(curl -s http://localhost:8080/health 2>/dev/null || echo '{"status":"unknown"}')
echo "   ✅ SweetGrass service running (PID: $SWEETGRASS_PID)"
echo "   ✅ Health: $(echo $SWEETGRASS_HEALTH | jq -r '.status' 2>/dev/null || echo 'ok')"
echo ""
sleep 1

# Step 4: Attempt integration
echo -e "${BLUE}Step 4: Testing integration...${NC}"
echo ""

if [ -n "$BEARDOG_PID" ] && kill -0 $BEARDOG_PID 2>/dev/null; then
    echo "   Both services running - attempting integration..."
    echo ""
    
    # Create a Braid
    echo "   Creating Braid in SweetGrass..."
    BRAID_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/braids \
        -H "Content-Type: application/json" \
        -d '{
          "data_hash": "sha256:test123",
          "mime_type": "text/plain",
          "size": 42,
          "was_attributed_to": {"did": "did:key:z6MkTest", "role": "Creator"}
        }')
    
    BRAID_ID=$(echo "$BRAID_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")
    
    if [ -n "$BRAID_ID" ] && [ "$BRAID_ID" != "null" ]; then
        echo "   ✅ Braid created: $BRAID_ID"
        echo ""
        
        # Try to sign the Braid with BearDog
        echo "   Attempting to sign Braid with BearDog..."
        
        # This will likely fail - we need to discover the actual API
        SIGN_RESPONSE=$(curl -s -X POST http://localhost:8091/sign \
            -H "Content-Type: application/json" \
            -d "{\"braid_id\": \"$BRAID_ID\"}" 2>&1)
        
        if echo "$SIGN_RESPONSE" | jq -e '.signature' > /dev/null 2>&1; then
            echo "   ✅ Signature obtained!"
            echo "$SIGN_RESPONSE" | jq '.signature'
        else
            # Document the actual API mismatch
            record_gap "BearDog Signing API Unknown" \
                "Attempted POST /sign but got: $SIGN_RESPONSE. Need to discover actual signing API." \
                "HIGH"
            
            # Try alternate endpoints
            echo "   ❌ /sign endpoint failed"
            echo "   Trying alternate endpoints..."
            
            if curl -s http://localhost:8091/api/v1/sign > /dev/null 2>&1; then
                echo "   💡 Found: /api/v1/sign"
                record_gap "BearDog Signing Endpoint Path" \
                    "Signing endpoint is at /api/v1/sign not /sign" \
                    "LOW"
            fi
        fi
    else
        record_gap "Braid Creation Failed" \
            "Failed to create Braid in SweetGrass: $BRAID_RESPONSE" \
            "CRITICAL"
    fi
else
    echo "   ⚠️  BearDog service not running - cannot test integration"
    echo "   This is expected if BearDog doesn't have server mode yet"
fi
echo ""
sleep 1

# Cleanup
echo -e "${YELLOW}Cleaning up...${NC}"

if [ -n "$SWEETGRASS_PID" ]; then
    kill $SWEETGRASS_PID 2>/dev/null || true
    wait $SWEETGRASS_PID 2>/dev/null || true
fi

if [ -n "$BEARDOG_PID" ]; then
    kill $BEARDOG_PID 2>/dev/null || true
    wait $BEARDOG_PID 2>/dev/null || true
fi

echo "   ✅ Services stopped"
echo ""

# Final summary
cat >> "$GAPS_FILE" <<EOF

## Summary

**Total Gaps Discovered**: $GAP_COUNT

## Next Steps

1. Review each gap above
2. Update integration specifications
3. Coordinate with BearDog team
4. Implement fixes
5. Re-run this test

## Test Artifacts

- SweetGrass logs: $OUTPUT_DIR/sweetgrass.log
- BearDog logs: $OUTPUT_DIR/beardog.log
- This report: $GAPS_FILE

EOF

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Integration Test Complete!${NC}"
echo ""
echo "📊 Results:"
echo "   • Gaps discovered: $GAP_COUNT"
echo "   • BearDog service: $([ -n "$BEARDOG_PID" ] && echo 'Started' || echo 'Not available')"
echo "   • SweetGrass service: ✅ Started"
echo "   • Real integration: $([ $GAP_COUNT -eq 0 ] && echo '✅ Success' || echo '⚠️  Issues found')"
echo ""

if [ $GAP_COUNT -gt 0 ]; then
    echo -e "${YELLOW}🐛 Integration gaps discovered!${NC}"
    echo ""
    echo "   This is GOOD - we found issues before production!"
    echo ""
    echo "   Gaps documented in:"
    echo "   $GAPS_FILE"
    echo ""
    echo "   Next steps:"
    echo "   1. Review the gaps file"
    echo "   2. Update integration specs"
    echo "   3. Coordinate with BearDog team"
    echo "   4. Implement fixes"
    echo "   5. Re-run this test"
else
    echo -e "${GREEN}✅ No gaps discovered!${NC}"
    echo ""
    echo "   Integration working perfectly (or needs more testing)"
fi
echo ""
echo "📁 Test output: $OUTPUT_DIR/"
echo ""
echo "🎓 What we learned:"
echo "   • Real service integration reveals actual issues"
echo "   • Mock demos hide problems"
echo "   • Discovering gaps early = better production"
echo ""

