#!/usr/bin/env python3
import psycopg2
import time

# Connect to PostgreSQL
conn = psycopg2.connect(
    host="/var/run/postgresql",
    database="tpch_test",
    user="openclaw",
    password="openclaw123",
)
conn.autocommit = True
cur = conn.cursor()

# Create tables
tables_sql = """
-- Customer table
CREATE TABLE IF NOT EXISTS customer (
    c_custkey INT PRIMARY KEY,
    c_name VARCHAR(25),
    c_address VARCHAR(40),
    c_nationkey INT,
    c_phone CHAR(15),
    c_acctbal DECIMAL(10,2),
    c_mktsegment CHAR(10),
    c_comment VARCHAR(117)
);

-- Nation table
CREATE TABLE IF NOT EXISTS nation (
    n_nationkey INT PRIMARY KEY,
    n_name CHAR(25),
    n_regionkey INT,
    n_comment VARCHAR(152)
);

-- Region table
CREATE TABLE IF NOT EXISTS region (
    r_regionkey INT PRIMARY KEY,
    r_name CHAR(25),
    r_comment VARCHAR(152)
);

-- Supplier table
CREATE TABLE IF NOT EXISTS supplier (
    s_suppkey INT PRIMARY KEY,
    s_name CHAR(25),
    s_address VARCHAR(40),
    s_nationkey INT,
    s_phone CHAR(15),
    s_acctbal DECIMAL(10,2),
    s_comment VARCHAR(101)
);

-- Part table
CREATE TABLE IF NOT EXISTS part (
    p_partkey INT PRIMARY KEY,
    p_name VARCHAR(55),
    p_mfgr CHAR(25),
    p_brand CHAR(10),
    p_type VARCHAR(25),
    p_size INT,
    p_container CHAR(10),
    p_retailprice DECIMAL(10,2),
    p_comment VARCHAR(23)
);

-- PartSupp table
CREATE TABLE IF NOT EXISTS partsupp (
    ps_partkey INT,
    ps_suppkey INT,
    ps_availqty INT,
    ps_supplycost DECIMAL(10,2),
    ps_comment VARCHAR(199),
    PRIMARY KEY (ps_partkey, ps_suppkey)
);

-- Orders table
CREATE TABLE IF NOT EXISTS orders (
    o_orderkey INT PRIMARY KEY,
    o_custkey INT,
    o_orderstatus CHAR(1),
    o_totalprice DECIMAL(10,2),
    o_orderdate DATE,
    o_orderpriority CHAR(15),
    o_clerk CHAR(15),
    o_shippriority INT,
    o_comment VARCHAR(79)
);

-- Lineitem table
CREATE TABLE IF NOT EXISTS lineitem (
    l_orderkey INT,
    l_partkey INT,
    l_suppkey INT,
    l_linenumber INT,
    l_quantity INT,
    l_extendedprice DECIMAL(10,2),
    l_discount DECIMAL(10,2),
    l_tax DECIMAL(10,2),
    l_returnflag CHAR(1),
    l_linestatus CHAR(1),
    l_shipdate DATE,
    l_commitdate DATE,
    l_receiptdate DATE,
    l_shipinstruct CHAR(25),
    l_shipmode CHAR(10),
    l_comment VARCHAR(44),
    PRIMARY KEY (l_orderkey, l_linenumber)
);
"""

for statement in tables_sql.split(";"):
    statement = statement.strip()
    if statement:
        cur.execute(statement)

print("Tables created")

# Create indexes
indexes = [
    "CREATE INDEX idx_customer_nationkey ON customer(c_nationkey)",
    "CREATE INDEX idx_orders_custkey ON orders(o_custkey)",
    "CREATE INDEX idx_orders_date ON orders(o_orderdate)",
    "CREATE INDEX idx_lineitem_orderkey ON lineitem(l_orderkey)",
    "CREATE INDEX idx_lineitem_shipdate ON lineitem(l_shipdate)",
    "CREATE INDEX idx_partsupp_partkey ON partsupp(ps_partkey)",
    "CREATE INDEX idx_partsupp_suppkey ON partsupp(ps_suppkey)",
    "CREATE INDEX idx_supplier_nationkey ON supplier(s_nationkey)",
    "CREATE INDEX idx_nation_regionkey ON nation(n_regionkey)",
]

for idx in indexes:
    cur.execute(idx)

print("Indexes created")


# Load data
def load_data(table, filepath):
    with open(filepath, "r") as f:
        for line in f:
            values = [v for v in line.strip().split("|") if v != ""]
            placeholders = ",".join(["%s"] * len(values))
            cur.execute(
                f"INSERT INTO {table} VALUES ({placeholders}) ON CONFLICT DO NOTHING",
                values,
            )
    print(f"Loaded {table}")


# Use SF0.1 for quick test
load_data("customer", "data/tpch-sf01/customer.tbl")
load_data("nation", "data/tpch-sf01/nation.tbl")
load_data("region", "data/tpch-sf01/region.tbl")
load_data("supplier", "data/tpch-sf01/supplier.tbl")
load_data("part", "data/tpch-sf01/part.tbl")
load_data("partsupp", "data/tpch-sf01/partsupp.tbl")
load_data("orders", "data/tpch-sf01/orders.tbl")
load_data("lineitem", "data/tpch-sf01/lineitem.tbl")

print("All data loaded")

conn.close()
print("PostgreSQL TPC-H setup complete")
