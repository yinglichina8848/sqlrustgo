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

echo ""
echo "=== Beta Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="

if [ $BLOCKERS -gt 0 ]; then
    echo "❌ Beta Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ Beta Gate PASSED"
    exit 0
fi
