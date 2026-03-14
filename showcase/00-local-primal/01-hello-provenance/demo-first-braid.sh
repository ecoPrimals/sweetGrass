#!/usr/bin/env bash
#
# 🌾 SweetGrass: Hello Provenance Demo
# Time: ~5 minutes
# Shows: Creating your first Braid, understanding provenance
#
# This demo uses a REAL SweetGrass service (no mocks!)
#

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "🌾 SweetGrass: Hello Provenance"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This demo creates your first Braid (provenance record)"
echo "using a REAL SweetGrass service."
echo ""
sleep 2

# Step 1: Build SweetGrass (if needed)
echo -e "${BLUE}Step 1: Preparing SweetGrass service...${NC}"
cd "$PROJECT_ROOT"

if [ ! -f "target/release/sweetgrass" ]; then
    echo "   Building SweetGrass (this may take a moment)..."
    cargo build --release -p sweet-grass-service > /dev/null 2>&1
    echo "   ✅ Build complete"
else
    echo "   ✅ SweetGrass binary ready"
fi
echo ""
sleep 1

# Step 2: Start real SweetGrass service
echo -e "${BLUE}Step 2: Starting real SweetGrass service...${NC}"
echo "   Configuration:"
echo "     • Port: 8080"
echo "     • Storage: Memory (ephemeral)"
echo "     • Default agent: did:key:z6MkLocalDemo"
echo ""

# Start service in background
RUST_LOG=error "$PROJECT_ROOT/target/release/sweetgrass" \
    --port 8080 \
    --storage memory \
    --default-agent "did:key:z6MkLocalDemo" \
    > "$OUTPUT_DIR/sweetgrass.log" 2>&1 &

SWEETGRASS_PID=$!
echo "   ✅ SweetGrass running (PID: $SWEETGRASS_PID)"
echo ""

# Wait for service to be ready
echo "   Waiting for service to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        break
    fi
    if [ $i -eq 30 ]; then
        echo "   ❌ Service failed to start"
        echo "   Check logs: $OUTPUT_DIR/sweetgrass.log"
        kill $SWEETGRASS_PID 2>/dev/null || true
        exit 1
    fi
    sleep 0.5
done

HEALTH=$(curl -s http://localhost:8080/health)
echo "   ✅ Service healthy: $(echo $HEALTH | jq -r '.status')"
echo ""
sleep 1

# Step 3: Create your first Braid
echo -e "${BLUE}Step 3: Creating your first Braid...${NC}"
echo ""
echo "   A Braid is a provenance record that tracks:"
echo "     • What: The data itself (hash, type, size)"
echo "     • Who: The agent(s) who created it"
echo "     • When: Timestamps"
echo "     • How: The activity that generated it"
echo ""
sleep 2

echo "   Creating Braid for: 'Hello, SweetGrass! This is my first provenance record.'"
echo ""

# Real API call to create a Braid
BRAID_REQUEST=$(cat <<EOF
{
  "data_hash": "sha256:7a8f3b2c1d9e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9",
  "mime_type": "text/plain",
  "size": 59,
  "was_attributed_to": {
    "did": "did:key:z6MkAlice",
    "role": "Creator"
  },
  "tags": ["hello-world", "first-braid", "demo"]
}
EOF
)

echo "$BRAID_REQUEST" > "$OUTPUT_DIR/braid-request.json"

RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "$BRAID_REQUEST")

# Save response
echo "$RESPONSE" > "$OUTPUT_DIR/braid-response.json"

# Check for errors
if echo "$RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    echo -e "   ${RED}❌ Failed to create Braid${NC}"
    echo "$RESPONSE" | jq '.'
    kill $SWEETGRASS_PID 2>/dev/null || true
    exit 1
fi

BRAID_ID=$(echo "$RESPONSE" | jq -r '.id')
echo -e "   ${GREEN}✅ Braid created successfully!${NC}"
echo ""
echo "   📦 Braid ID: $BRAID_ID"
echo "   📝 Data hash: sha256:7a8f3b2...e8f9"
echo "   👤 Creator: did:key:z6MkAlice"
echo "   🏷️  Tags: hello-world, first-braid, demo"
echo ""
sleep 2

