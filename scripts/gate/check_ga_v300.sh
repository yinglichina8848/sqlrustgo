#!/usr/bin/env bash
# ============================================================
# v3.0.0 GA Gate — Final Release Gate
#
# After all RC gates pass, run this to confirm GA readiness.
# NO code changes allowed at this stage.
#
# GA Requirements:
#   - All RC gates passed
#   - 100% test pass (cargo test --all-features)
#   - Coverage ≥ 85%
#   - All 22 TPC-H SF=1 queries pass (no OOM)
#   - Performance regression within 5% of baseline
#   - All formal proofs valid
#   - Release documentation complete
#
# Usage:
#   bash scripts/gate/check_ga_v300.sh
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1"; shift
    local cmd="$*"
    TOTAL=$((TOTAL+1))
    echo -n "[ga-v3.0.0] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then
        echo "PASS"; PASS=$((PASS+1))
    else
        echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
    fi
}

check_output() {
    local name="$1"; local threshold="$2"; shift 2
    local cmd="$*"
    TOTAL=$((TOTAL+1))
    echo -n "[ga-v3.0.0] $name ... "
    local output result
    output=$(eval "$cmd" 2>&1) || true
    result=$(echo "$output" | grep -oE '[0-9]+\.[0-9]+%' | head -1 | tr -d '%' || echo "0")
    if (( $(echo "$result >= $threshold" | bc -l) )); then
        echo "PASS (${result}%)"; PASS=$((PASS+1))
    else
        echo "FAIL (${result}% < ${threshold}%)"; BLOCKERS=$((BLOCKERS+1))
    fi
}

echo "=== v3.0.0 GA Gate ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'N/A')"
echo ""

# ---------- Pre-flight: no uncommitted code ----------
echo "[pre-flight] Checking for uncommitted changes..."
if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
    echo "⚠️  WARNING: uncommitted changes detected — GA gate requires clean state"
    git status --short 2>/dev/null | head -10
fi
echo ""

# GA-1: Build (release)
check "GA-1: cargo build --release" "cargo build --release --workspace"

# GA-2: 100% test pass
echo -n "[ga-v3.0.0] GA-2: cargo test --all-features (100%) ... "
TOTAL=$((TOTAL+1))
TEST_OUTPUT=$(cargo test --all-features 2>&1 || true)
FAILED=$(echo "$TEST_OUTPUT" | grep -c "test result: FAILED" || echo "0")
if [ "$FAILED" -eq 0 ]; then
    echo "PASS (0 failures)"; PASS=$((PASS+1))
else
    echo "FAIL ($FAILED test suites failed)"; BLOCKERS=$((BLOCKERS+1))
fi

# GA-3: Integration tests
check "GA-3: Integration tests" "bash scripts/test/run_integration.sh --quick"

# GA-4: Clippy
check "GA-4: clippy" "cargo clippy --all-features -- -D warnings"

# GA-5: Format
check "GA-5: fmt" "cargo fmt --all -- --check"

# GA-6: Coverage ≥ 85%
check_output "GA-6: Coverage ≥ 85%" 85 "cargo llvm-cov --all-features --lcov --output-path /tmp/lcov-ga.info 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1"

# GA-7: Security audit
check "GA-7: cargo audit" "cargo audit"

# GA-8: Docs links
check "GA-8: docs links" "bash scripts/gate/check_docs_links.sh"

# GA-9: TPC-H SF=1 22/22 (no OOM)
echo -n "[ga-v3.0.0] GA-9: TPC-H SF=1 (22/22) ... "
TOTAL=$((TOTAL+1))
TPCH_OUTPUT=$(bash scripts/gate/check_tpch.sh sf=1 2>&1 || true)
TPCH_PASSED=$(echo "$TPCH_OUTPUT" | grep -oE '[0-9]+/22' | head -1 || echo "0/22")
TPCH_FAIL=$(echo "$TPCH_OUTPUT" | grep -c "FAIL" || echo "0")
if echo "$TPCH_PASSED" | grep -q "^22/22" && [ "$TPCH_FAIL" -eq 0 ]; then
    echo "PASS ($TPCH_PASSED)"; PASS=$((PASS+1))
else
    echo "FAIL ($TPCH_PASSED)"; BLOCKERS=$((BLOCKERS+1))
fi

# GA-10: Performance regression (within 5%)
check "GA-10: perf regression" "bash scripts/gate/check_regression.sh"

