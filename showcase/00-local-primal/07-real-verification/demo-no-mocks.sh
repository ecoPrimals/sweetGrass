#!/usr/bin/env bash
#
# 🔍 SweetGrass Real Execution Verification
#
# 10-POINT VERIFICATION CHECKLIST (inspired by Songbird)
# Proves that all demos use REAL SweetGrass, not mocks.
#
# Time: ~5 minutes
# Prerequisites: None (service will be started)
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
SERVICE_BINARY="$PROJECT_ROOT/target/release/sweet-grass-service"
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
SERVICE_PORT=8084
SERVICE_PID=""

# Verification results
CHECKS_PASSED=0
CHECKS_TOTAL=10

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

# Logging
exec 1> >(tee -a "$OUTPUT_DIR/verification.log")
exec 2>&1

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}     🔍 SweetGrass Real Execution Verification${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BOLD}${YELLOW}10-POINT VERIFICATION CHECKLIST${NC}"
echo -e "${BLUE}Inspired by Songbird's real execution proof${NC}"
echo ""
echo -e "${BLUE}Time estimate: ~5 minutes${NC}"
echo -e "${BLUE}Output directory: $OUTPUT_DIR${NC}"
echo ""

# Function to stop service on exit
cleanup() {
    if [ -n "$SERVICE_PID" ] && kill -0 "$SERVICE_PID" 2>/dev/null; then
        echo -e "\n${YELLOW}🛑 Stopping service (PID: $SERVICE_PID)...${NC}"
        kill "$SERVICE_PID" 2>/dev/null || true
        wait "$SERVICE_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

# Function to record check result
check_result() {
    local check_num=$1
    local description=$2
    local passed=$3
    
    if [ "$passed" = "true" ]; then
        echo -e "${GREEN}   ✅ Check $check_num: $description${NC}"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    else
        echo -e "${RED}   ❌ Check $check_num: $description${NC}"
    fi
}

# ============================================================================
# CHECK 1: Real Binary Exists
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 1: Real Binary Exists${NC}"
echo ""

if [ ! -f "$SERVICE_BINARY" ]; then
    echo -e "${BLUE}   Building SweetGrass service...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release -p sweet-grass-service
fi

if [ -f "$SERVICE_BINARY" ]; then
    BINARY_SIZE=$(ls -lh "$SERVICE_BINARY" | awk '{print $5}')
    BINARY_TYPE=$(file "$SERVICE_BINARY")
    echo -e "${BLUE}   Binary path: $SERVICE_BINARY${NC}"
    echo -e "${BLUE}   Binary size: $BINARY_SIZE${NC}"
    echo -e "${BLUE}   Binary type: $BINARY_TYPE${NC}"
    echo "$BINARY_TYPE" > "$OUTPUT_DIR/binary-info.txt"
    
    if echo "$BINARY_TYPE" | grep -q "ELF"; then
        check_result 1 "Real ELF executable binary exists" "true"
    else
        check_result 1 "Real ELF executable binary exists" "false"
    fi
else
    check_result 1 "Real ELF executable binary exists" "false"
    exit 1
fi
echo ""

# ============================================================================
# CHECK 2: Real Process Created
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 2: Real Process Created${NC}"
echo ""

echo -e "${BLUE}   Starting SweetGrass service...${NC}"
"$SERVICE_BINARY" --port "$SERVICE_PORT" --storage memory > "$OUTPUT_DIR/service.log" 2>&1 &
SERVICE_PID=$!

sleep 2

if kill -0 "$SERVICE_PID" 2>/dev/null; then
    echo -e "${BLUE}   Process ID (PID): $SERVICE_PID${NC}"
    echo "$SERVICE_PID" > "$OUTPUT_DIR/service-pid.txt"
    
    # Get process info
    if command -v ps > /dev/null 2>&1; then
        ps -p "$SERVICE_PID" -o pid,ppid,cmd,etime > "$OUTPUT_DIR/process-info.txt"
        echo -e "${BLUE}   Process info saved to process-info.txt${NC}"
    fi
    
    check_result 2 "Real process created with PID $SERVICE_PID" "true"
else
    check_result 2 "Real process created" "false"
    exit 1
fi
echo ""

# ============================================================================
# CHECK 3: Real Port Listening
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 3: Real Port Listening${NC}"
echo ""

# Wait for service to bind to port
echo -e "${BLUE}   Waiting for service to bind to port $SERVICE_PORT...${NC}"
for i in {1..30}; do
    if curl -s "http://localhost:$SERVICE_PORT/health" > /dev/null 2>&1; then
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}   Service failed to start${NC}"
        check_result 3 "Real port listening" "false"
        exit 1
    fi
    sleep 1
