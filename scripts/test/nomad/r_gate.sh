#!/usr/bin/env bash
# r_gate.sh - RC Gate (R-Gate)
# Beta → RC 门禁
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$SCRIPT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/r_gate_artifacts}"
mkdir -p "$ARTIFACT_DIR"

PASS=0
FAIL=0
BLOCKERS=0

log_pass() { echo -e "${GREEN}✅ R-GATE: $*${NC}"; ((PASS++)); }
log_fail() { echo -e "${RED}❌ R-GATE: $*${NC}"; ((FAIL++)); ((BLOCKERS++)); }
log_warn() { echo -e "${YELLOW}⚠️  R-GATE: $*${NC}"; ((PASS++)); }
log_info() { echo -e "${YELLOW}⏳ R-GATE: $*${NC}"; }

report() {
    local exit_code=$?
    echo ""
    echo "=========================================="
    echo "RC Gate Report"
    echo "=========================================="
    echo "Timestamp: $TIMESTAMP"
    echo "Duration: $SECONDS seconds"
    echo "Result: $PASS / $((PASS+FAIL)) passed, $BLOCKERS blockers"
    echo ""
    
    cat > "$ARTIFACT_DIR/r_gate_report.json" << EOF
{
  "gate": "R-GATE",
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
        echo -e "${GREEN}✅ R-GATE PASSED - Ready for GA${NC}"
    else
        echo -e "${RED}❌ R-GATE FAILED - $BLOCKERS blockers${NC}"
    fi
    exit $exit_code
}

trap report EXIT

echo "=========================================="
echo "RC Gate - RC Validation"
echo "=========================================="
echo "Time: $TIMESTAMP"
echo ""

# R1: Build
log_info "R1: cargo build --release --all-features"
if cargo build --release --all-features >/dev/null 2>&1; then
    log_pass "R1 Build"
else
    log_fail "R1 Build"
fi

# R2: Test Suite >=90%
log_info "R2: L1 test (>=90%)"
if cargo test --lib >/dev/null 2>&1; then
    log_pass "R2 Test"
else
    log_fail "R2 Test"
fi

# R3: Clippy
log_info "R3: cargo clippy --all-features"
if cargo clippy --all-features -- -D warnings >/dev/null 2>&1; then
    log_pass "R3 Clippy"
else
    log_fail "R3 Clippy"
fi

# R4: Format
log_info "R4: cargo fmt --check --all"
if cargo fmt --check --all >/dev/null 2>&1; then
    log_pass "R4 Format"
else
    log_fail "R4 Format"
fi

# R5: Coverage >=85%
log_info "R5: L1 crates coverage >=85%"
COV=$(cargo llvm-cov --all-features --json 2>/dev/null | grep -o '"total_line"\:[0-9.]*' | grep -o '[0-9.]*$' | head -1 || echo "0")
if (( $(echo "$COV >= 85" | bc -l 2>/dev/null || echo 0) )); then
    log_pass "R5 Coverage ($COV%)"
else
    log_fail "R5 Coverage ($COV% < 85%)"
fi

# R6: Security
log_info "R6: cargo audit"
if cargo audit >/dev/null 2>&1; then
    log_pass "R6 Security"
else
    log_fail "R6 Security"
fi

# R7: SQL Ops >=95%
log_info "R7: SQL Operations >=95%"
if bash scripts/gate/check_sql_compat.sh >/dev/null 2>&1; then
    log_pass "R7 SQL Operations"
else
    log_fail "R7 SQL Operations"
fi

# R8: TPC-H SF=1 p99<5s
log_info "R8: TPC-H SF=1 (p99<5s)"
if bash scripts/gate/check_tpch.sh >/dev/null 2>&1; then
    log_pass "R8 TPC-H"
else
    log_fail "R8 TPC-H"
fi

# R9: Regression check
log_info "R9: regression check"
if bash scripts/gate/check_regression.sh >/dev/null 2>&1; then
    log_pass "R9 Regression"
else
    log_fail "R9 Regression"
fi

# R10: Formal proofs >=30
log_info "R10: formal proofs >=30"
if bash scripts/gate/check_proof.sh >/dev/null 2>&1; then
    log_pass "R10 Proofs"
else
    log_fail "R10 Proofs"
fi

# R11: Docs links
log_info "R11: docs links check"
if bash scripts/gate/check_docs_links.sh >/dev/null 2>&1; then
    log_pass "R11 Docs"
else
    log_warn "R11 Docs (broken links)"
fi

echo ""
echo "R-Gate: $PASS passed, $FAIL failed"
