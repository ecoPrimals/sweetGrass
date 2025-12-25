#!/usr/bin/env bash
#
# 🌾 SweetGrass: Fair Attribution Demo
# Time: ~10 minutes
# Shows: How attribution works with multiple contributors
#
# This demo uses a REAL SweetGrass service (no mocks!)
#

set -euo pipefail

# Colors
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
echo "🌾 SweetGrass: Fair Attribution Demo"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This demo shows how SweetGrass calculates fair credit"
echo "when multiple people contribute to a dataset."
echo ""
sleep 2

# Start service
echo -e "${BLUE}Starting SweetGrass service...${NC}"
cd "$PROJECT_ROOT"
RUST_LOG=error "$PROJECT_ROOT/target/release/sweet-grass-service" \
    --port 8080 --storage memory \
    > "$OUTPUT_DIR/sweetgrass.log" 2>&1 &
SWEETGRASS_PID=$!

# Wait for readiness
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then break; fi
    sleep 0.5
done
echo "   ✅ Service ready"
echo ""
sleep 1

# Scenario: ML Dataset Creation
echo -e "${CYAN}📖 Scenario: ML Training Dataset${NC}"
echo ""
echo "   A team creates a machine learning dataset:"
echo "   • Alice: Collected raw data"
echo "   • Bob: Cleaned and labeled data  "
echo "   • Carol: Validated labels"
echo "   • Dave: Curated final dataset"
echo ""
echo "   Question: How do we credit everyone fairly?"
echo ""
sleep 3

# Step 1: Create the final dataset Braid with all contributors
echo -e "${BLUE}Step 1: Creating dataset with attribution...${NC}"
echo ""

DATASET_BRAID=$(cat <<EOF
{
  "data_hash": "sha256:ml_dataset_final_v1_abc123def456",
  "mime_type": "application/x-hdf5",
  "size": 524288000,
  "was_attributed_to": [
    {
      "did": "did:key:z6MkAlice",
      "role": "DataProvider"
    },
    {
      "did": "did:key:z6MkBob",
      "role": "Transformer"
    },
    {
      "did": "did:key:z6MkCarol",
      "role": "Contributor"
    },
    {
      "did": "did:key:z6MkDave",
      "role": "Curator"
    }
  ],
  "tags": ["ml-dataset", "image-classification", "medical"]
}
EOF
)

RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "$DATASET_BRAID")

BRAID_ID=$(echo "$RESPONSE" | jq -r '.id')
echo "   ✅ Dataset Braid created: $BRAID_ID"
echo ""
echo "$RESPONSE" | jq '.was_attributed_to' 
echo ""
sleep 2

# Step 2: Calculate attribution
echo -e "${BLUE}Step 2: Calculating fair attribution...${NC}"
echo ""
echo "   SweetGrass uses role-based weights:"
echo "   • Creator: 1.0 (created from scratch)"
echo "   • DataProvider: 0.4 (provided raw data)"
echo "   • Transformer: 0.3 (processed/cleaned)"
echo "   • Contributor: 0.5 (significant contribution)"
echo "   • Curator: 0.2 (quality control)"
echo ""
sleep 2

ATTRIBUTION=$(curl -s "http://localhost:8080/api/v1/attribution/$BRAID_ID")
echo "$ATTRIBUTION" > "$OUTPUT_DIR/attribution.json"

echo "   📊 Attribution results:"
echo ""
echo "$ATTRIBUTION" | jq -r '.attributions[] | "   • \(.agent.did | split(":")[2]): \(.share * 100 | floor)% (role: \(.role), weight: \(.weight))"'
echo ""
echo "   Total shares sum to 100%"
echo ""
sleep 3

# Step 3: Show derivation chain impact
echo -e "${BLUE}Step 3: Derivation chain attribution...${NC}"
echo ""
echo "   Now let's say Eve uses this dataset to train a model."
echo "   How do we credit the original contributors?"
echo ""
sleep 2

# Create model Braid derived from dataset
MODEL_BRAID=$(cat <<EOF
{
  "data_hash": "sha256:trained_model_v1_xyz789",
  "mime_type": "application/x-pytorch",
  "size": 104857600,
  "was_attributed_to": {
    "did": "did:key:z6MkEve",
    "role": "Creator"
  },
  "was_derived_from": ["$BRAID_ID"],
  "tags": ["ml-model", "trained", "medical"]
}
EOF
)

MODEL_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/braids \
    -H "Content-Type: application/json" \
    -d "$MODEL_BRAID")

MODEL_ID=$(echo "$MODEL_RESPONSE" | jq -r '.id')
echo "   ✅ Model Braid created: $MODEL_ID"
echo "   Derived from: $BRAID_ID"
echo ""
sleep 1

# Calculate attribution with decay
MODEL_ATTRIBUTION=$(curl -s "http://localhost:8080/api/v1/attribution/$MODEL_ID?include_derived=true&decay=0.5")
echo "$MODEL_ATTRIBUTION" > "$OUTPUT_DIR/model-attribution.json"

echo "   📊 Model attribution (with 50% decay for derived):"
echo ""
echo "$MODEL_ATTRIBUTION" | jq -r '.attributions[] | "   • \(.agent.did | split(":")[2]): \(.share * 100 | floor)% (role: \(.role))"'
echo ""
echo "   Eve gets credit for training (1.0 weight)"
echo "   Dataset contributors get credit too (0.4, 0.3, 0.5, 0.2 weights × 0.5 decay)"
echo ""
sleep 3

# Step 4: Show why this matters
echo -e "${BLUE}Step 4: Why fair attribution matters...${NC}"
echo ""
echo "   🌐 For sunCloud reward distribution:"
echo "      • When the model is used, everyone gets paid"
echo "      • Proportional to their contribution"
echo "      • Automatically calculated"
echo ""
echo "   🔬 For open science:"
echo "      • All contributors get academic credit"
echo "      • Prevents 'ghost authorship'"
echo "      • Encourages data sharing"
echo ""
echo "   ⚖️  For fairness:"
echo "      • No forgotten contributors"
echo "      • Transparent calculation"
echo "      • Auditable trail"
echo ""
sleep 3

# Cleanup
kill $SWEETGRASS_PID 2>/dev/null || true
wait $SWEETGRASS_PID 2>/dev/null || true

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ Multiple contributors to one artifact"
echo "   ✅ Role-based attribution weights"
echo "   ✅ Derivation chain attribution"
echo "   ✅ Why fair credit matters"
echo ""
echo "💡 Real-world impact:"
echo "   • Dataset: Alice (29%), Bob (21%), Carol (36%), Dave (14%)"
echo "   • Model: Eve (67%), + dataset contributors (33% total)"
echo "   • Fair compensation when model is used"
echo ""
echo "📁 Output: $OUTPUT_DIR/"
echo ""
echo "⏭️  Next: Learn about querying provenance"
echo "   cd ../03-query-engine && ./demo-filters.sh"
echo ""

