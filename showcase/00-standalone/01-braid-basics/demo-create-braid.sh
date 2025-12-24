#!/usr/bin/env bash
#
# 🌾 SweetGrass Demo: Braid Basics
# Time: ~5 minutes
# Shows: Create, query, and derive Braids
# Prerequisites: SweetGrass built (`cargo build`)

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo -e "${BLUE}🌾 SweetGrass Demo: Braid Basics${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"
echo -e "${YELLOW}📁 Outputs will be saved to: $OUTPUT_DIR${NC}"
echo ""

sleep 1

# Step 1: Build project
echo -e "${BLUE}🔨 Step 1: Checking build status...${NC}"
if [ -f "$PROJECT_ROOT/target/debug/examples/demo" ] || [ -f "$PROJECT_ROOT/target/release/examples/demo" ]; then
    echo -e "${GREEN}   ✅ SweetGrass already built${NC}"
else
    echo -e "${YELLOW}   ⚙️  Building SweetGrass (first time may take a few minutes)...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --example demo --package sweet-grass-service > "$OUTPUT_DIR/build.log" 2>&1
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}   ✅ Build successful${NC}"
    else
        echo -e "${RED}   ❌ Build failed. See $OUTPUT_DIR/build.log${NC}"
        exit 1
    fi
fi
echo ""

sleep 1

# Step 2: Create a Braid
echo -e "${BLUE}📦 Step 2: Creating a Braid from data...${NC}"
echo ""
echo -e "${YELLOW}   Input Data:${NC}"
echo "   \"Hello, SweetGrass! This is provenance in action.\""
echo ""
echo -e "${YELLOW}   What's happening:${NC}"
echo "   • Computing SHA-256 content hash"
echo "   • Generating unique Braid ID (URN format)"
echo "   • Attaching provenance metadata"
echo "   • Creating cryptographic signature"
echo ""

# Run demo (capture output)
cd "$PROJECT_ROOT"
cargo run --example demo --package sweet-grass-service 2>&1 | tee "$OUTPUT_DIR/demo-output.txt" | head -50

echo ""
echo -e "${GREEN}   ✅ Braid created successfully!${NC}"
echo ""

sleep 1

# Step 3: Explain what happened
echo -e "${BLUE}🔍 Step 3: Understanding the Braid...${NC}"
echo ""
echo -e "${YELLOW}   Key Components:${NC}"
echo ""
echo "   • ${BLUE}BraidId${NC}: urn:braid:abc123-def456-..."
echo "     Unique identifier for this provenance record"
echo ""
echo "   • ${BLUE}ContentHash${NC}: sha256:7f83b1657ff..."
echo "     Cryptographic fingerprint of the data"
echo ""
echo "   • ${BLUE}was_attributed_to${NC}: did:key:z6MkAlice..."
echo "     Who created this (Decentralized Identifier)"
echo ""
echo "   • ${BLUE}was_generated_by${NC}: activity:process-123"
echo "     What activity created this"
echo ""
echo "   • ${BLUE}was_derived_from${NC}: [parent braids]"
echo "     Links to source data (empty for original)"
echo ""

sleep 2

# Step 4: Show use cases
echo -e "${BLUE}💡 Step 4: Real-World Use Cases${NC}"
echo ""
echo "   📊 Data Science:"
echo "      Track datasets through transformation pipelines"
echo ""
echo "   🎨 Content Creation:"
echo "      Attribution for remixes, derivatives, collaborations"
echo ""
echo "   🔬 Research:"
echo "      Reproducible science with full data lineage"
echo ""
echo "   💰 Rewards:"
echo "      Fair compensation based on contribution chains"
echo ""

sleep 2

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ Braids are immutable provenance records"
echo "   ✅ Each Braid has a unique ID and content hash"
echo "   ✅ Metadata follows W3C PROV-O standard"
echo "   ✅ Perfect for attribution and rewards"
echo ""
echo "📁 Output saved to: $OUTPUT_DIR"
echo ""
echo "⏭️  Next: Learn about Attribution"
echo "   cd ../02-attribution-engine && ./demo-attribution.sh"
echo ""
echo "🌾 Every piece of data has a story!"
echo ""
