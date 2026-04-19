#!/usr/bin/env bash

# docs_metrics.sh - 文档指标收集脚本
# 每周自动输出文档健康指标

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT"

echo "=== SQLRustGo 文档指标报告 ==="
echo "生成时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# 1. 失效链接数
echo "--- 失效链接检查 ---"
BROKEN_LINKS=0
TMP_FILE="$(mktemp)"
trap "rm -f $TMP_FILE" EXIT

if MD_FILES=($(git ls-files '*.md' 2>/dev/null || echo "")); then
    for file in "${MD_FILES[@]}"; do
        if [ -f "$file" ]; then
            file_dir="$(dirname "$file")"
            while IFS= read -r raw_target; do
                target="$raw_target"
                target="${target#<}"
                target="${target%>}"
                target="${target%%#*}"
                target="${target%%\?*}"

                if [ -z "$target" ]; then
                    continue
                fi

                case "$target" in
                    http://*|https://*|mailto:*|tel:*|ftp://*|data:*|javascript:*)
                        continue
                        ;;
                esac

                resolved=""
                if [[ "$target" = /* ]]; then
                    if [ -e "$target" ]; then
                        continue
                    fi
                    resolved="${REPO_ROOT}${target}"
                else
                    resolved="${file_dir}/${target}"
                fi

                if [ ! -e "$resolved" ]; then
                    printf '%s -> %s\n' "$file" "$raw_target" >> "$TMP_FILE"
                    BROKEN_LINKS=$((BROKEN_LINKS + 1))
                fi
            done < <(perl -ne 'while(/\[[^\]]+\]\(([^)]+)\)/g){print "$1\n"}' "$file" 2>/dev/null)
        fi
    done
fi

if [ "$BROKEN_LINKS" -gt 0 ]; then
    echo "❌ 失效链接数: $BROKEN_LINKS"
    echo ""
    echo "失效链接详情："
    sort -u "$TMP_FILE" | head -20
    if [ "$BROKEN_LINKS" -gt 20 ]; then
        echo "... (更多，请查看完整报告)"
    fi
else
    echo "✅ 失效链接数: 0"
fi
echo ""

# 2. 文档总数统计
echo "--- 文档统计 ---"
TOTAL_MD=$(git ls-files '*.md' 2>/dev/null | wc -l)
TOTAL_DOCS=$(find docs -name '*.md' 2>/dev/null | wc -l)
echo "Git 跟踪的 Markdown 文件: $TOTAL_MD"
echo "docs/ 目录下的 Markdown 文件: $TOTAL_DOCS"
echo ""

# 3. 目录结构统计
echo "--- 目录结构 ---"
echo "docs/ 子目录数量: $(find docs -type d 2>/dev/null | wc -l)"
echo "版本目录数量: $(find docs/releases -maxdepth 1 -type d 2>/dev/null | tail -n +2 | wc -l)"
echo ""

# 4. 重复文件检测（基于哈希）
echo "--- 重复文件检测 ---"
DUPLICATES=$(find docs -type f -name '*.md' -exec md5 {} \; 2>/dev/null | \
    awk '{print $1}' | sort | uniq -d | wc -l)
if [ "$DUPLICATES" -gt 0 ]; then
    echo "⚠️  发现重复文件: $DUPLICATES 个"
else
    echo "✅ 无重复文件"
fi
echo ""

# 5. 未索引文档检测
echo "--- 未索引文档检测 ---"
# 简单检测：检查 docs/ 下的 .md 文件是否在目录的 README.md 中被引用
# 这是一个简化检测
echo "注意: 完整检测需要解析 README.md 中的链接"
echo ""

# 总结
echo "=== 总结 ==="
echo "| 指标 | 状态 |"
echo "|-------|------|"
if [ "$BROKEN_LINKS" -eq 0 ]; then
    echo "| 失效链接 | ✅ 通过 |"
else
    echo "| 失效链接 | ❌ $BROKEN_LINKS |"
fi
if [ "$DUPLICATES" -eq 0 ]; then
    echo "| 重复文件 | ✅ 通过 |"
else
    echo "| 重复文件 | ⚠️  $DUPLICATES |"
fi
echo "| 文档总数 | $TOTAL_DOCS |"

echo ""
echo "=== 建议 ==="
if [ "$BROKEN_LINKS" -gt 0 ]; then
    echo "- 运行 'bash scripts/gate/check_docs_links.sh --all' 查看完整失效链接列表"
fi
if [ "$DUPLICATES" -gt 0 ]; then
    echo "- 检查重复文件，考虑合并或删除"
fi
