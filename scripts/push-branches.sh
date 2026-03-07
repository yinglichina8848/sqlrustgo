#!/bin/bash
# SQLRustGo 分支推送脚本
# 推送到远程仓库

cd /Users/liying/workspace/yinglichina/sqlrustgo

echo "========== 开始推送分支 =========="

# 推送到 origin (GitHub)
echo ">>> 推送到 GitHub..."

# 推送所有分支
git push origin main develop/v1.1.0 develop/v1.2.0 develop/v1.3.0 draft/v1.1.0 draft/v1.2.0 alpha/v1.1.0 beta/v1.1.0 rc/v1.1.0 release/v1.0.0 release/v1.1.0 archive/develop-legacy archive/baseline-legacy 2>&1

# 推送 2b-delete 分支（作为待删除标记）
git push origin 2b-delete/temp-main-reset 2b-delete/fix-v1.2.0-index-test 2b-delete/fix-v1.2.0-page-test 2b-delete/develop-v1.2.0-fixed 2b-delete/feature-2.0-engineering-setup 2b-delete/fix-v1.2.0-clippy 2b-delete/fix-v1.2.0-index-v2 2b-delete/docs-v1.2.0-design-docs 2b-delete/docs-whitepaper-update 2>&1

echo ""
echo "========== 推送到 Gitee (镜像) =========="
# 推送到 gitee
git push gitee main develop/v1.1.0 develop/v1.2.0 develop/v1.3.0 draft/v1.1.0 draft/v1.2.0 alpha/v1.1.0 beta/v1.1.0 rc/v1.1.0 release/v1.0.0 release/v1.1.0 archive/develop-legacy archive/baseline-legacy 2>&1

echo ""
echo "========== 推送完成 =========="
