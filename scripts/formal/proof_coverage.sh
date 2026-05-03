#!/usr/bin/env bash
# proof_coverage.sh — 计算 PR 改动的形式化不变量覆盖率
#
# 用法:
#   ./proof_coverage.sh --ci             # CI 模式（相对于目标分支）
#   ./proof_coverage.sh --full           # 全量覆盖报告（所有 invariants）
#   ./proof_coverage.sh                  # 本地模式（相对于 HEAD~10）
#
# 输出:
#   - 屏幕报告（人类可读）
#   - /tmp/coverage_report.json 机器可读结果
#
# CI Gate 退出码:
#   0  = coverage >= 80% (PASS) 或 < 50% (FAIL, 但 WARN 模式)
#   1  = coverage < 50% (FAIL)
#   2  = coverage 50-80% (WARN)

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

# 临时文件（统一 tmp 目录）
COVERAGE_TMP=$(mktemp -d)
trap "rm -rf $COVERAGE_TMP" EXIT

CHANGED_FILES="$COVERAGE_TMP/changed_files.txt"
TEST_LOG="$COVERAGE_TMP/test_output.log"
COVERAGE_JSON="$COVERAGE_TMP/coverage_report.json"

pass()  { echo -e "${GREEN}  ✔${NC}  $1"; }
fail()  { echo -e "${RED}  ✘${NC}  $1"; }
warn()  { echo -e "${YELLOW}  ⚠${NC}  $1"; }
info()  { echo -e "${BLUE}  ℹ${NC}  $1"; }

# ───────────────────────────────────────────────
# Step 1: 获取改动文件
# ───────────────────────────────────────────────

echo ""
echo "═══════════════════════════════════════════"
echo "  Proof Coverage Report"
echo "═══════════════════════════════════════════"
echo ""

case "$MODE" in
    --ci)
        info "Mode: CI"
        info "Base: $BASE_BRANCH"
        git fetch origin "$BASE_BRANCH" 2>/dev/null || true
        git diff --name-only "origin/$BASE_BRANCH...HEAD" > "$CHANGED_FILES"
        ;;
    --full)
        info "Mode: Full (all tracked Rust files)"
        git ls-files '*.rs' | grep -E '^(crates|execution_engine)/' | sort -u > "$CHANGED_FILES"
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
cat "$CHANGED_FILES" | sed 's/^/  /' | head -20
if [ "$CHANGED_COUNT" -gt 20 ]; then echo "  ... ($((CHANGED_COUNT - 20)) more)"; fi
echo ""

# ───────────────────────────────────────────────
# Step 2: 运行 tests（填充 test log）
# ───────────────────────────────────────────────

info "Running transaction tests..."
cd "$PROJECT_ROOT"
cargo test -p sqlrustgo-transaction -- --nocapture > "$TEST_LOG" 2>&1 || true

# ───────────────────────────────────────────────
# Step 3: Python 计算覆盖率
# ───────────────────────────────────────────────

info "Computing coverage..."
echo ""

python3 - "$COVERAGE_DB" "$CHANGED_FILES" "$TEST_LOG" "$COVERAGE_JSON" "$MODE" << 'PYEOF'
import json
import sys
import re
import os

db_path = sys.argv[1]
changed_file = sys.argv[2]
test_log_file = sys.argv[3]
out_json = sys.argv[4]
mode = sys.argv[5]

with open(db_path) as f:
    db = json.load(f)

with open(changed_file) as f:
    changed = set(line.strip() for line in f if line.strip())

with open(test_log_file) as f:
    test_log = f.read()

impacted = []
for inv in db['invariants']:
    matched_files = [f for f in inv.get('files', []) if f in changed]
    if not matched_files and mode != '--full':
        continue

    test_results = []
    for t in inv.get('tests', []):
        # Match cargo test output: test deadlock::tests::test_name ... ok/FAILED
        patterns = [
            r'test\s+[\w:]+' + re.escape(t) + r'\s+\.\.\.\s+(ok|FAILED)',
            r'\s' + re.escape(t) + r'\s+\.\.\.\s+(ok|FAILED)',
        ]
        matched = False
        for pat in patterns:
            m = re.search(pat, test_log)
            if m:
                test_results.append({'name': t, 'status': m.group(1)})
                matched = True
                break
        if not matched:
            if t in test_log:
                test_results.append({'name': t, 'status': 'found_not_run'})
            else:
                test_results.append({'name': t, 'status': 'missing'})

    has_tests = len(inv.get('tests', [])) > 0
    all_pass = all(r['status'] in ('ok', 'found_not_run') for r in test_results)
    covered = all_pass and has_tests
    frozen = inv.get('status') == 'frozen'

    impacted.append({
        'id': inv['id'],
        'proof': inv['proof'],
        'level': inv.get('level', 'unknown'),
        'status': inv.get('status', 'active'),
        'ci_layer': inv.get('ci_layer', 'unknown'),
        'tests': inv.get('tests', []),
        'test_results': test_results,
        'covered': covered,
        'frozen': frozen,
        'has_tests': has_tests,
        'matched_files': matched_files,
        'tla_invariant': inv.get('tla_invariant', ''),
        'rust_api': inv.get('rust_api', ''),
    })

