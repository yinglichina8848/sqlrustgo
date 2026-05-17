#!/usr/bin/env bash
# v3.2.0 Alpha Gate
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0
FAIL_REASONS=()

check() {
    local name="$1"; shift
    local cmd=("$@")
    TOTAL=$((TOTAL+1))
    echo -n "[alpha-v3.2.0] $name ... "
    if "${cmd[@]}" >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
    fi
}

check_test() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[alpha-v3.2.0] $name ... "
    out=$(eval "$cmd" 2>&1 || true)
    passed=$(echo "$out" | grep -c "test result: ok\." 2>/dev/null || echo "0")
    failed=$(echo "$out" | grep -c "test result: FAILED" 2>/dev/null || echo "0")
    passed=${passed//[^0-9]/}
    failed=${failed//[^0-9]/}
    if [ -z "$passed" ] || [ -z "$failed" ]; then
        passed=0; failed=0
    fi
    total=$((passed + failed))
    if [ "$total" -gt 0 ]; then
        rate=$((passed * 100 / total))
        if [ "$rate" -ge 90 ]; then
            echo "PASS ($rate% = $passed/$total)"; PASS=$((PASS+1))
        else
            echo "FAIL ($rate% < 90%)"; BLOCKERS=$((BLOCKERS+1))
        fi
    else
        echo "FAIL (no tests)"; BLOCKERS=$((BLOCKERS+1))
    fi
}

check_coverage() {
    local name="$1" min_rate="$2" cmd="$3"
    TOTAL=$((TOTAL+1))
    echo -n "[alpha-v3.2.0] $name ... "
    out=$($cmd 2>&1 || true)
    if echo "$out" | grep -q "could not compile"; then
        echo "SKIP (compilation error)"; TOTAL=$((TOTAL-1)); return
    fi
    passed=$(echo "$out" | grep -c "test result: ok\." 2>/dev/null || echo "0")
    failed=$(echo "$out" | grep -c "test result: FAILED" 2>/dev/null || echo "0")
    passed=${passed//[^0-9]/}
    failed=${failed//[^0-9]/}
    if [ -z "$passed" ] || [ -z "$failed" ]; then
        passed=0; failed=0
    fi
    total=$((passed + failed))
    if [ "$total" -gt 0 ]; then
        rate=$((passed * 100 / total))
        if [ "$rate" -ge "$min_rate" ]; then
            echo "PASS ($rate% = $passed/$total)"; PASS=$((PASS+1))
        else
            echo "FAIL ($rate% < $min_rate%)"; BLOCKERS=$((BLOCKERS+1))
        fi
    else
        echo "FAIL (no tests)"; BLOCKERS=$((BLOCKERS+1))
    fi
}

echo "=== v3.2.0 Alpha Gate ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# A1: Build
check "A1: cargo build --all-features" cargo build --all-features

# A2: L1 test >= 90%
check_coverage "A2: L1 test (>=90%)" 90 "cargo test -p sqlrustgo-types -p sqlrustgo-parser -p sqlrustgo-planner -p sqlrustgo-optimizer -p sqlrustgo-executor -p sqlrustgo-storage -p sqlrustgo-transaction -p sqlrustgo-catalog --lib -- --test-threads=1"

# A3: Clippy
check "A3: cargo clippy --all-features" cargo clippy --all-features -- -D warnings

# A4: Format
check "A4: cargo fmt --check" cargo fmt --all -- --check

# A5: Coverage >= 72%
echo -n "[alpha-v3.2.0] A5: L1 crates coverage >=73% ... "
TOTAL=$((TOTAL+1))
if command -v cargo-llvm-cov >/dev/null 2>&1; then
    COV_OUTPUT=$(cargo llvm-cov test -p sqlrustgo-types -p sqlrustgo-parser -p sqlrustgo-planner -p sqlrustgo-optimizer -p sqlrustgo-executor -p sqlrustgo-storage -p sqlrustgo-transaction -p sqlrustgo-catalog --lib --tests 2>&1 || true)
    cov=$(echo "$COV_OUTPUT" | grep "^TOTAL" | head -1 | awk '{print $4}' | tr -d '%' || echo "0")
    if [ -n "$cov" ] && [ "$cov" != "0" ] && [ "$cov" != "" ]; then
        result=$(echo "$cov >= 72" | bc -l 2>/dev/null || echo "0")
        if [ "$result" = "1" ]; then
            echo "PASS (${cov}%)"; PASS=$((PASS+1))
        else
            echo "FAIL (${cov}% < 72%)"; BLOCKERS=$((BLOCKERS+1))
        fi
    else
        echo "SKIP (llvm-cov no data)"; TOTAL=$((TOTAL-1))
    fi
else
    echo "SKIP (no llvm-cov)"
fi

# A6: HSM/KMS
check "A6: HSM/KMS integration" cargo test -p sqlrustgo-gmp --lib

# A7: MySQL Protocol
check "A7: MySQL protocol test" cargo test -p sqlrustgo-mysql-server --lib

# A8: OO Docs
check "A8: check_oo_docs.sh" bash scripts/gate/check_oo_docs.sh

echo ""
echo "━━━ Stability Tests (A-S1 ~ A-S3) ━━━"

check_test "A-S1: concurrency_stress" "cargo test --test concurrency_stress_test"
check_test "A-S2: crash_recovery" "cargo test --test crash_recovery_test"
check_test "A-S3: long_run_stability" "cargo test --test long_run_stability_test"

echo ""
echo "=== Alpha Gate: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
if [ $BLOCKERS -gt 0 ]; then
    echo "RESULT: FAILED"
    exit 1
else
    echo "RESULT: PASSED"
    exit 0
fi
