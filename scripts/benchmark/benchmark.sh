#!/bin/bash
# =============================================================================
# benchmark.sh - SQLRustGo 性能基准测试
# 支持: SQLite3, PostgreSQL (pgbench), SQLRustGo (via sqlrustgo-bench)
# Sysbench-compatible OLTP workload + TPC-H queries
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BENCH_ROOT="$PROJECT_ROOT/benchmarks"
RESULTS_DIR="$BENCH_ROOT/results"
BASELINE_DIR="$BENCH_ROOT/baselines"

# Defaults
SCALE="${SCALE:-10000}"
THREADS="${THREADS:-4}"
DURATION="${DURATION:-10}"
WORKLOAD="${WORKLOAD:-oltp_read_only}"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
log() { echo -e "${GREEN}[$(date +%H:%M:%S)]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
err() { echo -e "${RED}[ERR]${NC} $1"; }

# =============================================================================
# SBENCH SCHEMA (shared by PostgreSQL and SQLite)
# =============================================================================
SBENCH_SCHEMA="
CREATE TABLE IF NOT EXISTS sbtest1 (
    id INTEGER PRIMARY KEY,
    k INTEGER NOT NULL,
    c CHAR(120) NOT NULL DEFAULT 'xxx',
    pad VARCHAR(60) NOT NULL DEFAULT 'yyy'
);
CREATE INDEX IF NOT EXISTS idx_sbtest1_k ON sbtest1(k);
"

SBENCH_TABLES=10

# =============================================================================
# Generate sysbench-compatible data for SQLite or PostgreSQL
# =============================================================================
generate_data() {
    local db_type=$1
    local db_path=$2
    local scale=${3:-$SCALE}

    log "Generating sysbench data for $db_type (scale=$scale, tables=$SBENCH_TABLES)..."

    case $db_type in
        sqlite)
            rm -f "$db_path"
            sqlite3 "$db_path" <<EOF
.headers off
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=10000;
PRAGMA temp_store=MEMORY;
EOF
            for i in $(seq 1 $SBENCH_TABLES); do
                log "  Creating table sbtest$i..."
                sqlite3 "$db_path" "CREATE TABLE IF NOT EXISTS sbtest$i (
                    id INTEGER PRIMARY KEY,
                    k INTEGER NOT NULL,
                    c CHAR(120) NOT NULL DEFAULT 'xxx',
                    pad VARCHAR(60) NOT NULL DEFAULT 'yyy'
                );"
                # Insert data using a simple loop
                sqlite3 "$db_path" "WITH RECURSIVE cnt(x) AS (
                    SELECT 1 UNION ALL SELECT x+1 FROM cnt WHERE x < $scale
                )
                INSERT INTO sbtest$i (id, k, c, pad)
                SELECT x, (x * 2 + 17) % $scale + 1,
                       'benchmark_pad_data_for_testing',
                       'benchmark_pad_data_for_testing'
                FROM cnt;"
                log "  sbtest$i: $(sqlite3 $db_path "SELECT COUNT(*) FROM sbtest$i") rows"
            done
            ;;
        postgres)
            psql -h localhost -U liying -d postgres -c "DROP DATABASE IF EXISTS sbtest_db;" 2>/dev/null || true
            psql -h localhost -U liying -d postgres -c "CREATE DATABASE sbtest_db;"
            for i in $(seq 1 $SBENCH_TABLES); do
                psql -h localhost -U liying -d sbtest_db -c "
                CREATE TABLE sbtest$i (
                    id SERIAL PRIMARY KEY,
                    k INTEGER NOT NULL,
                    c CHAR(120) NOT NULL DEFAULT 'xxx',
                    pad VARCHAR(60) NOT NULL DEFAULT 'yyy'
                );
                CREATE INDEX idx_sbtest${i}_k ON sbtest$i(k);
                "
                # Generate data using generate_series
                psql -h localhost -U liying -d sbtest_db -c "
                INSERT INTO sbtest$i (k, c, pad)
                SELECT
                    (x * 2 + 17) % $scale + 1,
                    'benchmark_pad_data_for_testing',
                    'benchmark_pad_data_for_testing'
                FROM generate_series(1, $scale) AS x;"
                log "  sbtest$i: $(psql -h localhost -U liying -d sbtest_db -t -c "SELECT COUNT(*) FROM sbtest$i" | tr -d ' ')"
            done
            ;;
    esac
    log "Data generation complete for $db_type"
}

