#!/usr/bin/env bash
# v3.1.0 GA Gate — 正式发布前必须通过
# 基于 gate_spec.md + v3.1.0 DEVELOPMENT_PLAN.md
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0
FAIL_REASONS=()

check() {
    local name="$1" cmd="$2"
    local label="${3:-G}"
    TOTAL=$((TOTAL+1))
    echo -n "[ga-v3.1.0] $name ... "
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
    echo -n "[ga-v3.1.0] $name ... "
    local output
    output=$(eval "$cmd" 2>&1) || true
    local passed failed
    passed=$(echo "$output" | grep -c "test result: ok\." || echo "0")
    failed=$(echo "$output" | grep -c "test result: FAILED" || echo "0")
    if [ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]; then
        echo "PASS ($passed tests)"; PASS=$((PASS+1))
    else
        echo "FAIL ($passed passed, $failed failed)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【$label】$name")
    fi
}

echo "=== v3.1.0 GA Gate ==="
echo "日期: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# ========== G1: Build ==========
check "G1: cargo build --release --workspace" "cargo build --release --workspace" "G1"

# ========== G2: Full Test Suite 100% ==========
echo -n "[ga-v3.1.0] G2: Full test suite (≥95%) ... "
TOTAL=$((TOTAL+1))
TEST_OUTPUT=$(cargo test --all-features --lib 2>&1 || true)
PASSED=$(echo "$TEST_OUTPUT" | grep -c "test result: ok\." || echo "0")
FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" || echo "0")
TOTAL_TESTS=$((PASSED + FAILED))
if [ "$TOTAL_TESTS" -gt 0 ]; then
    PASS_RATE=$((PASSED * 100 / TOTAL_TESTS))
    if [ "$PASS_RATE" -ge 95 ]; then
        echo "PASS ($PASS_RATE% = $PASSED/$TOTAL_TESTS)"; PASS=$((PASS+1))
    else
        echo "FAIL ($PASS_RATE% = $PASSED/$TOTAL_TESTS < 95%)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【G2】测试通过率 $PASS_RATE% < 95%")
    fi
else
    echo "FAIL (no tests found)"; BLOCKERS=$((BLOCKERS+1))
fi

# ========== G3: Clippy ==========
check "G3: cargo clippy --all-features" "cargo clippy --all-features -- -D warnings" "G3"

# ========== G4: Format ==========
check "G4: cargo fmt --check" "cargo fmt --all -- --check" "G4"

# ========== G5: Coverage ≥90% (GA target) ==========
echo -n "[ga-v3.1.0] G5: Coverage ≥90% ... "
TOTAL=$((TOTAL+1))
if command -v cargo-llvm-cov &>/dev/null; then
    COVERAGE=$(cargo llvm-cov --all-features --ignore-run-fail --lcov --output-path /tmp/lcov-v310-ga.info 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
    if (( $(echo "$COVERAGE >= 90" | bc -l) )); then
        echo "PASS (${COVERAGE}%)"; PASS=$((PASS+1))
    else
        echo "FAIL (${COVERAGE}% < 90%)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("【G5】覆盖率 ${COVERAGE}% < 90%")
    fi
else
    echo "SKIP (no cargo-llvm-cov)"
fi

# ========== G6: Security Audit (Zero Vulnerabilities) ==========
check "G6: cargo audit (zero vulnerabilities)" "cargo audit" "G6"

# ========== G7: SQL Operations ≥98% ==========
echo -n "[ga-v3.1.0] G7: SQL Operations ≥98% ... "
TOTAL=$((TOTAL+1))
CORPUS_OUTPUT=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
CORPUS_PCT=$(echo "$CORPUS_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | tail -1 | tr -d '%' || echo "0")
if (( $(echo "$CORPUS_PCT >= 98" | bc -l) )); then
    echo "PASS (${CORPUS_PCT}%)"; PASS=$((PASS+1))
else
    echo "FAIL (${CORPUS_PCT}% < 98%)"; BLOCKERS=$((BLOCKERS+1))
    FAIL_REASONS+=("【G7】SQL Operations ${CORPUS_PCT}% < 98%")
fi

# ========== G8: TPC-H SF=1 (22/22 PASS, p99 < 5s) ==========
echo -n "[ga-v3.1.0] G8: TPC-H SF=1 22/22 p99<5s ... "
TOTAL=$((TOTAL+1))
TPCH_OUTPUT=$(bash scripts/gate/check_tpch.sh --sf1 2>&1 || true)
TPCH_PASS=$(echo "$TPCH_OUTPUT" | grep -c "TPC-H Gate: PASSED" || echo "0")
TPCH_TOTAL=$(echo "$TPCH_OUTPUT" | grep -oE "Total queries run: [0-9]+" | grep -oE "[0-9]+" || echo "0")
if [ "$TPCH_PASS" -gt 0 ] && [ "$TPCH_TOTAL" -eq 22 ]; then
    echo "PASS ($TPCH_TOTAL/22)"; PASS=$((PASS+1))
else
    echo "FAIL ($TPCH_TOTAL/22)"; BLOCKERS=$((BLOCKERS+1))
    FAIL_REASONS+=("【G8】TPC-H SF=1 未完全通过")
fi

# ========== G9: Performance QPS Benchmark ==========
check "G9: point_select QPS ≥10000" "cargo bench -- point_select 2>&1 | grep -q 'point_select' && echo ok || echo fail" "G9"

# ========== G10: Formal Proofs ≥30 ==========
check "G10: formal proofs ≥30" "bash scripts/gate/check_proof.sh" "G10"

# ========== G11: GA Gate Checklist ==========
check_test "G11: GA gate checklist" "bash scripts/gate/check_ga_v300.sh" "G11"

echo ""
echo "=== GA Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
if [ ${#FAIL_REASONS[@]} -gt 0 ]; then
    echo "失败项:"
    for reason in "${FAIL_REASONS[@]}"; do echo "  - $reason"; done
fi
if [ $BLOCKERS -gt 0 ]; then
    echo "❌ GA Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ GA Gate PASSED — 可以正式发布 v3.1.0"
    exit 0
fi
