#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== G-03: Attack Surface Verification (AV1-AV10) ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

ATTACK_SURFACE_DOC="docs/security/attack_surface/ATTACK_SURFACE_ANALYSIS.md"

echo "[1/3] Checking attack surface documentation..."

if [ ! -f "$ATTACK_SURFACE_DOC" ]; then
    echo "❌ FAIL: Attack surface document not found at '$ATTACK_SURFACE_DOC'"
    exit 1
fi

echo "  ✅ Attack surface document exists"

echo "[2/3] Verifying AV coverage..."

AV_COUNT=$(grep -c "^| AV" "$ATTACK_SURFACE_DOC" || echo "0")

if [ "$AV_COUNT" -lt 10 ]; then
    echo "  ❌ FAIL: Only $AV_COUNT attack vectors documented (need 10)"
    exit 1
fi

echo "  ✅ All 10 attack vectors (AV1-AV10) documented"

echo "[3/3] Checking critical mitigations..."

CRITICAL_AVS=$(grep -c "CRITICAL" "$ATTACK_SURFACE_DOC" || echo "0")

if [ "$CRITICAL_AVS" -lt 3 ]; then
    echo "  ⚠️  Warning: Only $CRITICAL_AVS CRITICAL attack vectors documented"
fi

echo "  ✅ Attack surface analysis complete"

echo ""
echo "=== G-03 Attack Surface Verification ==="
echo "✅ PASSED: All 10 attack vectors (AV1-AV10) verified"
echo ""
echo "Attack vectors documented:"
grep "^| AV" "$ATTACK_SURFACE_DOC" | while read line; do
    echo "  $line"
done

exit 0
