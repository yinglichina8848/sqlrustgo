#!/usr/bin/env bash
# v2.9.0 集成测试运行器
# 用途: 一键运行全部 28 个集成测试
# 使用: bash scripts/test/run_integration.sh [--verbose|--quick]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

VERBOSE=false
QUICK=false
for arg in "$@"; do
  case "$arg" in
    --verbose) VERBOSE=true ;;
    --quick) QUICK=true ;;
  esac
done

echo "=== v2.9.0 Integration Test Runner ==="
echo "Date: $(date)"
echo "Git: $(git rev-parse HEAD | head -c 12)"
echo ""

TESTS=(
  aggregate_functions_test
  binary_format_test
  boundary_test
  cbo_integration_test
  ci_test
  concurrency_stress_test
  crash_recovery_test
  distinct_test
  expression_operators_test
  in_value_list_test
  limit_clause_test
  mvcc_transaction_test
  parser_token_test
  regression_test
  stored_proc_catalog_test
  stored_procedure_parser_test
  wal_integration_test
  qps_benchmark_test
  page_io_benchmark_test
  long_run_stability_test
  data_loader
)

if [ "$QUICK" = false ]; then
  TESTS+=(
    e2e_query_test
    e2e_monitoring_test
    e2e_observability_test
    long_run_stability_72h_test
  )
fi

PASSED=0
FAILED=0
FAILED_TESTS=""

for test in "${TESTS[@]}"; do
  echo -n "[integration] $test ... "
  if [ "$VERBOSE" = true ]; then
    if cargo test --test "$test" --all-features 2>&1; then
      echo "✅ PASS"
      PASSED=$((PASSED + 1))
    else
      echo "❌ FAIL"
      FAILED=$((FAILED + 1))
      FAILED_TESTS="$FAILED_TESTS $test"
    fi
  else
    if cargo test --test "$test" --all-features --quiet 2>/dev/null; then
      echo "✅ PASS"
      PASSED=$((PASSED + 1))
    else
      echo "❌ FAIL"
      FAILED=$((FAILED + 1))
      FAILED_TESTS="$FAILED_TESTS $test"
    fi
  fi
done

echo ""
echo "=== Results: ${PASSED} passed, ${FAILED} failed ==="
if [ "$FAILED" -gt 0 ]; then
  echo "Failed tests:$FAILED_TESTS"
  exit 1
fi
echo "✅ All integration tests passed"
