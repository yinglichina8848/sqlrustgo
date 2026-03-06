#!/bin/bash
# SQLRustGo 分支整理脚本 - 第三批
# 修正分支命名

cd /Users/liying/workspace/yinglichina/sqlrustgo

echo "========== 开始分支命名修正 =========="

# 1. 将 2b-delete 中的阶段分支恢复到正确位置
echo ">>> 恢复阶段分支到正确位置..."
git branch -m 2b-delete/release-v1.1.0-alpha alpha/v1.1.0 2>/dev/null && echo "alpha/v1.1.0 创建完成" || echo "跳过"
git branch -m 2b-delete/release-v1.1.0-beta beta/v1.1.0 2>/dev/null && echo "beta/v1.1.0 创建完成" || echo "跳过"
git branch -m 2b-delete/release-v1.1.0-rc rc/v1.1.0 2>/dev/null && echo "rc/v1.1.0 创建完成" || echo "跳过"

# 2. 将 develop-vX.X.X 重命名为 develop/vX.X.X
echo ">>> 重命名 develop 分支..."
git branch -m develop-v1.1.0 develop/v1.1.0 2>/dev/null && echo "develop/v1.1.0 创建完成" || echo "跳过"
git branch -m develop-v1.2.0 develop/v1.2.0 2>/dev/null && echo "develop/v1.2.0 创建完成" || echo "跳过"
git branch -m develop-v1.3.0 develop/v1.3.0 2>/dev/null && echo "develop/v1.3.0 创建完成" || echo "跳过"

# 3. 检查是否还有其他 develop-v 分支需要处理
echo ""
echo "========== 当前分支列表 =========="
git branch
