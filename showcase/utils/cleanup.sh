#!/usr/bin/env bash
#
# 🌾 SweetGrass Showcase Cleanup
#
# Removes demo artifacts and resets state.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo ""
echo "🌾 Cleaning up showcase artifacts..."
echo ""

# Remove any output files
find "$SHOWCASE_ROOT" -name "*.output" -delete 2>/dev/null || true
find "$SHOWCASE_ROOT" -name "*.log" -delete 2>/dev/null || true
find "$SHOWCASE_ROOT" -name "*.tmp" -delete 2>/dev/null || true

# Remove any demo databases
find "$SHOWCASE_ROOT" -name "*.db" -delete 2>/dev/null || true
find "$SHOWCASE_ROOT" -type d -name "sled_data" -exec rm -rf {} + 2>/dev/null || true

echo "✅ Cleanup complete!"
echo ""

