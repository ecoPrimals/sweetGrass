#!/usr/bin/env bash
#
# 🌾 SweetGrass Primal Coordination Demos
# Shows: SweetGrass working with other primals (Songbird, Beardog, Nestgate, Squirrel)
# Time: ~20 minutes
# Prerequisites: Phase1 primals built at ../../bins/

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BINS_DIR="$PROJECT_ROOT/../bins"

echo ""
echo -e "${CYAN}🌾 SweetGrass: Primal Coordination Showcase${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${YELLOW}Prerequisites:${NC}"
echo "  ✅ SweetGrass built"
echo "  ✅ Phase1 primals at $BINS_DIR"
echo ""

sleep 1

# Check for binaries
echo -e "${BLUE}🔍 Checking for phase1 primal binaries...${NC}"
MISSING=()
for primal in songbird beardog nestgate squirrel; do
    if [ ! -f "$BINS_DIR/$primal" ]; then
        MISSING+=("$primal")
    fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
    echo -e "${RED}❌ Missing binaries: ${MISSING[*]}${NC}"
    echo ""
    echo "Please build phase1 primals first:"
    echo "  cd ../../phase1"
    echo "  ./build-all.sh"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ All required binaries found!${NC}"
echo ""

sleep 1

# Overview
echo -e "${BLUE}📚 What You'll Learn${NC}"
echo ""
echo "These demos show SweetGrass working with:"
echo ""
echo "  🐦 ${YELLOW}Songbird${NC}  - Discovery & Identity (DIDs)"
echo "  🐻 ${YELLOW}Beardog${NC}   - Compute & ML Training"
echo "  🏠 ${YELLOW}Nestgate${NC}  - Session & Attestation"
echo "  🐿️  ${YELLOW}Squirrel${NC}  - State Management"
echo ""
echo "All demos use capability-based discovery:"
echo "  • No hardcoded addresses or ports"
echo "  • Zero-knowledge startup"
echo "  • Runtime primal discovery"
echo ""

sleep 2

# Demo Menu
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Available Demos:${NC}"
echo ""
echo "  1. ${BLUE}Discovery Integration${NC}"
echo "     • SweetGrass registers with Songbird"
echo "     • Other primals discover SweetGrass capabilities"
echo "     • Time: ~5 minutes"
echo ""
echo "  2. ${BLUE}ML Training Provenance${NC}"
echo "     • Beardog runs ML training"
echo "     • SweetGrass tracks full provenance"
echo "     • Attribution across compute & data"
echo "     • Time: ~8 minutes"
echo ""
echo "  3. ${BLUE}Session-Aware Braids${NC}"
echo "     • Nestgate manages user sessions"
echo "     • SweetGrass links Braids to sessions"
echo "     • Audit trails with attestations"
echo "     • Time: ~7 minutes"
echo ""

sleep 1

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
read -p "$(echo -e ${YELLOW}Run all demos? [Y/n]:${NC} )" -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]] && [[ -n $REPLY ]]; then
    echo "Demos cancelled."
    exit 0
fi

# Create shared log directory
mkdir -p "$SCRIPT_DIR/logs"
mkdir -p "$SCRIPT_DIR/pids"

# Run demos
echo ""
echo -e "${GREEN}🚀 Starting Demo Sequence...${NC}"
echo ""

# Demo 1: Discovery
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Demo 1: Discovery Integration${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
cd "$SCRIPT_DIR/01-discovery-integration"
./demo-discovery.sh
echo ""
read -p "$(echo -e ${YELLOW}Press Enter to continue to Demo 2...${NC})"
echo ""

# Demo 2: ML Training
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Demo 2: ML Training Provenance${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
cd "$SCRIPT_DIR/02-ml-training-provenance"
./demo-ml-provenance.sh
echo ""
read -p "$(echo -e ${YELLOW}Press Enter to continue to Demo 3...${NC})"
echo ""

# Demo 3: Session-Aware
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Demo 3: Session-Aware Braids${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
cd "$SCRIPT_DIR/03-session-aware-braids"
./demo-session-braids.sh
echo ""

# Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 All Primal Coordination Demos Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "🎓 What you learned:"
echo "   ✅ SweetGrass integrates seamlessly with all primals"
echo "   ✅ Capability-based discovery (no hardcoding)"
echo "   ✅ Full provenance across primal boundaries"
echo "   ✅ ML training attribution with Beardog"
echo "   ✅ Session-aware audit trails with Nestgate"
echo ""
echo "💡 Key Insight:"
echo "   Primals compose to create powerful capabilities"
echo "   while maintaining sovereignty and zero-knowledge."
echo ""
echo "📁 Logs saved to: $SCRIPT_DIR/logs/"
echo ""
echo "🔍 Want to explore more?"
echo "   • View individual demo scripts for details"
echo "   • Check $SCRIPT_DIR/README.md for architecture"
echo "   • Experiment with your own scenarios!"
echo ""
echo "🌾 Thank you for exploring SweetGrass!"
echo ""

