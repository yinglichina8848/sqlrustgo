#!/bin/bash
# scripts/gate/check_release_readiness.sh
# 版本发布前完整性检查脚本

set -e
VERSION=${1:-"v3.1.0"}
PROJECT_ROOT=$(cd "$(dirname "$0")/../.." && pwd)

echo "=========================================="
echo "  SQLRustGo $VERSION Release Readiness Check"
echo "=========================================="
echo ""

PASS=0
FAIL=0

check() {
    local name="$1"
    local cmd="$2"
    echo "--- $name ---"
    if eval "$cmd"; then
        echo "✅ PASS"
        PASS=$((PASS+1))
    else
        echo "❌ FAIL"
        FAIL=$((FAIL+1))
    fi
    echo ""
}

# 1. 流程检查
echo "=== 1. 流程检查 ==="
echo ""

# 检查分支状态
check "所有必需分支存在" \
    "git fetch origin >/dev/null 2>&1 && \
     git ls-remote --heads origin release/v${VERSION} >/dev/null 2>&1 && \
     git ls-remote --heads origin rc/v${VERSION} >/dev/null 2>&1 && \
     git ls-remote --heads origin develop/v${VERSION} >/dev/null 2>&1"

# 检查分支同步
check "分支同步 (无落后)" \
    "git fetch origin >/dev/null 2>&1 && \
     R_REMOTE=\$(git rev-parse origin/rc/v${VERSION}) && \
     R_LOCAL=\$(git rev-parse origin/release/v${VERSION}) && \
     [ \"\$R_REMOTE\" = \"\$R_LOCAL\" ]"

# 2. 合规性检查
echo "=== 2. 合规性检查 ==="
echo ""

check "Governance 文档存在" \
    "[ -f \"$PROJECT_ROOT/docs/governance/GOVERNANCE_INDEX.md\" ]"

check "版本发布规范存在" \
    "[ -f \"$PROJECT_ROOT/docs/governance/RELEASE_LIFECYCLE.md\" ]"

check "Release Process 模板存在" \
    "[ -f \"$PROJECT_ROOT/docs/governance/RELEASE_PROCESS_TEMPLATE.md\" ]"

# 3. 文档完整性检查
echo "=== 3. 文档完整性检查 ==="
echo ""

REQUIRED_DOCS=(
    "DEVELOPMENT_PLAN.md"
    "TEST_PLAN.md"
    "USER_MANUAL.md"
    "API_REFERENCE.md"
    "RELEASE_NOTES.md"
    "CHANGELOG.md"
    "UPGRADE_GUIDE.md"
    "BENCHMARK.md"
    "TEST_REPORT.md"
    "SECURITY_ANALYSIS.md"
    "BETA_GATE_REPORT.md"
    "RC_GATE_REPORT.md"
    "GA_GATE_CHECKLIST.md"
)

for doc in "${REQUIRED_DOCS[@]}"; do
    check "必需文档: $doc" \
        "[ -f \"$PROJECT_ROOT/docs/releases/${VERSION}/${doc}\" ]"
done

# 4. Ignored 测试检查
echo "=== 4. Ignored 测试检查 ==="
echo ""

IGNORE_COUNT=$(grep -r "#\[ignore\]" "$PROJECT_ROOT/crates/" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
check "Ignored 测试数量 (应 < 30)" \
    "[ $IGNORE_COUNT -lt 30 ]"
echo "  当前: $IGNORE_COUNT 个"

# 5. OO 闭环追踪检查
echo "=== 5. OO 闭环追踪检查 ==="
echo ""

check "OO 文档存在" \
    "[ -f \"$PROJECT_ROOT/docs/releases/${VERSION}/OO_DOCUMENT_ANALYSIS.md\" ]"

# 检查 OO 文档更新时间 (应在 7 天内)
OO_DATE=$(stat -f '%Sm' -t '%Y%m%d' "$PROJECT_ROOT/docs/releases/${VERSION}/OO_DOCUMENT_ANALYSIS.md" 2>/dev/null || \
          stat -c '%Y' "$PROJECT_ROOT/docs/releases/${VERSION}/OO_DOCUMENT_ANALYSIS.md" 2>/dev/null)
TODAY=$(date +'%Y%m%d')
DAYS_AGO=$((TODAY - OO_DATE))
check "OO 文档已更新 (7天内)" \
    "[ $DAYS_AGO -lt 7 ]"
echo "  最后更新: $DAYS_AGO 天前"

# 6. 历史遗留问题检查
echo "=== 6. 历史遗留问题检查 ==="
echo ""

# 检查是否有 P0 问题未关闭
ISSUE_COUNT=$(curl -s "http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues?state=open&labels=P0&limit=10" \
    -H "Authorization: token cc19b2ad677e18c96b9d049f6dc2b46e02176883" 2>/dev/null | \
    python3 -c "import json,sys; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

check "无 P0 遗留问题" \
    "[ $ISSUE_COUNT -eq 0 ]"
echo "  P0 Open: $ISSUE_COUNT"

# 7. PR 状态检查
echo "=== 7. PR 状态检查 ==="
echo ""

PR_COUNT=$(curl -s "http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/pulls?state=open&limit=50" \
    -H "Authorization: token cc19b2ad677e18c96b9d049f6dc2b46e02176883" 2>/dev/null | \
    python3 -c "import json,sys; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")

check "无阻塞性 Open PR" \
    "[ $PR_COUNT -eq 0 ]"
echo "  Open PRs: $PR_COUNT"

# 8. 测试覆盖检查
echo "=== 8. 测试覆盖检查 ==="
echo ""

check "Coverage >= 85%" \
    "cd \"$PROJECT_ROOT\" && cargo llvm-cov test -p sqlrustgo-executor 2>/dev/null | grep -q '85\.'"

# 9. 版本标识检查
echo "=== 9. 版本标识检查 ==="
echo ""

check "VERSION 文件正确" \
    "[ \$(cat \"$PROJECT_ROOT/VERSION\") = 'release/v${VERSION}' ]"

check "README 版本更新" \
    "grep -q 'v${VERSION}' \"$PROJECT_ROOT/README.md\""

# 总结
echo "=========================================="
echo "  检查结果汇总"
echo "=========================================="
echo "PASS: $PASS"
echo "FAIL: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✅ 所有检查通过 - 可以发布"
    exit 0
else
    echo "❌ 有 $FAIL 项检查失败 - 需要修复"
    exit 1
fi
