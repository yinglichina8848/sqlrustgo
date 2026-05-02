#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== R10: Formal Proof Check ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

PROOF_DIR="docs/proof"
MIN_PROOFS=10

echo "[1/3] Checking proof directory..."

if [ ! -d "$PROOF_DIR" ]; then
    echo "⚠️  Proof directory '$PROOF_DIR' not found"
    echo ""
    echo "❌ R10: Formal Proof Check FAILED"
    echo "   Proof directory must exist at '$PROOF_DIR'"
    exit 1
fi

echo "[2/3] Counting proof files..."

PROOF_COUNT=$(find "$PROOF_DIR" -name "*.json" -type f 2>/dev/null | wc -l)

if [ "$PROOF_COUNT" -lt "$MIN_PROOFS" ]; then
    echo "Proof files found: $PROOF_COUNT"
    echo "Minimum required: $MIN_PROOFS"
    echo ""
    echo "❌ R10: Formal Proof Check FAILED"
    echo "   Insufficient formal proofs (found $PROOF_COUNT, need >= $MIN_PROOFS)"
    exit 1
fi

echo "[3/3] Validating proof files..."

INVALID=0
for proof_file in $(find "$PROOF_DIR" -name "*.json" -type f 2>/dev/null); do
    if ! python3 -c "import json; json.load(open('$proof_file'))" 2>/dev/null; then
        echo "  Invalid JSON: $proof_file"
        INVALID=$((INVALID + 1))
    fi
done

if [ "$INVALID" -gt 0 ]; then
    echo ""
    echo "❌ R10: Formal Proof Check FAILED"
    echo "   $INVALID proof file(s) have invalid JSON"
    exit 1
fi

echo ""
echo "✅ R10: Formal Proof Check PASSED"
echo "   Proof files: $PROOF_COUNT (>= $MIN_PROOFS required)"
echo "   All files valid JSON"
exit 0
