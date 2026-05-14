#!/usr/bin/env bash
# l2_integration.sh - L2 集成测试 (15min)
# 使用 test manifest 运行集成测试
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

# Integration tests from run_integration.sh
INTEGRATION_TESTS=(
    "aggregate_functions_test"
    "binary_format_test"
    "boundary_test"
    "cbo_integration_test"
    "concurrency_stress_test"
    "crash_recovery_test"
    "distinct_test"
    "expression_operators_test"
    "in_value_list_test"
    "limit_clause_test"
    "mvcc_transaction_test"
    "parser_token_test"
    "regression_test"
    "stored_proc_catalog_test"
    "wal_integration_test"
    "data_loader"
)

log_pass() { echo -e "${GREEN}✅ $*${NC}"; ((PASS++)); }
log_fail() { echo -e "${RED}❌ $*${NC}"; ((FAIL++)); }
log_info() { echo -e "${YELLOW}⏳ $*${NC}"; }

echo "=========================================="
echo "L2 Integration Tests"
echo "=========================================="
echo "Time: $TIMESTAMP"
echo "Tests: ${#INTEGRATION_TESTS[@]}"
echo ""

for test in "${INTEGRATION_TESTS[@]}"; do
    log_info "Running $test..."
    
    if cargo test --test "$test" --all-features --quiet 2>/dev/null; then
        log_pass "$test"
    else
        log_fail "$test"
    fi
done

echo ""
echo "=========================================="
echo "L2 Integration Test Report"
echo "=========================================="
echo "Timestamp: $TIMESTAMP"
echo "Duration: $SECONDS seconds"
echo "Result: $PASS passed, $FAIL failed"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ ALL PASS${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAIL TESTS FAILED${NC}"
    exit 1
fi
