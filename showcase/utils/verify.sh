#!/usr/bin/env bash
#
# 🌾 SweetGrass Showcase Verify
#
# Quick verification that everything is ready.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo ""
echo "🌾 Verifying SweetGrass..."
echo ""

cd "$PROJECT_ROOT"

# Quick build check
if cargo check --workspace -q 2>/dev/null; then
    echo "✅ Build: OK"
else
    echo "❌ Build: FAILED"
    exit 1
fi

# Quick test check
TEST_COUNT=$(cargo test --workspace 2>&1 | grep -E "test result:" | tail -1)
echo "✅ Tests: $TEST_COUNT"

# Version
VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo "✅ Version: $VERSION"

echo ""
echo "🌾 Ready for demos!"
echo ""

