#!/bin/bash
# SQLRustGo 分支整理脚本
# 将待删除分支移动到 2b-delete/ 前缀

cd /Users/liying/workspace/yinglichina/sqlrustgo

echo "========== 开始分支整理 =========="

# P0 临时测试分支
echo ">>> 移动 P0 临时测试分支..."
git branch -m temp-main-reset 2b-delete/temp-main-reset 2>/dev/null || echo "跳过: temp-main-reset 不存在"
git branch -m pr-148-test 2b-delete/pr-148-test 2>/dev/null || echo "跳过: pr-148-test 不存在"
git branch -m "pr/147" "2b-delete/pr-147" 2>/dev/null || echo "跳过: pr/147 不存在"
git branch -m fix/v1.2.0-index-test 2b-delete/fix-v1.2.0-index-test 2>/dev/null || echo "跳过: fix/v1.2.0-index-test 不存在"
git branch -m fix/v1.2.0-page-test 2b-delete/fix-v1.2.0-page-test 2>/dev/null || echo "跳过: fix/v1.2.0-page-test 不存在"

# P0 Release 临时分支
echo ">>> 移动 P0 Release 临时分支..."
git branch -m release/v1.1.0-to-main-v2 2b-delete/release-v1.1.0-to-main-v2 2>/dev/null || echo "跳过"
git branch -m release/v1.1.0-to-main-v3 2b-delete/release-v1.1.0-to-main-v3 2>/dev/null || echo "跳过"
git branch -m release/v1.1.0-to-main-v4 2b-delete/release-v1.1.0-to-main-v4 2>/dev/null || echo "跳过"
git branch -m release/v1.1.0-final 2b-delete/release-v1.1.0-final 2>/dev/null || echo "跳过"
git branch -m release/merge-v1.1.0-to-main 2b-delete/release-merge-v1.1.0-to-main 2>/dev/null || echo "跳过"

# P0 错误命名阶段分支
echo ">>> 移动 P0 错误命名阶段分支..."
git branch -m release/v1.1.0-alpha 2b-delete/release-v1.1.0-alpha 2>/dev/null || echo "跳过"
git branch -m release/v1.1.0-beta 2b-delete/release-v1.1.0-beta 2>/dev/null || echo "跳过"
git branch -m release/v1.1.0-rc 2b-delete/release-v1.1.0-rc 2>/dev/null || echo "跳过"
git branch -m rc/v1.0.0-1 2b-delete/rc-v1.0.0-1 2>/dev/null || echo "跳过"
git branch -m feature/v1.0.0-alpha 2b-delete/feature-v1.0.0-alpha 2>/dev/null || echo "跳过"
git branch -m feature/v1.0.0-beta 2b-delete/feature-v1.0.0-beta 2>/dev/null || echo "跳过"
git branch -m feature/v1.0.0-evaluation 2b-delete/feature-v1.0.0-evaluation 2>/dev/null || echo "跳过"

# P1 已合并的 fix/docs 分支
echo ">>> 移动 P1 已合并的 fix/docs 分支..."
git branch -m fix/v1.2.0-format-fix 2b-delete/fix-v1.2.0-format-fix 2>/dev/null || echo "跳过"
git branch -m fix/v1.2.0-directory-restructuring 2b-delete/fix-v1.2.0-directory-restructuring 2>/dev/null || echo "跳过"
git branch -m fix/v1.2.0-cargo-toml-fix 2b-delete/fix-v1.2.0-cargo-toml-fix 2>/dev/null || echo "跳过"
git branch -m fix/v1.2.0-clippy-warnings-merged 2b-delete/fix-v1.2.0-clippy-warnings-merged 2>/dev/null || echo "跳过"
git branch -m docs/v1.2.0-governance-report 2b-delete/docs-v1.2.0-governance-report 2>/dev/null || echo "跳过"
git branch -m docs/v1.2.0-consistency-fix 2b-delete/docs-v1.2.0-consistency-fix 2>/dev/null || echo "跳过"
git branch -m docs/v1.3.0-development-plan-discussion 2b-delete/docs-v1.3.0-development-plan-discussion 2>/dev/null || echo "跳过"

# 归档分支
echo ">>> 移动归档分支..."
git branch -m develop archive/develop-legacy 2>/dev/null || echo "跳过: develop 不存在"
git branch -m baseline archive/baseline-legacy 2>/dev/null || echo "跳过: baseline 不存在"

echo ""
echo "========== 分支整理完成 =========="
echo "当前分支列表:"
git branch
