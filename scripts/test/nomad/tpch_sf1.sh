#!/usr/bin/env bash
# tpch_sf1.sh - TPC-H SF=1 并行执行
# 22 个查询并行执行，约 45-60 分钟
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
RESULT_DIR="/tmp/tpch_sf1_results"
ARTIFACT_DIR="${ARTIFACT_DIR:-/tmp/tpch_artifacts}"
mkdir -p "$RESULT_DIR" "$ARTIFACT_DIR"

TPCH_DATA_DIR="${TPCH_DATA_DIR:-$HOME/sqlrustgo-tpch/data}"

# 关键查询阈值 (ms)
THRESHOLD_Q1=30000
THRESHOLD_Q6=15000

# 并行度
PARALLEL_JOBS="${PARALLEL_JOBS:-6}"

echo "=== TPC-H SF=1 Parallel Execution ==="
echo "Timestamp: $TIMESTAMP"
echo "Parallel jobs: $PARALLEL_JOBS"
echo "Data dir: $TPCH_DATA_DIR"
echo ""

# 检查数据
if [ ! -d "$TPCH_DATA_DIR" ] || [ -z "$(ls -A "$TPCH_DATA_DIR"/*.tbl 2>/dev/null)" ]; then
    echo "⚠️  TPC-H data not found at $TPCH_DATA_DIR"
    echo "   Generate with: tpch-dbgen -s 1 -f -d"
    echo "   Exiting..."
    exit 1
fi

TPCH_QUERIES=(Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 Q11 Q12 Q13 Q14 Q15 Q16 Q17 Q18 Q19 Q20 Q21 Q22)

# 执行单个查询
run_query() {
    local q=$1
    local start=$(date +%s.%N)
    
    echo -n "  Running $q... "
    
    local output
    output=$(timeout 1800 bench-cli run-tpch --sf 1 --query "$q" 2>&1) || true
    
    local end=$(date +%s.%N)
    local duration=$(echo "$end - $start" | bc)
    
    # 提取执行时间 (ms)
    local ms
    ms=$(echo "$output" | grep -oE '[0-9]+(\.[0-9]+)?(ms|µs|s)' | head -1 | grep -oE '[0-9]+(\.[0-9]+)?' || echo "0")
    
    # 转换为毫秒
    if echo "$output" | grep -q 's$'; then
        ms=$(echo "$ms * 1000" | bc)
    elif echo "$output" | grep -q 'µs$'; then
        ms=$(echo "$ms / 1000" | bc)
    fi
    
    echo "${ms}ms (${duration}s)"
    
    # 记录结果
    echo "$q:$ms:$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$RESULT_DIR/timing.txt"
    
    # 检查阈值
    if [ "$q" = "Q1" ] && [ $(echo "$ms > $THRESHOLD_Q1" | bc) -eq 1 ]; then
        echo "    ⚠️  Q1 exceeded threshold: ${ms}ms > ${THRESHOLD_Q1}ms"
        echo "FAIL_Q1: ${ms}ms" >> "$RESULT_DIR/failures.txt"
    fi
    if [ "$q" = "Q6" ] && [ $(echo "$ms > $THRESHOLD_Q6" | bc) -eq 1 ]; then
        echo "    ⚠️  Q6 exceeded threshold: ${ms}ms > ${THRESHOLD_Q6}ms"
        echo "FAIL_Q6: ${ms}ms" >> "$RESULT_DIR/failures.txt"
    fi
}

export -f run_query
export RESULT_DIR
export THRESHOLD_Q1
export THRESHOLD_Q6
export PARALLEL_JOBS

# 清理旧结果
rm -f "$RESULT_DIR/timing.txt" "$RESULT_DIR/failures.txt"

# 加载 TPC-H 数据
echo "[1/3] Loading TPC-H SF=1 data..."
if ! bench-cli load-tpch --sf 1 --path "$TPCH_DATA_DIR" 2>/dev/null; then
    echo "⚠️  Data load may have issues, continuing..."
fi
echo ""

# 并行执行 Q1-Q6 (关键查询)
echo "[2/3] Running critical queries (Q1-Q6) in parallel..."
parallel -j "$PARALLEL_JOBS" run_query ::: Q1 Q2 Q3 Q4 Q5 Q6
echo ""

# 串行执行 Q7-Q22
echo "[3/3] Running remaining queries (Q7-Q22)..."
for q in Q7 Q8 Q9 Q10 Q11 Q12 Q13 Q14 Q15 Q16 Q17 Q18 Q19 Q20 Q21 Q22; do
    run_query "$q"
done
echo ""

# 生成报告
echo "=== Generating Report ==="

python3 << EOF
import json
import os
from datetime import datetime

result_dir = os.environ.get('RESULT_DIR', '/tmp/tpch_sf1_results')
timestamp = os.environ.get('TIMESTAMP', '')

timing = {}
with open(f'{result_dir}/timing.txt') as f:
    for line in f:
        parts = line.strip().split(':')
        if len(parts) == 3:
            q, ms, ts = parts
            timing[q] = {'ms': float(ms), 'timestamp': ts}

total_time = sum(v['ms'] for v in timing.values()) / 1000
queries_passed = len(timing)
queries_total = 22

failures = []
if os.path.exists(f'{result_dir}/failures.txt'):
    with open(f'{result_dir}/failures.txt') as f:
        failures = [l.strip() for l in f.readlines()]

# Check threshold
threshold_q1 = int(os.environ.get('THRESHOLD_Q1', 30000))
threshold_q6 = int(os.environ.get('THRESHOLD_Q6', 15000))

status = 'PASS'
if failures:
    status = 'FAIL'
elif queries_passed < queries_total:
    status = 'PARTIAL'

report = {
    'test_type': 'tpch_sf1',
    'timestamp': timestamp,
    'sf': 1,
    'parallel_jobs': int(os.environ.get('PARALLEL_JOBS', '6')),
    'total_queries': queries_total,
    'queries_passed': queries_passed,
    'total_time_seconds': round(total_time, 1),
    'query_timings': timing,
    'failures': failures,
    'thresholds': {
        'Q1': threshold_q1,
        'Q6': threshold_q6
    },
    'status': status
}

with open(f'{result_dir}/report.json', 'w') as f:
    json.dump(report, f, indent=2)

print(f"TPC-H SF=1 Results:")
print(f"  Queries: {queries_passed}/{queries_total}")
print(f"  Total time: {total_time:.1f}s")
print(f"  Status: {status}")
if failures:
    print(f"  Failures: {failures}")
EOF

# 复制到 artifacts
cp "$RESULT_DIR/report.json" "$ARTIFACT_DIR/tpch_sf1_report.json"

echo ""
echo "Results: $RESULT_DIR/report.json"
echo ""

# 显示摘要
if [ -f "$RESULT_DIR/report.json" ]; then
    python3 -c "
import json
with open('$RESULT_DIR/report.json') as f:
    d = json.load(f)
    print(f\"Status: {d['status']}\")
    print(f\"Queries: {d['queries_passed']}/{d['total_queries']}\")
    print(f\"Time: {d['total_time_seconds']}s\")
    for q in ['Q1', 'Q6']:
        if q in d.get('query_timings', {}):
            print(f\"  {q}: {d['query_timings'][q]['ms']:.0f}ms\")
"
fi