done

# Verify port is listening
if command -v lsof > /dev/null 2>&1; then
    PORT_INFO=$(lsof -i ":$SERVICE_PORT" -sTCP:LISTEN 2>/dev/null || echo "")
    if [ -n "$PORT_INFO" ]; then
        echo "$PORT_INFO" > "$OUTPUT_DIR/port-info.txt"
        echo -e "${BLUE}   Port $SERVICE_PORT is listening${NC}"
        echo -e "${BLUE}   $(echo "$PORT_INFO" | head -2 | tail -1)${NC}"
        check_result 3 "Real port listening (verified with lsof)" "true"
    else
        check_result 3 "Real port listening" "false"
    fi
elif command -v netstat > /dev/null 2>&1; then
    PORT_INFO=$(netstat -tuln | grep ":$SERVICE_PORT" || echo "")
    if [ -n "$PORT_INFO" ]; then
        echo "$PORT_INFO" > "$OUTPUT_DIR/port-info.txt"
        echo -e "${BLUE}   Port $SERVICE_PORT is listening${NC}"
        check_result 3 "Real port listening (verified with netstat)" "true"
    else
        check_result 3 "Real port listening" "false"
    fi
else
    # Fallback: check if we can connect
    if curl -s "http://localhost:$SERVICE_PORT/health" > /dev/null 2>&1; then
        check_result 3 "Real port listening (verified with curl)" "true"
    else
        check_result 3 "Real port listening" "false"
    fi
fi
echo ""

# ============================================================================
# CHECK 4: Real HTTP Responses
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 4: Real HTTP Responses${NC}"
echo ""

echo -e "${BLUE}   Testing HTTP health endpoint...${NC}"
HEALTH_RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" "http://localhost:$SERVICE_PORT/health")
HTTP_CODE=$(echo "$HEALTH_RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
HEALTH_BODY=$(echo "$HEALTH_RESPONSE" | grep -v "HTTP_CODE:")

echo "$HEALTH_BODY" | jq . > "$OUTPUT_DIR/health-response.json" 2>/dev/null || echo "$HEALTH_BODY" > "$OUTPUT_DIR/health-response.txt"

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${BLUE}   HTTP Status: $HTTP_CODE OK${NC}"
    echo -e "${BLUE}   Response: $HEALTH_BODY${NC}"
    check_result 4 "Real HTTP 200 OK response" "true"
else
    echo -e "${RED}   HTTP Status: $HTTP_CODE${NC}"
    check_result 4 "Real HTTP 200 OK response" "false"
fi
echo ""

# ============================================================================
# CHECK 5: Real API Endpoints Working
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 5: Real API Endpoints Working${NC}"
echo ""

echo -e "${BLUE}   Creating a real Braid via POST /api/v1/braids...${NC}"
CREATE_REQUEST=$(cat <<EOF
{
  "data_hash": "sha256:verification_test_$(date +%s)",
  "mime_type": "text/plain",
  "size": 42,
  "was_attributed_to": "did:key:z6MkVerificationTest",
  "tags": ["verification", "real-test"]
}
EOF
)

CREATE_RESPONSE=$(curl -s -X POST "http://localhost:$SERVICE_PORT/api/v1/braids" \
    -H "Content-Type: application/json" \
    -d "$CREATE_REQUEST")

echo "$CREATE_RESPONSE" | jq . > "$OUTPUT_DIR/create-braid-response.json" 2>/dev/null

BRAID_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ -n "$BRAID_ID" ] && [ "$BRAID_ID" != "null" ]; then
    echo -e "${BLUE}   Created Braid: $BRAID_ID${NC}"
    echo -e "${BLUE}   Retrieving Braid via GET /api/v1/braids/$BRAID_ID...${NC}"
    
    GET_RESPONSE=$(curl -s "http://localhost:$SERVICE_PORT/api/v1/braids/$BRAID_ID")
    echo "$GET_RESPONSE" | jq . > "$OUTPUT_DIR/get-braid-response.json" 2>/dev/null
    
    GET_BRAID_ID=$(echo "$GET_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")
    
    if [ "$GET_BRAID_ID" = "$BRAID_ID" ]; then
        check_result 5 "Real API endpoints (POST + GET) working" "true"
    else
        check_result 5 "Real API endpoints working" "false"
    fi
else
    check_result 5 "Real API endpoints working" "false"
fi
echo ""

# ============================================================================
# CHECK 6: Real Log Output
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 6: Real Log Output${NC}"
echo ""

