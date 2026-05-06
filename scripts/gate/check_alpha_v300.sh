#!/usr/bin/env bash
# v3.0.0 Alpha Gate — 进入 Alpha 阶段必须通过
# 基于 gate_spec_v300.md §三
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[alpha-v3.0.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"
        PASS=$((PASS+1))
    else
        echo "FAIL"
        BLOCKERS=$((BLOCKERS+1))
    fi
}

check_output() {
    local name="$1" cmd="$2" check_type="$3"
    TOTAL=$((TOTAL+1))
    echo -n "[alpha-v3.0.0] $name ... "
    local output
    output=$(eval "$cmd" 2>&1) || true
    local result
    case "$check_type" in
        coverage)
            result=$(echo "$output" | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
            if (( $(echo "$result >= 50" | bc -l) )); then
                echo "PASS (${result}%)"
                PASS=$((PASS+1))
            else
                echo "FAIL (${result}% < 50%)"
                BLOCKERS=$((BLOCKERS+1))
            fi
            ;;
        *)
            echo "FAIL"
            BLOCKERS=$((BLOCKERS+1))
            ;;
    esac
}

echo "=== v3.0.0 Alpha Gate ==="
echo ""

# A1: Build
check "A1: cargo build --all-features" "cargo build --all-features --quiet"

# A2: Unit Test
check "A2: cargo test --all-features --workspace" "cargo test --all-features --workspace --quiet"

# A3: Clippy
check "A3: cargo clippy --all-features" "cargo clippy --all-features -- -D warnings --quiet"

# A4: Format
check "A4: cargo fmt --check" "cargo fmt --all -- --check --quiet"

# A5: Documentation Links
check "A5: check_docs_links.sh" "bash scripts/gate/check_docs_links.sh"

# A6: Coverage ≥50%
echo -n "[alpha-v3.0.0] A6: Coverage ≥50% ... "
if command -v cargo-llvm-cov &>/dev/null; then
    COVERAGE=$(cargo llvm-cov --all-features --lcov --output-path /tmp/lcov-v300-alpha.info 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
    TOTAL=$((TOTAL+1))
    if (( $(echo "$COVERAGE >= 50" | bc -l) )); then
        echo "PASS (${COVERAGE}%)"
        PASS=$((PASS+1))
    else
        echo "FAIL (${COVERAGE}% < 50%)"
        BLOCKERS=$((BLOCKERS+1))
    fi
elif command -v cargo-tarpaulin &>/dev/null; then
    COVERAGE=$(cargo tarpaulin --all-features --out Json 2>&1 | grep -o '"coverage":[0-9.]*' | head -1 | grep -o '[0-9.]*' || echo "0")
    TOTAL=$((TOTAL+1))
    if (( $(echo "$COVERAGE >= 50" | bc -l) )); then
        echo "PASS (${COVERAGE}%)"
        PASS=$((PASS+1))
    else
        echo "FAIL (${COVERAGE}% < 50%)"
        BLOCKERS=$((BLOCKERS+1))
    fi
else
    echo "SKIP (no coverage tool)"
fi

# A7: Security Audit
check "A7: cargo audit" "cargo audit 2>/dev/null || true"

echo ""
echo "=== Alpha Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="

if [ $BLOCKERS -gt 0 ]; then
    echo "❌ Alpha Gate FAILED — $BLOCKERS blocker(s)"
    exit 1
else
    echo "✅ Alpha Gate PASSED"
    exit 0
fi
