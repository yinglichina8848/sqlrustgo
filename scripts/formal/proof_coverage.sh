#!/usr/bin/env bash
# proof_coverage.sh — Proof Coverage v2: Risk-Weighted Formal Invariant Coverage
#
# 计算 PR 改动的形式化不变量风险覆盖率
#
# 用法:
#   ./proof_coverage.sh --ci             # CI 模式（相对于目标分支）
#   ./proof_coverage.sh --full           # 全量覆盖报告（所有 invariants）
#   ./proof_coverage.sh                  # 本地模式（相对于 HEAD~10）
#
# CI Gate:
#   0 = risk_score >= 0.70 (PASS)
#   1 = risk_score < 0.50 (FAIL)
#   2 = risk_score 0.50-0.70 (WARN)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COVERAGE_DB="$PROJECT_ROOT/docs/formal/PROOF_COVERAGE.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

MODE="${1:-local}"
BASE_BRANCH="${CI_MERGE_REQUEST_TARGET_BRANCH:-origin/develop/v2.9.0}"

COVERAGE_TMP=$(mktemp -d)
trap "rm -rf $COVERAGE_TMP" EXIT

CHANGED_FILES="$COVERAGE_TMP/changed_files.txt"
TEST_LOG="$COVERAGE_TMP/test_output.log"
COVERAGE_JSON="$COVERAGE_TMP/coverage_report.json"

info() { echo -e "${BLUE}  ℹ${NC}  $1"; }

# ───────────────────────────────────────────────
# Step 1: 获取改动文件
# ───────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════════"
echo "  Proof Coverage v2 — Risk-Weighted Report"
echo "══════════════════════════════════════════════"
echo ""

case "$MODE" in
    --ci)
        info "Mode: CI | Base: $BASE_BRANCH"
        git fetch origin "$BASE_BRANCH" 2>/dev/null || true
        git diff --name-only "origin/$BASE_BRANCH...HEAD" > "$CHANGED_FILES"
        ;;
    --full)
        info "Mode: Full (all transaction + execution_engine Rust files)"
        git ls-files '*.rs' | grep -E '^(crates/transaction|execution_engine)/' | sort -u > "$CHANGED_FILES"
        ;;
    *)
        info "Mode: Local (HEAD~10)"
        git diff --name-only HEAD~10...HEAD 2>/dev/null > "$CHANGED_FILES" || \
        git diff --name-only HEAD~5...HEAD > "$CHANGED_FILES"
        ;;
esac

CHANGED_COUNT=$(wc -l < "$CHANGED_FILES")
echo ""
echo "Changed files ($CHANGED_COUNT):"
cat "$CHANGED_FILES" | sed 's/^/  /' | head -15
[ "$CHANGED_COUNT" -gt 15 ] && echo "  ... ($((CHANGED_COUNT - 15)) more)"
echo ""

# ───────────────────────────────────────────────
# Step 2: 运行 tests
# ───────────────────────────────────────────────

info "Running transaction tests..."
cd "$PROJECT_ROOT"
cargo test -p sqlrustgo-transaction -- --nocapture > "$TEST_LOG" 2>&1 || true

# ───────────────────────────────────────────────
# Step 3: Python — 完整风险评分计算
# ───────────────────────────────────────────────

info "Computing risk-weighted coverage..."
echo ""

python3 "$SCRIPT_DIR/_proof_coverage_py.py" \
  "$COVERAGE_DB" "$CHANGED_FILES" "$TEST_LOG" "$COVERAGE_JSON" "$MODE"

# ───────────────────────────────────────────────
# Step 4: 读取 JSON 输出摘要
# ───────────────────────────────────────────────

RESULT_JSON=$(cat "$COVERAGE_JSON")
RISK_SCORE=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['risk_score'])")
GATE=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['gate'])")
GATE_EXIT=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['gate_exit'])")
RISK_MIN=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['risk_min'])")
WARN_RISK=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['warn_risk'])")

echo ""
echo "══════════════════════════════════════════════"
echo "  Summary"
echo "══════════════════════════════════════════════"
printf "  Risk Score:   %.3f / 1.000\n" "$RISK_SCORE"
printf "  Gate:         %s\n" "$GATE"
printf "  Threshold:    %.2f (PASS) | %.2f (WARN)\n" "$RISK_MIN" "$WARN_RISK"
echo "══════════════════════════════════════════════"
echo ""

case "$GATE" in
    PASS) echo -e "  ${GREEN}✔ PASS${NC} — Risk Score ${RISK_SCORE} >= ${RISK_MIN}" ;;
    WARN) echo -e "  ${YELLOW}⚠ WARN${NC} — Risk Score ${RISK_SCORE}" ;;
    FAIL) echo -e "  ${RED}✘ FAIL${NC} — Risk Score ${RISK_SCORE}" ;;
    SKIP) echo -e "  ${BLUE}ℹ SKIP${NC} — No active invariants" ;;
esac
echo ""

# Copy to project root for CI artifact
cp "$COVERAGE_JSON" "$PROJECT_ROOT/coverage_result.json" 2>/dev/null || true

exit $GATE_EXIT
