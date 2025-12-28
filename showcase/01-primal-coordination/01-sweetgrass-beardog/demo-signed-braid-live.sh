#!/bin/bash
# SweetGrass + BearDog Integration Demo — Architecture Gap Discovery
# Uses actual BearDog binary to demonstrate capabilities and design

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="$(cd "$PROJECT_ROOT/../../../primalBins" 2>/dev/null || cd "$PROJECT_ROOT/../bins")"

echo ""
echo "🌾 SweetGrass + BearDog Integration Demo"
echo "==========================================="
echo ""
echo "⚠️  ARCHITECTURE NOTE (Dec 28, 2025):"
echo "   BearDog: HTTP REST API (port 9000)"
echo "   SweetGrass: tarpc RPC (expected)"
echo "   Status: Architecture mismatch discovered"
echo "   Path Forward: HTTP adapter (2-3 hours to implement)"
echo ""
echo "📋 What This Demo Shows:"
echo "   1. BearDog capabilities (real CLI)"
echo "   2. Integration design (capability-based)"
echo "   3. Gap analysis & path forward"
echo ""
echo "📖 See: BEARDOG_INTEGRATION_GAP.md for full details"
echo ""

# Check for BearDog binary
if [ ! -x "$BINS_DIR/beardog" ]; then
    echo "❌ BearDog binary not found"
    echo "   Expected at: $BINS_DIR/beardog"
    exit 1
fi

echo "✅ BearDog binary found"
echo ""

# Show BearDog status
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 BearDog Capabilities (Real Binary)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

$BINS_DIR/beardog --version
echo ""

# Show key management
echo "🔐 Key Management:"
echo "──────────────────"
$BINS_DIR/beardog key --help 2>&1 | head -15
echo ""

# Show encryption capabilities
echo "🔒 Encryption:"
echo "──────────────"
$BINS_DIR/beardog encrypt --help 2>&1 | head -8
echo ""

# Show cross-primal capabilities
echo "🌐 Cross-Primal Integration:"
echo "────────────────────────────"
$BINS_DIR/beardog cross-primal --help 2>&1 | head -12
echo ""

# Architecture gap explanation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 Integration Architecture Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "SweetGrass Integration Expectations:"
echo "  • RPC: tarpc (TCP + bincode serialization)"
echo "  • Trait: SigningRpc { sign_braid(), verify_braid(), ... }"
echo "  • Discovery: Capability::Signing"
echo "  • Port: Discovered at runtime"
echo ""
echo "BearDog Current Implementation:"
echo "  • API: HTTP REST (JSON over HTTP)"
echo "  • Server: unified_api_server.rs (port 9000)"
echo "  • Capabilities: Genesis, BTSP, key management"
echo "  • Protocol: RESTful endpoints"
echo ""
echo "Gap Discovered:"
echo "  ⚠️  RPC architecture mismatch (tarpc vs HTTP)"
echo "  ⚠️  Cannot directly connect without adapter"
echo ""

# Integration paths
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Path Forward (3 Options)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Option A: HTTP REST Adapter (RECOMMENDED)"
echo "  • Create HttpSigningClient in sweet-grass-integration"
echo "  • Works with current BearDog"
echo "  • Time: 2-3 hours"
echo "  • Status: Unblocks showcase immediately"
echo ""
echo "Option B: BearDog tarpc Service"
echo "  • Add beardog-tarpc-service crate to BearDog"
echo "  • Unified RPC architecture"
echo "  • Time: 6-8 hours + BearDog team coordination"
echo "  • Status: Future enhancement"
echo ""
echo "Option C: Mock-Based Testing (CURRENT)"
echo "  • MockSigningClient in tests"
echo "  • Works now for unit/integration tests"
echo "  • Status: ✅ Active in SweetGrass test suite"
echo ""

# Design demonstration
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌾 Integration Design (Capability-Based)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
cat << 'EOF'
// SweetGrass discovers signing capability (not hardcoded to BearDog!)
let discovery = create_discovery().await;
let primal = discovery.find_one(&Capability::Signing).await?;

// Connect via discovered address (HTTP adapter would go here)
let client = create_signing_client_async(&primal).await?;

// Sign a Braid
let signed_braid = client.sign(&braid).await?;

// Verify signature
let is_valid = client.verify(&signed_braid).await?;
assert!(is_valid);

// Track in provenance
// Signature recorded as Activity in Braid provenance chain
EOF
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Demo Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📖 Read: BEARDOG_INTEGRATION_GAP.md (full analysis)"
echo "🔧 Next: Implement HTTP adapter (Option A)"
echo "🌾 Status: Gap documented honestly, path forward clear"
echo ""

