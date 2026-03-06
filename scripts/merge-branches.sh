#!/bin/bash
# SQLRustGo 分支合并脚本
# 合并 docs 分支和 fix 分支到 develop/v1.2.0

cd /Users/liying/workspace/yinglichina/sqlrustgo

echo "========== 开始分支合并 =========="

# 切换到 develop/v1.2.0
echo ">>> 切换到 develop/v1.2.0..."
git checkout develop/v1.2.0

# 合并 docs/whitepaper-update
echo ">>> 合并 docs/whitepaper-update..."
git merge docs/whitepaper-update --no-edit 2>&1 || echo "合并冲突或失败"

# 合并 docs/v1.2.0-design-docs
echo ">>> 合并 docs/v1.2.0-design-docs..."
git merge docs/v1.2.0-design-docs --no-edit 2>&1 || echo "合并冲突或失败"

# 合并 fix/v1.2.0-index-v2
echo ">>> 合并 fix/v1.2.0-index-v2..."
git merge fix/v1.2.0-index-v2 --no-edit 2>&1 || echo "合并冲突或失败"

echo ""
echo "========== 合并完成 =========="
echo "当前状态:"
git status

echo ""
echo "========== 移动分支到 2b-delete/ =========="

# 移动已合并的分支到 2b-delete
git branch -m docs/whitepaper-update 2b-delete/docs-whitepaper-update 2>/dev/null && echo "移动: docs/whitepaper-update" || echo "跳过"
git branch -m docs/v1.2.0-design-docs 2b-delete/docs-v1.2.0-design-docs 2>/dev/null && echo "移动: docs/v1.2.0-design-docs" || echo "跳过"
git branch -m fix/v1.2.0-index-v2 2b-delete/fix-v1.2.0-index-v2 2>/dev/null && echo "移动: fix/v1.2.0-index-v2" || echo "跳过"
git branch -m fix/v1.2.0-clippy 2b-delete/fix-v1.2.0-clippy 2>/dev/null && echo "移动: fix/v1.2.0-clippy" || echo "跳过"

echo ""
echo "========== 最终分支列表 =========="
git branch
