#!/bin/bash
# SQLRustGo 远程分支重命名脚本
# 将远程分支从 develop-v1.X.X 重命名为 develop/v1.X.X

cd /Users/liying/workspace/yinglichina/sqlrustgo

echo "========== 开始远程分支重命名 =========="

# 重命名开发分支
echo ">>> 重命名开发分支..."

# develop-v1.1.0 -> develop/v1.1.0
git push origin origin/develop-v1.1.0:refs/heads/develop/v1.1.0 2>&1 && git push origin --delete develop-v1.1.0 2>&1 || echo "跳过或失败"

# develop-v1.2.0 -> develop/v1.2.0
git push origin origin/develop-v1.2.0:refs/heads/develop/v1.2.0 2>&1 && git push origin --delete develop-v1.2.0 2>&1 || echo "跳过或失败"

# develop-v1.3.0 -> develop/v1.3.0
git push origin origin/develop-v1.3.0:refs/heads/develop/v1.3.0 2>&1 && git push origin --delete develop-v1.3.0 2>&1 || echo "跳过或失败"

echo "========== 重命名完成 =========="
echo ""
echo "当前远程分支列表:"
git ls-remote --heads origin
