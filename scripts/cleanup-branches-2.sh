#!/bin/bash
# SQLRustGo 分支整理脚本 - 第二批
# 将剩余待删除分支移动到 2b-delete/ 前缀

cd /Users/liying/workspace/yinglichina/sqlrustgo

echo "========== 开始第二批分支整理 =========="

# develop-v1.2.0-fixed
echo ">>> 移动 develop-v1.2.0-fixed..."
git branch -m develop-v1.2.0-fixed 2b-delete/develop-v1.2.0-fixed 2>/dev/null && echo "完成" || echo "跳过: 不存在"

# feature/2.0-engineering-setup
echo ">>> 移动 feature/2.0-engineering-setup..."
git branch -m feature/2.0-engineering-setup 2b-delete/feature-2.0-engineering-setup 2>/dev/null && echo "完成" || echo "跳过: 不存在"

echo ""
echo "========== 分支整理完成 =========="
echo "当前分支列表:"
git branch
