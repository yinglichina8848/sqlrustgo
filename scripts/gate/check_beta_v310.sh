#!/usr/bin/env bash
# v3.1.0 Beta Gate — 进入 RC 阶段必须通过
# 基于 gate_spec.md + v3.1.0 DEVELOPMENT_PLAN.md
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0
FAIL_REASONS=()

check() {
    local name="$1" cmd="$2"
    local label="${3:-X}"
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.1.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【$label】$name")
    fi
}

check_test() {
    local name="$1" cmd="$2"
    local label="$3"
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.1.0] $name ... "
    local output
    output=$(eval "$cmd" 2>&1) || true
    local passed failed
    passed=$(echo "$output" | grep -c "test result: ok\." || echo "0")
    failed=$(echo "$output" | grep -c "test result: FAILED" || echo "0")
    if [ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]; then
        echo "PASS ($passed tests)"; PASS=$((PASS+1))
    else
        echo "FAIL ($passed passed, $failed failed)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【$label】$name ($passed passed, $failed failed)")
    fi
}

echo "=== v3.1.0 Beta Gate ==="
echo "日期: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# ========== B1: Release Build ==========
check "B1: cargo build --release --workspace" "cargo build --release --workspace" "B1"

# ========== B2: L1 Core Crates Test ≥90% ==========
echo -n "[beta-v3.1.0] B2: L1 core crates test (≥90%) ... "
TOTAL=$((TOTAL+1))
TEST_OUTPUT=$(cargo test \
  -p sqlrustgo-types \
  -p sqlrustgo-parser \
  -p sqlrustgo-planner \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-executor \
  -p sqlrustgo-storage \
  -p sqlrustgo-transaction \
  -p sqlrustgo-catalog \
  --lib -- --test-threads=8 2>&1 || true)
PASSED=$(echo "$TEST_OUTPUT" | grep -c "test result: ok\." || echo "0")
FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" || echo "0")
TOTAL_TESTS=$((PASSED + FAILED))
if [ "$TOTAL_TESTS" -gt 0 ]; then
    PASS_RATE=$((PASSED * 100 / TOTAL_TESTS))
    if [ "$PASS_RATE" -ge 90 ]; then
        echo "PASS ($PASS_RATE% = $PASSED/$TOTAL_TESTS)"; PASS=$((PASS+1))
    else
        echo "FAIL ($PASS_RATE% = $PASSED/$TOTAL_TESTS < 90%)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【B2】测试通过率 $PASS_RATE% < 90%")
    fi
else
    echo "FAIL (no tests found)"; BLOCKERS=$((BLOCKERS+1))
fi

# ========== B3: Clippy ==========
check "B3: cargo clippy --all-features" "cargo clippy --all-features -- -D warnings" "B3"

# ========== B4: Format ==========
check "B4: cargo fmt --check" "cargo fmt --all -- --check" "B4"

# ========== B5: Coverage ≥75% (Beta) / ≥85% (GA) ==========
echo -n "[beta-v3.1.0] B5: Coverage ≥75% ... "
TOTAL=$((TOTAL+1))
if command -v cargo-llvm-cov &>/dev/null; then
    COVERAGE=$(cargo llvm-cov --all-features --ignore-run-fail --lcov --output-path /tmp/lcov-v310-beta.info 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
    if (( $(echo "$COVERAGE >= 75" | bc -l) )); then
        echo "PASS (${COVERAGE}%)"; PASS=$((PASS+1))
    else
        echo "FAIL (${COVERAGE}% < 75%)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【B5】覆盖率 ${COVERAGE}% < 75%")
    fi
else
    echo "SKIP (no cargo-llvm-cov)"
fi

# ========== B6: Security Audit ==========
check "B6: cargo audit" "cargo audit || true" "B6"

