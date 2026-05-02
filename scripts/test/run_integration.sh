#!/usr/bin/env bash
# v2.9.0 Integration Test Runner
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PASS=0; FAIL=0; FAILS=""
TESTS=(aggregate_functions_test binary_format_test boundary_test cbo_integration_test ci_test concurrency_stress_test crash_recovery_test distinct_test expression_operators_test in_value_list_test limit_clause_test mvcc_transaction_test parser_token_test regression_test stored_proc_catalog_test wal_integration_test qps_benchmark_test page_io_benchmark_test data_loader)
for t in "${TESTS[@]}"; do
  echo -n "[integration] $t ... "
  if cargo test --test "$t" --all-features --quiet 2>/dev/null; then echo "PASS"; PASS=$((PASS+1))
  else echo "FAIL"; FAIL=$((FAIL+1)); FAILS="$FAILS $t"; fi
done
echo "Results: ${PASS} passed, ${FAIL} failed"
[ "$FAIL" -eq 0 ] || { echo "Failed:$FAILS"; exit 1; }
