#!/bin/bash
# 🌾 SweetGrass Automated Tour
# Time: ~40 minutes
# Shows: All standalone capabilities

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

clear

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                    ║${NC}"
echo -e "${BLUE}║            🌾 Welcome to SweetGrass! 🌾           ║${NC}"
echo -e "${BLUE}║                                                    ║${NC}"
echo -e "${BLUE}║         Attribution & Provenance Platform         ║${NC}"
echo -e "${BLUE}║                                                    ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"
echo ""
echo "This automated tour demonstrates everything SweetGrass can do"
echo "as a standalone primal (no other services needed)."
echo ""
echo -e "${YELLOW}⏱️  Total Time: ~40 minutes${NC}"
echo -e "${YELLOW}🎓 What You'll Learn:${NC}"
echo "   • Create cryptographically signed provenance records (Braids)"
echo "   • Calculate fair attribution across contribution chains"
echo "   • Query provenance graphs (DAG traversal)"
echo "   • Export to W3C PROV-O standard"
echo "   • Apply GDPR-style privacy controls"
echo ""
echo -e "${GREEN}Press ENTER to start the tour, or Ctrl+C to exit...${NC}"
read

# Level 1: Braid Basics
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📦 Level 1: Braid Basics (5 min)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Learn the fundamentals of Braids - cryptographically signed"
echo "provenance records that tell the story of your data."
echo ""
echo -e "${GREEN}Press ENTER to continue...${NC}"
read

cd 01-braid-basics
./demo-create-braid.sh
cd ..

echo ""
echo -e "${GREEN}✅ Level 1 Complete!${NC}"
echo ""
echo -e "${YELLOW}What you learned:${NC}"
echo "  • Braids are immutable provenance records"
echo "  • Each Braid has a unique ID and content hash"
echo "  • Braids can be derived from other Braids"
echo "  • All metadata is W3C PROV-O compatible"
echo ""
echo -e "${GREEN}Press ENTER for Level 2...${NC}"
read

# Level 2: Attribution Engine
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}💰 Level 2: Attribution Engine (10 min)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Calculate fair attribution across contribution chains."
echo "See how Creator, Contributor, DataProvider roles get credit."
echo ""
echo -e "${GREEN}Press ENTER to continue...${NC}"
read

cd 02-attribution-engine
./demo-attribution.sh
cd ..

echo ""
echo -e "${GREEN}✅ Level 2 Complete!${NC}"
echo ""
echo -e "${YELLOW}What you learned:${NC}"
echo "  • Attribution flows through derivation chains"
echo "  • Different roles have different weights"
echo "  • Time decay reduces old contributions"
echo "  • Perfect for sunCloud reward distribution"
echo ""
echo -e "${GREEN}Press ENTER for Level 3...${NC}"
read

# Level 3: Provenance Queries
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔍 Level 3: Provenance Queries (10 min)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Traverse the provenance graph (DAG) to understand data history."
echo "Walk ancestors, descendants, query by agent or time."
echo ""
echo -e "${GREEN}Press ENTER to continue...${NC}"
read

cd 03-provenance-queries
./demo-queries.sh
cd ..

echo ""
echo -e "${GREEN}✅ Level 3 Complete!${NC}"
echo ""
echo -e "${YELLOW}What you learned:${NC}"
echo "  • Provenance forms a directed acyclic graph (DAG)"
echo "  • Can walk ancestors (sources) and descendants (derivatives)"
echo "  • Query by agent, time range, activity type"
echo "  • Depth limiting prevents infinite traversal"
echo ""
echo -e "${GREEN}Press ENTER for Level 4...${NC}"
read

# Level 4: PROV-O Export
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📤 Level 4: PROV-O Export (5 min)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Export to W3C standard JSON-LD format."
echo "Interoperate with other provenance systems."
echo ""
echo -e "${GREEN}Press ENTER to continue...${NC}"
read

cd 04-provo-export
./demo-export.sh
cd ..

echo ""
echo -e "${GREEN}✅ Level 4 Complete!${NC}"
echo ""
echo -e "${YELLOW}What you learned:${NC}"
echo "  • PROV-O is the W3C standard for provenance"
echo "  • JSON-LD enables linked data"
echo "  • SweetGrass is fully standards-compliant"
echo "  • Can share provenance with other systems"
echo ""
echo -e "${GREEN}Press ENTER for Level 5...${NC}"
read

# Level 5: Privacy Controls
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔒 Level 5: Privacy Controls (10 min)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "GDPR-inspired data subject rights."
echo "Privacy levels, retention policies, consent tracking."
echo ""
echo -e "${GREEN}Press ENTER to continue...${NC}"
read

cd 05-privacy-controls
./demo-privacy.sh
cd ..

echo ""
echo -e "${GREEN}✅ Level 5 Complete!${NC}"
echo ""
echo -e "${YELLOW}What you learned:${NC}"
echo "  • Privacy levels: Public, Private, Encrypted, etc."
echo "  • Retention policies: Duration, LegalHold, etc."
echo "  • Data subject rights: Access, Erasure, Portability"
echo "  • GDPR compliance built-in"
echo ""

# Final Summary
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                    ║${NC}"
echo -e "${GREEN}║        🎉 Congratulations! Tour Complete! 🎉      ║${NC}"
echo -e "${GREEN}║                                                    ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}You've completed the SweetGrass standalone tour!${NC}"
echo ""
echo "🎓 You now understand:"
echo "   ✅ Braids - Immutable provenance records"
echo "   ✅ Attribution - Fair reward calculation"
echo "   ✅ Queries - Provenance graph traversal"
echo "   ✅ PROV-O - Standards compliance"
echo "   ✅ Privacy - GDPR-style controls"
echo ""
echo "📚 Next Steps:"
echo ""
echo "  1️⃣  Explore integration with other primals:"
echo "     cd ../01-primal-coordination"
echo "     cat README.md"
echo ""
echo "  2️⃣  Try the full ecosystem demos:"
echo "     cd ../02-full-ecosystem"
echo "     cat README.md"
echo ""
echo "  3️⃣  Experiment with your own data:"
echo "     - Create complex derivation chains"
echo "     - Try different attribution weights"
echo "     - Test privacy access controls"
echo ""
echo "📁 All demo outputs saved in:"
echo "   showcase/00-standalone/*/outputs/"
echo ""
echo -e "${GREEN}🌾 Thank you for exploring SweetGrass! 🌾${NC}"
echo ""

