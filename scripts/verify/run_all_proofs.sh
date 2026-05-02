#!/usr/bin/env bash
# run_all_proofs.sh — 运行全部形式化证明验证
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Z6G4 工具链路径
DAFNY="/usr/bin/dafny"
TLA_JAR="/tmp/tla2tools.jar"
FORMULOG_JAR="/tmp/formulog-0.8.0.jar"
JAVA="/usr/bin/java"

echo "=== SQLRustGo Formal Verification ==="
echo "Date: $(date)"
echo ""

cd "$PROJECT_ROOT"

FAILED=0
PASSED=0
SKIPPED=0

verify_dafny() {
    local proof="$1"
    local file=$(ls docs/proof/${proof}-*.dfy 2>/dev/null | head -1)
    if [ ! -f "$file" ]; then
        echo "  SKIP $proof (no source .dfy file)"
        SKIPPED=$((SKIPPED + 1))
        return
    fi
    echo "[Dafny] Verifying $proof..."
    if $DAFNY verify "$file" --verification-time-limit:60 > /dev/null 2>&1; then
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
        echo "  SKIP $proof (no source .tla file)"
        SKIPPED=$((SKIPPED + 1))
        return
    fi
    local cfg="${file%.tla}.cfg"
    echo "[TLA+] Model checking $proof..."
    if $JAVA -cp "$TLA_JAR" tlc2.TLC -deadlock -workers auto -metadir /tmp/tlc_meta "$file" 2>&1 | \
       grep -q "Model checking completed"; then
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
        echo "  SKIP $proof (no source .formalog file)"
        SKIPPED=$((SKIPPED + 1))
        return
    fi
    echo "[Formulog] Checking $proof..."
    if $JAVA -jar "$FORMULOG_JAR" "$file" > /dev/null 2>&1; then
        echo "  PASS $proof"
        PASSED=$((PASSED + 1))
    else
        echo "  FAIL $proof"
        FAILED=$((FAILED + 1))
    fi
}

echo "=== S-01: Parser Correctness (Formulog) ==="
for proof in PROOF-001 PROOF-006 PROOF-007 PROOF-008 PROOF-010; do
    verify_formulog "$proof"
done

echo ""
echo "=== S-02: Type System Safety (Dafny) ==="
for proof in PROOF-002 PROOF-011; do
    verify_dafny "$proof"
done

echo ""
echo "=== S-03: Transaction ACID (TLA+) ==="
for proof in PROOF-003 PROOF-005 PROOF-012; do
    verify_tla "$proof"
done

echo ""
echo "=== S-04: B+Tree Invariants (Dafny) ==="
for proof in PROOF-004 PROOF-013; do
    verify_dafny "$proof"
done

echo ""
echo "=== S-05: Query Equivalence (Formulog) ==="
verify_formulog "PROOF-014"

echo ""
echo "=== Summary ==="
echo "Passed:  $PASSED"
echo "Failed:  $FAILED"
echo "Skipped: $SKIPPED"

if [ $FAILED -eq 0 ]; then
    echo "ALL formal verifications passed"
    exit 0
else
    echo "$FAILED verification(s) failed"
    exit 1
fi
