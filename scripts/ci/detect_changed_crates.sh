#!/bin/bash
# =============================================================================
# detect_changed_crates.sh - 检测 PR/分支相比目标分支变更的 crate 列表
# =============================================================================
# 用法:
#   detect_changed_crates.sh [target_branch] [base_commit]
#
# 输出:
#   stdout: 变更的 crate 名称列表（每行一个）
#   exit code: 0=成功, 1=无变更, 2=参数错误
#
# 示例:
#   detect_changed_crates.sh develop/v3.2.0
#   detect_changed_crates.sh develop/v3.2.0 HEAD~10

set -euo pipefail

TARGET_BRANCH="${1:-develop/v3.2.0}"
BASE_COMMIT="${2:-}"

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 如果没有指定 base_commit，使用 target_branch 的最新 commit
if [ -z "$BASE_COMMIT" ]; then
    BASE_COMMIT="origin/$TARGET_BRANCH"
fi

echo "[detect] Target: $TARGET_BRANCH, Base: $BASE_COMMIT" >&2

# 获取变更文件列表
CHANGED_FILES=$(git diff --name-only "$BASE_COMMIT" 2>/dev/null || git diff --name-only "$TARGET_BRANCH" 2>/dev/null || echo "")

if [ -z "$CHANGED_FILES" ]; then
    echo "[detect] No changes found between $BASE_COMMIT and $TARGET_BRANCH" >&2
    exit 1
fi

# 从文件路径提取变更的 crate
CHANGED_CRATES=$(echo "$CHANGED_FILES" \
    | grep '^crates/' \
    | sed 's|^crates/||' \
    | cut -d'/' -f1 \
    | sort -u)

if [ -z "$CHANGED_CRATES" ]; then
    echo "[detect] No crate-level changes found" >&2
    exit 1
fi

# 验证每个 crate 存在
VALID_CRATES=""
for crate in $CHANGED_CRATES; do
    if [ -d "$WORKSPACE_ROOT/crates/$crate" ]; then
        VALID_CRATES="$VALID_CRATES $crate"
    fi
done

if [ -z "$VALID_CRATES" ]; then
    echo "[detect] No valid crates found in changes" >&2
    exit 1
fi

echo "[detect] Changed crates:$VALID_CRATES" >&2
echo "$VALID_CRATES" | tr ' ' '\n' | grep -v '^$'
exit 0
