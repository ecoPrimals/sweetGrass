#!/bin/bash
# 🌾 SweetGrass Showcase - Quick Health Check
# 
# Verifies all showcase components are ready for demonstration
# Time: ~2 minutes

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${BOLD}${CYAN}🌾 SweetGrass Showcase Health Check${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo ""

ERRORS=0
WARNINGS=0

# Check 1: Local showcase structure
echo -e "${BLUE}[1/6]${NC} Checking local showcase structure..."
EXPECTED_LEVELS=8
FOUND_LEVELS=$(ls -d 00-local-primal/[0-9][0-9]-* 2>/dev/null | wc -l)

if [ "$FOUND_LEVELS" -eq "$EXPECTED_LEVELS" ]; then
    echo -e "  ${GREEN}✅ All $EXPECTED_LEVELS local levels present${NC}"
else
    echo -e "  ${RED}❌ Expected $EXPECTED_LEVELS levels, found $FOUND_LEVELS${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check 2: Demo scripts executable
echo -e "${BLUE}[2/6]${NC} Checking demo script permissions..."
NON_EXEC=$(find 00-local-primal 01-primal-coordination -name "demo-*.sh" ! -executable 2>/dev/null | wc -l)

if [ "$NON_EXEC" -eq 0 ]; then
    echo -e "  ${GREEN}✅ All demo scripts are executable${NC}"
else
    echo -e "  ${YELLOW}⚠️  $NON_EXEC scripts need chmod +x${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 3: Automated tour exists
echo -e "${BLUE}[3/6]${NC} Checking automated tour..."
if [ -x "00-local-primal/RUN_ME_FIRST.sh" ]; then
    echo -e "  ${GREEN}✅ RUN_ME_FIRST.sh present and executable${NC}"
else
    echo -e "  ${RED}❌ RUN_ME_FIRST.sh missing or not executable${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check 4: Phase 1 binary availability
echo -e "${BLUE}[4/6]${NC} Checking Phase 1 primal binaries..."
BINS_DIR="$(cd "$(dirname "$0")/../bins" && pwd 2>/dev/null)" || BINS_DIR="../bins"

EXPECTED_BINS=("songbird-cli" "nestgate" "toadstool-cli" "squirrel" "beardog")
FOUND_BINS=0

for bin in "${EXPECTED_BINS[@]}"; do
    if [ -x "$BINS_DIR/$bin" ]; then
        FOUND_BINS=$((FOUND_BINS + 1))
    fi
done

if [ "$FOUND_BINS" -eq "${#EXPECTED_BINS[@]}" ]; then
    echo -e "  ${GREEN}✅ All $FOUND_BINS primal binaries available${NC}"
elif [ "$FOUND_BINS" -gt 0 ]; then
    echo -e "  ${YELLOW}⚠️  $FOUND_BINS/${#EXPECTED_BINS[@]} binaries found (partial integration possible)${NC}"
    WARNINGS=$((WARNINGS + 1))
else
    echo -e "  ${YELLOW}⚠️  No Phase 1 binaries found (local showcase still works!)${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 5: Documentation completeness
echo -e "${BLUE}[5/6]${NC} Checking documentation..."
DOCS=("00_START_HERE.md" "README.md" "INTEGRATION_GAPS_REPORT.md")
MISSING_DOCS=0

for doc in "${DOCS[@]}"; do
    if [ ! -f "$doc" ]; then
        MISSING_DOCS=$((MISSING_DOCS + 1))
    fi
done

if [ "$MISSING_DOCS" -eq 0 ]; then
    echo -e "  ${GREEN}✅ All showcase documentation present${NC}"
else
    echo -e "  ${YELLOW}⚠️  $MISSING_DOCS documentation files missing${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Check 6: Syntax validation
echo -e "${BLUE}[6/6]${NC} Validating script syntax..."
SYNTAX_ERRORS=0

for script in 00-local-primal/RUN_ME_FIRST.sh 00-local-primal/*/demo-*.sh; do
    if [ -f "$script" ]; then
        if ! bash -n "$script" 2>/dev/null; then
            SYNTAX_ERRORS=$((SYNTAX_ERRORS + 1))
        fi
    fi
done

if [ "$SYNTAX_ERRORS" -eq 0 ]; then
    echo -e "  ${GREEN}✅ All scripts have valid bash syntax${NC}"
else
    echo -e "  ${RED}❌ $SYNTAX_ERRORS scripts have syntax errors${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Summary
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Health Check Summary${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo ""

if [ "$ERRORS" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
    echo -e "${GREEN}${BOLD}🎉 PERFECT HEALTH! 🎉${NC}"
    echo ""
    echo "  ✅ All 8 local levels present"
    echo "  ✅ All scripts executable"
    echo "  ✅ Automated tour ready"
    echo "  ✅ Phase 1 binaries available"
    echo "  ✅ Documentation complete"
    echo "  ✅ No syntax errors"
    echo ""
    echo -e "${GREEN}Status: READY FOR DEMONSTRATION ⭐⭐⭐${NC}"
    echo ""
    echo "Start the automated tour:"
    echo -e "  ${CYAN}cd 00-local-primal && ./RUN_ME_FIRST.sh${NC}"
    EXIT_CODE=0
elif [ "$ERRORS" -eq 0 ]; then
    echo -e "${YELLOW}${BOLD}⚠️  GOOD (with warnings)${NC}"
    echo ""
    echo -e "  ${YELLOW}Warnings: $WARNINGS${NC}"
    echo ""
    echo "Showcase is functional but has minor issues."
    echo "Review warnings above for details."
    echo ""
    echo -e "${YELLOW}Status: READY (address warnings when convenient)${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}${BOLD}❌ ISSUES DETECTED${NC}"
    echo ""
    echo -e "  ${RED}Errors: $ERRORS${NC}"
    echo -e "  ${YELLOW}Warnings: $WARNINGS${NC}"
    echo ""
    echo "Please address the errors above before demonstration."
    EXIT_CODE=1
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════${NC}"
echo ""

exit $EXIT_CODE

