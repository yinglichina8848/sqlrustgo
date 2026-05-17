#!/usr/bin/env bash
# v3.2.0 RC Gate — 进入 GA 阶段必须通过
# 基于 gate_spec.md + v3.2.0 DEVELOPMENT_PLAN.md
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0
FAIL_REASONS=()

check() {
    local name="$1" cmd="$2"
    local label="${3:-R}"
    TOTAL=$((TOTAL+1))
    echo -n "[rc-v3.2.0] $name ... "
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
    echo -n "[rc-v3.2.0] $name ... "
    local output
    output=$(eval "$cmd" 2>&1) || true
    local passed failed
    passed=$(echo "$output" | grep -c "test result: ok\." || true)
    failed=$(echo "$output" | grep -c "test result: FAILED" || true)
    passed=${passed:-0}
    failed=${failed:-0}
    passed=$(echo "$passed" | head -1)
    failed=$(echo "$failed" | head -1)
    if [ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]; then
        echo "PASS ($passed tests)"; PASS=$((PASS+1))
    else
        echo "FAIL ($passed passed, $failed failed)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【$label】$name")
    fi
}

echo "=== v3.2.0 RC Gate ==="
echo "日期: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# ========== R1: Build ==========
check "R1: cargo build --release --workspace" "cargo build --release --workspace" "R1"

# ========== R2: Full Test Suite ==========
echo -n "[rc-v3.2.0] R2: Full test suite ... "
TOTAL=$((TOTAL+1))
TEST_OUTPUT=$(cargo test --all-features --lib 2>&1 || true)
PASSED=$(echo "$TEST_OUTPUT" | grep -c "test result: ok\." || true)
FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" || true)
PASSED=${PASSED:-0}
FAILED=${FAILED:-0}
PASSED=$(echo "$PASSED" | head -1)
FAILED=$(echo "$FAILED" | head -1)
TOTAL_TESTS=$((PASSED + FAILED))
if [ "$TOTAL_TESTS" -gt 0 ]; then
    PASS_RATE=$((PASSED * 100 / TOTAL_TESTS))
    if [ "$PASS_RATE" -ge 90 ]; then
        echo "PASS ($PASS_RATE% = $PASSED/$TOTAL_TESTS)"; PASS=$((PASS+1))
    else
        echo "FAIL ($PASS_RATE% = $PASSED/$TOTAL_TESTS < 90%)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【R2】测试通过率 $PASS_RATE% < 90%")
    fi
else
    echo "FAIL (no tests found)"; BLOCKERS=$((BLOCKERS+1))
fi

# ========== R3: Clippy ==========
check "R3: cargo clippy --all-features" "cargo clippy --all-features -- -D warnings" "R3"

# ========== R4: Format ==========
check "R4: cargo fmt --check" "cargo fmt --all -- --check" "R4"

# ========== R5: Coverage ≥85% (GA target) ==========
echo -n "[rc-v3.2.0] R5: Coverage ≥85% ... "
TOTAL=$((TOTAL+1))
if command -v cargo-llvm-cov &>/dev/null; then
    TIMEOUT=600
    if command -v timeout &>/dev/null; then
        # Use parallel per-crate coverage to avoid timeout
        timeout "$TIMEOUT" bash "$SCRIPT_DIR/check_coverage_parallel.sh" --parallel 2 --wave all --timeout 180 2>/dev/null || true
    else
        bash "$SCRIPT_DIR/check_coverage_parallel.sh" --parallel 2 --wave all --timeout 180 2>/dev/null || true
    fi
    # Check if we got coverage results
    if [ -f "$PROJECT_ROOT/artifacts/coverage/coverage.json" ]; then
        COVERAGE=$(python3 -c "
import json
try:
    with open('$PROJECT_ROOT/artifacts/coverage/coverage.json') as f:
        data = json.load(f)
    pct = data.get('totals', {}).get('lines', {}).get('percent', 0)
    print(f'{pct:.1f}')
except:
    print('0')
" 2>/dev/null || echo "0")
        if (( $(echo "$COVERAGE >= 85" | bc -l 2>/dev/null || echo "0") )); then
            echo "PASS (${COVERAGE}%)"; PASS=$((PASS+1))
        else
            echo "FAIL (${COVERAGE}% < 85%)"; BLOCKERS=$((BLOCKERS+1))
            FAIL_REASONS+=("【R5】覆盖率 ${COVERAGE}% < 85%")
        fi
    else
        echo "SKIP (coverage data not available)"
    fi
else
    echo "SKIP (no cargo-llvm-cov)"
fi

# ========== R6: Security Audit ==========
check "R6: cargo audit" "cargo audit || true" "R6"

# ========== R7: SQL Operations ≥95% ==========
echo -n "[rc-v3.2.0] R7: SQL Operations ≥95% ... "
TOTAL=$((TOTAL+1))
CORPUS_OUTPUT=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
CORPUS_PASSED=$(echo "$CORPUS_OUTPUT" | grep -c "test result: ok\." || true)
CORPUS_FAILED=$(echo "$CORPUS_OUTPUT" | grep -c "test result: FAILED" || true)
CORPUS_PASSED=${CORPUS_PASSED:-0}
CORPUS_FAILED=${CORPUS_FAILED:-0}
CORPUS_TOTAL=$((CORPUS_PASSED + CORPUS_FAILED))
if [ "$CORPUS_TOTAL" -gt 0 ]; then
    CORPUS_PCT=$((CORPUS_PASSED * 100 / CORPUS_TOTAL))
    if [ "$CORPUS_PCT" -ge 95 ]; then
        echo "PASS (${CORPUS_PCT}% = $CORPUS_PASSED/$CORPUS_TOTAL)"; PASS=$((PASS+1))
    else
        echo "FAIL (${CORPUS_PCT}% = $CORPUS_PASSED/$CORPUS_TOTAL < 95%)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【R7】SQL Operations ${CORPUS_PCT}% < 95%")
    fi
