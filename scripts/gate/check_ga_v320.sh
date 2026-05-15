#!/usr/bin/env bash
# check_ga_v320.sh — v3.2.0 GA Gate 检查脚本
# 规范来源: gate_spec_v320.md (SSOT)
# 版本: 1.0
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0
FAIL_REASONS=()
SKIP_REASONS=()

log_info() { echo "[ga-v3.2.0] $1"; }
log_pass() { echo "✅ PASS: $1"; PASS=$((PASS+1)); TOTAL=$((TOTAL+1)); }
log_fail() { echo "❌ FAIL: $1"; BLOCKERS=$((BLOCKERS+1)); TOTAL=$((TOTAL+1)); FAIL_REASONS+=("$1"); }
log_skip() { echo "⏭️  SKIP: $1"; TOTAL=$((TOTAL+1)); SKIP_REASONS+=("$1"); }
log_warn() { echo "⚠️  WARN: $1"; }

check() {
    local name="$1" cmd="$2"
    local label="${3:-G}"
    TOTAL=$((TOTAL+1))
    echo -n "[ga-v3.2.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("【$label】$name")
    fi
}

check_test() {
    local name="$1" cmd="$2"
    local label="$3"
    TOTAL=$((TOTAL+1))
    echo -n "[ga-v3.2.0] $name ... "
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

echo "=========================================="
echo "  v3.2.0 GA Gate — 正式发布前必须通过"
echo "  规范来源: gate_spec_v320.md (SSOT)"
echo "=========================================="
echo "日期: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# ============================================================
# 第一部分: 代码层 Gate (G1-G6)
# ============================================================
echo "━━━ 第一部分: 代码层 Gate (G1-G6) ━━━"

# G1: Build
log_info "G1: cargo build --release --workspace"
TOTAL=$((TOTAL+1))
if cargo build --release --workspace >/dev/null 2>&1; then
    echo "✅ PASS"; PASS=$((PASS+1))
else
    echo "❌ FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G1: Build failed")
fi

# G2: Test (100%)
log_info "G2: Full test suite (100%)"
TOTAL=$((TOTAL+1))
TEST_OUTPUT=$(cargo test --all-features --lib 2>&1 || true)
PASSED=$(echo "$TEST_OUTPUT" | grep -c "test result: ok\." 2>/dev/null || echo "0")
FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" 2>/dev/null || echo "0")
PASSED=${PASSED//[^0-9]/}
FAILED=${FAILED//[^0-9]/}
if [ -z "$PASSED" ]; then PASSED=0; fi
if [ -z "$FAILED" ]; then FAILED=0; fi
TOTAL_TESTS=$((PASSED + FAILED))
if [ "$TOTAL_TESTS" -gt 0 ]; then
    if [ "$FAILED" -eq 0 ] && [ "$PASSED" -gt 0 ]; then
        echo "✅ PASS (100% = $PASSED/$TOTAL_TESTS)"; PASS=$((PASS+1))
    else
        echo "❌ FAIL ($PASSED passed, $FAILED failed)"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G2: Test $PASSED passed, $FAILED failed < 100%")
    fi
else
    echo "❌ FAIL (no tests found)"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G2: No tests found")
fi

# G3: Clippy
log_info "G3: cargo clippy --all-features"
TOTAL=$((TOTAL+1))
if cargo clippy --all-features -- -D warnings >/dev/null 2>&1; then
    echo "✅ PASS"; PASS=$((PASS+1))
else
    echo "❌ FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G3: Clippy warnings")
fi

# G4: Format
log_info "G4: cargo fmt --check"
TOTAL=$((TOTAL+1))
if cargo fmt --all -- --check >/dev/null 2>&1; then
    echo "✅ PASS"; PASS=$((PASS+1))
else
    echo "❌ FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G4: Format issues")
fi

