#!/usr/bin/env python3
"""
TPC-H Q1 Comparison: SQLRustGo vs PostgreSQL
"""

import time
import psycopg2

# Connect to PostgreSQL
pg_conn = psycopg2.connect(
    host="/var/run/postgresql",
    database="tpch_test",
    user="openclaw",
    password="openclaw123",
)

print("PostgreSQL TPC-H Q1 Comparison")
print("=" * 50)

# Q1: Pricing Summary Report
queries = {
    "Q1": "SELECT l_returnflag, SUM(l_quantity), SUM(l_extendedprice) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag",
}

for name, sql in queries.items():
    start = time.perf_counter()
    cur = pg_conn.cursor()
    cur.execute(sql)
    rows = cur.fetchall()
    elapsed = time.perf_counter() - start
    print(f"\n{name}: {elapsed * 1000:.2f}ms")
    print(f"  Rows: {len(rows)}")
    for row in rows:
        print(f"  {row}")

pg_conn.close()
print("\n" + "=" * 50)
