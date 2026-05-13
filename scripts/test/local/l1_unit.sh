#!/usr/bin/env bash
# l1_unit.sh - L1 单元测试 (10min)
# crate 级别全量测试
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
SKIP=0
CRATES=""

# Core crates for L1
CORE_CRATES=(
    "sqlrustgo-executor"
    "sqlrustgo-planner"
    "sqlrustgo-optimizer"
    "sqlrustgo-storage"
    "sqlrustgo-transaction"
    "sqlrustgo-catalog"
    "sqlrustgo-network"
    "sqlrustgo-gmp"
    "sqlrustgo-types"
    "sqlrustgo-parser"
    "sqlrustgo-leader"
    "sqlrustgo-distributed"
)

log_pass() { echo -e "${GREEN}✅ $*${NC}"; ((PASS++)); }
log_fail() { echo -e "${RED}❌ $*${NC}"; ((FAIL++)); }
log_skip() { echo -e "${YELLOW}⏭️  $*${NC}"; ((SKIP++)); }
log_info() { echo -e "${YELLOW}⏳ $*${NC}"; }

echo "=========================================="
echo "L1 Unit Tests"
echo "=========================================="
echo "Time: $TIMESTAMP"
echo "Crates: ${#CORE_CRATES[@]}"
echo ""

for crate in "${CORE_CRATES[@]}"; do
    log_info "Testing $crate..."
    
    # Check if crate exists
    if ! cargo metadata --format-version=1 -q 2>/dev/null | grep -q "\"name\":\"$crate\""; then
        log_skip "$crate (not found)"
        continue
    fi
    
    # Run tests
    if cargo test -p "$crate" --lib -- --test-threads=4 >/dev/null 2>&1; then
        log_pass "$crate"
    else
        log_fail "$crate"
    fi
done

echo ""
echo "=========================================="
echo "L1 Unit Test Report"
echo "=========================================="
echo "Timestamp: $TIMESTAMP"
echo "Duration: $SECONDS seconds"
echo "Result: $PASS passed, $FAIL failed, $SKIP skipped"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ ALL PASS${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAIL CRATES FAILED${NC}"
    exit 1
fi
