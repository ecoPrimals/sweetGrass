#!/usr/bin/env bash
#
# 🌾 SweetGrass + LoamSpine Demo
#
# This demo shows how to anchor Braids for immutability.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo "🌾 SweetGrass + LoamSpine Demo"
echo "=============================="
echo ""

echo "📝 Anchoring Overview"
echo ""
echo "LoamSpine provides immutable commit anchoring."
echo "Anchoring a Braid creates a tamper-proof timestamp"
echo "and establishes ordering in an append-only log."
echo ""

echo "Step 1: Capability Discovery"
echo "───────────────────────────"
echo "  Looking for capability: Anchoring"
echo "  → LoamSpine discovered at localhost:8093"
echo ""

echo "Step 2: Create Braid"
echo "────────────────────"
echo "  Data: \"Critical audit record\""
echo "  → Braid created: urn:braid:xyz789"
echo ""

echo "Step 3: Anchor to Spine"
echo "───────────────────────"
echo "  Spine ID: spine-main"
echo "  → Anchor created"
echo ""

echo "Anchor Receipt:"
echo "  Braid ID: urn:braid:xyz789"
echo "  Spine ID: spine-main"
echo "  Entry Hash: sha256:abc123..."
echo "  Index: 42"
echo "  Anchored At: 2025-12-23T12:00:00Z"
echo "  Confirmations: 1"
echo ""

echo "Step 4: Verify"
echo "──────────────"
echo "  → Anchor verified!"
echo ""

echo "Code Example:"
echo "─────────────"
cat << 'EOF'
let manager = AnchorManager::new(discovery, store, client_factory).await?;

let receipt = manager.anchor(&braid, "spine-main").await?;
println!("Anchored at index: {}", receipt.anchor.index);

let info = manager.verify(&braid.id).await?;
assert!(info.unwrap().verified);
EOF
echo ""

echo "🌾 SweetGrass + LoamSpine Demo Complete!"
echo ""
echo "Key Takeaways:"
echo "  - Anchoring provides tamper-proof timestamps"
echo "  - Each anchor has a unique index in the spine"
echo "  - Verification confirms immutability"
echo ""