# Step 4: Retrieve the Braid
echo -e "${BLUE}Step 4: Retrieving your Braid...${NC}"
echo ""

RETRIEVED=$(curl -s http://localhost:8080/api/v1/braids/$BRAID_ID)
echo "$RETRIEVED" > "$OUTPUT_DIR/braid-retrieved.json"

echo -e "   ${GREEN}✅ Retrieved successfully!${NC}"
echo ""
echo "   Full Braid structure:"
echo ""
echo "$RETRIEVED" | jq '.' | head -20
echo "   ... (full output in $OUTPUT_DIR/braid-retrieved.json)"
echo ""
sleep 2

# Step 5: Query for your Braid
echo -e "${BLUE}Step 5: Querying for Braids...${NC}"
echo ""
echo "   Query: Find all Braids with tag 'hello-world'"
echo ""

QUERY_RESULT=$(curl -s "http://localhost:8080/api/v1/braids?tag=hello-world")
echo "$QUERY_RESULT" > "$OUTPUT_DIR/query-result.json"

COUNT=$(echo "$QUERY_RESULT" | jq '.results | length')
echo -e "   ${GREEN}✅ Found $COUNT Braid(s)${NC}"
echo ""
sleep 1

# Step 6: Understanding the Braid structure
echo -e "${BLUE}Step 6: Understanding the Braid structure...${NC}"
echo ""
echo "   A Braid follows W3C PROV-O ontology:"
echo ""
echo "   • @context: JSON-LD context for semantic interoperability"
echo "   • @id: Unique identifier (URN format)"
echo "   • @type: 'prov:Entity' (W3C PROV-O)"
echo "   • data_hash: Content-addressable identifier"
echo "   • was_attributed_to: Agent(s) who created it"
echo "   • tags: Searchable metadata"
echo ""
echo "   This makes SweetGrass compatible with any PROV-O system!"
echo ""
sleep 2

# Step 7: Check service metrics
echo -e "${BLUE}Step 7: Service metrics...${NC}"
echo ""

METRICS=$(curl -s http://localhost:8080/status)
echo "$METRICS" > "$OUTPUT_DIR/metrics.json"

echo "   Service status:"
echo "$METRICS" | jq '{
  uptime: .uptime,
  total_braids: .metrics.total_braids,
  storage_backend: .storage_backend
}'
echo ""
sleep 1

# Cleanup
echo -e "${YELLOW}Cleaning up...${NC}"
kill $SWEETGRASS_PID 2>/dev/null || true
wait $SWEETGRASS_PID 2>/dev/null || true
echo "   ✅ Service stopped"
echo ""

# Success summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ Started a real SweetGrass service"
echo "   ✅ Created a provenance record (Braid)"
echo "   ✅ Retrieved it via REST API"
echo "   ✅ Queried for Braids by tag"
echo "   ✅ Saw W3C PROV-O structure"
echo ""
echo "💡 Key Concepts:"
echo "   • Braid = provenance record"
echo "   • Tracks data lineage (who, what, when, how)"
echo "   • W3C PROV-O compliant"
echo "   • RESTful API"
echo "   • Content-addressable (via data_hash)"
echo ""
echo "📁 Output saved to:"
echo "   $OUTPUT_DIR/"
echo "   ├── braid-request.json      (What we sent)"
echo "   ├── braid-response.json     (Braid created)"
echo "   ├── braid-retrieved.json    (Retrieved Braid)"
echo "   ├── query-result.json       (Query results)"
echo "   └── sweetgrass.log          (Service logs)"
echo ""
echo "🔍 Inspect the files to see the full structure!"
echo ""
echo "⏭️  Next: Learn about attribution"
echo "   cd ../02-attribution-basics && ./demo-fair-credit.sh"
echo ""
echo "🌾 You've created your first provenance record!"
echo ""

