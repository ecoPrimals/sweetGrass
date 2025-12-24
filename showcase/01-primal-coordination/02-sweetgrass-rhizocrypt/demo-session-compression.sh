#!/usr/bin/env bash
#
# 🌾 SweetGrass + RhizoCrypt Demo
#
# This demo shows how to compress sessions to Braids.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo "🌾 SweetGrass + RhizoCrypt Demo"
echo "==============================="
echo ""

cd "$PROJECT_ROOT"

# Try to run the demo which includes session compression
if cargo run --example demo --package sweet-grass-service 2>&1 | grep -A 25 "Part 5: Session Compression"; then
    echo ""
else
    echo "📝 Session Compression Overview"
    echo ""
    echo "RhizoCrypt captures edit sessions as DAGs."
    echo "SweetGrass compresses these into Braids."
    echo ""
    
    echo "0/1/Many Compression Model:"
    echo "  Rollback/Empty → 0 Braids (discarded)"
    echo "  Single Commit  → 1 Braid (coherent)"
    echo "  Branched DAG   → N Braids (per branch)"
    echo ""
    
    echo "Example Session:"
    echo "  Event: SessionStarted (session-456)"
    echo "  Event: VertexAdded (v1 - import by Alice)"
    echo "  Event: VertexAdded (v2 - transform by Bob)"
    echo "  Event: VertexAdded (v3 - compute by Charlie)"
    echo "  Event: SessionCommitted"
    echo ""
    
    echo "Compression Result:"
    echo "  Model: Single (linear session)"
    echo "  Braids: 1"
    echo "  Derived from: 3 vertices"
    echo "  Attribution:"
    echo "    Alice: 40%"
    echo "    Bob: 35%"
    echo "    Charlie: 25%"
    echo ""
fi

echo ""
echo "🌾 Session Compression Demo Complete!"
echo ""
echo "Key Takeaways:"
echo "  - Sessions compress to 0, 1, or many Braids"
echo "  - Attribution flows from session vertices"
echo "  - Branches become separate Braids"
echo ""

