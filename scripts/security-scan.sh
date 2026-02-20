#!/bin/bash

# 安全扫描脚本
# 根据 Issue 19 审核意见的建议

echo "🔒 开始安全扫描..."
echo "="

# 检查 cargo-audit 是否已安装
if ! command -v cargo-audit &> /dev/null; then
    echo "⚠️  cargo-audit 未安装，正在安装..."
    cargo install cargo-audit
    if [ $? -ne 0 ]; then
        echo "❌ 安装 cargo-audit 失败"
        exit 1
    fi
fi

# 运行安全扫描
echo "🔍 运行 cargo audit 安全扫描..."
echo "-"
cargo audit

if [ $? -eq 0 ]; then
    echo "✅ 安全扫描通过，未发现安全漏洞"
else
    echo "❌ 安全扫描发现问题，请查看输出"
    exit 1
fi

echo "="
echo "🔒 安全扫描完成"
