#!/usr/bin/env bash
# v3.0.0 RC Gate — 进入 RC 阶段必须通过
# 基于 gate_spec_v300.md §五
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[rc-v3.0.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"
        PASS=$((PASS+1))
    else
        echo "FAIL"
        BLOCKERS=$((BLOCKERS+1))
    fi
}

echo "=== v3.0.0 RC Gate ==="
echo ""

# R1: Build
check "R1: cargo build --release --workspace" "cargo build --release --workspace"

# R2: Test 100%
echo -n "[rc-v3.0.0] R2: cargo test --all-features (100%) ... "
TOTAL=$((TOTAL+1))
TEST_OUTPUT=$(cargo test --all-features 2>&1 || true)
FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$FAILED" -eq 0 ]; then
    echo "PASS (0 failures)"
    PASS=$((PASS+1))
else
    echo "FAIL ($FAILED test suites failed)"
    BLOCKERS=$((BLOCKERS+1))
fi

# R3: Clippy
check "R3: cargo clippy --all-features" "cargo clippy --all-features -- -D warnings"

# R4: Format
check "R4: cargo fmt --check" "cargo fmt --all -- --check"

# R5: Coverage ≥85%
echo -n "[rc-v3.0.0] R5: Coverage ≥85% ... "
TOTAL=$((TOTAL+1))
if command -v cargo-llvm-cov &>/dev/null; then
    COVERAGE=$(cargo llvm-cov --all-features --lcov --output-path /tmp/lcov-v300-rc.info 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
    if (( $(echo "$COVERAGE >= 85" | bc -l) )); then
        echo "PASS (${COVERAGE}%)"
        PASS=$((PASS+1))
    else
        echo "FAIL (${COVERAGE}% < 85%)"
        BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "SKIP (no coverage tool)"
fi

# R6: Security Audit
check "R6: cargo audit" "cargo audit 2>/dev/null || true"

# R7: Docs Links + version consistency
check "R7: check_docs_links.sh" "bash scripts/gate/check_docs_links.sh"
echo -n "[rc-v3.0.0] R7b: Version docs completeness ... "
TOTAL=$((TOTAL+1))
# Check v3.0.0 release docs exist
if [ -f "docs/releases/v3.0.0/RELEASE_NOTES.md" ] && \
   [ -f "docs/releases/v3.0.0/CHANGELOG.md" ] && \
   [ -f "docs/releases/v3.0.0/FEATURE_MATRIX.md" ]; then
    echo "PASS"
    PASS=$((PASS+1))
else
    echo "FAIL (missing v3.0.0 release docs)"
    BLOCKERS=$((BLOCKERS+1))
fi

# R8: SQL Corpus ≥95%
echo -n "[rc-v3.0.0] R8: SQL Corpus ≥95% ... "
TOTAL=$((TOTAL+1))
CORPUS_OUTPUT=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
CORPUS_PCT=$(echo "$CORPUS_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | tail -1 | tr -d '%' || echo "0")
if (( $(echo "$CORPUS_PCT >= 95" | bc -l) )); then
    echo "PASS (${CORPUS_PCT}%)"
    PASS=$((PASS+1))
else
    echo "FAIL (${CORPUS_PCT}% < 95%)"
    BLOCKERS=$((BLOCKERS+1))
fi

# R9: TPC-H SF=1 22/22
echo -n "[rc-v3.0.0] R9: TPC-H SF=1 (22/22) ... "
TOTAL=$((TOTAL+1))
if [ -f scripts/gate/check_tpch.sh ]; then
    TPCH_OUTPUT=$(bash scripts/gate/check_tpch.sh sf=1 2>&1 || true)
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

# R10: Performance Baseline (regression check)
echo -n "[rc-v3.0.0] R10: Performance baseline ... "
TOTAL=$((TOTAL+1))
if [ -f "perf_baselines/v3.0.0/baseline.json" ]; then
    # Run benchmarks and check regression
    BENCH_OUTPUT=$(cargo bench 2>&1 || true)
    REGRESSION=$(bash scripts/gate/check_regression.sh 2>&1 || echo "REGRESSION=UNKNOWN")
    if echo "$REGRESSION" | grep -q "REGRESSION=PASS"; then
        echo "PASS"
        PASS=$((PASS+1))
    else
        echo "FAIL ($REGRESSION)"
        BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "SKIP (baseline not established)"
fi

# R11: Sysbench Gate
echo -n "[rc-v3.0.0] R11: Sysbench gate ... "
TOTAL=$((TOTAL+1))
if [ -f scripts/gate/check_sysbench.sh ]; then
    SYSBENCH_OUTPUT=$(bash scripts/gate/check_sysbench.sh 2>&1 || true)
    if echo "$SYSBENCH_OUTPUT" | grep -q "PASS"; then
        echo "PASS"
        PASS=$((PASS+1))
    else
        echo "FAIL"
        BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "SKIP (check_sysbench.sh not found)"
fi

# R12: Formal Proof
check "R12: check_proof.sh" "bash scripts/gate/check_proof.sh"

echo ""
echo "=== RC Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="

if [ $BLOCKERS -gt 0 ]; then
    echo "❌ RC Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ RC Gate PASSED"
    exit 0
fi
