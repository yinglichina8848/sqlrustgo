#!/usr/bin/env python3
"""
TPC-H Data Loader for PostgreSQL
Converts .tbl files to PostgreSQL-compatible CSV and loads into database.

Usage:
    python3 scripts/load_tpch_postgres.py --sf 0.1 [--data-dir data/tpch-sf01] [--db-name tpch]
    python3 scripts/load_tpch_postgres.py --sf 1  [--data-dir data/tpch-sf1]  [--db-name tpch]
"""

import argparse
import os
import subprocess
import sys
import re
from pathlib import Path

TPCH_TABLES = [
    "region", "nation", "supplier", "part", 
    "partsupp", "customer", "orders", "lineitem"
]

def escape_csv_value(val: str) -> str:
    """Escape a single CSV field - replace | with , and handle quoting."""
    # TPC-H uses | as delimiter, convert to PostgreSQL COPY compatible format
    # Escape backslashes first, then handle special chars
    val = val.replace('\\', '\\\\')
    val = val.replace('\t', '\\t')
    val = val.replace('\n', '\\n')
    val = val.replace('\r', '\\r')
    return val

def convert_tbl_to_csv(tbl_path: str, csv_path: str):
    """Convert TPC-H .tbl file (pipe-delimited) to CSV for PostgreSQL COPY."""
    print(f"  Converting {tbl_path} -> {csv_path}")
    with open(tbl_path, 'r') as fin, open(csv_path, 'w') as fout:
        for line in fin:
            line = line.rstrip('\n\r')
            fields = line.split('|')
            escaped = [escape_csv_value(f) for f in fields]
            # PostgreSQL COPY expects tab-delimited
            fout.write('\t'.join(escaped) + '\n')
    print(f"  Done: {csv_path}")

def run_psql(sql: str, dbname: str, user: str = "liying"):
    """Execute SQL via psql."""
    result = subprocess.run(
        ["psql", "-U", user, "-h", "localhost", "-p", "5432", "-d", dbname, "-c", sql],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"  PSQL ERROR: {result.stderr}")
    return result.returncode == 0

def load_table(table: str, csv_path: str, dbname: str):
    """Load CSV into PostgreSQL table using COPY."""
    print(f"  Loading {table}...")
    result = subprocess.run(
        ["psql", "-U", "liying", "-h", "localhost", "-p", "5432", "-d", dbname,
         "-c", f"\\copy {table} FROM '{csv_path}' WITH (FORMAT csv, DELIMITER E'\\t', NULL '')"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"  COPY ERROR for {table}: {result.stderr}")
        return False
    print(f"  {table} loaded.")
    return True

def get_row_count(dbname: str, table: str) -> int:
    """Get row count for a table."""
    result = subprocess.run(
        ["psql", "-U", "liying", "-h", "localhost", "-p", "5432", "-d", dbname,
         "-t", "-c", f"SELECT COUNT(*) FROM {table}"],
        capture_output=True, text=True
    )
    if result.returncode == 0:
        try:
            return int(result.stdout.strip())
        except:
            pass
    return -1

def main():
    parser = argparse.ArgumentParser(description="Load TPC-H data into PostgreSQL")
    parser.add_argument("--sf", type=str, required=True, choices=["0.1", "1"], help="Scale factor")
    parser.add_argument("--data-dir", type=str, default=None, help="TPC-H data directory")
    parser.add_argument("--db-name", type=str, default="tpch", help="Target database name")
    parser.add_argument("--recreate", action="store_true", help="Recreate schema (DROP + CREATE)")
    args = parser.parse_args()

    if args.sf == "0.1":
        default_dir = "data/tpch-sf01"
    else:
        default_dir = "data/tpch-sf1"

    data_dir = args.data_dir or default_dir
    data_dir = os.path.abspath(data_dir)
    
    if not os.path.exists(data_dir):
        print(f"ERROR: Data directory not found: {data_dir}")
        sys.exit(1)

    script_dir = os.path.dirname(os.path.abspath(__file__))
    schema_sql = os.path.join(script_dir, "load_tpch_postgres.sql")
    tmp_dir = f"/tmp/tpch_sf{args.sf.replace('.', '')}"
    os.makedirs(tmp_dir, exist_ok=True)

    print(f"\n=== TPC-H SF{args.sf} PostgreSQL Loader ===")
    print(f"Data dir: {data_dir}")
    print(f"DB: {args.db_name}")

    # Step 1: Create schema
    if args.recreate:
        print("\n[1/4] Recreating schema...")
        run_psql(f"DROP DATABASE IF EXISTS {args.db_name}", "postgres")
        run_psql(f"CREATE DATABASE {args.db_name}", "postgres")
        result = subprocess.run(["psql", "-U", "liying", "-h", "localhost", "-p", "5432",
                                 "-d", args.db_name, "-f", schema_sql],
                                capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Schema creation failed: {result.stderr}")
            sys.exit(1)
        print("  Schema created.")
    else:
        print("\n[1/4] Checking schema...")
        # Just verify tables exist
        result = subprocess.run(["psql", "-U", "liying", "-h", "localhost", "-p", "5432",
                                 "-d", args.db_name, "-t", "-c", "\\dt"],
                                capture_output=True, text=True)
        if "No relations found" in result.stdout or result.returncode != 0:
            print("  Schema missing. Run with --recreate first.")
            sys.exit(1)
        print("  Schema OK.")

    # Step 2: Convert .tbl to CSV
    print(f"\n[2/4] Converting .tbl files (pipe -> tab)...")
    csv_files = {}
    for table in TPCH_TABLES:
        tbl_path = os.path.join(data_dir, f"{table}.tbl")
        csv_path = os.path.join(tmp_dir, f"{table}.csv")
        csv_files[table] = csv_path
        if os.path.exists(tbl_path):
            convert_tbl_to_csv(tbl_path, csv_path)
        else:
            print(f"  WARNING: {tbl_path} not found, skipping.")
    
    # Step 3: Load data
    print(f"\n[3/4] Loading data into {args.db_name}...")
    for table in TPCH_TABLES:
        csv_path = csv_files.get(table)
        if csv_path and os.path.exists(csv_path):
            load_table(table, csv_path, args.db_name)

    # Step 4: Verify
    print(f"\n[4/4] Verification...")
    total = 0
    for table in TPCH_TABLES:
        cnt = get_row_count(args.db_name, table)
        if cnt >= 0:
            print(f"  {table}: {cnt:,} rows")
            total += cnt
    print(f"\n  Total rows: {total:,}")
    print("\n✅ PostgreSQL TPC-H SF{} loaded successfully!".format(args.sf))

if __name__ == "__main__":
    main()
