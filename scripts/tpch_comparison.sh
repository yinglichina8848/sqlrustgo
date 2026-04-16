#!/bin/bash
# TPC-H Comparison Test Script for MySQL, PostgreSQL, and SQLite

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="$SCRIPT_DIR/data/tpch-sf1"
RESULTS_DIR="$SCRIPT_DIR/results"

mkdir -p "$RESULTS_DIR"

echo "=== TPC-H Comparison Test ==="
echo "Data Directory: $DATA_DIR"
echo ""

# Function to run SQL and measure time
run_sql() {
    local db_cmd="$1"
    local sql="$2"
    local start_time=$(date +%s%N)

    echo "$sql" | $db_cmd > /dev/null 2>&1
    local end_time=$(date +%s%N)
    local elapsed=$(( (end_time - start_time) / 1000000 ))
    echo "$elapsed"
}

# TPC-H Q1 Query
Q1="SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag"

# TPC-H Q2 Query
Q2="SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10"

# TPC-H Q3 Query
Q3="SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey"

# TPC-H Q4 Query
Q4="SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority"

# TPC-H Q5 Query
Q5="SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name"

# TPC-H Q6 Query
Q6="SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'"

# TPC-H Q10 Query
Q10="SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey"

# TPC-H Q13 Query
Q13="SELECT c_mktsegment, COUNT(*) FROM customer GROUP BY c_mktsegment"

# TPC-H Q14 Query
Q14="SELECT p_type, COUNT(*) FROM part GROUP BY p_type"

# TPC-H Q19 Query
Q19="SELECT p_brand, SUM(p_retailprice) FROM part GROUP BY p_brand"

# TPC-H Q20 Query
Q20="SELECT s_nationkey, COUNT(*) FROM supplier GROUP BY s_nationkey"

# TPC-H Q22 Query
Q22="SELECT c_nationkey, COUNT(*) FROM customer WHERE c_acctbal > 0 GROUP BY c_nationkey"

echo "Running SQLite comparison..."
echo "=========================="

SQLITE_DB="$RESULTS_DIR/tpch_sqlite.db"
rm -f "$SQLITE_DB"

# Create SQLite tables and import data
sqlite3 "$SQLITE_DB" << 'SQL'
.mode csv
.import data/tpch-sf1/customer.tbl customer
.import data/tpch-sf1/orders.tbl orders
.import data/tpch-sf1/lineitem.tbl lineitem
.import data/tpch-sf1/part.tbl part
.import data/tpch-sf1/partsupp.tbl partsupp
.import data/tpch-sf1/supplier.tbl supplier
.import data/tpch-sf1/nation.tbl nation
.import data/tpch-sf1/region.tbl region

CREATE INDEX IF NOT EXISTS idx_lineitem_shipdate ON lineitem(l_shipdate);
CREATE INDEX IF NOT EXISTS idx_lineitem_orderkey ON lineitem(l_orderkey);
CREATE INDEX IF NOT EXISTS idx_orders_custkey ON orders(o_custkey);
CREATE INDEX IF NOT EXISTS idx_orders_date ON orders(o_orderdate);
CREATE INDEX IF NOT EXISTS idx_partsupp_partkey ON partsupp(ps_partkey);
CREATE INDEX IF NOT EXISTS idx_partsupp_suppkey ON partsupp(ps_suppkey);
CREATE INDEX IF NOT EXISTS idx_customer_nationkey ON customer(c_nationkey);
CREATE INDEX IF NOT EXISTS idx_supplier_nationkey ON supplier(s_nationkey);
CREATE INDEX IF NOT EXISTS idx_nation_regionkey ON nation(n_regionkey);
SQL

echo "SQLite database created with indexes"
echo ""

# Run SQLite benchmarks
echo "Running Q1..."
sqlite3 "$SQLITE_DB" "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag;" > /dev/null

echo "Running Q2..."
sqlite3 "$SQLITE_DB" "SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10;" > /dev/null

echo "Running Q3..."
sqlite3 "$SQLITE_DB" "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey;" > /dev/null

echo "Running Q4..."
sqlite3 "$SQLITE_DB" "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority;" > /dev/null

echo "Running Q6..."
sqlite3 "$SQLITE_DB" "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01';" > /dev/null

echo "Running Q10..."
sqlite3 "$SQLITE_DB" "SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey;" > /dev/null

echo "Running Q13..."
sqlite3 "$SQLITE_DB" "SELECT c_mktsegment, COUNT(*) FROM customer GROUP BY c_mktsegment;" > /dev/null

echo "Running Q14..."
sqlite3 "$SQLITE_DB" "SELECT p_type, COUNT(*) FROM part GROUP BY p_type;" > /dev/null

echo "Running Q22..."
sqlite3 "$SQLITE_DB" "SELECT c_nationkey, COUNT(*) FROM customer WHERE c_acctbal > 0 GROUP BY c_nationkey;" > /dev/null

echo ""
echo "SQLite benchmarks completed!"

echo ""
echo "=== Sample MySQL/PostgreSQL Commands ==="
echo "To run MySQL comparison:"
echo "  mysql -u root tpch_test < mysql_tpch_setup.sql"
echo "  mysql -u root tpch_test < mysql_tpch_queries.sql"
echo ""
echo "To run PostgreSQL comparison:"
echo "  psql -U postgres -d tpch_test -f pg_tpch_setup.sql"
echo "  psql -U postgres -d tpch_test -f pg_tpch_queries.sql"
echo ""
echo "Results saved to: $RESULTS_DIR"