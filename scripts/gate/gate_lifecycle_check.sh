#!/usr/bin/env bash
# gate_lifecycle_check.sh — 门禁生命周期追踪健康检查
# 每次门禁 FAIL 后执行，或每周定期执行
set -euo pipefail

REPO="openclaw/sqlrustgo"
GATE_DOC="docs/governance/gate_lifecycle_tracking.md"
DEVPLAN="docs/releases/v3.0.0/DEVELOPMENT_PLAN.md"
EXEMPTIONS="docs/governance/GATE_EXEMPTIONS.md"

echo "=== 门禁追踪健康检查 ==="
echo "时间: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# ========== 检查 1: OPEN Issue 是否都有 milestone ==========
echo "【检查 1】OPEN Issue milestone 覆盖率"
echo "-----------------------------------"
OPEN_ISSUES=$(gh issue list --state open --repo "$REPO" --json number,title,milestone,labels \
  --jq '.[] | @json' 2>/dev/null || echo "[]")

if [ -z "$OPEN_ISSUES" ] || [ "$OPEN_ISSUES" = "[]" ]; then
    echo "  ⚠️  无法获取 Issue 列表（可能网络问题）"
else
    TOTAL_OPEN=$(echo "$OPEN_ISSUES" | jq 'length')
    WITH_MILESTONE=$(echo "$OPEN_ISSUES" | jq '[.[] | select(.milestone != null)] | length')
    WITHOUT_MILESTONE=$(echo "$OPEN_ISSUES" | jq '[.[] | select(.milestone == null)] | length')

    echo "  总 OPEN Issue: $TOTAL_OPEN"
    echo "  有 milestone: $WITH_MILESTONE"
    echo "  无 milestone: $WITHOUT_MILESTONE"

    if [ "$WITHOUT_MILESTONE" -gt 0 ]; then
        echo ""
        echo "  ⚠️  以下 Issue 缺少 milestone:"
        echo "$OPEN_ISSUES" | jq -r '.[] | select(.milestone == null) | "  - #\(.number): \(.title)"'
    else
        echo "  ✅ 所有 OPEN Issue 均有 milestone"
    fi
fi
echo ""

# ========== 检查 2: source/gate-* Issue 是否关联到门禁文档 ==========
echo "【检查 2】source/gate-* Issue 追踪状态"
echo "--------------------------------------------"
GATE_ISSUES=$(gh issue list --state open --repo "$REPO" --label "source/gate-beta,source/gate-rc,source/gate-ga" \
  --json number,title,labels --jq '.[] | @json' 2>/dev/null || echo "[]")

if [ -z "$GATE_ISSUES" ] || [ "$GATE_ISSUES" = "[]" ]; then
    echo "  ℹ️  无 OPEN 的 source/gate-* Issue"
else
    GATE_COUNT=$(echo "$GATE_ISSUES" | jq 'length')
    echo "  OPEN source/gate-* Issue: $GATE_COUNT 个"
    echo "$GATE_ISSUES" | jq -r '.[] | "  - #\(.number): \(.title)"'
fi
echo ""

# ========== 检查 3: Issue 是否有对应 PR ==========
echo "【检查 3】OPEN Issue 中有 PR 关联但未关闭的"
echo "----------------------------------------------------"
ISSUES_WITH_PR=$(gh issue list --state open --repo "$REPO" \
  --json number,title,closedByPullRequestsReferences \
  --jq '[.[] | select(.closedByPullRequestsReferences | length > 0)] | .[] | @json' 2>/dev/null || echo "[]")

if [ -z "$ISSUES_WITH_PR" ] || [ "$ISSUES_WITH_PR" = "[]" ]; then
    echo "  ℹ️  无 OPEN Issue 有关联 PR"
else
    echo "  ⚠️  以下 Issue 已有 PR 但未关闭（可能 PR 被关闭或未合并）:"
    echo "$ISSUES_WITH_PR" | jq -r '. | "  - #\(.number): \(.title)"'
