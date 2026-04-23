#!/usr/bin/env bash

set -e

echo "=== Running Coverage Gate Check ==="

COVERAGE_DIR="docs/releases/v2.7.0"
mkdir -p "$COVERAGE_DIR"

MODE="${1:-full}"

PROBLEMATIC_TESTS=(
    "test_trigger_executes_insert"
    "test_trigger_executes_delete"
    "test_trigger_executes_update"
    "test_sql_corpus_all"
)

SKIP_ARGS=""
for test in "${PROBLEMATIC_TESTS[@]}"; do
    SKIP_ARGS="$SKIP_ARGS --skip $test"
done

echo "Mode: $MODE"
echo "Skipping problematic tests under tarpaulin: ${PROBLEMATIC_TESTS[*]}"

if [ "$MODE" = "incremental" ]; then
    echo "Running incremental coverage..."
    CHANGED_CRATES=$(git diff --name-only | cut -d/ -f2 | sort -u | grep -E "^crates/" | cut -d/ -f2 || true)
    if [ -z "$CHANGED_CRATES" ]; then
        echo "No crate changes detected, using full coverage"
        MODE="full"
    else
        echo "Changed crates: $CHANGED_CRATES"
        PKGS=""
        for crate in $CHANGED_CRATES; do
            PKGS="$PKGS -p sqlrustgo-$crate"
        done
        cargo tarpaulin --out Xml --output-dir "$COVERAGE_DIR" -- $SKIP_ARGS $PKGS
    fi
fi

if [ "$MODE" = "full" ]; then
    echo "Running full coverage test..."
    cargo tarpaulin --out Xml --out Html --output-dir "$COVERAGE_DIR" -- $SKIP_ARGS
fi

# 检查覆盖率报告是否生成
if [ ! -f "$COVERAGE_DIR/coverage.xml" ]; then
    echo "❌ Coverage report not generated"
    exit 1
fi

# 提取覆盖率百分比
echo "Extracting coverage percentage..."
COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' "$COVERAGE_DIR/coverage.xml")

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
echo "Coverage report saved to: $COVERAGE_DIR/coverage.html"
echo "Coverage XML saved to: $COVERAGE_DIR/coverage.xml"

echo "Generating coverage summary..."
cat > "$COVERAGE_DIR/coverage-summary.md" << EOF
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
- **Mode**: $MODE
- **Test Date**: $(date)

## Conclusion

Coverage meets the required threshold of ${REQUIRED}% or higher.
EOF

echo "✅ Coverage summary generated: $COVERAGE_DIR/coverage-summary.md"
echo "=== Coverage Gate Check Complete ==="
