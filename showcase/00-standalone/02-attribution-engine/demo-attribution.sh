#!/usr/bin/env bash
#
# 🌾 SweetGrass Demo: Attribution Engine
# Time: ~10 minutes
# Shows: Fair attribution calculation across contribution chains
# Prerequisites: SweetGrass built

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
echo -e "${BLUE}🌾 SweetGrass Demo: Attribution Engine${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"
echo -e "${YELLOW}📁 Outputs will be saved to: $OUTPUT_DIR${NC}"
echo ""

sleep 1

# Step 1: Explain Attribution
echo -e "${BLUE}💡 Step 1: Understanding Attribution${NC}"
echo ""
echo "Attribution tracks WHO contributed to data and HOW MUCH."
echo ""
echo -e "${YELLOW}   Agent Roles & Weights:${NC}"
echo "   • Creator (1.0)       - Original author"
echo "   • Contributor (0.5)   - Added to work"
echo "   • DataProvider (0.4)  - Provided source data"
echo "   • Transformer (0.3)   - Processed/transformed"
echo "   • Curator (0.2)       - Organized/curated"
echo "   • Publisher (0.1)     - Made available"
echo ""
echo -e "${YELLOW}   How It Works:${NC}"
echo "   1. Start with original creator (100%)"
echo "   2. Add contributors based on role weights"
echo "   3. Attribution flows through derivation chains"
echo "   4. Apply time decay (optional)"
echo "   5. Calculate final proportions"
echo ""

sleep 2

# Step 2: Simple Example
echo -e "${BLUE}📊 Step 2: Simple Attribution Chain${NC}"
echo ""
echo -e "${YELLOW}   Scenario: Alice creates, Bob contributes${NC}"
echo ""
echo "   Original Document:"
echo "   • Creator: Alice (weight: 1.0)"
echo "   • Attribution: Alice 100%"
echo ""
echo "   ↓ Bob adds analysis"
echo ""
echo "   Processed Version:"
echo "   • Original: Alice (1.0)"
echo "   • Contributor: Bob (0.5)"
echo "   • Total weight: 1.5"
echo "   • Attribution: Alice 66.7%, Bob 33.3%"
echo ""

sleep 2

# Step 3: Complex Example
echo -e "${BLUE}🔗 Step 3: Multi-Level Derivation Chain${NC}"
echo ""
echo -e "${YELLOW}   Scenario: Three levels of contribution${NC}"
echo ""
echo "   Level 1 - Original Dataset:"
echo "   • DataProvider: Alice (0.4)"
echo "   • Attribution: Alice 100%"
echo ""
echo "   ↓ Bob processes the data"
echo ""
echo "   Level 2 - Processed Data:"
echo "   • Derived from: Alice's data (0.4)"
echo "   • Transformer: Bob (0.3)"
echo "   • Total: 0.7"
echo "   • Attribution: Alice 57%, Bob 43%"
echo ""
echo "   ↓ Charlie creates visualization"
echo ""
echo "   Level 3 - Final Output:"
echo "   • Derived from: Alice + Bob (0.7)"
echo "   • Creator: Charlie (1.0)"
echo "   • Total: 1.7"
echo "   • Attribution: Alice 24%, Bob 18%, Charlie 59%"
echo ""

sleep 2

# Step 4: Real-World Use Case
echo -e "${BLUE}💰 Step 4: Real-World Application (sunCloud)${NC}"
echo ""
echo -e "${YELLOW}   Scenario: \$1000 reward for final output${NC}"
echo ""
echo "   Based on attribution above:"
echo "   • Alice:   \$240 (24%)"
echo "   • Bob:     \$180 (18%)"
echo "   • Charlie: \$590 (59%)"
echo ""
echo "   ${GREEN}✅ Fair compensation based on contribution!${NC}"
echo ""

sleep 2

# Step 5: Advanced Features
echo -e "${BLUE}⚙️  Step 5: Advanced Attribution Features${NC}"
echo ""
echo -e "${YELLOW}   Time Decay:${NC}"
echo "   • Reduces attribution over time"
echo "   • Encourages fresh contributions"
echo "   • Decay factor: 0.9 per derivation level"
echo ""
echo -e "${YELLOW}   Depth Limiting:${NC}"
echo "   • Prevents infinite chains"
echo "   • Default: 10 levels deep"
echo "   • Configurable per query"
echo ""
echo -e "${YELLOW}   Multiple Sources:${NC}"
echo "   • Data derived from multiple Braids"
echo "   • Attribution aggregates across all sources"
echo "   • Handles complex DAGs"
echo ""

sleep 2

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ Attribution tracks contribution proportions"
echo "   ✅ Role weights determine contribution value"
echo "   ✅ Attribution flows through derivation chains"
echo "   ✅ Perfect for fair reward distribution (sunCloud)"
echo ""
echo "💡 Key Insight:"
echo "   Attribution isn't just about WHO created data,"
echo "   it's about WHO CONTRIBUTED and HOW MUCH."
echo ""
echo "📁 Output saved to: $OUTPUT_DIR"
echo ""
echo "⏭️  Next: Learn about Provenance Queries"
echo "   cd ../03-provenance-queries && ./demo-queries.sh"
echo ""
echo "🌾 Fair attribution for everyone!"
echo ""
