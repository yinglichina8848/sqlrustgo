-- PostgreSQL TPC-H Setup Script
-- Run: psql -U postgres -d tpch_test -f pg_tpch_setup.sql

DROP DATABASE IF EXISTS tpch_test;
CREATE DATABASE tpch_test;
\c tpch_test

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

-- Load data (run from shell)
-- \copy customer FROM 'data/tpch-sf1/customer.tbl' WITH (FORMAT csv, DELIMITER '|');
-- \copy orders FROM 'data/tpch-sf1/orders.tbl' WITH (FORMAT csv, DELIMITER '|');
-- \copy lineitem FROM 'data/tpch-sf1/lineitem.tbl' WITH (FORMAT csv, DELIMITER '|');
-- \copy part FROM 'data/tpch-sf1/part.tbl' WITH (FORMAT csv, DELIMITER '|');
-- \copy partsupp FROM 'data/tpch-sf1/partsupp.tbl' WITH (FORMAT csv, DELIMITER '|');
-- \copy supplier FROM 'data/tpch-sf1/supplier.tbl' WITH (FORMAT csv, DELIMITER '|');
-- \copy nation FROM 'data/tpch-sf1/nation.tbl' WITH (FORMAT csv, DELIMITER '|');
-- \copy region FROM 'data/tpch-sf1/region.tbl' WITH (FORMAT csv, DELIMITER '|');

-- Create indexes
CREATE INDEX idx_customer_nationkey ON customer(c_nationkey);
CREATE INDEX idx_orders_custkey ON orders(o_custkey);
CREATE INDEX idx_orders_date ON orders(o_orderdate);
CREATE INDEX idx_lineitem_orderkey ON lineitem(l_orderkey);
CREATE INDEX idx_lineitem_shipdate ON lineitem(l_shipdate);
CREATE INDEX idx_partsupp_partkey ON partsupp(ps_partkey);
CREATE INDEX idx_partsupp_suppkey ON partsupp(ps_suppkey);
CREATE INDEX idx_supplier_nationkey ON supplier(s_nationkey);
CREATE INDEX idx_nation_regionkey ON nation(n_regionkey);

VACUUM ANALYZE;