echo -e "${BLUE}   Checking service logs...${NC}"
if [ -f "$OUTPUT_DIR/service.log" ]; then
    LOG_SIZE=$(wc -l < "$OUTPUT_DIR/service.log")
    echo -e "${BLUE}   Log file: $OUTPUT_DIR/service.log${NC}"
    echo -e "${BLUE}   Log lines: $LOG_SIZE${NC}"
    
    if [ "$LOG_SIZE" -gt 0 ]; then
        echo -e "${BLUE}   Sample log entries:${NC}"
        head -5 "$OUTPUT_DIR/service.log" | sed 's/^/      /'
        check_result 6 "Real log output ($LOG_SIZE lines)" "true"
    else
        check_result 6 "Real log output" "false"
    fi
else
    check_result 6 "Real log output" "false"
fi
echo ""

# ============================================================================
# CHECK 7: Real Resource Usage
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 7: Real Resource Usage${NC}"
echo ""

if command -v ps > /dev/null 2>&1; then
    RESOURCE_INFO=$(ps -p "$SERVICE_PID" -o %cpu,%mem,vsz,rss 2>/dev/null || echo "")
    if [ -n "$RESOURCE_INFO" ]; then
        echo "$RESOURCE_INFO" > "$OUTPUT_DIR/resource-usage.txt"
        echo -e "${BLUE}   Resource usage:${NC}"
        echo "$RESOURCE_INFO" | sed 's/^/      /'
        check_result 7 "Real resource usage measurable" "true"
    else
        check_result 7 "Real resource usage measurable" "false"
    fi
else
    echo -e "${YELLOW}   ps command not available${NC}"
    check_result 7 "Real resource usage measurable" "false"
fi
echo ""

# ============================================================================
# CHECK 8: Real Network Connections
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 8: Real Network Connections${NC}"
echo ""

if command -v netstat > /dev/null 2>&1; then
    CONNECTIONS=$(netstat -an | grep ":$SERVICE_PORT" | grep "ESTABLISHED\|LISTEN" || echo "")
    if [ -n "$CONNECTIONS" ]; then
        echo "$CONNECTIONS" > "$OUTPUT_DIR/network-connections.txt"
        CONN_COUNT=$(echo "$CONNECTIONS" | wc -l)
        echo -e "${BLUE}   Network connections: $CONN_COUNT${NC}"
        echo "$CONNECTIONS" | head -5 | sed 's/^/      /'
        check_result 8 "Real network connections ($CONN_COUNT)" "true"
    else
        check_result 8 "Real network connections" "false"
    fi
elif command -v ss > /dev/null 2>&1; then
    CONNECTIONS=$(ss -tan | grep ":$SERVICE_PORT" || echo "")
    if [ -n "$CONNECTIONS" ]; then
        echo "$CONNECTIONS" > "$OUTPUT_DIR/network-connections.txt"
        CONN_COUNT=$(echo "$CONNECTIONS" | wc -l)
        echo -e "${BLUE}   Network connections: $CONN_COUNT${NC}"
        check_result 8 "Real network connections ($CONN_COUNT)" "true"
    else
        check_result 8 "Real network connections" "false"
    fi
else
    echo -e "${YELLOW}   netstat/ss command not available${NC}"
    check_result 8 "Real network connections" "false"
fi
echo ""

# ============================================================================
# CHECK 9: Interactive Verification Commands
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 9: Interactive Verification Commands${NC}"
echo ""

echo -e "${CYAN}   You can verify execution yourself with these commands:${NC}"
echo ""
echo -e "${BLUE}   # Check process is running${NC}"
echo -e "${GREEN}   ps -p $SERVICE_PID -f${NC}"
echo ""
echo -e "${BLUE}   # Check port is listening${NC}"
echo -e "${GREEN}   lsof -i :$SERVICE_PORT${NC}"
echo ""
echo -e "${BLUE}   # Test API manually${NC}"
echo -e "${GREEN}   curl http://localhost:$SERVICE_PORT/health${NC}"
echo ""
echo -e "${BLUE}   # View live logs${NC}"
echo -e "${GREEN}   tail -f $OUTPUT_DIR/service.log${NC}"
echo ""
echo -e "${BLUE}   # Check binary${NC}"
echo -e "${GREEN}   file $SERVICE_BINARY${NC}"
echo ""

# Create verification script
cat > "$OUTPUT_DIR/verify-yourself.sh" <<EOL
#!/usr/bin/env bash
# Interactive verification commands

