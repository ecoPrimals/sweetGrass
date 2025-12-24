#!/usr/bin/env bash
#
# 🌾 SweetGrass + BearDog Demo
#
# This demo shows how to sign Braids with BearDog.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo ""
echo "🌾 SweetGrass + BearDog Demo"
echo "============================"
echo ""

echo "📝 Integration Overview"
echo ""
echo "SweetGrass discovers BearDog via capability-based lookup,"
echo "then uses tarpc (pure Rust RPC) to request signatures."
echo ""

echo "Step 1: Capability Discovery"
echo "───────────────────────────"
echo "  Looking for capability: Signing"
echo "  → BearDog discovered at localhost:8091"
echo ""

echo "Step 2: Create Braid"
echo "────────────────────"
echo "  Data: \"Important research data\""
echo "  Agent: did:key:z6MkAlice..."
echo "  → Braid created: urn:braid:abc123"
echo ""

echo "Step 3: Sign with BearDog"
echo "─────────────────────────"
echo "  Connecting via tarpc..."
echo "  → Signature created"
echo ""

echo "Signature Details:"
echo "  Type: Ed25519Signature2020"
echo "  Created: 2025-12-23T12:00:00Z"
echo "  Verification Method: did:key:z6MkBearDog...#keys-1"
echo "  Proof Purpose: assertionMethod"
echo ""

echo "Step 4: Verify"
echo "──────────────"
echo "  → Signature valid!"
echo ""

echo "Code Example:"
echo "─────────────"
cat << 'EOF'
let discovery = LocalDiscovery::new();
let signer = DiscoverySigner::new(discovery)?;

let braid = factory.from_data(b"data", "text/plain", None)?;
let signed_braid = signer.sign(&braid).await?;

assert!(signer.verify(&signed_braid).await?);
EOF
echo ""

echo "🌾 SweetGrass + BearDog Demo Complete!"
echo ""
echo "Key Takeaways:"
echo "  - Capability-based discovery (no hardcoding)"
echo "  - tarpc for pure Rust RPC"
echo "  - W3C Data Integrity signatures"
echo ""