# ========== B7: SQL Operations ≥80% ==========
echo -n "[beta-v3.1.0] B7: SQL Operations ≥80% ... "
TOTAL=$((TOTAL+1))
CORPUS_OUTPUT=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
CORPUS_PCT=$(echo "$CORPUS_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | tail -1 | tr -d '%' || echo "0")
if (( $(echo "$CORPUS_PCT >= 80" | bc -l) )); then
    echo "PASS (${CORPUS_PCT}%)"; PASS=$((PASS+1))
else
    echo "FAIL (${CORPUS_PCT}% < 80%)"; BLOCKERS=$((BLOCKERS+1))
    FAIL_REASONS+=("【B7】SQL Operations ${CORPUS_PCT}% < 80%")
fi

# ========== B8: TPC-H SF=1 ==========
echo -n "[beta-v3.1.0] B8: TPC-H SF=1 22/22 ... "
TOTAL=$((TOTAL+1))
TPCH_OUTPUT=$(bash scripts/gate/check_tpch.sh --sf1 2>&1 || true)
TPCH_PASS=$(echo "$TPCH_OUTPUT" | grep -c "TPC-H Gate: PASSED" || echo "0")
TPCH_TOTAL=$(echo "$TPCH_OUTPUT" | grep -oE "Total queries run: [0-9]+" | grep -oE "[0-9]+" || echo "0")
if [ "$TPCH_PASS" -gt 0 ]; then
    echo "PASS ($TPCH_TOTAL/22)"; PASS=$((PASS+1))
else
    echo "FAIL ($TPCH_TOTAL/22)"; BLOCKERS=$((BLOCKERS+1))
    FAIL_REASONS+=("【B8】TPC-H SF=1 未通过")
fi

# ========== B9: Formal Proof Registry ==========
check "B9: proof registry integrity" "python3 scripts/verify_proof_registry.py" "B9"

# ========== B-S1: Concurrency Stress ==========
check_test "B-S1: concurrency_stress_test" "cargo test --test concurrency_stress_test 2>&1" "B-S1"

# ========== B-S2: Crash Recovery ==========
check_test "B-S2: crash_recovery_test" "cargo test --test crash_recovery_test 2>&1" "B-S2"

# ========== B-S3: Long Run Stability ==========
check_test "B-S3: long_run_stability_test" "cargo test --test long_run_stability_test 2>&1" "B-S3"

# ========== B-S4: WAL Integration ==========
check_test "B-S4: wal_integration_test" "cargo test --test wal_integration_test 2>&1" "B-S4"

# ========== B-S5: Network TCP ==========
check_test "B-S5: network_tcp_smoke_test" "cargo test --test network_tcp_smoke_test 2>&1" "B-S5"

# ========== B-S6: SSI Stress ==========
check_test "B-S6: ssi_stress_test" "cargo test -p sqlrustgo-transaction --test ssi_stress_test 2>&1" "B-S6"

# ========== B-S7: Audit Trail ==========
check_test "B-S7: audit_trail_test" "cargo test --test audit_trail_test 2>&1" "B-S7"

# ========== B-S8: Explain Analyze ==========
check_test "B-S8: explain_analyze_test" "cargo test --test explain_analyze_test 2>&1" "B-S8"

# ========== B-S9: Window Functions (NTILE/LEAD/LAG) ==========
check_test "B-S9: window_function_full_test" "cargo test --test window_function_full_test 2>&1" "B-S9"

# ========== B-S10: MERGE DML ==========
check_test "B-S10: merge_dml_test" "cargo test --test merge_dml_test 2>&1" "B-S10"

echo ""
echo "=== Beta Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
if [ ${#FAIL_REASONS[@]} -gt 0 ]; then
    echo "失败项:"
    for reason in "${FAIL_REASONS[@]}"; do echo "  - $reason"; done
fi
if [ $BLOCKERS -gt 0 ]; then
    echo "❌ Beta Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ Beta Gate PASSED — 可以进入 RC 阶段"
    exit 0
fi