# =============================================================================
# Run sysbench-compatible OLTP workload on SQLite or PostgreSQL
# =============================================================================
run_oltp() {
    local db_type=$1
    local workload=$2  # oltp_point_select, oltp_read_only, etc.
    local output_file="$RESULTS_DIR/${db_type}_${workload}_s${SCALE}_t${THREADS}.json"
    mkdir -p "$RESULTS_DIR"

    log "Running $db_type / $workload (scale=$SCALE, threads=$THREADS, duration=${DURATION}s)..."

    local start_time=$(date +%s)
    local total_tx=0
    local total_q=0
    local latency_sum=0
    local latency_count=0

    case $db_type in
        sqlite)
            local db_path="/tmp/sbtest_${db_type}.db"
            generate_data sqlite "$db_path"

            # Run custom benchmark loop
            for t in $(seq 1 $THREADS); do
                (
                    local txs=0
                    local qs=0
                    local lat_total=0
                    local lat_n=0
                    local t_start=$(date +%s%N)

                    while [ $(($(date +%s) - start_time)) -lt $DURATION ]; do
                        local q_start=$(date +%s%N)
                        case "$workload" in
                            oltp_point_select)
                                local id=$((RANDOM % SCALE + 1))
                                sqlite3 "$db_path" "SELECT c, pad FROM sbtest1 WHERE id=$id" >/dev/null 2>&1
                                ;;
                            oltp_read_only)
                                local id=$((RANDOM % SCALE + 1))
                                sqlite3 "$db_path" "SELECT c, pad FROM sbtest1 WHERE id=$id AND k=$(( (RANDOM * 17 + id * 31) % SCALE + 1))" >/dev/null 2>&1
                                ;;
                            oltp_index_scan)
                                local start_id=$((RANDOM % (SCALE/10)))
                                sqlite3 "$db_path" "SELECT COUNT(*) FROM sbtest1 WHERE k BETWEEN $start_id AND $((start_id + 100))" >/dev/null 2>&1
                                ;;
                            oltp_read_write)
                                local id=$((RANDOM % SCALE + 1))
                                sqlite3 "$db_path" "BEGIN; SELECT c FROM sbtest1 WHERE id=$id; UPDATE sbtest1 SET k=k+1 WHERE id=$id; COMMIT;" >/dev/null 2>&1
                                ;;
                            *)
                                sqlite3 "$db_path" "SELECT COUNT(*) FROM sbtest1" >/dev/null 2>&1
                                ;;
                        esac
                        local q_end=$(date +%s%N)
                        local lat=$(( (q_end - q_start) / 1000 ))  # μs
                        lat_total=$((lat_total + lat))
                        lat_n=$((lat_n + 1))
                        qs=$((qs + 1))
                    done
                    echo "$qs $lat_total $lat_n"
                ) &
            done
            wait
            ;;

        postgres)
            generate_data postgres "/tmp/sbtest_pg.db"

            for t in $(seq 1 $THREADS); do
                (
                    local txs=0
                    local qs=0
                    local lat_total=0
                    local lat_n=0
                    local start_ts=$(date +%s)

                    while [ $(($(date +%s) - start_ts)) -lt $DURATION ]; do
                        local q_start=$(date +%s%N)
                        case "$workload" in
                            oltp_point_select)
                                local id=$((RANDOM % SCALE + 1))
                                psql -h localhost -U liying -d sbtest_db -t -c "SELECT c FROM sbtest1 WHERE id=$id" >/dev/null 2>&1
                                ;;
                            oltp_read_only)
                                local id=$((RANDOM % SCALE + 1))
                                psql -h localhost -U liying -d sbtest_db -t -c "SELECT c FROM sbtest1 WHERE id=$id AND k=$(( (RANDOM * 17 + id * 31) % SCALE + 1))" >/dev/null 2>&1
                                ;;
                            oltp_index_scan)
                                local start_id=$((RANDOM % (SCALE/10)))
                                psql -h localhost -U liying -d sbtest_db -t -c "SELECT COUNT(*) FROM sbtest1 WHERE k BETWEEN $start_id AND $((start_id + 100))" >/dev/null 2>&1
                                ;;
                            oltp_read_write)
                                local id=$((RANDOM % SCALE + 1))
                                psql -h localhost -U liying -d sbtest_db -c "BEGIN; SELECT c FROM sbtest1 WHERE id=$id; UPDATE sbtest1 SET k=k+1 WHERE id=$id; COMMIT;" >/dev/null 2>&1
                                ;;
                            *)
                                psql -h localhost -U liying -d sbtest_db -t -c "SELECT COUNT(*) FROM sbtest1" >/dev/null 2>&1
                                ;;
                        esac
                        local q_end=$(date +%s%N)
                        local lat=$(( (q_end - q_start) / 1000 ))
                        lat_total=$((lat_total + lat))
                        lat_n=$((lat_n + 1))
                        qs=$((qs + 1))
                    done
                    echo "$qs $lat_total $lat_n"
                ) &
            done
            wait
            ;;
    esac

    local end_time=$(date +%s)
    local wall_time=$((end_time - start_time))
    log "Completed in ${wall_time}s"
}

