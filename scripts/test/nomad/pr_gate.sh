#!/usr/bin/env bash
# pr_gate.sh - PR 门禁 (PR-Gate)
# 每个 PR 必须通过的门禁
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$SCRIPT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/pr_gate_artifacts}"
mkdir -p "$ARTIFACT_DIR"

PASS=0
FAIL=0

log_pass() { echo -e "${GREEN}✅ PR-GATE: $*${NC}"; ((PASS++)); }
log_fail() { echo -e "${RED}❌ PR-GATE: $*${NC}"; ((FAIL++)); }
log_info() { echo -e "${YELLOW}⏳ PR-GATE: $*${NC}"; }

report() {
    local exit_code=$?
    echo ""
    echo "=========================================="
    echo "PR Gate Report"
    echo "=========================================="
    echo "Timestamp: $TIMESTAMP"
    echo "Duration: $SECONDS seconds"
    echo "Result: $PASS / $((PASS+FAIL))"
    echo ""
    
    # Write JSON report
    cat > "$ARTIFACT_DIR/pr_gate_report.json" << EOF
{
  "gate": "PR-GATE",
  "timestamp": "$TIMESTAMP",
  "duration_seconds": $SECONDS,
  "status": "$([ $FAIL -eq 0 ] && echo "PASS" || echo "FAIL")",
  "passed": $PASS,
  "failed": $FAIL,
  "checks": [
    {"name": "build", "status": "$(grep -q 'build' <<< "$FAILURES" && echo FAIL || echo PASS)"},
    {"name": "test", "status": "$(grep -q 'test' <<< "$FAILURES" && echo FAIL || echo PASS)"},
    {"name": "clippy", "status": "$(grep -q 'clippy' <<< "$FAILURES" && echo FAIL || echo PASS)"},
    {"name": "format", "status": "$(grep -q 'format' <<< "$FAILURES" && echo FAIL || echo PASS)"},
    {"name": "coverage", "status": "$(grep -q 'coverage' <<< "$FAILURES" && echo FAIL || echo PASS)"}
  ]
}
EOF
    
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✅ PR-GATE PASSED${NC}"
    else
        echo -e "${RED}❌ PR-GATE FAILED${NC}"
    fi
    exit $exit_code
}

trap report EXIT

echo "=========================================="
echo "PR Gate - Pull Request Validation"
echo "=========================================="
echo "Time: $TIMESTAMP"
echo ""

# PR-Gate checks
log_info "B1: cargo build --release --workspace"
if cargo build --release --workspace >/dev/null 2>&1; then
    log_pass "B1 Build"
else
    log_fail "B1 Build"
fi

log_info "B2: cargo test --lib (quick check)"
if cargo test --lib -- --test-threads=8 >/dev/null 2>&1; then
    log_pass "B2 Test"
else
    log_fail "B2 Test"
fi

log_info "B3: cargo clippy --all-features"
if cargo clippy --all-features -- -D warnings >/dev/null 2>&1; then
    log_pass "B3 Clippy"
else
    log_fail "B3 Clippy"
fi

log_info "B4: cargo fmt --check --all"
if cargo fmt --check --all >/dev/null 2>&1; then
    log_pass "B4 Format"
else
    log_fail "B4 Format"
fi

log_info "B5: coverage check (>=60%)"
COV=$(cargo llvm-cov --all-features --json 2>/dev/null | grep -o '"total_line"\:[0-9.]*' | grep -o '[0-9.]*$' | head -1 || echo "0")
if (( $(echo "$COV >= 60" | bc -l 2>/dev/null || echo 0) )); then
    log_pass "B5 Coverage ($COV%)"
else
    log_fail "B5 Coverage ($COV% < 60%)"
fi

echo ""
echo "PR-Gate: $PASS passed, $FAIL failed"
