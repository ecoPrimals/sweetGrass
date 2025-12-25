#!/usr/bin/env bash
#
# 🌾 SweetGrass Local Showcase - Automated Tour
# 
# Time: ~50 minutes (6 progressive levels)
# Philosophy: "SweetGrass BY ITSELF is Amazing"
# Following: NestGate's local-first pattern
#
# This script runs all local showcase demos in sequence with:
# - Explanatory text between demos
# - Pauses for observation
# - Progress tracking
# - Colored, narrative output
#
# Usage: ./RUN_ME_FIRST.sh

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Progress tracking
TOTAL_LEVELS=7
CURRENT_LEVEL=0
START_TIME=$(date +%s)

# Print header
clear
echo ""
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}${BLUE}  🌾 SweetGrass Local Showcase - Automated Tour${NC}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Time Estimate:${NC} ~55 minutes (7 progressive levels)"
echo -e "${YELLOW}Philosophy:${NC} \"SweetGrass BY ITSELF is Amazing\""
echo -e "${YELLOW}Pattern:${NC} Local-first (inspired by NestGate)"
echo ""
echo -e "${GREEN}What you'll learn:${NC}"
echo "  • Create and query Braids (provenance records)"
echo "  • Calculate fair attribution across contributors"
echo "  • Traverse provenance graphs"
echo "  • Export to W3C PROV-O standard"
echo "  • Configure privacy controls (GDPR-inspired)"
echo "  • Use multiple storage backends"
echo "  • Verify real execution (no mocks!)"
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}Press Enter to start the tour, or Ctrl+C to exit${NC}"
read -r

# Function to show progress
show_progress() {
    local level=$1
    local total=$2
    local pct=$((level * 100 / total))
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}Progress: Level $level/$total ($pct%)${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Function to pause between levels
pause_between_levels() {
    local next_level=$1
    local next_name=$2
    local next_time=$3
    
    echo ""
    echo -e "${GREEN}✓ Level $((CURRENT_LEVEL)) complete!${NC}"
    echo ""
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}Next: Level $next_level - $next_name${NC} (${CYAN}~$next_time${NC})"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Press Enter to continue, or Ctrl+C to stop here"
    read -r
    clear
}

# Function to show summary
show_summary() {
    local end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    local minutes=$((duration / 60))
    local seconds=$((duration % 60))
    
    clear
    echo ""
    echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${GREEN}  🎉 Showcase Complete! Congratulations!${NC}"
    echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${YELLOW}Time Taken:${NC} ${minutes}m ${seconds}s"
    echo -e "${YELLOW}Levels Completed:${NC} $TOTAL_LEVELS/$TOTAL_LEVELS"
    echo ""
    echo -e "${GREEN}What you've learned:${NC}"
    echo "  ✓ Level 1: Created your first Braid"
    echo "  ✓ Level 2: Calculated fair attribution"
    echo "  ✓ Level 3: Traversed provenance graphs"
    echo "  ✓ Level 4: Exported to W3C PROV-O"
    echo "  ✓ Level 5: Configured privacy controls"
    echo "  ✓ Level 6: Used multiple storage backends"
    echo "  ✓ Level 7: Verified real execution (no mocks!)"
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${BOLD}📚 What's Next?${NC}"
    echo ""
    echo -e "${YELLOW}Level 1: Inter-Primal Integration${NC}"
    echo "  → cd ../01-primal-coordination"
    echo "  → See SweetGrass integrate with Songbird, NestGate, ToadStool"
    echo "  → Time: ~60 minutes"
    echo ""
    echo -e "${YELLOW}Level 2: Federation${NC}"
    echo "  → cd ../02-federation (when available)"
    echo "  → Multi-tower SweetGrass mesh"
    echo "  → Time: ~45 minutes"
    echo ""
    echo -e "${YELLOW}Real-World Value${NC}"
    echo "  → cd ../03-real-world"
    echo "  → See $40M+ demonstrated value"
    echo "  → Time: ~90 minutes"
    echo ""
    echo -e "${GREEN}You're now ready to:${NC}"
    echo "  • Integrate SweetGrass into your projects"
    echo "  • Build provenance-tracked applications"
    echo "  • Calculate fair attribution"
    echo "  • Export to standard formats"
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${BOLD}${BLUE}🌾 Thank you for exploring SweetGrass! 🌾${NC}"
    echo ""
}

# ============================================================================
# Level 1: Hello Provenance (~5 minutes)
# ============================================================================

CURRENT_LEVEL=1
show_progress $CURRENT_LEVEL $TOTAL_LEVELS

echo -e "${BOLD}${BLUE}Level 1: Hello Provenance${NC} (${CYAN}~5 minutes${NC})"
echo ""
echo "Your first Braid - understanding provenance basics."
echo ""
echo -e "${YELLOW}What you'll see:${NC}"
echo "  • Create a Braid from raw data"
echo "  • Query the Braid by ID and hash"
echo "  • Understand content-addressable storage"
echo "  • See provenance metadata"
echo ""
sleep 2

cd "$SCRIPT_DIR/01-hello-provenance"
./demo-first-braid.sh

pause_between_levels 2 "Fair Credit" "10 minutes"

# ============================================================================
# Level 2: Attribution Basics (~10 minutes)
# ============================================================================

