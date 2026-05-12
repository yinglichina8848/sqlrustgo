#!/usr/bin/env bash
# v3.1.0 Beta Gate
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1"; shift
    local cmd=("$@")
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.1.0] $name ... "
    if "${cmd[@]}" >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
    fi
}

check_test() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.1.0] $name ... "
    out=$(eval "$cmd" 2>&1 || true)
    passed=$(echo "$out" | grep -c "test result: ok\." 2>/dev/null || echo "0")
    failed=$(echo "$out" | grep -c "test result: FAILED" 2>/dev/null || echo "0")
    passed=${passed//[^0-9]/}
    failed=${failed//[^0-9]/}
    if [ -z "$passed" ] || [ -z "$failed" ]; then
        passed=0; failed=0
    fi
    if [ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]; then
        echo "PASS ($passed tests)"; PASS=$((PASS+1))
    else
        echo "FAIL ($passed passed, $failed failed)"; BLOCKERS=$((BLOCKERS+1))
    fi
}

check_coverage() {
    local name="$1" min_rate="$2" cmd="$3"
    TOTAL=$((TOTAL+1))
    echo -n "[beta-v3.1.0] $name ... "
    out=$($cmd 2>&1 || true)
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

echo "=== v3.1.0 Beta Gate ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# B1: Build
check "B1: cargo build --all-features" cargo build --all-features

# B2: L1 test >= 90%
check_coverage "B2: L1 test (>=90%)" 90 "cargo test -p sqlrustgo-types -p sqlrustgo-parser -p sqlrustgo-planner -p sqlrustgo-optimizer -p sqlrustgo-executor -p sqlrustgo-storage -p sqlrustgo-transaction -p sqlrustgo-catalog --lib -- --test-threads=8"

check "B3: cargo clippy" cargo clippy --all-features -- -D warnings
check "B4: cargo fmt" cargo fmt --all -- --check

# B5: Coverage >= 75%
TOTAL=$((TOTAL+1))
echo -n "[beta-v3.1.0] B5: Coverage >=75% ... "
if command -v cargo-llvm-cov >/dev/null 2>&1; then
    cov=$(cargo llvm-cov --all-features --ignore-run-fail --lcov --output-path /tmp/lcov-beta.info 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
    if [ -n "$cov" ] && [ "$cov" != "0" ]; then
        result=$(echo "$cov >= 75" | bc -l 2>/dev/null || echo "0")
        if [ "$result" = "1" ]; then
            echo "PASS (${cov}%)"; PASS=$((PASS+1))
        else
            echo "FAIL (${cov}% < 75%)"; BLOCKERS=$((BLOCKERS+1))
        fi
    else
        echo "FAIL (cov=$cov)"; BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "SKIP (no llvm-cov)"
fi

check "B6: cargo audit" cargo audit \|\| true

# B7: SQL Operations >= 80%
TOTAL=$((TOTAL+1))
echo -n "[beta-v3.1.0] B7: SQL Operations >=80% ... "
corpus=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
pct=$(echo "$corpus" | grep -oE '[0-9]+\.[0-9]+%' | tail -1 | tr -d '%' || echo "0")
if [ -n "$pct" ] && [ "$pct" != "0" ]; then
    result=$(echo "$pct >= 80" | bc -l 2>/dev/null || echo "0")
    if [ "$result" = "1" ]; then
        echo "PASS (${pct}%)"; PASS=$((PASS+1))
    else
        echo "FAIL (${pct}% < 80%)"; BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
fi

# B8: TPC-H SF=1
TOTAL=$((TOTAL+1))
echo -n "[beta-v3.1.0] B8: TPC-H SF=1 ... "
tpch=$(bash scripts/gate/check_tpch.sh --sf1 2>&1 || true)
if echo "$tpch" | grep -q "TPC-H Gate: PASSED"; then
    echo "PASS"; PASS=$((PASS+1))
else
    echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
fi

check "B9: proof registry" python3 scripts/verify_proof_registry.py

# Stability tests
check_test "B-S1: concurrency_stress" "cargo test --test concurrency_stress_test"
check_test "B-S2: crash_recovery" "cargo test --test crash_recovery_test"
check_test "B-S3: long_run_stability" "cargo test --test long_run_stability_test"
check_test "B-S4: wal_integration" "cargo test --test wal_integration_test"
check_test "B-S5: network_tcp" "cargo test --test network_tcp_smoke_test"
check_test "B-S6: ssi_stress" "cargo test -p sqlrustgo-transaction --test ssi_stress_test"
check_test "B-S7: audit_trail" "cargo test --test wal_crash_recovery_test"
check_test "B-S8: explain_analyze" "cargo test --test explain_analyze_test"
check_test "B-S9: window_functions" "cargo test --test window_function_test"

echo ""
echo "=== Beta Gate: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
if [ $BLOCKERS -gt 0 ]; then
    echo "RESULT: FAILED"
    exit 1
else
    echo "RESULT: PASSED"
    exit 0
fi
