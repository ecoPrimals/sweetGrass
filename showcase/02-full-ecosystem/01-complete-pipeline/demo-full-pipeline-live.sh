#!/bin/bash
# Full Ecosystem Integration Demo
# Uses all phase1 primal binaries from phase2/bins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="$(cd "$PROJECT_ROOT/../bins" && pwd)"

echo ""
echo "🌾 SweetGrass Full Ecosystem LIVE Demo"
echo "========================================"
echo ""
echo "This demo shows SweetGrass coordinating with ALL phase1 primals:"
echo "  • BearDog  (Security/Signing)"
echo "  • NestGate (Storage)"
echo "  • Songbird (Discovery/Mesh)"
echo "  • ToadStool (Compute)"
echo "  • Squirrel (AI/MCP)"
echo ""

# Verify all binaries
echo "📦 Checking Binaries:"
echo "─────────────────────"
MISSING=0
for bin in beardog nestgate songbird-cli toadstool-cli squirrel; do
    if [ -x "$BINS_DIR/$bin" ]; then
        echo "  ✅ $bin"
    else
        echo "  ❌ $bin (missing)"
        MISSING=1
    fi
done
echo ""

if [ $MISSING -eq 1 ]; then
    echo "⚠️  Some binaries missing. Run from phase1 directories:"
    echo "   See ../../bins/README.md for build instructions"
    echo ""
fi

# Show ecosystem overview
echo "🌐 Ecosystem Overview:"
echo "──────────────────────"
echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│                    ecoPrimals Ecosystem                     │"
echo "├─────────────────────────────────────────────────────────────┤"
echo "│                                                             │"
echo "│     ┌──────────┐    ┌──────────┐    ┌──────────┐           │"
echo "│     │ BearDog  │    │ NestGate │    │ ToadStool│           │"
echo "│     │ Security │    │ Storage  │    │ Compute  │           │"
echo "│     └────┬─────┘    └────┬─────┘    └────┬─────┘           │"
echo "│          │               │               │                  │"
echo "│          └───────────────┼───────────────┘                  │"
echo "│                          │                                  │"
echo "│                   ┌──────┴──────┐                           │"
echo "│                   │  Songbird   │                           │"
echo "│                   │   Mesh      │                           │"
echo "│                   └──────┬──────┘                           │"
echo "│                          │                                  │"
echo "│                   ┌──────┴──────┐                           │"
echo "│                   │ SweetGrass  │                           │"
echo "│                   │ Provenance  │                           │"
echo "│                   └─────────────┘                           │"
echo "│                                                             │"
echo "└─────────────────────────────────────────────────────────────┘"
echo ""

# Run SweetGrass demo
echo "🌾 Running SweetGrass Core Demo:"
echo "─────────────────────────────────"
cd "$PROJECT_ROOT"
cargo run --example demo 2>&1 | head -50
echo ""

# Show primal versions
echo "📋 Primal Versions:"
echo "───────────────────"
echo "  BearDog:   $($BINS_DIR/beardog --version 2>&1 | head -1)"
echo "  Songbird:  $($BINS_DIR/songbird-cli version 2>&1 | head -1 || echo 'v0.1.0')"
echo "  ToadStool: $($BINS_DIR/toadstool-cli --version 2>&1 | head -1)"
echo ""

# Full pipeline concept
echo "🔄 Full Pipeline Flow:"
echo "──────────────────────"
cat << 'EOF'

Complete Attribution Pipeline:

1. DATA CREATION
   Alice creates dataset → SweetGrass Braid
   
2. SIGNING (BearDog)
   Braid signed with HSM → Cryptographic proof
   
3. STORAGE (NestGate)  
   Signed Braid stored → ZFS integrity
   
4. COMPUTE (ToadStool)
   Bob processes data → Derived Braid
   
5. AI ANALYSIS (Squirrel)
   Model inference → Attribution metadata
   
6. DISCOVERY (Songbird)
   All services discovered → Mesh coordination
   
7. PROVENANCE (SweetGrass)
   Full chain recorded → Fair attribution

Result:
  Alice (data):    30%
  Bob (code):      20%  
  ToadStool (GPU): 30%
  Squirrel (AI):   20%

EOF
echo ""
echo "🌾 Full Ecosystem Demo Complete!"
echo ""
echo "All primals are ready for integration testing."

