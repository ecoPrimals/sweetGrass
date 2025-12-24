#!/usr/bin/env bash
#
# 🌾 SweetGrass Showcase Setup
#
# Verifies prerequisites and prepares the environment.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo ""
echo "🌾 SweetGrass Showcase Setup"
echo "============================"
echo ""

ERRORS=0

# Check Rust
echo "Checking Rust..."
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "  ✅ Rust: $RUST_VERSION"
else
    echo "  ❌ Rust not found. Install from https://rustup.rs/"
    ERRORS=$((ERRORS + 1))
fi

# Check build
echo ""
echo "Checking build..."
cd "$PROJECT_ROOT"
if cargo build -q 2>/dev/null; then
    echo "  ✅ Build successful"
else
    echo "  ❌ Build failed. Run 'cargo build' for details."
    ERRORS=$((ERRORS + 1))
fi

# Check tests
echo ""
echo "Checking tests..."
if cargo test --workspace -q 2>/dev/null; then
    echo "  ✅ All tests passing"
else
    echo "  ⚠️  Some tests failed. Run 'cargo test' for details."
fi

# Summary
echo ""
echo "============================"
if [[ $ERRORS -eq 0 ]]; then
    echo "✅ Setup complete! Ready for demos."
    echo ""
    echo "Start with:"
    echo "  cd showcase/00-standalone/01-braid-basics"
    echo "  ./demo-create-braid.sh"
else
    echo "❌ Setup incomplete. Fix $ERRORS error(s) above."
    exit 1
fi
echo ""

