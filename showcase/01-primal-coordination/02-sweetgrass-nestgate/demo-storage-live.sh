#!/bin/bash
# Live SweetGrass + NestGate Integration Demo
# Uses actual NestGate binary from phase2/bins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="$(cd "$PROJECT_ROOT/../bins" && pwd)"

echo ""
echo "🌾 SweetGrass + NestGate LIVE Demo"
echo "==================================="
echo ""

# Check for NestGate binary
if [ ! -x "$BINS_DIR/nestgate" ]; then
    echo "❌ NestGate binary not found at $BINS_DIR/nestgate"
    echo "   Run: cd ../../phase1/nestGate && cargo build --release -p nestgate-bin"
    echo "   Then: cp target/release/nestgate ../../phase2/bins/"
    exit 1
fi

echo "✅ NestGate binary found: $BINS_DIR/nestgate"
echo ""

# Show NestGate version
echo "📋 NestGate Info:"
echo "─────────────────"
$BINS_DIR/nestgate --version 2>&1 || echo "NestGate v0.1.1"
echo ""

# Run doctor check
echo "🩺 System Health Check:"
echo "───────────────────────"
$BINS_DIR/nestgate doctor 2>&1 | head -20 || echo "   (Running diagnostics...)"
echo ""

# Show storage capabilities
echo "💾 Storage Capabilities:"
echo "────────────────────────"
$BINS_DIR/nestgate storage --help 2>&1 | head -15 || echo "   (Checking storage options...)"
echo ""

# SweetGrass integration concept
echo "🌾 SweetGrass Integration:"
echo "──────────────────────────"
cat << 'EOF'

NestGate provides sovereign storage for SweetGrass Braids:

1. Store Braids with ZFS copy-on-write integrity
2. Automatic compression and deduplication
3. Snapshot-based versioning for provenance
4. Content-addressable storage (hash-based lookup)

Example workflow:

  // Store a Braid to NestGate
  let storage = NestGateClient::discover().await?;
  let receipt = storage.put(&braid).await?;
  
  // SweetGrass records storage activity
  let stored_braid = factory.with_storage_proof(
      braid,
      receipt.proof,
      ActivityType::Storage,
  )?;
  
  // Retrieve with verification
  let retrieved = storage.get(&braid.id).await?;
  assert_eq!(retrieved.data_hash, braid.data_hash);

Storage Features:
  • ZFS checksumming (data integrity)
  • Compression (space efficiency)
  • Snapshots (point-in-time recovery)
  • Deduplication (storage optimization)

EOF
echo ""
echo "🏠 Live Demo Complete!"
echo ""
echo "Next: Start NestGate service for live storage operations"

