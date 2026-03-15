#!/usr/bin/env bash
#
# Pre-commit quality checks for SweetGrass
# Run this before committing to ensure code quality
#
# Usage: ./scripts/check.sh

set -e

echo "🌾 SweetGrass Quality Checks"
echo "=============================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_pass() {
    echo -e "${GREEN}✓${NC} $1"
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Format check
echo ""
echo "1️⃣  Checking formatting..."
if cargo fmt --all -- --check; then
    check_pass "Code formatting is correct"
else
    check_fail "Code formatting failed. Run: cargo fmt --all"
fi

# 2. Clippy (pedantic + nursery)
echo ""
echo "2️⃣  Running clippy (pedantic + nursery)..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    check_pass "Clippy passed with zero warnings"
else
    check_fail "Clippy found issues. Fix them and try again."
fi

# 3. Build
echo ""
echo "3️⃣  Building (all features)..."
if cargo build --all-features --quiet; then
    check_pass "Build successful"
else
    check_fail "Build failed"
fi

# 4. Tests (without Docker)
echo ""
echo "4️⃣  Running tests (unit + integration)..."
if cargo test --all-features --quiet; then
    check_pass "All tests passed"
else
    check_fail "Tests failed"
fi

# 5. Documentation
echo ""
echo "5️⃣  Checking documentation..."
if cargo doc --no-deps --all-features --quiet 2>&1 | grep -q "^warning:"; then
    check_fail "Documentation has warnings"
else
    check_pass "Documentation is clean"
fi

# 6. Check for unwraps in production code
echo ""
echo "6️⃣  Checking for production unwraps..."
UNWRAPS=$(grep -r "\.unwrap()\|\.expect(" crates/*/src --include="*.rs" \
    | grep -v "^crates/.*/tests/" \
    | grep -v "^crates/.*/examples/" \
    | grep -v "test" \
    | wc -l || true)

if [ "$UNWRAPS" -eq 0 ]; then
    check_pass "Zero production unwraps"
else
    check_warn "Found $UNWRAPS potential production unwraps"
    echo "    Review these carefully - they should be in test code only"
fi

# 7. Check for unsafe code
echo ""
echo "7️⃣  Checking for unsafe code..."
UNSAFE=$(grep -r "unsafe" crates/*/src --include="*.rs" \
    | grep -v "forbid(unsafe_code)" \
    | wc -l || true)

if [ "$UNSAFE" -eq 0 ]; then
    check_pass "Zero unsafe code"
else
    check_fail "Found unsafe code blocks"
fi

# 8. Check file sizes
echo ""
echo "8️⃣  Checking file sizes (max 1000 lines)..."
LARGE_FILES=$(find crates -name "*.rs" -exec bash -c 'lines=$(wc -l < "$1"); if [ "$lines" -gt 1000 ]; then echo "$1: $lines"; fi' _ {} \; | wc -l)

if [ "$LARGE_FILES" -eq 0 ]; then
    check_pass "All files under 1000 lines"
else
    check_fail "Found $LARGE_FILES files over 1000 lines"
    find crates -name "*.rs" -exec bash -c 'lines=$(wc -l < "$1"); if [ "$lines" -gt 1000 ]; then echo "  $1: $lines lines"; fi' _ {} \;
fi

# 9. Check for TODO/FIXME in production
echo ""
echo "9️⃣  Checking for TODO/FIXME markers..."
TODOS=$(grep -r "TODO\|FIXME\|XXX\|HACK" crates/*/src --include="*.rs" \
    | grep -v "test" \
    | wc -l || true)

if [ "$TODOS" -eq 0 ]; then
    check_pass "No TODO/FIXME markers in production"
else
    check_warn "Found $TODOS TODO/FIXME markers"
fi

# Summary
echo ""
echo "=============================="
echo -e "${GREEN}✓ All checks passed!${NC}"
echo ""
echo "Optional: Run with Docker for full coverage"
echo "  docker-compose up -d"
echo "  cargo test --all-features"
echo "  cargo llvm-cov --all-features --workspace"
echo "  docker-compose down"
echo ""