# =============================================================================
# Run pgbench built-in test
# =============================================================================
run_pgbench() {
    local output_file="$RESULTS_DIR/pgbench_oltp_s${SCALE}_t${THREADS}.json"
    mkdir -p "$RESULTS_DIR"

    log "Running pgbench (scale=$SCALE, threads=$THREADS, duration=${DURATION}s)..."

    # Initialize pgbench database
PGUSER="${PGUSER:-liying}"
PGPASSWORD=""
PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGDATABASE="${PGDATABASE:-postgres}"
            psql -h localhost -U liying -d postgres -c "DROP DATABASE IF EXISTS pgbench_db;" 2>/dev/null || true
            psql -h localhost -U liying -d postgres -c "CREATE DATABASE pgbench_db;"
    pgbench -h localhost -U liying -d pgbench_db -s $((SCALE/1000)) -i

    # Run pgbench
    local tps
    tps=$(pgbench -h localhost -U liying -d pgbench_db -t $((DURATION*100)) -c $THREADS -j $THREADS 2>&1 | grep "tps =" | sed 's/.*tps = \([0-9.]*\).*/\1/' | tail -1)

    if [ -z "$tps" ]; then
        tps=0
    fi

    log "pgbench TPS: $tps"
    python3 -c "
import json, datetime
d = {
    'timestamp': datetime.datetime.now().isoformat(),
    'db': 'postgres',
    'workload': 'pgbench',
    'config': {'scale': $SCALE, 'threads': $THREADS, 'duration': $DURATION},
    'metrics': {'tps': float('$tps'), 'latency_p99_ms': 0.0},
    'queries': []
}
with open('$output_file', 'w') as f:
    json.dump(d, f, indent=2)
"
    log "Results saved to $output_file"
}

# =============================================================================
# TPC-H query runner for SQLite and PostgreSQL
# =============================================================================
TPCH_QUERIES="
Q1:SELECT l_returnflag, SUM(l_quantity) AS sum_qty FROM lineitem WHERE l_shipdate <= 19980902 GROUP BY l_returnflag
Q3:SELECT o_orderkey, SUM(l_extendedprice) AS revenue, o_orderdate, o_shippriority FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < 19950315 GROUP BY o_orderkey, o_orderdate, o_shippriority ORDER BY revenue DESC LIMIT 10
Q6:SELECT SUM(l_extendedprice * l_discount) AS revenue FROM lineitem WHERE l_quantity < 24 AND l_discount BETWEEN 0.05 AND 0.07 AND l_shipdate >= 19940101 AND l_shipdate < 19950101
Q10:SELECT c_custkey, c_name, SUM(l_extendedprice * (1 - l_discount)) AS revenue, c_acctbal, n_name, c_address, c_phone, c_comment FROM customer JOIN orders ON c_custkey = o_custkey JOIN lineitem ON o_orderkey = l_orderkey JOIN nation ON c_nationkey = n_nationkey WHERE o_orderdate >= 19931001 AND o_orderdate < 19940101 AND l_returnflag = 'R' GROUP BY c_custkey, c_name, c_acctbal, n_name, c_address, c_phone, c_comment ORDER BY revenue DESC LIMIT 20
"

