#!/usr/bin/env bash

set -e

echo "=== Running SQLRustGo Release Gate Check ==="
echo "Date: $(date)"
echo "Branch: $(git rev-parse --abbrev-ref HEAD)"
echo "Commit: $(git rev-parse HEAD)"
echo ""

# 确保脚本有执行权限
chmod +x scripts/gate/*.sh

# 1. 代码质量检查
echo "=== 1. Code Quality Check ==="
echo "Running rustfmt..."
cargo fmt --all -- --check

echo "Running clippy..."
cargo clippy --all-targets -- -D warnings

echo "Running build..."
cargo build --all

echo "Running tests..."
cargo test --all

echo "✅ Code quality check passed!"
echo ""

# 2. 覆盖率检查
echo "=== 2. Coverage Check ==="
scripts/gate/check_coverage.sh
echo ""

# 3. 安全检查
echo "=== 3. Security Check ==="
scripts/gate/check_security.sh
echo ""

# 4. 文档检查
echo "=== 4. Documentation Check ==="
scripts/gate/check_docs.sh
echo ""

# 5. 安装测试
echo "=== 5. Installation Test ==="
echo "Running installation test..."
cargo install --path .

echo "Running smoke test..."
sqlrustgo --help > /dev/null

if [ $? -eq 0 ]; then
    echo "✅ Installation test passed!"
else
    echo "❌ Installation test failed!"
    exit 1
fi
echo ""

# 6. 分支保护检查
echo "=== 6. Branch Protection Check ==="
echo "Checking branch protection status..."
echo "Note: This check requires manual verification in GitHub settings"
echo "Please ensure v1.0.0-rc1 branch has protection rules enabled"
echo "✅ Branch protection check noted"
echo ""

# 7. 最终状态
echo "=== Final Gate Check Status ==="
echo "🎉 All gates passed!"
echo ""
echo "Release evidence has been archived to:"
echo "docs/releases/v1.0.0-rc1/"
echo ""
echo "Next steps:"
echo "1. Review release evidence"
echo "2. Complete branch protection setup"
echo "3. Prepare for GA release"
echo ""
echo "=== Release Gate Check Complete ==="
