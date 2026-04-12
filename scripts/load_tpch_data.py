#!/usr/bin/env python3
"""Load TPC-H SF=0.1 data into SQLite for compliance testing."""

import sqlite3
import os

DATA_DIR = "data/tpch-sf01"
DB_PATH = f"{DATA_DIR}/tpch.db"


def create_schema(conn):
    """Create TPC-H schema."""
    cursor = conn.cursor()
    cursor.executescript("""
        CREATE TABLE IF NOT EXISTS region (
            r_regionkey INTEGER PRIMARY KEY,
            r_name TEXT,
            r_comment TEXT
        );
        
        CREATE TABLE IF NOT EXISTS nation (
            n_nationkey INTEGER PRIMARY KEY,
            n_name TEXT,
            n_regionkey INTEGER,
            n_comment TEXT
        );
        
        CREATE TABLE IF NOT EXISTS customer (
            c_custkey INTEGER PRIMARY KEY,
            c_name TEXT,
            c_address TEXT,
            c_nationkey INTEGER,
            c_phone TEXT,
            c_acctbal REAL,
            c_mktsegment TEXT,
            c_comment TEXT
        );
        
        CREATE TABLE IF NOT EXISTS supplier (
            s_suppkey INTEGER PRIMARY KEY,
            s_name TEXT,
            s_address TEXT,
            s_nationkey INTEGER,
            s_phone TEXT,
            s_acctbal REAL,
            s_comment TEXT
        );
        
        CREATE TABLE IF NOT EXISTS part (
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
        
        CREATE TABLE IF NOT EXISTS partsupp (
            ps_partkey INTEGER,
            ps_suppkey INTEGER,
            ps_availqty INTEGER,
            ps_supplycost REAL,
            ps_comment TEXT,
            PRIMARY KEY (ps_partkey, ps_suppkey)
        );
        
        CREATE TABLE IF NOT EXISTS orders (
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
        
        CREATE TABLE IF NOT EXISTS lineitem (
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
    """)
    conn.commit()


def load_table(conn, table_name, columns):
    """Load a single table from .tbl file."""
    filepath = f"{DATA_DIR}/{table_name}.tbl"
    if not os.path.exists(filepath):
        print(f"  Skipping {table_name} (not found)")
        return

    print(f"  Loading {table_name}...")

    with open(filepath, "r") as f:
        rows = []
        for line in f:
            # TPC-H files are pipe-delimited, strip trailing pipe and split
            line = line.strip()
            if line.endswith("|"):
                line = line[:-1]
            parts = line.split("|")
            # Only take the columns we need
            row = parts[: len(columns)]
            rows.append(tuple(row))

        placeholders = ",".join(["?"] * len(columns))
        insert_sql = f"INSERT INTO {table_name} VALUES ({placeholders})"
        cursor = conn.cursor()
        cursor.executemany(insert_sql, rows)
        print(f"    Loaded {len(rows)} rows into {table_name}")


def main():
    print("=== TPC-H Data Loader ===")
    print(f"Data directory: {DATA_DIR}")
    print(f"Database: {DB_PATH}")

    # Remove existing database
    if os.path.exists(DB_PATH):
        os.remove(DB_PATH)

    # Create connection
    conn = sqlite3.connect(DB_PATH)

    # Create schema
    print("Creating schema...")
    create_schema(conn)

    # Define tables and their columns
    tables = {
        "region": ["r_regionkey", "r_name", "r_comment"],
        "nation": ["n_nationkey", "n_name", "n_regionkey", "n_comment"],
        "customer": [
            "c_custkey",
            "c_name",
            "c_address",
            "c_nationkey",
            "c_phone",
            "c_acctbal",
            "c_mktsegment",
            "c_comment",
        ],
        "supplier": [
            "s_suppkey",
            "s_name",
            "s_address",
            "s_nationkey",
            "s_phone",
            "s_acctbal",
            "s_comment",
        ],
        "part": [
            "p_partkey",
            "p_name",
            "p_mfgr",
            "p_brand",
            "p_type",
            "p_size",
            "p_container",
            "p_retailprice",
            "p_comment",
        ],
        "partsupp": [
            "ps_partkey",
            "ps_suppkey",
            "ps_availqty",
            "ps_supplycost",
            "ps_comment",
        ],
        "orders": [
            "o_orderkey",
            "o_custkey",
            "o_orderstatus",
            "o_totalprice",
            "o_orderdate",
            "o_orderpriority",
            "o_clerk",
            "o_shippriority",
            "o_comment",
        ],
        "lineitem": [
            "l_orderkey",
            "l_partkey",
            "l_suppkey",
            "l_linenumber",
            "l_quantity",
            "l_extendedprice",
            "l_discount",
            "l_tax",
            "l_returnflag",
            "l_linestatus",
            "l_shipdate",
            "l_commitdate",
            "l_receiptdate",
            "l_shipinstruct",
            "l_shipmode",
            "l_comment",
        ],
    }

    # Load tables in order (respecting foreign keys)
    print("\nLoading tables...")
    for table_name, columns in tables.items():
        load_table(conn, table_name, columns)

    # Commit and vacuum
    print("\nOptimizing database...")
    conn.commit()
    conn.execute("VACUUM")

    # Print statistics
    print("\n=== Data Loaded ===")
    cursor = conn.cursor()
    cursor.execute("""
        SELECT 'region' as table_name, COUNT(*) FROM region
        UNION ALL SELECT 'nation', COUNT(*) FROM nation
        UNION ALL SELECT 'customer', COUNT(*) FROM customer
        UNION ALL SELECT 'supplier', COUNT(*) FROM supplier
        UNION ALL SELECT 'part', COUNT(*) FROM part
        UNION ALL SELECT 'partsupp', COUNT(*) FROM partsupp
        UNION ALL SELECT 'orders', COUNT(*) FROM orders
        UNION ALL SELECT 'lineitem', COUNT(*) FROM lineitem
    """)
    for row in cursor.fetchall():
        print(f"  {row[0]}: {row[1]} rows")

    conn.close()
    print(f"\nDatabase created: {DB_PATH}")
    print(f"Size: {os.path.getsize(DB_PATH) / 1024 / 1024:.1f} MB")


if __name__ == "__main__":
    main()
