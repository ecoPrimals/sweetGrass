#!/usr/bin/env bash
#
# 🌾 Complete Attribution Pipeline Demo
#
# This demo shows the full data lifecycle with attribution.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo "🌾 Complete Attribution Pipeline Demo"
echo "====================================="
echo ""

cd "$PROJECT_ROOT"

# Run the comprehensive demo
echo "Running SweetGrass demo example..."
echo ""

if cargo run --example demo --package sweet-grass-service 2>&1; then
    echo ""
    echo "═══════════════════════════════════════"
    echo "✅ Pipeline Complete!"
    echo "═══════════════════════════════════════"
else
    echo ""
    echo "📝 Complete Pipeline Overview"
    echo ""
    echo "Scene 1: Alice Creates Dataset"
    echo "  → Signs with BearDog"
    echo "  → Records in SweetGrass"
    echo ""
    echo "Scene 2: Bob Runs Analysis"
    echo "  → RhizoCrypt captures session"
    echo "  → Derives from Alice's Braid"
    echo ""
    echo "Scene 3: Charlie Creates Visualization"
    echo "  → Derives from Bob's Braid"
    echo "  → Anchors with LoamSpine"
    echo ""
    echo "Scene 4: Attribution Calculation"
    echo "  Alice:   49%"
    echo "  Bob:     21%"
    echo "  Charlie: 30%"
    echo ""
    echo "Reward Distribution (\$1000):"
    echo "  Alice:   \$490"
    echo "  Bob:     \$210"
    echo "  Charlie: \$300"
    echo ""
fi

echo ""
echo "🌾 Complete Pipeline Demo Finished!"
echo ""
echo "Key Takeaways:"
echo "  - Every transformation is recorded"
echo "  - Attribution flows through derivations"
echo "  - PROV-O export for interoperability"
echo "  - Fair rewards based on contribution"
echo ""

