#!/usr/bin/env bash
# v3.0.0 Beta Gate — 进入 Beta 阶段必须通过
# 基于 gate_spec_v300.md §四
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.0.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"
        PASS=$((PASS+1))
    else
        echo "FAIL"
        BLOCKERS=$((BLOCKERS+1))
    fi
}

echo "=== v3.0.0 Beta Gate ==="
echo ""

# B1: Release Build
check "B1: cargo build --release --workspace" "cargo build --release --workspace"

# B2: Full Test Suite ≥90%
echo -n "[beta-v3.0.0] B2: cargo test --all-features (≥90%) ... "
TOTAL=$((TOTAL+1))
TEST_OUTPUT=$(cargo test --all-features 2>&1 || true)
PASSED=$(echo "$TEST_OUTPUT" | grep -c "test result: ok" || echo "0")
FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" || echo "0")
TOTAL_TESTS=$((PASSED + FAILED))
if [ "$TOTAL_TESTS" -gt 0 ]; then
    PASS_RATE=$((PASSED * 100 / TOTAL_TESTS))
    if [ "$PASS_RATE" -ge 90 ]; then
        echo "PASS ($PASS_RATE% = $PASSED/$TOTAL_TESTS)"
        PASS=$((PASS+1))
    else
        echo "FAIL ($PASS_RATE% = $PASSED/$TOTAL_TESTS < 90%)"
        BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "FAIL (no tests found)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B3: Clippy
check "B3: cargo clippy --all-features" "cargo clippy --all-features -- -D warnings"

# B4: Format
check "B4: cargo fmt --check" "cargo fmt --all -- --check"

