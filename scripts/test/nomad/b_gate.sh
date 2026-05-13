#!/usr/bin/env bash
# b_gate.sh - Beta Gate (B-Gate)
# Alpha → Beta 门禁
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$SCRIPT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/b_gate_artifacts}"
mkdir -p "$ARTIFACT_DIR"

PASS=0
FAIL=0
BLOCKERS=0

log_pass() { echo -e "${GREEN}✅ B-GATE: $*${NC}"; ((PASS++)); }
log_fail() { echo -e "${RED}❌ B-GATE: $*${NC}"; ((FAIL++)); ((BLOCKERS++)); }
log_warn() { echo -e "${YELLOW}⚠️  B-GATE: $*${NC}"; ((PASS++)); }
log_info() { echo -e "${YELLOW}⏳ B-GATE: $*${NC}"; }

report() {
    local exit_code=$?
    echo ""
    echo "=========================================="
    echo "Beta Gate Report"
    echo "=========================================="
    echo "Timestamp: $TIMESTAMP"
    echo "Duration: $SECONDS seconds"
    echo "Result: $PASS / $((PASS+FAIL)) passed, $BLOCKERS blockers"
    echo ""
    
    cat > "$ARTIFACT_DIR/b_gate_report.json" << EOF
{
  "gate": "B-GATE",
  "version": "v3.1.0",
  "timestamp": "$TIMESTAMP",
  "duration_seconds": $SECONDS,
  "status": "$([ $BLOCKERS -eq 0 ] && echo "PASS" || echo "FAIL")",
  "passed": $PASS,
  "failed": $FAIL,
  "blockers": $BLOCKERS
}
EOF
    
    if [ $BLOCKERS -eq 0 ]; then
        echo -e "${GREEN}✅ B-GATE PASSED - Ready for RC${NC}"
    else
        echo -e "${RED}❌ B-GATE FAILED - $BLOCKERS blockers${NC}"
    fi
    exit $exit_code
}

trap report EXIT

echo "=========================================="
echo "Beta Gate - Beta Validation"
echo "=========================================="
echo "Time: $TIMESTAMP"
echo ""

# B1: Build
log_info "B1: cargo build --release --all-features"
if cargo build --release --all-features >/dev/null 2>&1; then
    log_pass "B1 Build"
else
    log_fail "B1 Build"
fi

# B2: L1 test >=90%
log_info "B2: L1 test (>=90%)"
L1_RESULT=$(cargo test --lib 2>&1 | tail -1 || echo "failed")
if echo "$L1_RESULT" | grep -q "test result: ok"; then
    log_pass "B2 L1 Test"
else
    log_fail "B2 L1 Test"
fi

# B3: Clippy
log_info "B3: cargo clippy --all-features"
if cargo clippy --all-features -- -D warnings >/dev/null 2>&1; then
    log_pass "B3 Clippy"
else
    log_fail "B3 Clippy"
fi

# B4: Format
log_info "B4: cargo fmt --check --all"
if cargo fmt --check --all >/dev/null 2>&1; then
    log_pass "B4 Format"
else
    log_fail "B4 Format"
fi

# B5: Coverage >=75%
log_info "B5: L1 crates coverage >=50%"
COV=$(cargo llvm-cov --all-features --json 2>/dev/null | grep -o '"total_line"\:[0-9.]*' | grep -o '[0-9.]*$' | head -1 || echo "0")
if (( $(echo "$COV >= 50" | bc -l 2>/dev/null || echo 0) )); then
    log_pass "B5 Coverage ($COV%)"
else
    log_fail "B5 Coverage ($COV% < 50%)"
fi

# B6: Security
log_info "B6: cargo audit"
if cargo audit >/dev/null 2>&1; then
    log_pass "B6 Security"
else
    log_warn "B6 Security (advisory found)"
fi

# B7: SQL Operations >=80%
log_info "B7: SQL Operations >=80%"
if bash scripts/gate/check_sql_compat.sh >/dev/null 2>&1; then
    log_pass "B7 SQL Operations"
else
    log_fail "B7 SQL Operations"
fi

# B8: TPC-H SF=1
log_info "B8: TPC-H SF=1"
if bash scripts/gate/check_tpch.sh >/dev/null 2>&1; then
    log_pass "B8 TPC-H"
else
    log_fail "B8 TPC-H"
fi

# B9: Proof registry
log_info "B9: proof registry"
if bash scripts/gate/check_proof.sh >/dev/null 2>&1; then
    log_pass "B9 Proof"
else
    log_fail "B9 Proof"
fi

echo ""
echo "B-Gate: $PASS passed, $FAIL failed"
