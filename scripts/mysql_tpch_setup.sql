-- MySQL TPC-H Setup Script
-- Run: mysql -u root tpch_test < mysql_tpch_setup.sql

CREATE DATABASE IF NOT EXISTS tpch_test;
USE tpch_test;

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
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/customer.tbl' INTO TABLE customer FIELDS TERMINATED BY '|';
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/orders.tbl' INTO TABLE orders FIELDS TERMINATED BY '|';
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/lineitem.tbl' INTO TABLE lineitem FIELDS TERMINATED BY '|';
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/part.tbl' INTO TABLE part FIELDS TERMINATED BY '|';
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/partsupp.tbl' INTO TABLE partsupp FIELDS TERMINATED BY '|';
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/supplier.tbl' INTO TABLE supplier FIELDS TERMINATED BY '|';
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/nation.tbl' INTO TABLE nation FIELDS TERMINATED BY '|';
-- LOAD DATA LOCAL INFILE 'data/tpch-sf1/region.tbl' INTO TABLE region FIELDS TERMINATED BY '|';

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