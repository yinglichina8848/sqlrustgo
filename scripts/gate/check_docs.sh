#!/usr/bin/env bash

set -e

echo "=== Running Documentation Gate Check ==="

# 创建文档报告目录
mkdir -p docs/releases/v1.0.0-rc1
mkdir -p docs/releases/v1.0.0/api-doc

# 生成API文档
echo "Building API documentation..."
cargo doc --no-deps --document-private-items

# 检查API文档是否生成
if [ ! -d "target/doc/sqlrustgo" ]; then
    echo "❌ API documentation not generated"
    exit 1
fi

# 复制API文档到发布证据目录
echo "Copying API documentation to release evidence..."
cp -r target/doc/sqlrustgo docs/releases/v1.0.0/api-doc/

# 检查核心文档是否存在
echo "Checking core documentation files..."

DOCS_TO_CHECK=(
    "README.md"
    "CHANGELOG.md"
    "CONTRIBUTING.md"
    "docs/v1.0/rc1/SECURITY_REPORT.md"
    "docs/v1.0/rc1/INSTALL_TEST.md"
    "docs/v1.0/rc1/验收文档/门禁验收/RC1门禁验收清单.md"
)

MISSING_DOCS=()

for doc in "${DOCS_TO_CHECK[@]}"; do
    if [ ! -f "$doc" ]; then
        MISSING_DOCS+=($doc)
    else
        echo "✅ $doc exists"
    fi

done

if [ ${#MISSING_DOCS[@]} -gt 0 ]; then
    echo "❌ Missing documentation files:"
    for doc in "${MISSING_DOCS[@]}"; do
        echo "  - $doc"
    done
    exit 1
fi

# 检查文档链接
echo "Checking documentation links..."

# 检查内部链接（简单检查）
MARKDOWN_FILES=$(find docs -name "*.md" -type f)

BROKEN_LINKS=()

for file in $MARKDOWN_FILES; do
    # 检查相对链接
    LINKS=$(grep -oP '\[.*?\]\(\K[^)]+' "$file" | grep -v '^http')
    
    for link in $LINKS; do
        # 跳过锚点链接
        if [[ $link == *"#"* ]]; then
            continue
        fi
        
        # 检查链接是否存在
        if [ ! -f "$(dirname "$file")/$link" ] && [ ! -d "$(dirname "$file")/$link" ]; then
            BROKEN_LINKS+=("$file: $link")
        fi
    done
done

if [ ${#BROKEN_LINKS[@]} -gt 0 ]; then
    echo "⚠️  Potential broken links found:"
    for link in "${BROKEN_LINKS[@]}"; do
        echo "  - $link"
    done
else
    echo "✅ No broken links found"
fi

# 生成文档检查摘要
echo "Generating documentation check summary..."
cat > docs/releases/v1.0.0-rc1/docs-summary.md << EOF
# Documentation Check Report Summary

## Documentation Status

- **API Documentation**: ✅ Generated
- **Core Documentation**: ✅ All required files exist
- **Link Check**: $(if [ ${#BROKEN_LINKS[@]} -eq 0 ]; then echo "✅ PASS"; else echo "⚠️  WARNING"; fi)

## Report Files

- **API Documentation**: docs/releases/v1.0.0/api-doc/
- **Core Documentation**: Various files in docs/

## Check Details

- **API Doc Command**: cargo doc --no-deps --document-private-items
- **Link Check**: Basic relative link validation
- **Check Date**: $(date)

## Conclusion

API documentation has been generated and all core documentation files are present.
EOF

echo "✅ Documentation summary generated: docs/releases/v1.0.0-rc1/docs-summary.md"
echo "=== Documentation Gate Check Complete ==="
