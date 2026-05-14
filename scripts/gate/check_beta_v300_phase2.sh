#!/usr/bin/env bash
# v3.0.0 Beta Phase 2 Gate — GMP 可信度基础设施验证
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.0.0-phase2] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"
        PASS=$((PASS+1))
    else
        echo "FAIL"
        BLOCKERS=$((BLOCKERS+1))
    fi
}

check_test() {
    local name="$1" test_cmd="$2" expected="$3"
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.0.0-phase2] $name ... "
    TEST_OUTPUT=$(eval "$test_cmd" 2>&1 || true)
    PASSED=$(echo "$TEST_OUTPUT" | grep -c "test result: ok" || echo "0")
    FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" || echo "0")
    if [ "$FAILED" -eq 0 ] && [ "$PASSED" -gt 0 ]; then
        echo "PASS ($PASSED tests)"
        PASS=$((PASS+1))
    else
        echo "FAIL ($PASSED passed, $FAILED failed)"
        BLOCKERS=$((BLOCKERS+1))
    fi
}

echo "=== v3.0.0 Beta Phase 2 Gate ==="
echo ""

# BP2-1: Audit Trail
check_test "BP2-1: audit_trail_test" "cargo test --test audit_trail_test 2>&1" "all pass"

# BP2-2: WAL Crash Validation
check_test "BP2-2: crash_inject_test" "cargo test --test crash_inject_test 2>&1" "100 iterations pass"

# BP2-3: Differential Testing
echo -n "[beta-v3.0.0-phase2] BP2-3: Differential Testing (≥85%) ... "
TOTAL=$((TOTAL+1))
CORPUS_OUTPUT=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
CORPUS_PCT=$(echo "$CORPUS_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | tail -1 | tr -d '%' || echo "0")
if (( $(echo "$CORPUS_PCT >= 85" | bc -l) )); then
    echo "PASS (${CORPUS_PCT}%)"
    PASS=$((PASS+1))
else
    echo "FAIL (${CORPUS_PCT}% < 85%)"
    BLOCKERS=$((BLOCKERS+1))
fi

# BP2-4: INFORMATION_SCHEMA Extension
check_test "BP2-4: information_schema_test" "cargo test --test information_schema_test 2>&1" "TRIGGERS/ROUTINES pass"

# BP2-5: EXPLAIN ANALYZE
check_test "BP2-5: explain_analyze_test" "cargo test --test explain_analyze_test 2>&1" "actual_rows output correct"

# BP2-6: Window Functions
check_test "BP2-6: window_function_test" "cargo test --test window_function_test 2>&1" "LEAD/LAG/NTILE correct"

# BP2-7: RANGE Partition (P1)
check_test "BP2-7: partition_test" "cargo test --test partition_test 2>&1" "partition pruning correct"

# BP2-8: Cursor (P1)
check_test "BP2-8: cursor_test" "cargo test --test cursor_test 2>&1" "FETCH correct"

# BP2-9: Trigger Chain (P1)
check_test "BP2-9: trigger_chain_test" "cargo test --test trigger_chain_test 2>&1" "ordered execution correct"

# BP2-QA1: Soak Test 72h (QA Gate)
echo -n "[beta-v3.0.0-phase2] BP2-QA1: Soak Test 72h ... "
TOTAL=$((TOTAL+1))
# Note: This test is long-running and may be skipped in fast CI
if [ "${CI:-}" = "true" ]; then
    echo "SKIP (72h test not run in CI)"
else
    SOAK_OUTPUT=$(cargo test --test long_run_stability_72h_test 2>&1 || true)
    SOAK_PASSED=$(echo "$SOAK_OUTPUT" | grep -c "test result: ok" || echo "0")
    SOAK_FAILED=$(echo "$SOAK_OUTPUT" | grep -c "test result: FAILED" || echo "0")
    if [ "$SOAK_FAILED" -eq 0 ] && [ "$SOAK_PASSED" -gt 0 ]; then
        echo "PASS ($SOAK_PASSED tests)"
        PASS=$((PASS+1))
    else
        echo "FAIL ($SOAK_PASSED passed, $SOAK_FAILED failed)"
        BLOCKERS=$((BLOCKERS+1))
    fi
fi

# Keep existing Beta Gate items (B-S1 ~ B-S6)
check_test "B-S1: concurrency_stress_test" "cargo test --test concurrency_stress_test 2>&1" "all pass"
check_test "B-S2: crash_recovery_test" "cargo test --test crash_recovery_test 2>&1" "8/8 pass"
check_test "B-S3: long_run_stability_test" "cargo test --test long_run_stability_test 2>&1" "10/10 pass"
check_test "B-S4: wal_integration_test" "cargo test --test wal_integration_test 2>&1" "all pass"
check_test "B-S5: network_tcp_smoke_test" "cargo test --test network_tcp_smoke_test 2>&1" "6/6 pass"
check_test "B-S6: ssi_stress_test" "cargo test -p sqlrustgo-transaction --test ssi_stress_test 2>&1" "all pass"

echo ""
echo "=== Beta Phase 2 Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="

if [ $BLOCKERS -gt 0 ]; then
    echo "❌ Beta Phase 2 Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ Beta Phase 2 Gate PASSED"
    exit 0
fi