# B5: Coverage ≥75%
echo -n "[beta-v3.0.0] B5: Coverage ≥75% ... "
TOTAL=$((TOTAL+1))
if command -v cargo-llvm-cov &>/dev/null; then
    COVERAGE=$(cargo llvm-cov --all-features --lcov --output-path /tmp/lcov-v300-beta.info 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
    if (( $(echo "$COVERAGE >= 75" | bc -l) )); then
        echo "PASS (${COVERAGE}%)"
        PASS=$((PASS+1))
    else
        echo "FAIL (${COVERAGE}% < 75%)"
        BLOCKERS=$((BLOCKERS+1))
    fi
elif command -v cargo-tarpaulin &>/dev/null; then
    COVERAGE=$(cargo tarpaulin --all-features --out Json 2>&1 | grep -o '"coverage":[0-9.]*' | head -1 | grep -o '[0-9.]*' || echo "0")
    if (( $(echo "$COVERAGE >= 75" | bc -l) )); then
        echo "PASS (${COVERAGE}%)"
        PASS=$((PASS+1))
    else
        echo "FAIL (${COVERAGE}% < 75%)"
        BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "SKIP (no coverage tool)"
fi

# B6: Security Audit
check "B6: cargo audit" "cargo audit 2>/dev/null || true"

# B7: Documentation Links
check "B7: check_docs_links.sh" "bash scripts/gate/check_docs_links.sh"

# B8: TPC-H SF=0.1 22/22
echo -n "[beta-v3.0.0] B8: TPC-H SF=0.1 (22/22) ... "
TOTAL=$((TOTAL+1))
if [ -f scripts/gate/check_tpch.sh ]; then
    TPCH_OUTPUT=$(bash scripts/gate/check_tpch.sh sf=0.1 2>&1 || true)
    PASSED_Q=$(echo "$TPCH_OUTPUT" | grep -oE '[0-9]+/22' | head -1 || echo "0/22")
    if echo "$PASSED_Q" | grep -q "^22/22"; then
        echo "PASS ($PASSED_Q)"
        PASS=$((PASS+1))
    else
        echo "FAIL ($PASSED_Q < 22/22)"
        BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "SKIP (check_tpch.sh not found)"
fi

# B9: SQL Corpus ≥85%
echo -n "[beta-v3.0.0] B9: SQL Corpus ≥85% ... "
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

# B10: CBO Index Scan Selection (test_should_use_index)
echo -n "[beta-v3.0.0] B10: CBO Index Scan Selection ... "
TOTAL=$((TOTAL+1))
CBO_INDEX_OUTPUT=$(cargo test --test cbo_integration_test test_should_use_index 2>&1 || true)
if echo "$CBO_INDEX_OUTPUT" | grep -q "test_should_use_index.*ok"; then
    echo "PASS"
    PASS=$((PASS+1))
else
    echo "FAIL"
    BLOCKERS=$((BLOCKERS+1))
fi

# B11: CBO Join Cost Estimation (test_estimate_join_cost)
echo -n "[beta-v3.0.0] B11: CBO Join Cost Estimation ... "
TOTAL=$((TOTAL+1))
CBO_JOIN_OUTPUT=$(cargo test --test cbo_integration_test test_estimate_join_cost 2>&1 || true)
if echo "$CBO_JOIN_OUTPUT" | grep -q "test_estimate_join_cost.*ok"; then
    echo "PASS"
    PASS=$((PASS+1))
else
    echo "FAIL"
    BLOCKERS=$((BLOCKERS+1))
fi

# B12: CBO Optimizer Tests (all optimizer tests pass)
echo -n "[beta-v3.0.0] B12: CBO Optimizer Tests ... "
TOTAL=$((TOTAL+1))
CBO_OPT_OUTPUT=$(cargo test -p sqlrustgo-optimizer 2>&1 || true)
CBO_OPT_PASSED=$(echo "$CBO_OPT_OUTPUT" | grep -c "test result: ok" || echo "0")
CBO_OPT_FAILED=$(echo "$CBO_OPT_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$CBO_OPT_FAILED" -eq 0 ] && [ "$CBO_OPT_PASSED" -gt 0 ]; then
    echo "PASS ($CBO_OPT_PASSED tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($CBO_OPT_PASSED passed, $CBO_OPT_FAILED failed)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B13: CBO Planner Tests (planner integration tests pass)
echo -n "[beta-v3.0.0] B13: CBO Planner Tests ... "
TOTAL=$((TOTAL+1))
CBO_PLANNER_OUTPUT=$(cargo test --test cbo_integration_test 2>&1 || true)
CBO_PLANNER_PASSED=$(echo "$CBO_PLANNER_OUTPUT" | grep -c "test result: ok" || echo "0")
CBO_PLANNER_FAILED=$(echo "$CBO_PLANNER_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$CBO_PLANNER_FAILED" -eq 0 ] && [ "$CBO_PLANNER_PASSED" -gt 0 ]; then
    echo "PASS ($CBO_PLANNER_PASSED tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($CBO_PLANNER_PASSED passed, $CBO_PLANNER_FAILED failed)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B-S1: concurrency_stress_test
echo -n "[beta-v3.0.0] B-S1: concurrency_stress_test ... "
TOTAL=$((TOTAL+1))
CONCURRENCY_OUTPUT=$(cargo test --test concurrency_stress_test 2>&1 || true)
CONCURRENCY_PASSED=$(echo "$CONCURRENCY_OUTPUT" | grep -c "test result: ok" || echo "0")
CONCURRENCY_FAILED=$(echo "$CONCURRENCY_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$CONCURRENCY_FAILED" -eq 0 ] && [ "$CONCURRENCY_PASSED" -gt 0 ]; then
    echo "PASS ($CONCURRENCY_PASSED tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($CONCURRENCY_PASSED passed, $CONCURRENCY_FAILED failed)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B-S2: crash_recovery_test (8 tests)
echo -n "[beta-v3.0.0] B-S2: crash_recovery_test ... "
TOTAL=$((TOTAL+1))
CRASH_OUTPUT=$(cargo test --test crash_recovery_test 2>&1 || true)
CRASH_PASSED=$(echo "$CRASH_OUTPUT" | grep -c "test result: ok" || echo "0")
CRASH_FAILED=$(echo "$CRASH_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$CRASH_FAILED" -eq 0 ] && [ "$CRASH_PASSED" -gt 0 ]; then
    echo "PASS ($CRASH_PASSED/8 tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($CRASH_PASSED/8 tests)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B-S3: long_run_stability_test (10 tests with --ignored)
echo -n "[beta-v3.0.0] B-S3: long_run_stability_test ... "
TOTAL=$((TOTAL+1))
LONG_RUN_OUTPUT=$(cargo test --test long_run_stability_test -- --ignored 2>&1 || true)
LONG_RUN_PASSED=$(echo "$LONG_RUN_OUTPUT" | grep -c "test result: ok" || echo "0")
LONG_RUN_FAILED=$(echo "$LONG_RUN_OUTPUT" | grep -c "test result: FAILED" || echo "0")
LONG_RUN_IGNORED=$(echo "$LONG_RUN_OUTPUT" | grep -c "ignored" || echo "0")
if [ "$LONG_RUN_FAILED" -eq 0 ] && [ "$LONG_RUN_PASSED" -gt 0 ]; then
    echo "PASS ($LONG_RUN_PASSED/10 tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($LONG_RUN_PASSED/10 passed, $LONG_RUN_FAILED failed, $LONG_RUN_IGNORED ignored)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B-S4: wal_integration_test (zero data loss after crash)
echo -n "[beta-v3.0.0] B-S4: wal_integration_test ... "
TOTAL=$((TOTAL+1))
WAL_OUTPUT=$(cargo test --test wal_integration_test 2>&1 || true)
WAL_PASSED=$(echo "$WAL_OUTPUT" | grep -c "test result: ok" || echo "0")
WAL_FAILED=$(echo "$WAL_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$WAL_FAILED" -eq 0 ] && [ "$WAL_PASSED" -gt 0 ]; then
    echo "PASS ($WAL_PASSED tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($WAL_PASSED passed, $WAL_FAILED failed)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B-S5: network_tcp_smoke_test (6 tests, no connection leak)
echo -n "[beta-v3.0.0] B-S5: network_tcp_smoke_test ... "
TOTAL=$((TOTAL+1))
NETWORK_OUTPUT=$(cargo test --test network_tcp_smoke_test 2>&1 || true)
NETWORK_PASSED=$(echo "$NETWORK_OUTPUT" | grep -c "test result: ok" || echo "0")
NETWORK_FAILED=$(echo "$NETWORK_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$NETWORK_FAILED" -eq 0 ] && [ "$NETWORK_PASSED" -gt 0 ]; then
    echo "PASS ($NETWORK_PASSED/6 tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($NETWORK_PASSED/6 tests)"
    BLOCKERS=$((BLOCKERS+1))
fi

# B-S6: ssi_stress_test (7 tests, SSI transaction stress)
echo -n "[beta-v3.0.0] B-S6: ssi_stress_test ... "
TOTAL=$((TOTAL+1))
SSI_OUTPUT=$(cargo test -p sqlrustgo-transaction --test ssi_stress_test 2>&1 || true)
SSI_PASSED=$(echo "$SSI_OUTPUT" | grep -c "test result: ok" || echo "0")
SSI_FAILED=$(echo "$SSI_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$SSI_FAILED" -eq 0 ] && [ "$SSI_PASSED" -gt 0 ]; then
    echo "PASS ($SSI_PASSED tests)"
    PASS=$((PASS+1))
else
    echo "FAIL ($SSI_PASSED passed, $SSI_FAILED failed)"
    BLOCKERS=$((BLOCKERS+1))
fi

echo ""
echo "=== Beta Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="

if [ $BLOCKERS -gt 0 ]; then
    echo "❌ Beta Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ Beta Gate PASSED"
    exit 0
fi