CURRENT_LEVEL=2
show_progress $CURRENT_LEVEL $TOTAL_LEVELS

echo -e "${BOLD}${BLUE}Level 2: Fair Credit${NC} (${CYAN}~10 minutes${NC})"
echo ""
echo "Calculate attribution across contribution chains."
echo ""
echo -e "${YELLOW}What you'll see:${NC}"
echo "  • Role-based weights (Creator: 1.0, Contributor: 0.5)"
echo "  • Attribution propagation through derivations"
echo "  • Time decay calculations"
echo "  • Final reward proportions"
echo ""
echo -e "${GREEN}Example:${NC}"
echo "  Alice creates document (100%)"
echo "  Bob adds analysis → Alice: 70%, Bob: 30%"
echo "  Charlie visualizes → Alice: 49%, Bob: 21%, Charlie: 30%"
echo ""
sleep 2

cd "$SCRIPT_DIR/02-attribution-basics"
./demo-fair-credit.sh

pause_between_levels 3 "Provenance Queries" "10 minutes"

# ============================================================================
# Level 3: Query Engine (~10 minutes)
# ============================================================================

CURRENT_LEVEL=3
show_progress $CURRENT_LEVEL $TOTAL_LEVELS

echo -e "${BOLD}${BLUE}Level 3: Provenance Queries${NC} (${CYAN}~10 minutes${NC})"
echo ""
echo "Traverse the provenance graph to understand data history."
echo ""
echo -e "${YELLOW}What you'll see:${NC}"
echo "  • Build provenance graph from any Braid"
echo "  • Walk ancestors (sources)"
echo "  • Walk descendants (derivatives)"
echo "  • Filter by activity, agent, time"
echo ""
sleep 2

cd "$SCRIPT_DIR/03-query-engine"
./demo-filters.sh

pause_between_levels 4 "PROV-O Standard" "5 minutes"

# ============================================================================
# Level 4: PROV-O Export (~5 minutes)
# ============================================================================

CURRENT_LEVEL=4
show_progress $CURRENT_LEVEL $TOTAL_LEVELS

echo -e "${BOLD}${BLUE}Level 4: PROV-O Standard${NC} (${CYAN}~5 minutes${NC})"
echo ""
echo "Export to W3C standard JSON-LD format for interoperability."
echo ""
echo -e "${YELLOW}What you'll see:${NC}"
echo "  • Convert Braids to PROV-O entities"
echo "  • Activities as prov:Activity"
echo "  • Agents as prov:Agent"
echo "  • Standard JSON-LD context"
echo ""
sleep 2

cd "$SCRIPT_DIR/04-prov-o-standard"
./demo-prov-o-export.sh

pause_between_levels 5 "Privacy Controls" "10 minutes"

# ============================================================================
# Level 5: Privacy Controls (~10 minutes)
# ============================================================================

CURRENT_LEVEL=5
show_progress $CURRENT_LEVEL $TOTAL_LEVELS

echo -e "${BOLD}${BLUE}Level 5: Privacy Controls${NC} (${CYAN}~10 minutes${NC})"
echo ""
echo "GDPR-inspired data subject rights built into SweetGrass."
echo ""
echo -e "${YELLOW}What you'll see:${NC}"
echo "  • Privacy levels (Public, Private, Encrypted)"
echo "  • Retention policies"
echo "  • Data subject requests (Access, Erasure)"
echo "  • Consent tracking"
echo ""
sleep 2

cd "$SCRIPT_DIR/05-privacy-controls"
./demo-privacy.sh

pause_between_levels 6 "Storage Backends" "10 minutes"

# ============================================================================
# Level 6: Storage Backends (~10 minutes)
# ============================================================================

CURRENT_LEVEL=6
show_progress $CURRENT_LEVEL $TOTAL_LEVELS

echo -e "${BOLD}${BLUE}Level 6: Storage Backends${NC} (${CYAN}~10 minutes${NC})"
echo ""
echo "Multiple storage options for different use cases."
echo ""
echo -e "${YELLOW}What you'll see:${NC}"
echo "  • Memory backend (testing, ephemeral)"
echo "  • PostgreSQL backend (production, multi-node)"
echo "  • Sled backend (embedded, single-node, Pure Rust)"
echo ""
sleep 2

cd "$SCRIPT_DIR/06-storage-backends"
./demo-backends.sh

pause_between_levels 7 "Real Verification" "5 minutes"

# ============================================================================
# Level 7: Real Verification (~5 minutes)
# ============================================================================

CURRENT_LEVEL=7
show_progress $CURRENT_LEVEL $TOTAL_LEVELS

echo -e "${BOLD}${BLUE}Level 7: Real Verification${NC} (${CYAN}~5 minutes${NC})"
echo ""
echo "Prove that all demos use REAL SweetGrass, not mocks."
echo ""
echo -e "${YELLOW}What you'll see:${NC}"
echo "  • 10-point verification checklist"
echo "  • Real binary validation"
echo "  • Real process and port verification"
echo "  • Real HTTP API responses"
echo "  • Interactive verification commands"
echo ""
sleep 2

cd "$SCRIPT_DIR/07-real-verification"
./demo-no-mocks.sh

# ============================================================================
# Summary
# ============================================================================

show_summary

# Success exit
exit 0
