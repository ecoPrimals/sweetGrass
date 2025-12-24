#!/bin/bash
# Live SweetGrass + ToadStool Integration Demo
# Uses actual ToadStool binary from phase2/bins

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINS_DIR="$(cd "$PROJECT_ROOT/../bins" && pwd)"

echo ""
echo "🍄 SweetGrass + ToadStool LIVE Demo"
echo "===================================="
echo ""

# Check for ToadStool binary
if [ ! -x "$BINS_DIR/toadstool-cli" ]; then
    echo "❌ ToadStool binary not found at $BINS_DIR/toadstool-cli"
    echo "   Run: cd ../../phase1/toadStool && PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release"
    echo "   Then: cp target/release/toadstool-cli ../../phase2/bins/"
    exit 1
fi

echo "✅ ToadStool binary found: $BINS_DIR/toadstool-cli"
echo ""

# Show ToadStool version
echo "📋 ToadStool Info:"
echo "──────────────────"
$BINS_DIR/toadstool-cli --version
echo ""

# Show system capabilities
echo "🖥️  System Capabilities:"
echo "────────────────────────"
$BINS_DIR/toadstool-cli capabilities 2>&1 | head -30
echo ""

# Ecosystem discovery
echo "🌐 Ecosystem Discovery:"
echo "───────────────────────"
$BINS_DIR/toadstool-cli ecosystem discover 2>&1 | head -15 || echo "   (Checking ecosystem...)"
echo ""

# Universal compute options
echo "🚀 Universal Compute:"
echo "─────────────────────"
$BINS_DIR/toadstool-cli universal --help 2>&1 | head -15
echo ""

# Zero-config deployment
echo "⚡ Zero-Config Deployment:"
echo "──────────────────────────"
$BINS_DIR/toadstool-cli zero-config --help 2>&1 | head -10
echo ""

# SweetGrass integration concept
echo "🌾 SweetGrass Integration:"
echo "──────────────────────────"
echo "
When ToadStool processes compute jobs, SweetGrass tracks:

1. Input Braids: Data entering the compute pipeline
2. Activity: The transformation/computation performed  
3. Output Braids: Results with full provenance
4. Compute Units: Resource usage for fair rewards

Example workflow:

  // Submit job to ToadStool
  let job = ComputeJob::new(input_braid, code_braid);
  let result = toadstool.execute(job).await?;
  
  // SweetGrass records provenance
  let output_braid = factory.derived_from(
      result.data,
      vec![input_ref, code_ref],
      ActivityType::Computation,
  )?;
  
  // Compute units tracked for attribution
  output_braid.ecop.compute_units = result.compute_units;
"
echo ""
echo "🍄 Live Demo Complete!"
echo ""
echo "Next: Define a biome.yaml to run compute workloads"

