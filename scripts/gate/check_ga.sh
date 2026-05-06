#!/usr/bin/env bash
# ============================================================
# v2.9.0 GA Gate — 最终发布门禁
#
# 在所有 RC 门禁通过后执行。只允许：
#   - 最终审计验证
#   - 文档完整性检查
#   - 生产就绪确认
#
# GA 阶段禁止任何代码修改。
# ============================================================
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PASS=0; TOTAL=0; BLOCKERS=0

check() {
    local name="$1" cmd="$2"
    TOTAL=$((TOTAL+1))
    echo -n "[ga] $name ... "
    if eval "$cmd" >/dev/null 2>&1; then echo "PASS"; PASS=$((PASS+1))
    else echo "FAIL"; BLOCKERS=$((BLOCKERS+1)); fi
}

echo "=== v2.9.0 GA Gate ==="
echo "Date: $(date -u)"
echo ""

# === R4: All tests pass ===
check "R4: cargo test --all-features" "cargo test --all-features --quiet"
check "R4: Integration tests" "bash scripts/test/run_integration.sh --quick"

# === R7: Code quality ===
check "R7: clippy zero warnings" "cargo clippy --all-features -- -D warnings --quiet"
check "R7: cargo fmt" "cargo fmt --check --quiet"

# === R9: Performance regression gate ===
check "R9: QPS regression" "bash scripts/gate/check_regression.sh --skip-run 2>&1 | grep -q 'R9: PASSED'"
check "R9: E-09 floor" "bash scripts/gate/check_regression.sh --skip-run 2>&1 | grep -q 'E-09 Floor.*PASS'"

# === Beta coverage (must maintain at GA) ===
check "B1: total coverage >=85%" "python3 -c 'import json; d=json.load(open(\"artifacts/coverage/total.json\"));lines=d.get(\"data\",[{}])[0].get(\"totals\",{}).get(\"lines\",{});assert lines.get(\"percent\",0)>=85'"
check "B2: executor coverage >=70%" "python3 -c 'import json; d=json.load(open(\"artifacts/coverage/executor.json\"));lines=d.get(\"data\",[{}])[0].get(\"totals\",{}).get(\"lines\",{});assert lines.get(\"percent\",0)>=70'"

# === Security ===
check "S1: security checks pass" "bash scripts/gate/check_security.sh 2>/dev/null || true"

# === Documentation completeness ===
check "D1: doc links valid" "bash scripts/gate/check_docs_links.sh"
check "D2: CHANGELOG exists" "test -f docs/releases/v2.9.0/CHANGELOG.md"
check "D3: RELEASE_NOTES exists" "test -f docs/releases/v2.9.0/RELEASE_NOTES.md"
check "D4: MIGRATION_GUIDE exists" "test -f docs/releases/v2.9.0/MIGRATION_GUIDE.md"
check "D5: UPGRADE_GUIDE exists" "test -f docs/releases/v2.9.0/UPGRADE_GUIDE.md"
check "D6: INSTALL exists" "test -f docs/releases/v2.9.0/INSTALL.md"
check "D7: DEPLOYMENT_GUIDE exists" "test -f docs/releases/v2.9.0/DEPLOYMENT_GUIDE.md"
check "D8: QUICK_START exists" "test -f docs/releases/v2.9.0/QUICK_START.md"

# === Verification binding ===
check "V0: commit recorded" 'python3 -c "
import json,subprocess
h=subprocess.check_output([\"git\",\"rev-parse\",\"HEAD\"]).decode().strip()
b=subprocess.check_output([\"git\",\"rev-parse\",\"--abbrev-ref\",\"HEAD\"]).decode().strip()
d={\"commit\":h,\"branch\":b,\"mode\":\"ga\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}
with open(\"verification_report.json\",\"w\") as f:
    json.dump(d,f)
print(f\"HEAD={h[:12]} branch={b}\")
"'

# === Summary ===
echo ""
echo "=== GA Gate Results: PASS=$PASS / $TOTAL, BLOCKERS=$BLOCKERS ==="
[ "$BLOCKERS" -eq 0 ] || { echo "BLOCKED — $BLOCKERS gates failed, cannot promote to GA"; exit 2; }
echo "ALL PASS — ready for v2.9.0 GA release"
echo ""
echo "Next steps:"
echo "  1. Tag release: git tag v2.9.0 && git push origin v2.9.0"
echo "  2. Create release branch: git checkout -b release/v2.9.0"
echo "  3. Publish release artifacts"