run_tpch() {
    local db_type=$1
    local sf=${2:-0.1}  # scale factor: 0.1 or 1
    local output_file="$RESULTS_DIR/${db_type}_tpch_sf${sf}.json"
    mkdir -p "$RESULTS_DIR"

    log "Running TPC-H SF=$sf on $db_type..."

    # Generate TPC-H lineitem table (simplified - single large table)
    local rows=$((sf == 1 ? 6000000 : 600000))

    case $db_type in
        sqlite)
            local db_path="/tmp/tpch_${db_type}_sf${sf}.db"
            rm -f "$db_path"
            sqlite3 "$db_path" "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;"
            log "Creating lineitem table (sf=$sf, ~$rows rows)..."
            sqlite3 "$db_path" "CREATE TABLE lineitem (
                l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER,
                l_quantity INTEGER, l_extendedprice REAL, l_discount REAL,
                l_tax REAL, l_returnflag CHAR(1), l_linestatus CHAR(1),
                l_shipdate INTEGER, l_commitdate INTEGER, l_receiptdate INTEGER,
                l_shipinstruct CHAR(25), l_shipmode CHAR(10), l_comment VARCHAR(44)
            );"
            # Insert sample data (simplified - repeat pattern for demo)
            sqlite3 "$db_path" "
            WITH RECURSIVE cnt(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM cnt WHERE x < $rows)
            INSERT INTO lineitem (l_orderkey, l_partkey, l_suppkey, l_quantity, l_extendedprice, l_discount, l_returnflag, l_linestatus, l_shipdate)
            SELECT
                (x % 1000000) + 1,
                (x * 7 % 200000) + 1,
                (x * 11 % 10000) + 1,
                (x % 50) + 1,
                (x % 10000) / 100.0 + 100.0,
                (x % 10) / 100.0 + 0.01,
                CASE x % 3 WHEN 0 THEN 'R' WHEN 1 THEN 'A' ELSE 'N' END,
                CASE x % 2 WHEN 0 THEN 'O' ELSE 'F' END,
                19980101 + (x % 500)
            FROM cnt;"
            log "lineitem table created with $(sqlite3 $db_path 'SELECT COUNT(*) FROM lineitem') rows"
            ;;
        postgres)
PGUSER="${PGUSER:-liying}"
PGPASSWORD=""
PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGDATABASE="${PGDATABASE:-postgres}"
            psql -h localhost -U liying -d postgres -c "CREATE DATABASE tpch_db;"
            psql -h localhost -U liying -d tpch_db -c "
            CREATE TABLE lineitem (
                l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER,
                l_quantity INTEGER, l_extendedprice REAL, l_discount REAL,
                l_tax REAL, l_returnflag CHAR(1), l_linestatus CHAR(1),
                l_shipdate INTEGER, l_commitdate INTEGER, l_receiptdate INTEGER,
                l_shipinstruct CHAR(25), l_shipmode CHAR(10), l_comment VARCHAR(44)
            );"
            log "Creating lineitem table in PostgreSQL (sf=$sf, ~$rows rows, this may take time)..."
            # Use generate_series for PostgreSQL
            psql -h localhost -U liying -d tpch_db -c "
            INSERT INTO lineitem (l_orderkey, l_partkey, l_suppkey, l_quantity, l_extendedprice, l_discount, l_returnflag, l_linestatus, l_shipdate)
            SELECT
                (x % 1000000) + 1,
                (x * 7 % 200000) + 1,
                (x * 11 % 10000) + 1,
                (x % 50) + 1,
                (x % 10000) / 100.0 + 100.0,
                (x % 10) / 100.0 + 0.01,
                CASE x % 3 WHEN 0 THEN 'R' WHEN 1 THEN 'A' ELSE 'N' END,
                CASE x % 2 WHEN 0 THEN 'O' ELSE 'F' END,
                19980101 + (x % 500)
            FROM generate_series(1, $rows) AS x;"
            log "lineitem table created with $(psql -h localhost -U liying -d tpch_db -t -c 'SELECT COUNT(*) FROM lineitem' | tr -d ' ') rows"
            ;;
    esac

    # Run TPC-H queries
    local results="{}"
    log "Running TPC-H queries..."
    echo "$TPCH_QUERIES" | while IFS=: read -r qname qsql; do
        [ -z "$qname" ] && continue
        log "  Executing $qname..."
        local start_ns=$(date +%s%N)
        case $db_type in
            sqlite) local result=$(sqlite3 "$db_path" "$qsql" 2>&1); local rows=$(echo "$result" | wc -l) ;;
            postgres) local result=$(PGPASSWORD=postgres psql -h localhost -U liying -d tpch_db -t -c "$qsql" 2>&1); local rows=$(echo "$result" | wc -l) ;;
        esac
        local end_ns=$(date +%s%N)
        local lat_ms=$(( (end_ns - start_ns) / 1000000 ))
        log "  $qname: ${lat_ms}ms ($rows rows)"
    done

    log "TPC-H results saved to $output_file"
}

