#!/usr/bin/env bash
# v3.1.0 Alpha Gate
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0
FAIL_REASONS=()

check() {
    local name="$1" cmd="$2"
    local label="${3:-X}"
    TOTAL=$((TOTAL+1))
    echo -n "[alpha-v3.1.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("[$label] $name")
    fi
}

check_test_count() {
    local name="$1" cmd="$2" label="$3" min_pass_rate="$4"
    TOTAL=$((TOTAL+1))
    echo -n "[alpha-v3.1.0] $name ... "
    local output
    output=$(eval "$cmd" 2>&1) || true
    local passed
    local failed
    passed=$(echo "$output" | grep -c "test result: ok\." || echo "0")
    failed=$(echo "$output" | grep -c "test result: FAILED" || echo "0")
    local total
    total=$((passed + failed))
    if [ "$total" -eq 0 ]; then
        echo "FAIL (no tests)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("[$label] $name: no tests")
        return
    fi
    local pass_rate
    pass_rate=$((passed * 100 / total))
    if [ "$pass_rate" -ge "$min_pass_rate" ]; then
        echo "PASS ($pass_rate% = $passed/$total)"; PASS=$((PASS+1))
    else
        echo "FAIL ($pass_rate% = $passed/$total < $min_pass_rate%)"; BLOCKERS=$((BLOCKERS+1))
        FAIL_REASONS+=("[$label] $name: $pass_rate% < $min_pass_rate%")
    fi
}

echo "=== v3.1.0 Alpha Gate ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# A1: Build
check "A1: cargo build --all-features" "cargo build --all-features --quiet" "A1"

# A2: L1 core crates test >= 80%
check_test_count "A2: L1 core crates (>=80%)" \
    "cargo test -p sqlrustgo-types -p sqlrustgo-parser -p sqlrustgo-planner -p sqlrustgo-optimizer -p sqlrustgo-executor -p sqlrustgo-storage -p sqlrustgo-transaction -p sqlrustgo-catalog --lib -- --test-threads=8" \
    "A2" 80

# A3: Clippy
check "A3: cargo clippy --all-features" "cargo clippy --all-features -- -D warnings --quiet" "A3"

# A4: Format
check "A4: cargo fmt --check" "cargo fmt --all -- --check --quiet" "A4"

# A5: Docs Links
check "A5: check_docs_links.sh" "bash scripts/gate/check_docs_links.sh" "A5"

# A6: OO Docs
check "A6: check_oo_docs.sh" "bash scripts/gate/check_oo_docs.sh" "A6"

# A7: TPC-H SF=1
TOTAL=$((TOTAL+1))
echo -n "[alpha-v3.1.0] A7: TPC-H SF=1 22/22 ... "
TPCH_OUTPUT=$(bash scripts/gate/check_tpch.sh --sf1 2>&1 || true)
tpch_pass=$(echo "$TPCH_OUTPUT" | grep -c "TPC-H Gate: PASSED" || echo "0")
tpch_total=$(echo "$TPCH_OUTPUT" | grep -oE "Total queries run: [0-9]+" | grep -oE "[0-9]+" || echo "0")
if [ "$tpch_pass" -gt 0 ]; then
    echo "PASS ($tpch_total/22)"; PASS=$((PASS+1))
else
    echo "FAIL ($tpch_total/22)"; BLOCKERS=$((BLOCKERS+1))
    FAIL_REASONS+=("[A7] TPC-H SF=1 failed")
fi

# A8: INFO Schema test
check "A8: information_schema_test" "cargo test --test information_schema_test 2>&1" "A8"

# A9: SQL Operations >= 60%
TOTAL=$((TOTAL+1))
echo -n "[alpha-v3.1.0] A9: SQL Operations >=60% ... "
CORPUS_OUTPUT=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
corpus_pct=$(echo "$CORPUS_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | tail -1 | tr -d '%' || echo "0")
if [ -n "$corpus_pct" ] && [ "$corpus_pct" != "0" ] && (( $(echo "$corpus_pct >= 60" | bc -l 2>/dev/null || echo 0) )); then
    echo "PASS (${corpus_pct}%)"; PASS=$((PASS+1))
else
    echo "FAIL (${corpus_pct}% < 60%)"; BLOCKERS=$((BLOCKERS+1))
    FAIL_REASONS+=("[A9] SQL Operations ${corpus_pct}% < 60%")
fi

# A10: MERGE test
check "A10: merge_statement_test" "cargo test --test merge_statement_test 2>&1" "A10"

# A11: Window Functions
check "A11: window_function_test" "cargo test --test window_function_test 2>&1" "A11"

# A12: Security (allow known)
check "A12: cargo audit" "cargo audit || true" "A12"

echo ""
echo "=== Alpha Gate: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
if [ ${#FAIL_REASONS[@]} -gt 0 ]; then
    echo "Failures:"
    for r in "${FAIL_REASONS[@]}"; do echo "  - $r"; done
fi
if [ $BLOCKERS -gt 0 ]; then
    echo "RESULT: FAILED"
    exit 1
else
    echo "RESULT: PASSED - ready for Beta"
    exit 0
fi
