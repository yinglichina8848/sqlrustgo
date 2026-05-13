#!/usr/bin/env bash
# l0_smoke.sh - L0 冒烟测试 (<5min)
# 每次 push 前强制执行
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$SCRIPT_DIR"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
PASS=0
FAIL=0
FAILURES=""

log_pass() { echo -e "${GREEN}✅ $*${NC}"; ((PASS++)); }
log_fail() { echo -e "${RED}❌ $*${NC}"; ((FAIL++)); FAILURES="$FAILURES\n  - $*"; }
log_info() { echo -e "${YELLOW}⏳ $*${NC}"; }

report() {
    echo ""
    echo "=========================================="
    echo "L0 Smoke Test Report"
    echo "=========================================="
    echo "Timestamp: $TIMESTAMP"
    echo "Duration: $SECONDS seconds"
    echo "Result: $PASS / $((PASS+FAIL))"
    echo ""
    if [ $FAIL -eq 0 ]; then
        echo -e "${GREEN}✅ ALL PASS - Branch is ready${NC}"
        exit 0
    else
        echo -e "${RED}❌ FAILURES:${NC}$FAILURES"
        exit 1
    fi
}

trap report EXIT

echo "=========================================="
echo "L0 Smoke Test"
echo "=========================================="
echo "Time: $TIMESTAMP"
echo ""

# Check 1: Build
log_info "【构建】 cargo build --release --workspace"
if cargo build --release --workspace >/dev/null 2>&1; then
    log_pass "Build"
else
    log_fail "Build failed"
fi

# Check 2: Format
log_info "【格式】 cargo fmt --check --all"
if cargo fmt --check --all >/dev/null 2>&1; then
    log_pass "Format"
else
    log_fail "Format check failed (run 'cargo fmt --all' to fix)"
fi

# Check 3: Clippy
log_info "【Clippy】 cargo clippy --all-features"
if cargo clippy --all-features -- -D warnings >/dev/null 2>&1; then
    log_pass "Clippy"
else
    log_fail "Clippy warnings found"
fi

# Check 4: Types test
log_info "【Types 冒烟】 cargo test -p sqlrustgo-types --lib"
if cargo test -p sqlrustgo-types --lib -- --test-threads=4 >/dev/null 2>&1; then
    log_pass "Types test"
else
    log_fail "Types test failed"
fi

# Check 5: Parser test
log_info "【Parser 冒烟】 cargo test -p sqlrustgo-parser --lib"
if cargo test -p sqlrustgo-parser --lib -- --test-threads=4 >/dev/null 2>&1; then
    log_pass "Parser test"
else
    log_fail "Parser test failed"
fi

echo ""
echo "L0 Smoke: $PASS passed, $FAIL failed"
