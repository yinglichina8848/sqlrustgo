#!/bin/bash
# =============================================================================
# perf_regression_gate.sh - Nightly 性能回归检测 (V3 Enhancement #7)
# =============================================================================
# 规则: if perf_delta > 5% → FAIL
# 比较昨天 vs 今天 benchmark 数据

set -euo pipefail

RESULTS_DIR="${1:-ci_artifacts/bench}"
BASELINE_DIR="${2:-$HOME/.sqlrustgo/bench-baseline}"
THRESHOLD="${3:-5}"  # percent
OUTPUT="${4:-ci_artifacts/perf_check.txt}"

mkdir -p "$BASELINE_DIR" "$(dirname "$OUTPUT")"

save_baseline() {
    if [ -f "$RESULTS_DIR/latest.json" ]; then
        cp "$RESULTS_DIR/latest.json" "$BASELINE_DIR/latest.json"
        echo "[perf] Baseline saved"
    else
        echo "[perf] ERROR: $RESULTS_DIR/latest.json not found"
        exit 1
    fi
}

compare() {
    local current="$RESULTS_DIR/latest.json"
    local baseline="$BASELINE_DIR/latest.json"

    if [ ! -f "$current" ]; then
        echo "[perf] ERROR: $current not found"
        exit 1
    fi

    if [ ! -f "$baseline" ]; then
        echo "[perf] No baseline - saving current as baseline"
        save_baseline
        echo "0.0" > "$OUTPUT"
        exit 0
    fi

    echo "[perf] Comparing: baseline vs current"
    echo "[perf] Threshold: ${THRESHOLD}% regression -> FAIL"

    python3 -c "
import json, sys

with open('$baseline') as f:
    base = json.load(f)
with open('$current') as f:
    cur = json.load(f)

regressions = []
queries = set(base.keys()) & set(cur.keys())

for q in sorted(queries):
    b = base[q]
    c = cur[q]

    # Handle qps (higher is better) or duration_ms (lower is better)
    if isinstance(b, dict):
        if 'qps' in b:
            b_val, c_val = b['qps'], c['qps']
            higher_better = True
        elif 'duration_ms' in b:
            b_val, c_val = b['duration_ms'], c['duration_ms']
            higher_better = False
        else:
            continue
    else:
        continue

    if b_val == 0:
        continue

    delta = ((c_val - b_val) / b_val) * 100

    if higher_better:
        if delta < -$THRESHOLD:
            regressions.append((q, delta, b_val, c_val))
    else:
        if delta > $THRESHOLD:
            regressions.append((q, delta, b_val, c_val))

    print(f'{q}: {b_val:.2f} -> {c_val:.2f} ({delta:+.1f}%)')

if regressions:
    print()
    print('[perf] REGRESSIONS DETECTED:')
    for q, delta, b, c in regressions:
        print(f'  {q}: {b:.2f} -> {c:.2f} ({delta:+.1f}%)')
    sys.exit(1)
else:
    print()
    print('[perf] No significant regressions')
    sys.exit(0)
"
}

case "${1:-compare}" in
    compare|check) compare ;;
    save-baseline) save_baseline ;;
    *)
        echo "Usage: $0 [compare|save-baseline] [results_dir] [baseline_dir]"
        exit 1 ;;
esac
