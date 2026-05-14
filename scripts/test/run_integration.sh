#!/usr/bin/env bash
# v3.1.0 Integration Test Runner
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PASS=0; FAIL=0; FAILS=""
TESTS=(aggregate_functions_test binary_format_test boundary_test cbo_integration_test ci_test concurrency_stress_test crash_recovery_test distinct_test expression_operators_test in_value_list_test limit_clause_test mvcc_transaction_test parser_token_test regression_test stored_proc_catalog_test wal_integration_test qps_benchmark_test page_io_benchmark_test data_loader merge_execution_test gis_spatial_test event_scheduler_test window_function_test window_function_execution_test engine_test engine_expr_test engine_subquery_test explain_analyze_test trigger_chain_test stored_procedure_parser_test partition_test planner_multi_join_test mvcc_snapshot_isolation_test gap_locking_e2e_test idempotency_test coverage_null_matrix_tests coverage_error_path_tests coverage_operator_matrix_tests cbo_performance_test cbo_perf_compare_test optimizer_cbo_accuracy_test long_run_stability_test crash_inject_test mvcc_visibility_counterexample_test multi_table_dml_test on_duplicate_key_test replace_into_test)
for t in "${TESTS[@]}"; do
  echo -n "[integration] $t ... "
  if cargo test --test "$t" --all-features --quiet 2>/dev/null; then echo "PASS"; PASS=$((PASS+1))
  else echo "FAIL"; FAIL=$((FAIL+1)); FAILS="$FAILS $t"; fi
done
echo "Results: ${PASS} passed, ${FAIL} failed"
[ "$FAIL" -eq 0 ] || { echo "Failed:$FAILS"; exit 1; }
