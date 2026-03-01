#!/usr/bin/env bash

set -e

echo "=== Running Coverage Gate Check ==="

# 创建覆盖率报告目录
mkdir -p docs/releases/v1.0.0-rc1

# 运行覆盖率测试
echo "Running coverage test..."
cargo tarpaulin --out Xml --out Html --output-dir docs/releases/v1.0.0-rc1

# 检查覆盖率报告是否生成
if [ ! -f "docs/releases/v1.0.0-rc1/coverage.xml" ]; then
    echo "❌ Coverage report not generated"
    exit 1
fi

# 提取覆盖率百分比
echo "Extracting coverage percentage..."
COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' docs/releases/v1.0.0-rc1/coverage.xml)

if [ -z "$COVERAGE" ]; then
    echo "❌ Failed to extract coverage percentage"
    exit 1
fi

# 转换为整数百分比
COVERAGE_INT=$(echo "$COVERAGE * 100" | bc | cut -d. -f1)
REQUIRED=80

echo "Current coverage: ${COVERAGE_INT}%"
echo "Required coverage: ${REQUIRED}%"

if [ "$COVERAGE_INT" -lt "$REQUIRED" ]; then
    echo "❌ Coverage too low! Need at least ${REQUIRED}%"
    exit 1
fi

echo "✅ Coverage check passed!"
echo "Coverage report saved to: docs/releases/v1.0.0-rc1/coverage.html"
echo "Coverage XML saved to: docs/releases/v1.0.0-rc1/coverage.xml"

# 生成覆盖率摘要
echo "Generating coverage summary..."
cat > docs/releases/v1.0.0-rc1/coverage-summary.md << EOF
# Coverage Report Summary

## Coverage Statistics

- **Total Coverage**: ${COVERAGE_INT}%
- **Required Coverage**: ${REQUIRED}%
- **Status**: ✅ PASS

## Report Files

- **HTML Report**: coverage.html
- **XML Report**: coverage.xml

## Test Details

- **Test Command**: cargo tarpaulin --out Xml --out Html
- **Test Date**: $(date)

## Conclusion

Coverage meets the required threshold of ${REQUIRED}% or higher.
EOF

echo "✅ Coverage summary generated: docs/releases/v1.0.0-rc1/coverage-summary.md"
echo "=== Coverage Gate Check Complete ==="