# G5: Coverage >= 85%
log_info "G5: Coverage >= 85%"
TOTAL=$((TOTAL+1))
if command -v cargo-llvm-cov &>/dev/null; then
    cargo llvm-cov test --workspace --all-features --no-fail-fast >/dev/null 2>&1 || true
    cargo llvm-cov report --lcov --output-path /tmp/lcov-v320-ga.info 2>/dev/null || true
    TOTAL_LINES=$(grep "^LF:" /tmp/lcov-v320-ga.info 2>/dev/null | cut -d: -f2 | awk '{sum+=$1} END {print sum}' || echo "0")
    COVERED_LINES=$(grep "^LH:" /tmp/lcov-v320-ga.info 2>/dev/null | cut -d: -f2 | awk '{sum+=$1} END {print sum}' || echo "0")
    if [ "$TOTAL_LINES" -gt 0 ]; then
        COVERAGE=$(echo "scale=2; $COVERED_LINES * 100 / $TOTAL_LINES" | bc)
        if (( $(echo "$COVERAGE >= 85" | bc -l) )); then
            echo "✅ PASS (${COVERAGE}%)"; PASS=$((PASS+1))
        else
            echo "❌ FAIL (${COVERAGE}% < 85%)"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G5: Coverage ${COVERAGE}% < 85%")
        fi
    else
        echo "⏭️  SKIP (no coverage data)"; SKIP_REASONS+=("G5: No coverage data")
    fi
else
    echo "⏭️  SKIP (cargo-llvm-cov not installed)"; SKIP_REASONS+=("G5: No cargo-llvm-cov")
fi

# G6: Security Audit
log_info "G6: cargo audit"
TOTAL=$((TOTAL+1))
if cargo audit >/dev/null 2>&1; then
    echo "✅ PASS"; PASS=$((PASS+1))
else
    echo "❌ FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G6: Security vulnerabilities found")
fi

# ============================================================
# 第二部分: 功能 Gate (G7-G12)
# ============================================================
echo ""
echo "━━━ 第二部分: 功能 Gate (G7-G12) ━━━"

# G7: HSM/KMS Integration
check "G7: HSM/KMS integration" "cargo test -p sqlrustgo-hsm --lib" "G7"

# G8: MySQL Protocol
check "G8: MySQL protocol" "cargo test --test mysql_protocol_test" "G8"

# G9: Window Functions
check_test "G9: window_function_test" "cargo test --test window_function_test" "G9"

# G10: Multi-table DML
check_test "G10: dml_multi_table_test" "cargo test --test dml_multi_table_test" "G10"

# G11: HASH JOIN
check_test "G11: hash_join_test" "cargo test --test hash_join_test" "G11"

# G12: TPC-H SF=1
log_info "G12: TPC-H SF=1 22/22"
TOTAL=$((TOTAL+1))
if bash scripts/gate/check_tpch.sh --sf1 >/dev/null 2>&1; then
    echo "✅ PASS"; PASS=$((PASS+1))
else
    echo "❌ FAIL"; BLOCKERS=$((BLOCKERS+1)); FAIL_REASONS+=("G12: TPC-H SF=1 failed")
fi

# ============================================================
# 第三部分: QA Enhancement Tests (G-QA1 ~ G-QA10)
# ============================================================
echo ""
echo "━━━ 第三部分: QA Enhancement Tests (G-QA1 ~ G-QA10) ━━━"

check_test "G-QA1: fts_tests" "cargo test -p sqlrustgo-executor --test fts_tests 2>&1" "G-QA1"
check_test "G-QA2: gis_spatial_test" "cargo test --test gis_spatial_test 2>&1" "G-QA2"
check_test "G-QA3: event_scheduler_test" "cargo test --test event_scheduler_test 2>&1" "G-QA3"
check_test "G-QA4: merge_execution_test" "cargo test --test merge_execution_test 2>&1" "G-QA4"
check_test "G-QA5: set_operation_test" "cargo test --test set_operation_test 2>&1" "G-QA5"
check_test "G-QA6: explain_analyze_test" "cargo test --test explain_analyze_test 2>&1" "G-QA6"
check_test "G-QA7: ddl_statement_test" "cargo test --test ddl_statement_test 2>&1" "G-QA7"
check_test "G-QA8: gmp_digital_signature_test" "cargo test --test gmp_digital_signature_test 2>&1" "G-QA8"
check_test "G-QA9: gmp_electronic_signature_test" "cargo test --test gmp_electronic_signature_test 2>&1" "G-QA9"
check "G-QA10: QA Enhancement Suite" "GATE_STAGE=ga bash scripts/gate/check_qa_enhancement.sh" "G-QA10"

