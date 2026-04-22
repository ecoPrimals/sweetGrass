#!/usr/bin/env bash
#
# 🌾 SweetGrass Quick Demo
#
# A 5-minute overview of SweetGrass capabilities.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$SHOWCASE_ROOT/.." && pwd)"

echo ""
echo "🌾 SweetGrass Quick Demo (5 minutes)"
echo "====================================="
echo ""
echo "This demo provides a quick overview of SweetGrass's attribution"
echo "and provenance capabilities."
echo ""

# Check build
echo "📦 Checking build..."
if ! cargo build --manifest-path="$PROJECT_ROOT/Cargo.toml" -q 2>/dev/null; then
    echo "❌ Build failed. Run 'cargo build' first."
    exit 1
fi
echo "✅ Build OK"
echo ""

# Run the demo example
echo "🚀 Running demo..."
echo ""

cd "$PROJECT_ROOT"
if cargo run --example demo --package sweet-grass-service 2>&1; then
    echo ""
    echo "✅ Demo completed successfully!"
else
    echo ""
    echo "⚠️  Demo example failed (BearDog may not be running)."
    echo ""
    echo "📝 What SweetGrass does:"
    echo ""
    echo "  1. BRAIDS - Cryptographic provenance records"
    echo "     - Track who created data"
    echo "     - Track how data was derived"
    echo "     - Track when things happened"
    echo ""
    echo "  2. ATTRIBUTION - Fair credit for contributors"
    echo "     - Role-based weights (Creator > Contributor > Curator)"
    echo "     - Chain propagation (sources get credit in derivatives)"
    echo "     - Time decay (recent contributions weighted more)"
    echo ""
    echo "  3. PROVENANCE - Data history traversal"
    echo "     - Query ancestors (what was used?)"
    echo "     - Query descendants (what was derived?)"
    echo "     - Export to W3C PROV-O standard"
    echo ""
    echo "  4. PRIVACY - GDPR-style data subject rights"
    echo "     - Privacy levels (Public/Private/Encrypted)"
    echo "     - Retention policies (Duration/LegalHold)"
    echo "     - Data subject requests (Access/Erasure)"
    echo ""
fi

echo ""
echo "====================================="
echo "🌾 Quick Demo Complete!"
echo ""
echo "Next steps:"
echo "  - Level 0 (Local Primal): cd 00-local-primal"
echo "  - Level 1 (Coordination): cd 01-primal-coordination"
echo "  - Level 2 (Ecosystem):    cd 02-full-ecosystem"
echo ""
echo "🌾 Every piece of data has a story!"
echo ""