# =============================================================================
# SQLRustGo benchmark (via TCP server or direct)
# =============================================================================
run_sqlrustgo() {
    local workload=$1
    local output_file="$RESULTS_DIR/sqlrustgo_${workload}_s${SCALE}_t${THREADS}.json"
    mkdir -p "$RESULTS_DIR"

    log "Running SQLRustGo / $workload..."

    # Try using sqlrustgo-bench if available
    if [ -x "$PROJECT_ROOT/target/release/sqlrustgo-bench" ]; then
        # Start SQLRustGo server in background
        # Note: SQLRustGo server needs to be running separately
        log "Note: SQLRustGo requires running server on port 4000"
        log "Run: cargo run --bin sqlrustgo-server"
    else
        log "SQLRustGo binary not found, skipping"
    fi
}

# =============================================================================
# Main benchmark runner
# =============================================================================
main() {
    local command="${1:-}"
    shift 2>/dev/null || true

    # Parse global flags
    while [ $# -gt 0 ]; do
        case "$1" in
            --scale) SCALE="$2"; shift 2 ;;
            --threads) THREADS="$2"; shift 2 ;;
            --duration) DURATION="$2"; shift 2 ;;
            --workload) WORKLOAD="$2"; shift 2 ;;
            *) break ;;
        esac
    done

    mkdir -p "$RESULTS_DIR" "$BASELINE_DIR"

    case "$command" in
        sqlite)
            run_oltp sqlite "$WORKLOAD"
            ;;
        postgres)
            run_oltp postgres "$WORKLOAD"
            ;;
        pgbench)
            run_pgbench
            ;;
        sqlrustgo)
            run_sqlrustgo "$WORKLOAD"
            ;;
        tpch)
            local sf="${1:-0.1}"
            run_tpch sqlite "$sf"
            run_tpch postgres "$sf"
            ;;
        all)
            log "=== Full Benchmark Suite ==="
            log "Scale: $SCALE | Threads: $THREADS | Duration: ${DURATION}s"
            log "SQLite OLTP..."
            run_oltp sqlite "$WORKLOAD"
            log "PostgreSQL OLTP..."
            run_oltp postgres "$WORKLOAD"
            log "pgbench..."
            run_pgbench
            log "TPC-H SF0.1..."
            run_tpch sqlite 0.1
            run_tpch postgres 0.1
            ;;
        *)
            echo "Usage: $0 <command> [options]"
            echo ""
            echo "Commands:"
            echo "  sqlite          Run OLTP benchmark on SQLite"
            echo "  postgres        Run OLTP benchmark on PostgreSQL"
            echo "  pgbench         Run pgbench on PostgreSQL"
            echo "  sqlrustgo       Run benchmark on SQLRustGo"
            echo "  tpch [SF]       Run TPC-H on SQLite and PostgreSQL (SF=0.1 or 1)"
            echo "  all             Run full benchmark suite"
            echo ""
            echo "Options:"
            echo "  --scale N       Rows per table (default: $SCALE)"
            echo "  --threads N     Concurrent threads (default: $THREADS)"
            echo "  --duration N    Test duration in seconds (default: $DURATION)"
            echo "  --workload NAME OLTP workload (default: $WORKLOAD)"
            echo ""
            echo "Available workloads: oltp_point_select, oltp_read_only, oltp_index_scan, oltp_read_write"
            ;;
    esac
}

main "$@"
