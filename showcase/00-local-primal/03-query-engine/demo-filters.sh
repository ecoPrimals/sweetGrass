#!/usr/bin/env bash
#
# 🌾 SweetGrass: Query Engine Demo
# Time: ~10 minutes
# Shows: Filtering and searching Braids
#

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo ""
echo "🌾 SweetGrass: Query Engine Demo"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This demo shows powerful provenance queries."
echo ""
sleep 2

# Start service
echo -e "${BLUE}Starting SweetGrass...${NC}"
cd "$PROJECT_ROOT"
RUST_LOG=error "$PROJECT_ROOT/target/release/sweetgrass" \
    --port 8080 --storage memory > "$OUTPUT_DIR/sweetgrass.log" 2>&1 &
SWEETGRASS_PID=$!

for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then break; fi
    sleep 0.5
done
echo "   ✅ Service ready"
echo ""

# Populate with test data
echo -e "${BLUE}Step 1: Creating test dataset...${NC}"
echo ""
echo "   Creating 10 Braids with different attributes:"
echo ""

BRAID_IDS=()

# Create diverse Braids
for i in {1..10}; do
    TYPE=$([ $((i % 2)) -eq 0 ] && echo "Dataset" || echo "Model")
    AGENT="did:key:z6MkAgent$i"
    TAG1=$([ $i -le 5 ] && echo "research" || echo "production")
    TAG2=$([ $((i % 3)) -eq 0 ] && echo "ml" || echo "analysis")
    
    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/braids \
        -H "Content-Type: application/json" \
        -d "{
          \"data_hash\": \"sha256:data_${i}_abc123\",
          \"mime_type\": \"application/octet-stream\",
          \"size\": $((1000 * i)),
          \"was_attributed_to\": {\"did\": \"$AGENT\", \"role\": \"Creator\"},
          \"tags\": [\"$TAG1\", \"$TAG2\", \"test-data\"]
        }")
    
    BRAID_ID=$(echo "$RESPONSE" | jq -r '.id')
    BRAID_IDS+=("$BRAID_ID")
    echo "   ✅ Braid $i: $TYPE by Agent$i (tags: $TAG1, $TAG2)"
done

echo ""
echo "   Total: ${#BRAID_IDS[@]} Braids created"
echo ""
sleep 2

# Query 1: By tag
echo -e "${BLUE}Step 2: Query by tag...${NC}"
echo ""
echo "   Query: tag=research"
echo ""

RESULT=$(curl -s "http://localhost:8080/api/v1/braids?tag=research")
COUNT=$(echo "$RESULT" | jq '.results | length')
echo "   📊 Found $COUNT Braids"
echo "$RESULT" | jq -r '.results[] | "      • \(.id | split(":")[2][0:8])... (tags: \(.tags | join(", ")))"'
echo ""
sleep 2

# Query 2: By agent
echo -e "${BLUE}Step 3: Query by agent...${NC}"
echo ""
echo "   Query: agent=did:key:z6MkAgent3"
echo ""

RESULT=$(curl -s "http://localhost:8080/api/v1/braids?agent=did:key:z6MkAgent3")
COUNT=$(echo "$RESULT" | jq '.results | length')
echo "   📊 Found $COUNT Braid(s)"
echo "$RESULT" | jq -r '.results[] | "      • Created by: \(.was_attributed_to[0].did | split(":")[2])"'
echo ""
sleep 2

# Query 3: Combination
echo -e "${BLUE}Step 4: Combined query...${NC}"
echo ""
echo "   Query: tag=production AND tag=ml"
echo ""

RESULT=$(curl -s "http://localhost:8080/api/v1/braids?tag=production&tag=ml")
COUNT=$(echo "$RESULT" | jq '.results | length')
echo "   📊 Found $COUNT Braid(s) matching both tags"
echo "$RESULT" | jq -r '.results[] | "      • \(.id | split(":")[2][0:8])... (tags: \(.tags | join(", ")))"'
echo ""
sleep 2

# Query 4: Pagination
echo -e "${BLUE}Step 5: Pagination...${NC}"
echo ""
echo "   Query: limit=3, offset=0 (first page)"
echo ""

RESULT=$(curl -s "http://localhost:8080/api/v1/braids?limit=3&offset=0")
COUNT=$(echo "$RESULT" | jq '.results | length')
TOTAL=$(echo "$RESULT" | jq '.total')
echo "   📊 Page 1: $COUNT of $TOTAL total"
echo ""

echo "   Query: limit=3, offset=3 (second page)"
echo ""

RESULT=$(curl -s "http://localhost:8080/api/v1/braids?limit=3&offset=3")
COUNT=$(echo "$RESULT" | jq '.results | length')
echo "   📊 Page 2: $COUNT of $TOTAL total"
echo ""
sleep 2

# Query 5: Ordering
echo -e "${BLUE}Step 6: Ordering results...${NC}"
echo ""
echo "   Query: order=size_desc (largest first)"
echo ""

RESULT=$(curl -s "http://localhost:8080/api/v1/braids?order=size_desc&limit=3")
echo "   📊 Top 3 largest Braids:"
echo "$RESULT" | jq -r '.results[] | "      • Size: \(.size) bytes"'
echo ""
sleep 1

# Cleanup
kill $SWEETGRASS_PID 2>/dev/null || true
wait $SWEETGRASS_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ Query by tag"
echo "   ✅ Query by agent"
echo "   ✅ Combined filters"
echo "   ✅ Pagination (limit/offset)"
echo "   ✅ Result ordering"
echo ""
echo "💡 Query capabilities:"
echo "   • Filter by: tag, agent, type, time range"
echo "   • Combine multiple filters"
echo "   • Paginate results"
echo "   • Order by: created, size, etc."
echo "   • Efficient indexing"
echo ""
echo "📁 Output: $OUTPUT_DIR/"
echo ""
echo "⏭️  Next: Learn about PROV-O export"
echo "   cd ../04-prov-o-standard && ./demo-prov-o-export.sh"
echo ""

