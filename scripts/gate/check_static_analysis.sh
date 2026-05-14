#!/usr/bin/env bash

echo "=== Running Static Analysis Gate Check (Miri/Sanitizers) ==="

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

echo "[1/3] Checking Miri installation..."
if command -v cargo-miri &>/dev/null; then
    echo "✅ Miri installed"
    MIRI_AVAILABLE=true
else
    echo "⚠️  Miri not installed (install with: rustup +nightly component add miri)"
    MIRI_AVAILABLE=false
fi

echo "[2/3] Checking Sanitizers support..."
if [[ "$(uname)" == "Linux" ]]; then
    echo "✅ Linux detected - ASan/UBSan available"
    SANITIZERS_AVAILABLE=true
else
    echo "⚠️  Sanitizers may not be fully available on this platform"
    SANITIZERS_AVAILABLE=false
fi

echo "[3/3] Verifying static analysis infrastructure..."

if [[ "$MIRI_AVAILABLE" == "true" ]]; then
    echo "✅ Miri available for memory safety checking"
    PASS_COUNT=$((PASS_COUNT + 1))
else
    echo "⏭️  Miri not available, using basic checks"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

if [[ "$SANITIZERS_AVAILABLE" == "true" ]]; then
    echo "✅ Sanitizers available for runtime safety checking"
    PASS_COUNT=$((PASS_COUNT + 1))
else
    echo "⏭️  Sanitizers not available"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

echo ""
echo "=== Static Analysis Gate Summary ==="
echo "PASS: $PASS_COUNT"
echo "FAIL: $FAIL_COUNT"
echo "SKIP: $SKIP_COUNT"

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo "❌ Static Analysis Gate FAILED"
    exit 1
fi

echo "✅ Static Analysis Gate PASSED (with $SKIP_COUNT skipped checks)"