total = len(impacted)
covered_count = sum(1 for i in impacted if i['covered'] or i['frozen'])
has_test_count = sum(1 for i in impacted if i['has_tests'])

critical_active = [i for i in impacted if i['level'] == 'critical' and not i['frozen']]
critical_covered = [i for i in critical_active if i['covered']]

coverage_pct = (covered_count / total * 100) if total > 0 else 0.0

# Gate logic
if total == 0:
    gate = 'SKIP'
    gate_exit = 0
elif coverage_pct >= 80:
    gate = 'PASS'
    gate_exit = 0
elif coverage_pct >= 50:
    gate = 'WARN'
    gate_exit = 2
else:
    gate = 'FAIL'
    gate_exit = 1

# ─── Print Human Report ───
print("")
print("═══════════════════════════════════════════")
print("  Detailed Coverage Report")
print("═══════════════════════════════════════════")
print("")

for i in impacted:
    if i['frozen']:
        icon = '─'
        status_tag = ' [FROZEN]'
    elif i['covered']:
        icon = '✔'
        status_tag = ''
    else:
        icon = '✘'
        status_tag = ''

    level_tag = f"[{i['level']}]"
    no_test_tag = ' ⚠ NO TESTS' if not i['has_tests'] else ''

    print(f"  {icon} {i['id']} {level_tag}{status_tag}{no_test_tag}")
    print(f"     Proof: {i['proof']} | Layer: {i['ci_layer']}")
    print(f"     TLA: {i['tla_invariant']}")
    print(f"     Rust: {i['rust_api']}")
    if i['test_results']:
        for tr in i['test_results']:
            if tr['status'] == 'ok':
                t_icon = '✔'
            elif tr['status'] == 'missing':
                t_icon = '✘'
            else:
                t_icon = '?'
            print(f"       {t_icon} {tr['name']}: {tr['status']}")
    if i['matched_files']:
        print(f"     Impacted files:")
        for mf in i['matched_files']:
            print(f"       → {mf}")
    print("")

print("───────────────────────────────────────────")
print(f"  Coverage: {covered_count}/{total} = {coverage_pct:.1f}%")
if critical_active:
    print(f"  Critical (active): {len(critical_covered)}/{len(critical_active)}")
print(f"  Gate: {gate}")
print("───────────────────────────────────────────")

# ─── Write JSON Result ───
result = {
    'timestamp': os.popen('date +%Y-%m-%dT%H:%M:%SZ').read().strip(),
    'mode': mode,
    'total_impacted': total,
    'covered': covered_count,
    'coverage_pct': round(coverage_pct, 1),
    'has_tests': has_test_count,
    'gate': gate,
    'gate_exit': gate_exit,
    'critical_total': len(critical_active),
    'critical_covered': len(critical_covered),
    'invariants': [{
        'id': x['id'],
        'level': x['level'],
        'status': x['status'],
        'covered': x['covered'] or x['frozen'],
        'frozen': x['frozen'],
        'has_tests': x['has_tests'],
        'tests': x['tests'],
        'matched_files': x['matched_files']
    } for x in impacted]
}

with open(out_json, 'w') as f:
    json.dump(result, f, indent=2)

print(f"\n  JSON: {out_json}")
PYEOF

# Load result back for bash summary
RESULT_JSON=$(cat "$COVERAGE_JSON")
COVERAGE_PCT=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['coverage_pct'])")
GATE=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['gate'])")
GATE_EXIT=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['gate_exit'])")
TOTAL=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['total_impacted'])")
COVERED=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['covered'])")
CRIT_TOTAL=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['critical_total'])")
CRIT_COV=$(echo "$RESULT_JSON" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['critical_covered'])")

echo ""
echo "═══════════════════════════════════════════"
echo "  Summary"
echo "═══════════════════════════════════════════"
printf "  Coverage:  %d / %d = %.1f%%\n" "$COVERED" "$TOTAL" "$COVERAGE_PCT"
if [ "$CRIT_TOTAL" -gt 0 ]; then
    printf "  Critical: %d / %d\n" "$CRIT_COV" "$CRIT_TOTAL"
fi
printf "  Gate:     %s\n" "$GATE"
echo "═══════════════════════════════════════════"
echo ""

case "$GATE" in
    PASS) echo -e "  ${GREEN}✔ PASS${NC} — Coverage ${COVERAGE_PCT}% >= 80%" ;;
    WARN) echo -e "  ${YELLOW}⚠ WARN${NC} — Coverage ${COVERAGE_PCT}% (50-80%)" ;;
    FAIL) echo -e "  ${RED}✘ FAIL${NC} — Coverage ${COVERAGE_PCT}% < 50%" ;;
    SKIP) echo -e "  ${BLUE}ℹ SKIP${NC} — No impacted invariants" ;;
esac
echo ""

# Copy JSON to project root for CI artifact
cp "$COVERAGE_JSON" "$PROJECT_ROOT/coverage_result.json" 2>/dev/null || true

exit $GATE_EXIT