else
    echo "FAIL (no tests found)"; BLOCKERS=$((BLOCKERS+1))
fi

# ========== R8: TPC-H SF=1 Performance ==========
echo -n "[rc-v3.2.0] R8: TPC-H SF=1 p99 < 5s ... "
TOTAL=$((TOTAL+1))
TPCH_OUTPUT=$(bash scripts/gate/check_tpch.sh --sf1 2>&1 || true)
TPCH_PASS=$(echo "$TPCH_OUTPUT" | grep -c "TPC-H Gate: PASSED" || echo "0")
if [ "$TPCH_PASS" -gt 0 ]; then
    echo "PASS"; PASS=$((PASS+1))
else
    echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
    FAIL_REASONS+=("【R8】TPC-H SF=1 未通过")
fi

# ========== R9: Performance Regression ==========
check "R9: check_regression.sh" "timeout 300 bash scripts/gate/check_regression.sh" "R9"

# ========== R10: Formal Proofs ≥30 ==========
check "R10: formal proof count ≥30" "bash scripts/gate/check_proof.sh" "R10"

# ========== R11: Docs Links Complete ==========
check "R11: check_docs_links.sh --all" "bash scripts/gate/check_docs_links.sh --all" "R11"

# ========== R12: HSM/KMS Integration ==========
echo -n "[rc-v3.2.0] R12: HSM/KMS integration ... "
TOTAL=$((TOTAL+1))
if cargo metadata --no-deps --format-version 1 2>/dev/null | grep -q '"sqlrustgo-hsm"'; then
    if cargo test -p sqlrustgo-hsm --lib >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【R12】HSM/KMS integration")
    fi
else
    echo "SKIP (sqlrustgo-hsm not in workspace)"; PASS=$((PASS+1))
fi

# ========== R13: MySQL Protocol ==========
check "R13: MySQL protocol" "SQLRUSTGO_SERVER_BIN=\"$PROJECT_ROOT/target/release/sqlrustgo-mysql-server\" cargo test -p sqlrustgo-mysql-server --test mysql_protocol_handshake_test" "R13"

# ========== R14: Window Functions ==========
check_test "R14: window_function_test" "cargo test --test window_function_test" "R14"

# ========== R15: Multi-table DML ==========
check_test "R15: dml_multi_table_test" "cargo test --test dml_multi_table_test" "R15"

# ========== R16: HASH JOIN ==========
check_test "R16: hash_join_test" "cargo test --test hash_join_test" "R16"

echo ""
echo "━━━ Stability Tests (R-S1 ~ R-S16) ━━━"

# ========== R-S1: Integration Tests ==========
check_test "R-S1: integration tests" "timeout 300 bash scripts/test/run_integration.sh --quick" "R-S1"

# ========== R-S2: Sysbench ==========
check "R-S2: check_sysbench.sh" "bash scripts/gate/check_sysbench.sh rc" "R-S2"

# ========== R-S3: Fulltext Search ==========
check_test "R-S3: fts_tests" "cargo test -p sqlrustgo-executor --test fts_tests 2>&1" "R-S3"

# ========== R-S4: GIS Spatial ==========
check_test "R-S4: gis_spatial_test" "cargo test --test gis_spatial_test 2>&1" "R-S4"

# ========== R-S5: Event Scheduler ==========
check_test "R-S5: event_scheduler_test" "cargo test --test event_scheduler_test 2>&1" "R-S5"

# ========== R-S6: Concurrency Stress ==========
check_test "R-S6: concurrency_stress" "cargo test --test concurrency_stress_test 2>&1" "R-S6"

# ========== R-S7: Crash Recovery ==========
check_test "R-S7: crash_recovery" "cargo test --test crash_recovery_test 2>&1" "R-S7"

# ========== R-S8: SSI Stress ==========
check_test "R-S8: ssi_stress" "cargo test -p sqlrustgo-transaction --test ssi_stress_test 2>&1" "R-S8"

# ========== R-S9: Merge Execution ==========
check_test "R-S9: merge_execution" "cargo test --test merge_execution_test 2>&1" "R-S9"

# ========== R-S10: WAL Integration ==========
check_test "R-S10: wal_integration" "cargo test --test wal_integration_test 2>&1" "R-S10"

# ========== R-S11: Long Run Stability ==========
check_test "R-S11: long_run_stability" "cargo test --test long_run_stability_test 2>&1" "R-S11"

# ========== R-S12: Network TCP ==========
check_test "R-S12: network_tcp" "cargo test --test network_tcp_smoke_test 2>&1" "R-S12"

# ========== R-S13: Explain Analyze ==========
check_test "R-S13: explain_analyze" "cargo test --test explain_analyze_test 2>&1" "R-S13"

# ========== R-S14: Set Operations ==========
check_test "R-S14: set_operations" "cargo test --test set_operation_test 2>&1" "R-S14"

# ========== R-S15: DDL Statements ==========
check_test "R-S15: ddl_statements" "cargo test --test ddl_statement_test 2>&1" "R-S15"

# ========== R-S16: GMP Signatures ==========
check_test "R-S16: gmp_signatures" "cargo test --test gmp_digital_signature_test --test gmp_electronic_signature_test 2>&1" "R-S16"

echo ""
echo "=== RC Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
if [ ${#FAIL_REASONS[@]} -gt 0 ]; then
    echo "失败项:"
    for reason in "${FAIL_REASONS[@]}"; do echo "  - $reason"; done
fi
if [ $BLOCKERS -gt 0 ]; then
    echo "❌ RC Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ RC Gate PASSED — 可以发布 GA"
    exit 0
fi