fi
echo ""

# ========== 检查 4: GATE_EXEMPTIONS 豁免过期 ==========
echo "【检查 4】GATE_EXEMPTIONS.md 豁免复审日期"
echo "--------------------------------------------"
if [ -f "$EXEMPTIONS" ]; then
    TODAY=$(date -u +%Y-%m-%d)
    echo "  今天: $TODAY"
    # 提取所有复审日期并检查是否过期
    EXPIRED=$(grep -E "\| v[0-9]" "$EXEMPTIONS" | grep -E "[0-9]{4}-[0-9]{2}-[0-9]{2}" | \
      awk -F'|' '{gsub(/ /,"",$10); if($10 ~ /[0-9]{4}/ && $10 < "'"$TODAY"'") print $2, $10}')
    if [ -n "$EXPIRED" ]; then
        echo "  ⚠️  以下豁免已过期但未更新:"
        echo "$EXPIRED" | while read id date; do
            echo "  - $id (复审日期: $date)"
        done
    else
        echo "  ✅ 所有豁免在有效期内"
    fi
else
    echo "  ⚠️  GATE_EXEMPTIONS.md 不存在"
fi
echo ""

# ========== 检查 5: DEVELOPMENT_PLAN.md §6 是否与实际 Issue 对应 ==========
echo "【检查 5】延续任务映射完整性"
echo "-----------------------------------"
if [ -f "$DEVPLAN" ]; then
    CARRIED_ISSUES=$(grep -E "^#|## " "$DEVPLAN" | grep -c "延续任务" || echo "0")
    echo "  DEVELOPMENT_PLAN.md §6 延续任务章节: $CARRIED_ISSUES 个"
    # 检查 §6 中引用的 Issue 是否在 Gitea 存在且为 OPEN
    echo "  ℹ️  请手动验证: grep -E '#[0-9]+' docs/releases/v3.0.0/DEVELOPMENT_PLAN.md"
else
    echo "  ⚠️  DEVELOPMENT_PLAN.md 不存在"
fi
echo ""

# ========== 检查 6: gate_lifecycle_tracking.md §7 登记完整性 ==========
echo "【检查 6】gate_lifecycle_tracking.md §7 登记"
echo "------------------------------------------------"
if [ -f "$GATE_DOC" ]; then
    TRACKED_COUNT=$(grep -c "GA-GAP\|#451\|#379\|#380\|#381\|#392" "$GATE_DOC" || echo "0")
    echo "  gate_lifecycle_tracking.md §7 登记的 Issue 引用: $TRACKED_COUNT 处"
    echo "  ℹ️  请验证 §7 内容与实际追踪状态一致"
else
    echo "  ⚠️  gate_lifecycle_tracking.md 不存在"
fi
echo ""

# ========== 检查 7: 文档与脚本同步 ==========
echo "【检查 7】门禁规范与脚本一致性（Beta/RC/GA）"
echo "------------------------------------------------"
SCRIPT_DIR="$(dirname "$0")"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# 检查规范中定义的 G1-G15 是否在 check_ga_v300.sh 中
if [ -f "$PROJECT_ROOT/scripts/gate/check_ga_v300.sh" ]; then
    GA_SPEC_ITEMS=$(grep -oE '\bG[0-9]+\b' "$PROJECT_ROOT/docs/governance/gate_spec_v300.md" | sort -u)
    GA_SCRIPT_ITEMS=$(grep -oE '\bG[0-9]+\b' "$PROJECT_ROOT/scripts/gate/check_ga_v300.sh" | sort -u)
    echo "  GA 规范项: $(echo "$GA_SPEC_ITEMS" | wc -l) 个"
    echo "  GA 脚本项: $(echo "$GA_SCRIPT_ITEMS" | wc -l) 个"
fi
echo ""

# ========== 总结 ==========
echo "=== 检查完成 ==="
echo "建议: 对上述 ⚠️ 项进行修复或登记"
