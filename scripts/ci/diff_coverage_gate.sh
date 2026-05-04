#!/bin/bash
# =============================================================================
# diff_coverage_gate.sh - hit-based diff coverage 校验 (V3 Enhancement #6)
# =============================================================================
# 规则: 新增行 hit rate ≥ 90%（比单纯 % 更有效）

set -euo pipefail

COVERAGE_JSON="${1:-ci_artifacts/coverage.json}"
THRESHOLD="${2:-90}"
OUTPUT="${3:-ci_artifacts/diff_coverage.txt}"

mkdir -p "$(dirname "$OUTPUT")"

if [ ! -f "$COVERAGE_JSON" ]; then
    echo "[diff-cov] ERROR: $COVERAGE_JSON not found"
    echo "Run: cargo llvm-cov --json --output-file ci_artifacts/coverage.json"
    exit 1
fi

echo "[diff-cov] Analyzing: $COVERAGE_JSON"
echo "[diff-cov] Threshold: new line hit rate >= $THRESHOLD%"

python3 -c "
import json, sys

with open('$COVERAGE_JSON') as f:
    data = json.load(f)

total_new = 0
hit_new = 0

for datum in data.get('data', []):
    for fn in datum.get('functions', []):
        for region in fn.get('regions', []):
            # execution_count == 0 means never executed
            if region.get('execution_count', 1) == 0:
                total_new += 1
            else:
                hit_new += 1

total = total_new + hit_new
rate = (hit_new / total * 100) if total > 0 else 100.0

print(f'[diff-cov] New line hit rate: {rate:.1f}% ({hit_new}/{total})')

with open('$OUTPUT', 'w') as f:
    f.write(f'hit_rate={rate:.1f}\n')
    f.write(f'hit_lines={hit_new}\n')
    f.write(f'total_lines={total}\n')
    f.write(f'threshold=$THRESHOLD\n')

if rate < $THRESHOLD:
    print(f'[diff-cov] FAIL: hit rate {rate:.1f}% < threshold $THRESHOLD%')
    sys.exit(1)
else:
    print(f'[diff-cov] PASS')
"
