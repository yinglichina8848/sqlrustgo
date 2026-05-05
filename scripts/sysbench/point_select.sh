#!/bin/bash
# Sysbench OLTP Point Select Test for SQLRustGo
# Usage: ./sysbench_point_select.sh [threads] [time]

set -e

THREADS=${1:-8}
TIME=${2:-30}
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-3306}
USER=${USER:-mysql}
PASSWORD=${PASSWORD:-mysql}
DB=${DB:-sbtest}
TABLE_SIZE=${TABLE_SIZE:-10000}

RESULTS_DIR="./sysbench_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "========================================"
echo "SQLRustGo Sysbench Point Select Test"
echo "========================================"
echo "Threads: $THREADS"
echo "Time: ${TIME}s"
echo "Host: $HOST:$PORT"
echo "Database: $DB"
echo "Table size: $TABLE_SIZE"
echo "Results dir: $RESULTS_DIR"
echo "========================================"

# Prepare tables if they don't exist
echo "Preparing tables..."
sysbench oltp_insert \
  --db-driver=mysql \
  --mysql-host=$HOST \
  --mysql-port=$PORT \
  --mysql-user=$USER \
  --mysql-password=$PASSWORD \
  --mysql-db=$DB \
  --table-size=$TABLE_SIZE \
  --tables=1 \
  --random-points=1 \
  prepare 2>/dev/null || true

# Run point select test
echo "Running point select test..."
OUTPUT_FILE="$RESULTS_DIR/point_select_t${THREADS}.log"

sysbench oltp_point_select \
  --db-driver=mysql \
  --mysql-host=$HOST \
  --mysql-port=$PORT \
  --mysql-user=$USER \
  --mysql-password=$PASSWORD \
  --mysql-db=$DB \
  --table-size=$TABLE_SIZE \
  --tables=1 \
  --threads=$THREADS \
  --time=$TIME \
  --report-interval=5 \
  run 2>&1 | tee "$OUTPUT_FILE"

echo ""
echo "========================================"
echo "Results saved to: $OUTPUT_FILE"
echo "========================================"