# GA-11: Formal proofs ≥ 10 files valid
echo -n "[ga-v3.0.0] GA-11: Formal proofs ... "
TOTAL=$((TOTAL+1))
PROOF_COUNT=$(find docs/proof -name "*.json" -type f 2>/dev/null | wc -l || echo "0")
INVALID_PROOF=0
for pf in $(find docs/proof -name "*.json" -type f 2>/dev/null); do
    python3 -c "import json; json.load(open('$pf'))" 2>/dev/null || INVALID_PROOF=$((INVALID_PROOF+1))
done
if [ "$PROOF_COUNT" -ge 10 ] && [ "$INVALID_PROOF" -eq 0 ]; then
    echo "PASS ($PROOF_COUNT proofs, 0 invalid)"; PASS=$((PASS+1))
else
    echo "FAIL ($PROOF_COUNT proofs, $INVALID_PROOF invalid)"; BLOCKERS=$((BLOCKERS+1))
fi

# GA-12: Sysbench gate
echo -n "[ga-v3.0.0] GA-12: Sysbench gate ... "
TOTAL=$((TOTAL+1))
if bash scripts/gate/check_sysbench.sh 2>&1 | grep -q "PASSED"; then
    echo "PASS"; PASS=$((PASS+1))
else
    echo "FAIL"; BLOCKERS=$((BLOCKERS+1))
fi

# GA-13: Release documentation completeness
echo -n "[ga-v3.0.0] GA-13: release docs ... "
TOTAL=$((TOTAL+1))
GA_DOCS=(
    "docs/releases/v3.0.0/RELEASE_NOTES.md"
    "docs/releases/v3.0.0/CHANGELOG.md"
    "docs/releases/v3.0.0/FEATURE_MATRIX.md"
    "docs/releases/v3.0.0/TEST_PLAN.md"
    "docs/releases/v3.0.0/PERFORMANCE_TARGETS.md"
    "docs/releases/v3.0.0/INSTALL.md"
    "docs/releases/v3.0.0/DEPLOYMENT_GUIDE.md"
    "docs/releases/v3.0.0/QUICK_START.md"
)
MISSING=0
for doc in "${GA_DOCS[@]}"; do
    [ -f "$PROJECT_ROOT/$doc" ] || MISSING=$((MISSING+1))
done
if [ "$MISSING" -eq 0 ]; then
    echo "PASS"; PASS=$((PASS+1))
else
    echo "FAIL ($MISSING docs missing)"; BLOCKERS=$((BLOCKERS+1))
fi

# GA-14: SQL Corpus ≥ 95%
echo -n "[ga-v3.0.0] GA-14: SQL Corpus ≥ 95% ... "
TOTAL=$((TOTAL+1))
CORPUS_OUTPUT=$(cargo test -p sqlrustgo-sql-corpus 2>&1 || true)
CORPUS_PCT=$(echo "$CORPUS_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | tail -1 | tr -d '%' || echo "0")
if (( $(echo "$CORPUS_PCT >= 95" | bc -l) )); then
    echo "PASS (${CORPUS_PCT}%)"; PASS=$((PASS+1))
else
    echo "FAIL (${CORPUS_PCT}% < 95%)"; BLOCKERS=$((BLOCKERS+1))
fi

# GA-15: Version consistency
echo -n "[ga-v3.0.0] GA-15: version consistency ... "
TOTAL=$((TOTAL+1))
VERSION_IN_CARGO=$(grep -m1 '^version' "$PROJECT_ROOT/Cargo.toml" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || echo "N/A")
VERSION_IN_DOCS=$(grep -r "v3.0.0" "$PROJECT_ROOT/docs/releases/v3.0.0/" 2>/dev/null | grep -c "version\|Version" || echo "0")
if [ "$VERSION_IN_CARGO" = "3.0.0" ] && [ "$VERSION_IN_DOCS" -gt 5 ]; then
    echo "PASS (cargo=$VERSION_IN_CARGO)"; PASS=$((PASS+1))
else
    echo "FAIL (cargo=$VERSION_IN_CARGO, docs refs=$VERSION_IN_DOCS)"; BLOCKERS=$((BLOCKERS+1))
fi

# ---------- Summary ----------
echo ""
echo "=== GA Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
echo ""

if [ $BLOCKERS -gt 0 ]; then
    echo "❌ GA Gate FAILED — $BLOCKERS blocker(s) must be resolved"
    exit 1
fi

echo "✅ GA Gate PASSED — v3.0.0 is ready for release"
echo ""
echo "Next steps:"
echo "  1. Tag: git tag v3.0.0 && git push origin v3.0.0"
echo "  2. Create release branch: git checkout -b release/v3.0.0"
echo "  3. Publish release artifacts"
exit 0