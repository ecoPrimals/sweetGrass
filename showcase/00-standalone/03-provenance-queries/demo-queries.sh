#!/usr/bin/env bash
#
# 🌾 SweetGrass Demo: Provenance Queries
# Time: ~10 minutes
# Shows: Traverse the provenance DAG to understand data history
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
echo -e "${BLUE}🌾 SweetGrass Demo: Provenance Queries${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create output directory
OUTPUT_DIR="$SCRIPT_DIR/outputs/demo-$(date +%s)"
mkdir -p "$OUTPUT_DIR"
echo -e "${YELLOW}📁 Outputs will be saved to: $OUTPUT_DIR${NC}"
echo ""

sleep 1

# Step 1: Understanding the Provenance Graph
echo -e "${BLUE}🕸️  Step 1: The Provenance Graph (DAG)${NC}"
echo ""
echo "Provenance forms a Directed Acyclic Graph (DAG):"
echo ""
echo "   Raw Data A ──┐"
echo "                ├──> Processed Data C ──> Final Output E"
echo "   Raw Data B ──┘          │"
echo "                            └──> Analysis Report D"
echo ""
echo -e "${YELLOW}   Key Concepts:${NC}"
echo "   • ${BLUE}Ancestors${NC}: Sources (what came before)"
echo "   • ${BLUE}Descendants${NC}: Derivatives (what came after)"
echo "   • ${BLUE}Depth${NC}: How far to traverse"
echo "   • ${BLUE}Cycle Prevention${NC}: DAG ensures no loops"
echo ""

sleep 2

# Step 2: Ancestor Queries
echo -e "${BLUE}🔼 Step 2: Walking Up (Ancestor Queries)${NC}"
echo ""
echo -e "${YELLOW}   Question: Where did this data come from?${NC}"
echo ""
echo "   Query: provenance_ancestors(Final Output E, depth=3)"
echo ""
echo "   Results:"
echo "   ┌─ Level 1: Processed Data C"
echo "   │  ├─ Level 2: Raw Data A"
echo "   │  └─ Level 2: Raw Data B"
echo "   └─ (max depth reached)"
echo ""
echo "   ${GREEN}✅ Shows complete data lineage!${NC}"
echo ""

sleep 2

# Step 3: Descendant Queries
echo -e "${BLUE}🔽 Step 3: Walking Down (Descendant Queries)${NC}"
echo ""
echo -e "${YELLOW}   Question: What was created from this data?${NC}"
echo ""
echo "   Query: provenance_descendants(Processed Data C, depth=2)"
echo ""
echo "   Results:"
echo "   ┌─ Level 1: Final Output E"
echo "   └─ Level 1: Analysis Report D"
echo ""
echo "   ${GREEN}✅ Shows data impact!${NC}"
echo ""

sleep 2

# Step 4: Agent Queries
echo -e "${BLUE}👤 Step 4: Querying by Agent${NC}"
echo ""
echo -e "${YELLOW}   Question: What has Alice contributed?${NC}"
echo ""
echo "   Query: braids_by_agent(did:key:z6MkAlice...)"
echo ""
echo "   Results:"
echo "   • Raw Data A (Creator)"
echo "   • Processed Data C (Contributor)"
echo "   • Final Output E (DataProvider)"
echo ""
echo "   ${GREEN}✅ Shows all of Alice's work!${NC}"
echo ""

sleep 2

# Step 5: Time Range Queries
echo -e "${BLUE}📅 Step 5: Querying by Time Range${NC}"
echo ""
echo -e "${YELLOW}   Question: What happened last week?${NC}"
echo ""
echo "   Query: braids_by_time_range("
echo "       start: 2025-12-17T00:00:00Z,"
echo "       end:   2025-12-24T00:00:00Z"
echo "   )"
echo ""
echo "   Results: All Braids created in that week"
echo ""
echo "   ${GREEN}✅ Temporal provenance!${NC}"
echo ""

sleep 2

# Step 6: Activity Type Queries
echo -e "${BLUE}⚡ Step 6: Querying by Activity Type${NC}"
echo ""
echo -e "${YELLOW}   Question: Show me all ML training runs${NC}"
echo ""
echo "   Query: braids_by_activity(ActivityType::MLTraining)"
echo ""
echo "   Results:"
echo "   • Model Training v1"
echo "   • Model Training v2"
echo "   • Model Training v3"
echo ""
echo "   ${GREEN}✅ Filter by what happened!${NC}"
echo ""

sleep 2

# Step 7: Advanced Features
echo -e "${BLUE}🚀 Step 7: Advanced Query Features${NC}"
echo ""
echo -e "${YELLOW}   Depth Limiting:${NC}"
echo "   • Prevents traversing entire graph"
echo "   • Default: 5 levels"
echo "   • Configurable per query"
echo ""
echo -e "${YELLOW}   Cycle Detection:${NC}"
echo "   • DAG structure prevents cycles"
echo "   • Safe to traverse any path"
echo "   • No infinite loops possible"
echo ""
echo -e "${YELLOW}   Efficient Traversal:${NC}"
echo "   • Uses indexes for fast lookups"
echo "   • Lazy loading of Braid details"
echo "   • Pagination for large results"
echo ""

sleep 2

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ Provenance forms a DAG (Directed Acyclic Graph)"
echo "   ✅ Walk ancestors to find sources"
echo "   ✅ Walk descendants to find derivatives"
echo "   ✅ Query by agent, time, activity type"
echo "   ✅ Depth limiting prevents infinite traversal"
echo ""
echo "💡 Key Insight:"
echo "   The provenance graph tells the COMPLETE STORY"
echo "   of where data came from and where it went."
echo ""
echo "📁 Output saved to: $OUTPUT_DIR"
echo ""
echo "⏭️  Next: Learn about PROV-O Export"
echo "   cd ../04-provo-export && ./demo-export.sh"
echo ""
echo "🌾 Every query reveals part of the story!"
echo ""
