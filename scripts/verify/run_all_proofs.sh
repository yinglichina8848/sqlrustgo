#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Setup tool paths
export PATH="$HOME/.local/z3/z3-4.12.2-x64-glibc-2.35/bin:$HOME/.dotnet/tools:$HOME/.dotnet:$PATH"
export DOTNET_ROOT="$HOME/.dotnet"

echo "=== SQLRustGo Formal Verification ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

FAILED=0
PASSED=0

verify_dafny() {
    local proof="$1"
    local file=$(ls docs/proof/${proof}-*.dfy 2>/dev/null | head -1)
    if [ ! -f "$file" ]; then
        echo "SKIP $proof not found"
        return
    fi
    echo "[Dafny] Verifying $proof..."
    if dafny verify "$file" > /dev/null 2>&1; then
        echo "  PASS $proof"
        PASSED=$((PASSED + 1))
    else
        echo "  FAIL $proof"
        FAILED=$((FAILED + 1))
    fi
}

verify_tla() {
    local proof="$1"
    local file=$(ls docs/proof/${proof}-*.tla 2>/dev/null | head -1)
    if [ ! -f "$file" ]; then
        echo "SKIP $proof not found"
        return
    fi
    echo "[TLA+] Model checking $proof..."
    if docker run --rm -v "$(pwd):/workspace" -w /workspace tlatools/tlatools tlc -workers auto "$file" > /dev/null 2>&1; then
        echo "  PASS $proof"
        PASSED=$((PASSED + 1))
    else
        echo "  FAIL $proof"
        FAILED=$((FAILED + 1))
    fi
}

verify_formulog() {
    local proof="$1"
    local file=$(ls docs/proof/${proof}-*.formalog 2>/dev/null | head -1)
    if [ ! -f "$file" ]; then
        echo "SKIP $proof not found"
        return
    fi
    echo "[Formulog] Checking $proof..."
    if formulog check "$file" > /dev/null 2>&1; then
        echo "  PASS $proof"
        PASSED=$((PASSED + 1))
    else
        echo "  FAIL $proof"
        FAILED=$((FAILED + 1))
    fi
}

echo "=== S-01: Parser (Formulog) ==="
for proof in PROOF-001 PROOF-006 PROOF-007 PROOF-008 PROOF-010; do
    verify_formulog "$proof"
done

echo ""
echo "=== S-02: Type System (Dafny) ==="
for proof in PROOF-002 PROOF-011; do
    verify_dafny "$proof"
done

echo ""
echo "=== S-03: Transaction ACID (TLA+) ==="
for proof in PROOF-003 PROOF-005 PROOF-012; do
    verify_tla "$proof"
done

echo ""
echo "=== S-04: B+Tree (Dafny) ==="
for proof in PROOF-004 PROOF-013; do
    verify_dafny "$proof"
done

echo ""
echo "=== S-05: Query Equivalence (Formulog) ==="
verify_formulog "PROOF-014"

echo ""
echo "=== Summary ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ $FAILED -eq 0 ]; then
    echo "ALL formal verifications passed"
    exit 0
else
    echo "$FAILED verification(s) failed"
    exit 1
fi