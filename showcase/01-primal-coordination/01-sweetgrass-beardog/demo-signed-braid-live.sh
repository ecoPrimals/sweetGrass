#!/bin/bash
# Live SweetGrass + BearDog Integration Demo
# Uses actual BearDog binary from phase2/bins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="$(cd "$PROJECT_ROOT/../bins" && pwd)"

echo ""
echo "🌾 SweetGrass + BearDog LIVE Demo"
echo "=================================="
echo ""

# Check for BearDog binary
if [ ! -x "$BINS_DIR/beardog" ]; then
    echo "❌ BearDog binary not found at $BINS_DIR/beardog"
    echo "   Run: cd ../../phase1/bearDog && cargo build --release -p beardog-cli"
    echo "   Then: cp target/release/beardog ../../phase2/bins/"
    exit 1
fi

echo "✅ BearDog binary found: $BINS_DIR/beardog"
echo ""

# Show BearDog status
echo "📋 BearDog Status:"
echo "─────────────────"
$BINS_DIR/beardog --version
echo ""

# Discover HSMs
echo "🔍 Discovering HSMs:"
echo "────────────────────"
$BINS_DIR/beardog hsm discover 2>&1 || echo "   (No physical HSMs - using software fallback)"
echo ""

# Show capabilities  
echo "🛡️  Security Capabilities:"
echo "──────────────────────────"
$BINS_DIR/beardog hsm capabilities 2>&1 | head -20 || echo "   (Checking capabilities...)"
echo ""

# Create a test key for signing
echo "🔐 Key Operations:"
echo "──────────────────"
$BINS_DIR/beardog key --help 2>&1 | head -10
echo ""

# Cross-primal discovery
echo "🌐 Cross-Primal Discovery:"
echo "──────────────────────────"
$BINS_DIR/beardog cross-primal discover-primals 2>&1 | head -10 || echo "   (No primals currently advertising)"
echo ""

# SweetGrass integration concept
echo "🌾 SweetGrass Integration:"
echo "──────────────────────────"
echo "
When BearDog is running as a service, SweetGrass can:

1. Discover BearDog via capability-based lookup
2. Request signatures for Braids using tarpc RPC
3. Verify signatures using BearDog's verification service
4. Track signature provenance in the attribution chain

Example integration code:

  let discovery = LocalDiscovery::new();
  let signer = DiscoverySigner::new(discovery)?;
  
  // BearDog signs the braid
  let signed_braid = signer.sign(&braid).await?;
  
  // Signature recorded in provenance
  assert!(signed_braid.signatures.len() > 0);
"
echo ""
echo "🌾 Live Demo Complete!"
echo ""
echo "Next: Start BearDog as a service to enable live signing"