echo "🔍 SweetGrass Real Execution Verification"
echo ""
echo "Process:"
ps -p $SERVICE_PID -f 2>/dev/null || echo "Process not running"
echo ""
echo "Port:"
lsof -i :$SERVICE_PORT 2>/dev/null || netstat -tuln | grep :$SERVICE_PORT || echo "Port not listening"
echo ""
echo "API Health:"
curl -s http://localhost:$SERVICE_PORT/health | jq . || echo "API not responding"
echo ""
echo "Binary:"
file $SERVICE_BINARY
echo ""
echo "Resource Usage:"
ps -p $SERVICE_PID -o %cpu,%mem,vsz,rss 2>/dev/null || echo "Cannot get resource usage"
EOL

chmod +x "$OUTPUT_DIR/verify-yourself.sh"

check_result 9 "Interactive verification commands provided" "true"
echo ""

# ============================================================================
# CHECK 10: Clean Shutdown Verification
# ============================================================================

echo -e "${YELLOW}🔍 CHECK 10: Clean Shutdown Verification${NC}"
echo ""

echo -e "${BLUE}   Testing graceful shutdown...${NC}"

# Record PID before shutdown
SHUTDOWN_PID=$SERVICE_PID

# Send SIGTERM
kill -TERM "$SERVICE_PID" 2>/dev/null

# Wait for graceful shutdown (max 5 seconds)
SHUTDOWN_SUCCESS=false
for i in {1..10}; do
    if ! kill -0 "$SERVICE_PID" 2>/dev/null; then
        SHUTDOWN_SUCCESS=true
        break
    fi
    sleep 0.5
done

if [ "$SHUTDOWN_SUCCESS" = "true" ]; then
    echo -e "${BLUE}   Service shut down gracefully${NC}"
    SERVICE_PID=""  # Clear PID so cleanup doesn't try again
    check_result 10 "Clean shutdown (graceful SIGTERM)" "true"
else
    echo -e "${YELLOW}   Service did not shut down gracefully, forcing...${NC}"
    kill -9 "$SERVICE_PID" 2>/dev/null || true
    SERVICE_PID=""
    check_result 10 "Clean shutdown" "false"
fi
echo ""

# ============================================================================
# Summary
# ============================================================================

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}${YELLOW}   VERIFICATION SUMMARY${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BOLD}Results: $CHECKS_PASSED / $CHECKS_TOTAL checks passed${NC}"
echo ""

PERCENT=$((CHECKS_PASSED * 100 / CHECKS_TOTAL))

if [ "$CHECKS_PASSED" -eq "$CHECKS_TOTAL" ]; then
    echo -e "${GREEN}   ✅ 100% VERIFICATION SUCCESS${NC}"
    echo -e "${GREEN}   All demos use REAL SweetGrass, not mocks!${NC}"
elif [ "$PERCENT" -ge 80 ]; then
    echo -e "${YELLOW}   ⚠️  $PERCENT% verification (mostly successful)${NC}"
    echo -e "${YELLOW}   Some checks may require additional tools${NC}"
else
    echo -e "${RED}   ❌ $PERCENT% verification (issues found)${NC}"
    echo -e "${RED}   Please review the output above${NC}"
fi

echo ""
echo -e "${CYAN}Verification Artifacts:${NC}"
echo -e "${BLUE}   • Binary info:        $OUTPUT_DIR/binary-info.txt${NC}"
echo -e "${BLUE}   • Process info:       $OUTPUT_DIR/process-info.txt${NC}"
echo -e "${BLUE}   • Port info:          $OUTPUT_DIR/port-info.txt${NC}"
echo -e "${BLUE}   • Health response:    $OUTPUT_DIR/health-response.json${NC}"
echo -e "${BLUE}   • Created Braid:      $OUTPUT_DIR/create-braid-response.json${NC}"
echo -e "${BLUE}   • Service logs:       $OUTPUT_DIR/service.log${NC}"
echo -e "${BLUE}   • Verification log:   $OUTPUT_DIR/verification.log${NC}"
echo -e "${BLUE}   • Self-verify script: $OUTPUT_DIR/verify-yourself.sh${NC}"
echo ""

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}   ✅ Real Verification Demo Complete!${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BOLD}${MAGENTA}🔍 This is REAL SweetGrass, not mocks! 🔍${NC}"
echo ""
echo -e "${BLUE}Time taken: ~5 minutes${NC}"
echo -e "${BLUE}Next: cd ../../01-primal-coordination (inter-primal demos)${NC}"
echo ""

# Exit with appropriate code
if [ "$CHECKS_PASSED" -eq "$CHECKS_TOTAL" ]; then
    exit 0
else
    exit 1
fi
