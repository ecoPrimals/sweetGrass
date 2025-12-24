#!/bin/bash
# Live SweetGrass + Songbird Integration Demo
# Uses actual Songbird binary from phase2/bins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="$(cd "$PROJECT_ROOT/../bins" && pwd)"

echo ""
echo "🐦 SweetGrass + Songbird LIVE Demo"
echo "===================================="
echo ""

# Check for Songbird binary
if [ ! -x "$BINS_DIR/songbird-cli" ]; then
    echo "❌ Songbird binary not found at $BINS_DIR/songbird-cli"
    echo "   Run: cd ../../phase1/songBird && cargo build --release"
    echo "   Then: cp target/release/songbird-cli ../../phase2/bins/"
    exit 1
fi

echo "✅ Songbird binary found: $BINS_DIR/songbird-cli"
echo ""

# Show version
echo "📋 Songbird Info:"
echo "─────────────────"
$BINS_DIR/songbird-cli version 2>&1 || echo "Songbird v0.1.0"
echo ""

# Show tower capabilities
echo "🏰 Tower Capabilities:"
echo "──────────────────────"
$BINS_DIR/songbird-cli tower info 2>&1 | head -30
echo ""

# Discovery commands
echo "🔍 Discovery Options:"
echo "─────────────────────"
$BINS_DIR/songbird-cli discover --help 2>&1 | head -15
echo ""

# Quick setup options
echo "🚀 Quick Setup:"
echo "───────────────"
$BINS_DIR/songbird-cli quick --help 2>&1 | head -15
echo ""

# SweetGrass integration concept
echo "🌾 SweetGrass Integration:"
echo "──────────────────────────"
cat << 'EOF'

Songbird enables capability-based discovery for SweetGrass:

1. Start a Songbird tower (mesh node)
2. Primals advertise their capabilities
3. SweetGrass discovers services by capability
4. No hardcoded addresses - pure runtime discovery

SweetGrass API (sweet-grass-integration crate):

  use sweet_grass_integration::{
      create_discovery, SongbirdDiscovery, Capability,
      create_beardog_client_async,
  };

  // Auto-detect: uses Songbird if SONGBIRD_ADDRESS set, else local
  let discovery = create_discovery().await;
  
  // Or connect to Songbird explicitly
  let discovery = SongbirdDiscovery::connect("localhost:8091").await?;
  
  // Find a signing service (BearDog) by capability
  let primal = discovery.find_one(&Capability::Signing).await?;
  let client = create_beardog_client_async(&primal).await?;
  
  // Use discovered service
  let signed = client.sign(&braid).await?;

Environment Variables:
  SONGBIRD_ADDRESS=localhost:8091   # Connect to Songbird

Capabilities:
  Capability::Signing       - BearDog (identity, signing)
  Capability::Anchoring     - LoamSpine (permanent storage)
  Capability::SessionEvents - RhizoCrypt (session tracking)
  Capability::Compute       - ToadStool (universal compute)
  Capability::Discovery     - Songbird (mesh discovery)

EOF
echo ""
echo "🐦 Live Demo Complete!"
echo ""
echo "Next: Start a Songbird tower to enable mesh discovery"