# ============================================================
# 第四部分: 稳定性测试 Gate (G-S1 ~ G-S20)
# ============================================================
echo ""
echo "━━━ 第四部分: 稳定性测试 Gate (G-S1 ~ G-S20) ━━━"

check_test "G-S1: concurrency_stress" "cargo test --test concurrency_stress_test 2>&1" "G-S1"
check_test "G-S2: crash_recovery" "cargo test --test crash_recovery_test 2>&1" "G-S2"
check_test "G-S3: long_run_stability" "cargo test --test long_run_stability_test 2>&1" "G-S3"
check_test "G-S4: wal_integration" "cargo test --test wal_integration_test 2>&1" "G-S4"
check_test "G-S5: network_tcp" "cargo test --test network_tcp_smoke_test 2>&1" "G-S5"
check_test "G-S6: ssi_stress" "cargo test -p sqlrustgo-transaction --test ssi_stress_test 2>&1" "G-S6"
check_test "G-S7: audit_trail" "cargo test -p sqlrustgo-server --test wal_crash_recovery_test 2>&1" "G-S7"
check_test "G-S8: integration_tests" "bash scripts/test/run_integration.sh --quick 2>&1" "G-S8"
check_test "G-S9: sysbench" "bash scripts/gate/check_sysbench.sh ga 2>&1" "G-S9"
check_test "G-S10: regression_check" "bash scripts/gate/check_regression.sh 2>&1" "G-S10"
check_test "G-S11: proof_count" "bash scripts/gate/check_proof.sh 2>&1" "G-S11"
check_test "G-S12: docs_links" "bash scripts/gate/check_docs_links.sh 2>&1" "G-S12"
check_test "G-S13: sql_corpus" "cargo test -p sqlrustgo-sql-corpus 2>&1" "G-S13"
check_test "G-S14: coverage_check" "bash scripts/gate/check_coverage.sh 2>&1" "G-S14"
check_test "G-S15: security_audit" "cargo audit 2>&1" "G-S15"
check_test "G-S16: static_analysis" "bash scripts/gate/check_static_analysis.sh 2>&1" "G-S16"
check_test "G-S17: mutants_check" "bash scripts/gate/check_mutants.sh 2>&1" "G-S17"
check_test "G-S18: oo_docs" "bash scripts/gate/check_oo_docs.sh 2>&1" "G-S18"
check_test "G-S19: information_schema" "cargo test --test information_schema_test 2>&1" "G-S19"
check_test "G-S20: benchmark_baseline" "bash scripts/gate/check_perf_baseline.sh 2>&1" "G-S20"

# ============================================================
# 结果汇总
# ============================================================
echo ""
echo "=========================================="
echo "  GA Gate 结果汇总"
echo "=========================================="
echo "✅ PASS: $PASS"
echo "⏭️  SKIP: ${#SKIP_REASONS[@]} (需手动验证)"
echo "❌ FAIL: $BLOCKERS"
echo ""

if [ ${#FAIL_REASONS[@]} -gt 0 ]; then
    echo "失败项:"
    for reason in "${FAIL_REASONS[@]}"; do
        echo "  - $reason"
    done
fi

if [ ${#SKIP_REASONS[@]} -gt 0 ]; then
    echo ""
    echo "手动验证项 (需人工确认):"
    for reason in "${SKIP_REASONS[@]}"; do
        echo "  - $reason"
    done
fi

echo ""
if [ $BLOCKERS -gt 0 ]; then
    echo "❌ GA Gate FAILED — $BLOCKERS blocker(s) detected"
    exit 1
else
    echo "✅ GA Gate PASSED (with ${#SKIP_REASONS[@]} manual verifications pending)"
    exit 0
fi
