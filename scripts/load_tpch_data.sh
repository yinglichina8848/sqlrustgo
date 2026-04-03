#!/bin/bash
# TPC-H SF=0.1 Data Loader
# Converts .tbl files (pipe-delimited) to SQLite for TPC-H compliance testing

set -e

DATA_DIR="data/tpch-sf01"
DB_PATH="$DATA_DIR/tpch.db"

echo "=== TPC-H Data Loader ==="
echo "Data directory: $DATA_DIR"
echo "Database: $DB_PATH"

# Remove existing database
rm -f "$DB_PATH"

# Create SQLite database
sqlite3 "$DB_PATH" <<'EOF'
-- Region table
CREATE TABLE region (
    r_regionkey INTEGER PRIMARY KEY,
    r_name TEXT,
    r_comment TEXT
);

-- Nation table
CREATE TABLE nation (
    n_nationkey INTEGER PRIMARY KEY,
    n_name TEXT,
    n_regionkey INTEGER,
    n_comment TEXT
);

-- Customer table
CREATE TABLE customer (
    c_custkey INTEGER PRIMARY KEY,
    c_name TEXT,
    c_address TEXT,
    c_nationkey INTEGER,
    c_phone TEXT,
    c_acctbal REAL,
    c_mktsegment TEXT,
    c_comment TEXT
);

-- Supplier table
CREATE TABLE supplier (
    s_suppkey INTEGER PRIMARY KEY,
    s_name TEXT,
    s_address TEXT,
    s_nationkey INTEGER,
    s_phone TEXT,
    s_acctbal REAL,
    s_comment TEXT
);

-- Part table
CREATE TABLE part (
    p_partkey INTEGER PRIMARY KEY,
    p_name TEXT,
    p_mfgr TEXT,
    p_brand TEXT,
    p_type TEXT,
    p_size INTEGER,
    p_container TEXT,
    p_retailprice REAL,
    p_comment TEXT
);

-- PartSupp table
CREATE TABLE partsupp (
    ps_partkey INTEGER,
    ps_suppkey INTEGER,
    ps_availqty INTEGER,
    ps_supplycost REAL,
    ps_comment TEXT,
    PRIMARY KEY (ps_partkey, ps_suppkey)
);

-- Orders table
CREATE TABLE orders (
    o_orderkey INTEGER PRIMARY KEY,
    o_custkey INTEGER,
    o_orderstatus TEXT,
    o_totalprice REAL,
    o_orderdate TEXT,
    o_orderpriority TEXT,
    o_clerk TEXT,
    o_shippriority INTEGER,
    o_comment TEXT
);

-- LineItem table
CREATE TABLE lineitem (
    l_orderkey INTEGER,
    l_partkey INTEGER,
    l_suppkey INTEGER,
    l_linenumber INTEGER,
    l_quantity INTEGER,
    l_extendedprice REAL,
    l_discount REAL,
    l_tax REAL,
    l_returnflag TEXT,
    l_linestatus TEXT,
    l_shipdate TEXT,
    l_commitdate TEXT,
    l_receiptdate TEXT,
    l_shipinstruct TEXT,
    l_shipmode TEXT,
    l_comment TEXT,
    PRIMARY KEY (l_orderkey, l_linenumber)
);
EOF

echo "Schema created."

# Load data using pipe delimiter
echo "Loading region..."
sqlite3 "$DB_PATH" ".mode list"
sqlite3 "$DB_PATH" ".import '$DATA_DIR/region.tbl' region"

echo "Loading nation..."
sqlite3 "$DB_PATH" ".import '$DATA_DIR/nation.tbl' nation"

echo "Loading customer..."
sqlite3 "$DB_PATH" ".import '$DATA_DIR/customer.tbl' customer"

echo "Loading supplier..."
sqlite3 "$DB_PATH" ".import '$DATA_DIR/supplier.tbl' supplier"

echo "Loading part..."
sqlite3 "$DB_PATH" ".import '$DATA_DIR/part.tbl' part"

echo "Loading partsupp..."
sqlite3 "$DB_PATH" ".import '$DATA_DIR/partsupp.tbl' partsupp"

echo "Loading orders..."
sqlite3 "$DB_PATH" ".import '$DATA_DIR/orders.tbl' orders"

echo "Loading lineitem (this may take a while)..."
sqlite3 "$DB_PATH" ".import '$DATA_DIR/lineitem.tbl' lineitem"

# Run VACUUM to optimize
echo "Optimizing database..."
sqlite3 "$DB_PATH" "VACUUM;"

# Print statistics
echo ""
echo "=== Data Loaded ==="
echo "Tables:"
sqlite3 "$DB_PATH" <<'EOF'
SELECT 'region' as table_name, COUNT(*) as row_count FROM region
UNION ALL
SELECT 'nation', COUNT(*) FROM nation
UNION ALL
SELECT 'customer', COUNT(*) FROM customer
UNION ALL
SELECT 'supplier', COUNT(*) FROM supplier
UNION ALL
SELECT 'part', COUNT(*) FROM part
UNION ALL
SELECT 'partsupp', COUNT(*) FROM partsupp
UNION ALL
SELECT 'orders', COUNT(*) FROM orders
UNION ALL
SELECT 'lineitem', COUNT(*) FROM lineitem;
EOF

echo ""
echo "Database created: $DB_PATH"
echo "Size: $(du -h "$DB_PATH" | cut -f1)"
