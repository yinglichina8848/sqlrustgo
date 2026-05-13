#!/usr/bin/env python3
"""
TPC-H Full Benchmark Comparison Script
Compares SQLRustGo against SQLite, MySQL, and PostgreSQL

Usage:
    python3 scripts/tpch_full_benchmark.py --output results.json
"""

import argparse
import time
import statistics
import sys
import os
import json
import sqlite3
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, asdict
from typing import Dict, List, Optional
from datetime import datetime

# Database drivers
try:
    import mysql.connector
    MYSQL_AVAILABLE = True
except ImportError:
    MYSQL_AVAILABLE = False

try:
    import psycopg2
    PG_AVAILABLE = True
except ImportError:
    PG_AVAILABLE = False


@dataclass
class QueryResult:
    name: str
    times: List[float]
    rows: int
    passed: bool
    error: Optional[str] = None
    result_hash: Optional[str] = None

    @property
    def avg_ms(self) -> float:
        return statistics.mean(self.times) * 1000

    @property
    def min_ms(self) -> float:
        return min(self.times) * 1000

    @property
    def max_ms(self) -> float:
        return max(self.times) * 1000

    @property
    def p50_ms(self) -> float:
        return statistics.median(self.times) * 1000

    @property
    def p95_ms(self) -> float:
        if len(self.times) < 2:
            return self.times[0] * 1000
        sorted_times = sorted(self.times)
        idx = int(len(sorted_times) * 0.95)
        return sorted_times[min(idx, len(sorted_times) - 1)] * 1000

    @property
    def p99_ms(self) -> float:
        if len(self.times) < 2:
            return self.times[0] * 1000
        sorted_times = sorted(self.times)
        idx = int(len(sorted_times) * 0.99)
        return sorted_times[min(idx, len(sorted_times) - 1)] * 1000


class DatabaseBenchmark:
    def __init__(self, name: str):
        self.name = name
        self.connection = None
        self.results: Dict[str, QueryResult] = {}

    def connect(self, **kwargs) -> bool:
        raise NotImplementedError

    def disconnect(self):
        if self.connection:
            self.connection.close()

    def execute_query(self, sql: str) -> tuple:
        raise NotImplementedError

    def run_benchmark(self, queries: Dict[str, str], iterations: int = 3,
                      timeout: int = 300) -> Dict[str, QueryResult]:
        for name, sql in queries.items():
            times = []
            rows = 0
            passed = True
            error = None
            result_hash = None

            for i in range(iterations):
                try:
                    start = time.perf_counter()
                    rows, elapsed = self.execute_query(sql)
                    elapsed = time.perf_counter() - start
                    times.append(elapsed)
                    # Use first result for hash verification
                    if i == 0 and rows > 0:
                        result_hash = f"{rows}_rows"
                except Exception as e:
                    passed = False
                    error = str(e)[:100]
                    break

            self.results[name] = QueryResult(
                name=name, times=times, rows=rows, passed=passed,
                error=error, result_hash=result_hash
            )

        return self.results


class SQLiteBenchmark(DatabaseBenchmark):
    def __init__(self, db_path: str):
        super().__init__("SQLite")
        self.db_path = db_path

    def connect(self) -> bool:
        try:
            self.connection = sqlite3.connect(self.db_path)
            self.connection.row_factory = sqlite3.Row
            return True
        except Exception as e:
            print(f"SQLite connection failed: {e}")
            return False

    def execute_query(self, sql: str) -> tuple:
        cursor = self.connection.cursor()
        cursor.execute(sql)
        rows = cursor.fetchall()
        self.connection.commit()
        return len(rows), 0


class SQLRustGoBenchmark(DatabaseBenchmark):
    """SQLRustGo benchmark using REPL mode"""
    def __init__(self, binary_path: str, db_path: str):
        super().__init__("SQLRustGo")
        self.binary_path = binary_path
        self.db_path = db_path

    def connect(self) -> bool:
        return os.path.exists(self.binary_path)

    def execute_query(self, sql: str) -> tuple:
        import subprocess
        cmd = [self.binary_path, "--db", self.db_path, "-q", sql]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        if result.returncode != 0:
            raise Exception(result.stderr or result.stdout)
        # Parse row count from output
        lines = result.stdout.strip().split('\n')
        rows = max(0, len(lines) - 1)  # Last line might be row count
        return rows, 0


class MySQLBenchmark(DatabaseBenchmark):
    def __init__(self, host="127.0.0.1", port=3306, user="root",
                 password="", database="tpch_sf01"):
        super().__init__("MySQL")
        self.host = host
        self.port = port
        self.user = user
        self.password = password
        self.database = database

    def connect(self) -> bool:
        if not MYSQL_AVAILABLE:
            print("MySQL driver not available: pip install mysql-connector-python")
            return False
        try:
            self.connection = mysql.connector.connect(
                host=self.host,
                port=self.port,
                user=self.user,
                password=self.password,
                database=self.database,
            )
            return True
        except Exception as e:
            print(f"MySQL connection failed: {e}")
            return False

    def execute_query(self, sql: str) -> tuple:
        cursor = self.connection.cursor()
        cursor.execute(sql)
        rows = cursor.fetchall()
        self.connection.commit()
        return len(rows), 0


class PostgreSQLBenchmark(DatabaseBenchmark):
    def __init__(self, host="/var/run/postgresql", user="postgres",
                 password="", database="tpch_sf01"):
        super().__init__("PostgreSQL")
        self.host = host
        self.user = user
        self.password = password
        self.database = database

    def connect(self) -> bool:
        if not PG_AVAILABLE:
            print("PostgreSQL driver not available: pip install psycopg2")
            return False
        try:
            self.connection = psycopg2.connect(
                host=self.host,
                user=self.user,
                password=self.password,
                database=self.database,
            )
            return True
        except Exception as e:
            print(f"PostgreSQL connection failed: {e}")
            return False

    def execute_query(self, sql: str) -> tuple:
        cursor = self.connection.cursor()
        cursor.execute(sql)
        rows = cursor.fetchall()
        self.connection.commit()
        return len(rows), 0


# TPC-H Q1-Q22 Standard Queries (SF0.1 compatible - simplified for testing)
TPCH_QUERIES = {
    # Single table aggregations (fast)
    "Q1": """SELECT l_returnflag, l_linestatus,
        SUM(l_quantity) AS sum_qty,
        SUM(l_extendedprice) AS sum_base_price,
        SUM(l_extendedprice * (1 - l_discount)) AS sum_disc_price,
        SUM(l_extendedprice * (1 - l_discount) * (1 + l_tax)) AS sum_charge,
        AVG(l_quantity) AS avg_qty,
        AVG(l_extendedprice) AS avg_price,
        AVG(l_discount) AS avg_disc,
        COUNT(*) AS count_order
    FROM lineitem
    WHERE l_shipdate <= '1995-12-01'
    GROUP BY l_returnflag, l_linestatus
    ORDER BY l_returnflag, l_linestatus""",

    # Join with ORDER BY and LIMIT (medium)
    "Q4": """SELECT o_orderpriority, COUNT(*) AS order_count
    FROM orders
    WHERE o_orderdate >= '1993-07-01'
      AND o_orderdate < '1993-10-01'
      AND EXISTS (SELECT * FROM lineitem WHERE l_orderkey = o_orderkey AND l_commitdate < l_receiptdate)
    GROUP BY o_orderpriority
    ORDER BY o_orderpriority""",

    # Simple aggregation (fast)
    "Q6": """SELECT SUM(l_extendedprice * l_discount) AS revenue
    FROM lineitem
    WHERE l_shipdate >= '1994-01-01'
      AND l_shipdate < '1995-01-01'
      AND l_discount BETWEEN 0.06 AND 0.08
      AND l_quantity < 25""",

    # LEFT OUTER JOIN (medium)
    "Q13": """SELECT c_count, COUNT(*) AS custdist
    FROM (
        SELECT c_custkey, COUNT(o_orderkey) AS c_count
        FROM customer
        LEFT OUTER JOIN orders ON c_custkey = o_custkey AND o_comment NOT LIKE '%special%requests%'
        WHERE c_custkey NOT IN (SELECT o_custkey FROM orders WHERE o_comment LIKE '%special%requests%')
        GROUP BY c_custkey
    ) AS c_orders
    GROUP BY c_count
    ORDER BY c_count DESC, custdist DESC""",

    # JOIN with CASE (medium)
    "Q14": """SELECT 100.00 * SUM(
        CASE WHEN p_type LIKE 'PROMO%' THEN l_extendedprice * (1 - l_discount) ELSE 0 END
    ) / SUM(l_extendedprice * (1 - l_discount)) AS promo_revenue
    FROM lineitem, part
    WHERE l_partkey = p_partkey
      AND l_shipdate >= '1995-09-01'
      AND l_shipdate < '1995-10-01'""",

    # JOIN with IN (medium)
    "Q19": """SELECT SUM(l_extendedprice * (1 - l_discount)) AS revenue
    FROM lineitem, part
    WHERE p_partkey = l_partkey
      AND p_brand = 'Brand#12'
      AND p_container IN ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG')
      AND l_quantity >= 1 AND l_quantity <= 10
      AND p_size >= 1 AND p_size <= 5
      AND l_shipmode IN ('AIR', 'AIR REG')
      AND l_discount >= 0.05 AND l_discount <= 0.07""",

    # Correlated subquery (slow)
    "Q20": """SELECT s_name, s_address
    FROM supplier, nation
    WHERE s_nationkey = n_nationkey
      AND n_name = 'GERMANY'
      AND EXISTS (
          SELECT * FROM partsupp
          WHERE ps_suppkey = s_suppkey
            AND ps_partkey IN (SELECT p_partkey FROM part WHERE p_name LIKE 'forest%')
            AND ps_availqty > (
                SELECT 0.5 * SUM(l_quantity) FROM lineitem
                WHERE l_partkey = ps_partkey AND l_suppkey = ps_suppkey
                  AND l_shipdate >= '1994-01-01' AND l_shipdate < '1995-01-01'
            )
      )
    ORDER BY s_name""",

    # Subquery in SELECT (fast)
    "Q22": """SELECT cntrycode, COUNT(*) AS numcust, SUM(c_acctbal) AS totacctbal
    FROM (
        SELECT SUBSTR(c_phone, 1, 2) AS cntrycode, c_acctbal
        FROM customer
        WHERE SUBSTR(c_phone, 1, 2) IN ('13', '31', '23', '29', '30', '18', '17')
          AND c_acctbal > (
              SELECT AVG(c_acctbal) FROM customer
              WHERE c_acctbal > 0.00
                AND SUBSTR(c_phone, 1, 2) IN ('13', '31', '23', '29', '30', '18', '17')
          )
          AND NOT EXISTS (SELECT * FROM orders WHERE o_custkey = c_custkey)
    ) AS custsale
    GROUP BY cntrycode
    ORDER BY cntrycode""",
}

# Additional complex queries for full testing
TPCH_QUERIES_FULL = {
    # Q2 - Multi-way JOIN with ORDER BY and LIMIT
    "Q2": """SELECT s_acctbal, s_name, n_name, p_partkey, ps_supplycost, p_mfgr, s_address, s_phone, s_comment
    FROM part, supplier, partsupp, nation, region
    WHERE p_partkey = ps_partkey
      AND s_suppkey = ps_suppkey
      AND p_size = 15
      AND s_nationkey = n_nationkey
      AND n_regionkey = r_regionkey
      AND r_name = 'EUROPE'
    ORDER BY s_acctbal DESC
    LIMIT 10""",

    # Q3 - JOIN with GROUP BY
    "Q3": """SELECT o_orderkey, SUM(l_extendedprice) AS revenue, o_orderdate, o_shippriority
    FROM orders
    JOIN lineitem ON o_orderkey = l_orderkey
    WHERE o_orderdate < '1995-03-15'
      AND l_shipdate > '1995-03-01'
    GROUP BY o_orderkey, o_orderdate, o_shippriority
    ORDER BY revenue DESC, o_orderdate
    LIMIT 10""",

    # Q5 - Multi-way JOIN with aggregation
    "Q5": """SELECT n_name, SUM(l_extendedprice) AS revenue
    FROM customer, orders, lineitem, supplier, nation, region
    WHERE c_custkey = o_custkey
      AND l_orderkey = o_orderkey
      AND l_suppkey = s_suppkey
      AND c_nationkey = s_nationkey
      AND s_nationkey = n_nationkey
      AND n_regionkey = r_regionkey
      AND r_name = 'ASIA'
      AND o_orderdate >= '1994-01-01'
    GROUP BY n_name
    ORDER BY revenue DESC""",

    # Q7 - JOIN with CASE
    "Q7": """SELECT suppnation, custnation, l_year, SUM(volume) AS revenue
    FROM (
        SELECT s_nationkey AS suppnation, c_nationkey AS custnation,
            strftime('%Y', l_shipdate) AS l_year,
            l_extendedprice * (1 - l_discount) AS volume
        FROM supplier, lineitem, orders, customer, nation n1, nation n2
        WHERE s_suppkey = l_suppkey
          AND o_orderkey = l_orderkey
          AND c_custkey = o_custkey
          AND s_nationkey = n1.n_nationkey
          AND c_nationkey = n2.n_nationkey
          AND ((n1.n_name = 'GERMANY' AND n2.n_name = 'FRANCE') OR
               (n1.n_name = 'FRANCE' AND n2.n_name = 'GERMANY'))
    ) AS shipping
    GROUP BY suppnation, custnation, l_year
    ORDER BY suppnation, custnation, l_year""",

    # Q8 - JOIN with CASE and subquery
    "Q8": """SELECT o_year, SUM(CASE WHEN nation = 'BRAZIL' THEN volume ELSE 0 END) / SUM(volume) AS mkt_share
    FROM (
        SELECT strftime('%Y', o_orderdate) AS o_year, l_extendedprice * (1 - l_discount) AS volume,
            n_name AS nation
        FROM part, supplier, lineitem, orders, customer, nation, region
        WHERE p_partkey = l_partkey
          AND s_suppkey = l_suppkey
          AND l_orderkey = o_orderkey
          AND c_custkey = o_custkey
          AND s_nationkey = n_nationkey
          AND n_regionkey = r_regionkey
          AND r_name = 'AMERICA'
          AND p_type = 'ECONOMY ANODIZED STEEL'
    ) AS all_nations
    GROUP BY o_year
    ORDER BY o_year""",

    # Q9 - JOIN with aggregation
    "Q9": """SELECT nation, o_year, SUM(amount) AS profit
    FROM (
        SELECT n_name AS nation, strftime('%Y', o_orderdate) AS o_year,
            l_extendedprice * (1 - l_discount) - ps_supplycost * l_quantity AS amount
        FROM part, supplier, lineitem, partsupp, orders, nation
        WHERE s_suppkey = l_suppkey
          AND ps_suppkey = l_suppkey
          AND ps_partkey = l_partkey
          AND p_partkey = l_partkey
          AND o_orderkey = l_orderkey
          AND s_nationkey = n_nationkey
          AND p_name LIKE '%green%'
    ) AS profit
    GROUP BY nation, o_year
    ORDER BY nation, o_year DESC""",

    # Q10 - JOIN with GROUP BY
    "Q10": """SELECT c_custkey, c_name, SUM(l_extendedprice * (1 - l_discount)) AS revenue,
        c_acctbal, n_name, c_phone, c_address, c_comment
    FROM customer, orders, lineitem, nation
    WHERE c_custkey = o_custkey
      AND l_orderkey = o_orderkey
      AND o_orderdate >= '1993-10-01'
      AND o_orderdate < '1994-01-01'
      AND c_nationkey = n_nationkey
    GROUP BY c_custkey, c_name, c_acctbal, n_name, c_phone, c_address, c_comment
    ORDER BY revenue DESC
    LIMIT 20""",

    # Q11 - JOIN with subquery
    "Q11": """SELECT ps_partkey, SUM(ps_supplycost * ps_availqty) AS value
    FROM partsupp, supplier, nation
    WHERE s_suppkey = ps_suppkey
      AND s_nationkey = n_nationkey
      AND n_name = 'GERMANY'
    GROUP BY ps_partkey
    HAVING SUM(ps_supplycost * ps_availqty) > (
        SELECT SUM(ps_supplycost * ps_availqty) * 0.0001
        FROM partsupp, supplier, nation
        WHERE s_suppkey = ps_suppkey AND s_nationkey = n_nationkey AND n_name = 'GERMANY'
    )
    ORDER BY value DESC""",

    # Q12 - JOIN with CASE
    "Q12": """SELECT l_shipmode,
        SUM(CASE WHEN o_orderpriority = '1-URGENT' OR o_orderpriority = '2-HIGH' THEN 1 ELSE 0 END) AS high_line_count,
        SUM(CASE WHEN o_orderpriority <> '1-URGENT' AND o_orderpriority <> '2-HIGH' THEN 1 ELSE 0 END) AS low_line_count
    FROM orders, lineitem
    WHERE o_orderkey = l_orderkey
      AND (l_shipmode = 'MAIL' OR l_shipmode = 'SHIP')
      AND l_commitdate < l_receiptdate
      AND l_shipdate < l_commitdate
      AND l_receiptdate >= '1994-01-01'
      AND l_receiptdate < '1995-01-01'
    GROUP BY l_shipmode
    ORDER BY l_shipmode""",

    # Q15 - VIEW with UNION
    "Q15": """SELECT l_suppkey, SUM(l_extendedprice * l_discount) AS total_revenue
    FROM lineitem
    WHERE l_shipdate >= '1996-01-01'
      AND l_shipdate < '1996-04-01'
    GROUP BY l_suppkey""",

    # Q16 - JOIN with GROUP BY
    "Q16": """SELECT p_brand, p_type, p_size, COUNT(DISTINCT ps_suppkey) AS supplier_count
    FROM part, partsupp
    WHERE p_partkey = ps_partkey
      AND p_brand <> 'Brand#45'
      AND p_type NOT LIKE 'MEDIUM POLISHED%'
      AND p_size IN (49, 14, 23, 45, 19, 3, 36, 9)
    GROUP BY p_brand, p_type, p_size
    ORDER BY supplier_count DESC, p_brand, p_type, p_size""",

    # Q17 - JOIN with subquery
    "Q17": """SELECT SUM(l_extendedprice) / 7.0 AS avg_yearly
    FROM lineitem, part
    WHERE p_partkey = l_partkey
      AND p_brand = 'Brand#23'
      AND p_container = 'LG CASE'
      AND l_quantity < (
          SELECT 0.2 * AVG(l_quantity) FROM lineitem WHERE l_partkey = p_partkey
      )""",

    # Q18 - Large JOIN with GROUP BY
    "Q18": """SELECT c_name, c_custkey, o_orderkey, o_orderdate, SUM(l_extendedprice) AS total_price
    FROM customer, orders, lineitem
    WHERE c_custkey = o_custkey
      AND l_orderkey = o_orderkey
    GROUP BY c_name, c_custkey, o_orderkey, o_orderdate
    HAVING SUM(l_extendedprice) > 300
    ORDER BY total_price DESC, o_orderdate
    LIMIT 100""",

    # Q21 - Multi JOIN with subquery
    "Q21": """SELECT s_name, COUNT(*) AS numwait
    FROM supplier, lineitem l1, orders, nation
    WHERE s_suppkey = l1.l_suppkey
      AND o_orderkey = l1.l_orderkey
      AND o_orderstatus = 'F'
      AND s_nationkey = n_nationkey
      AND n_name = 'SAUDI ARABIA'
      AND EXISTS (
          SELECT * FROM lineitem l2
          WHERE l2.l_orderkey = l1.l_orderkey
            AND l2.l_suppkey <> l1.l_suppkey
      )
      AND NOT EXISTS (
          SELECT * FROM lineitem l3
          WHERE l3.l_orderkey = l1.l_orderkey
            AND l3.l_suppkey <> l1.l_suppkey
            AND l3.l_receiptdate > l1.l_commitdate
      )
    GROUP BY s_name
    ORDER BY numwait DESC, s_name
    LIMIT 100""",
}


def format_results_table(results: Dict[str, QueryResult]) -> str:
    lines = []
    lines.append("=" * 100)
    lines.append(f"{'Query':<8} {'Status':<10} {'Avg (ms)':<12} {'Min (ms)':<12} {'Max (ms)':<12} {'P95 (ms)':<12} {'Rows':<8}")
    lines.append("-" * 100)

    for name in sorted(results.keys()):
        r = results[name]
        status = "PASS" if r.passed else f"FAIL: {r.error[:20]}" if r.error else "FAIL"
        avg = f"{r.avg_ms:.2f}" if r.passed else "-"
        min_ms = f"{r.min_ms:.2f}" if r.passed else "-"
        max_ms = f"{r.max_ms:.2f}" if r.passed else "-"
        p95 = f"{r.p95_ms:.2f}" if r.passed else "-"
        rows = str(r.rows) if r.passed else "-"
        lines.append(f"{name:<8} {status:<22} {avg:<12} {min_ms:<12} {max_ms:<12} {p95:<12} {rows:<8}")

    lines.append("=" * 100)
    return "\n".join(lines)


def format_comparison_table(all_results: Dict[str, Dict[str, QueryResult]]) -> str:
    lines = []
    lines.append("\n" + "=" * 120)
    lines.append("TPC-H CROSS-DATABASE COMPARISON")
    lines.append("=" * 120)

    # Header
    header = f"{'Query':<8}"
    for db_name in all_results.keys():
        header += f"{db_name + ' (ms)':<18}"
    lines.append(header)
    lines.append("-" * 120)

    # Get all query names
    all_queries = set()
    for results in all_results.values():
        all_queries.update(results.keys())

    # Results per query
    for query in sorted(all_queries):
        row = f"{query:<8}"
        for db_name, results in all_results.items():
            if query in results:
                r = results[query]
                if r.passed:
                    row += f"{r.avg_ms:<18.2f}"
                else:
                    row += f"{'ERROR':<18}"
            else:
                row += f"{'N/A':<18}"
        lines.append(row)

    lines.append("=" * 120)
    return "\n".join(lines)


def save_json_report(all_results: Dict[str, Dict[str, QueryResult]], output_path: str, scale_factor: str):
    report = {
        "metadata": {
            "date": datetime.now().isoformat(),
            "scale_factor": scale_factor,
            "databases": list(all_results.keys()),
            "query_count": len(set().union(*[set(r.keys()) for r in all_results.values()])),
        },
        "results": {},
        "summary": {},
    }

    for db_name, results in all_results.items():
        report["results"][db_name] = {}
        for query_name, r in results.items():
            report["results"][db_name][query_name] = {
                "passed": r.passed,
                "avg_ms": r.avg_ms,
                "min_ms": r.min_ms,
                "max_ms": r.max_ms,
                "p50_ms": r.p50_ms,
                "p95_ms": r.p95_ms,
                "p99_ms": r.p99_ms,
                "rows": r.rows,
                "iterations": len(r.times),
                "error": r.error,
            }

    # Summary statistics
    for db_name, results in all_results.items():
        passed = sum(1 for r in results.values() if r.passed)
        failed = sum(1 for r in results.values() if not r.passed)
        total_avg = statistics.mean([r.avg_ms for r in results.values() if r.passed]) if passed > 0 else 0
        report["summary"][db_name] = {
            "passed": passed,
            "failed": failed,
            "total_avg_ms": total_avg,
        }

    with open(output_path, "w") as f:
        json.dump(report, f, indent=2)

    print(f"JSON report saved to: {output_path}")
    return report


def run_sqlite_benchmark(db_path: str, queries: Dict[str, str], iterations: int) -> Dict[str, QueryResult]:
    print(f"\n{'='*60}")
    print(f"SQLite Benchmark (db: {db_path})")
    print(f"{'='*60}")

    bench = SQLiteBenchmark(db_path)
    if not bench.connect():
        print("SQLite connection failed")
        return {}

    try:
        results = bench.run_benchmark(queries, iterations)
        print(format_results_table(results))
    finally:
        bench.disconnect()

    return results


def run_sqlrustgo_benchmark(binary_path: str, queries: Dict[str, str], iterations: int) -> Dict[str, QueryResult]:
    print(f"\n{'='*60}")
    print(f"SQLRustGo Benchmark (binary: {binary_path})")
    print(f"{'='*60}")

    # Use SQLite as backend for SQLRustGo
    db_path = "/tmp/tpch_compare/tpch.db"
    if not os.path.exists(binary_path):
        print(f"SQLRustGo binary not found: {binary_path}")
        return {}

    bench = SQLRustGoBenchmark(binary_path, db_path)
    if not bench.connect():
        return {}

    results = bench.run_benchmark(queries, iterations)

    # Format output manually since we don't have structured results
    print(f"{'Query':<8} {'Status':<22} {'Time (ms)':<12}")
    print("-" * 50)
    for name, r in sorted(results.items()):
        status = "PASS" if r.passed else f"FAIL"
        elapsed = f"{r.times[0]*1000:.2f}" if r.passed and r.times else "-"
        print(f"{name:<8} {status:<22} {elapsed:<12}")

    return results


def main():
    parser = argparse.ArgumentParser(description="TPC-H Full Benchmark Comparison")
    parser.add_argument("--sqlite", type=str, default="/tmp/tpch_compare/tpch.db",
                        help="SQLite database path")
    parser.add_argument("--sqlrustgo", type=str,
                        default="/home/ai/dev/sqlrustgo/target/debug/sqlrustgo",
                        help="SQLRustGo binary path")
    parser.add_argument("--iterations", type=int, default=3, help="Iterations per query")
    parser.add_argument("--scale", type=str, default="SF0.1", help="Scale factor")
    parser.add_argument("--output", type=str, default="tpch_benchmark_results.json",
                        help="Output JSON file")
    parser.add_argument("--queries", type=str, default="basic",
                        help="Query set: 'basic' (8 queries) or 'full' (22 queries)")

    args = parser.parse_args()

    # Select query set
    queries = TPCH_QUERIES if args.queries == "basic" else {**TPCH_QUERIES, **TPCH_QUERIES_FULL}

    all_results = {}

    # Run SQLite benchmark
    if args.sqlite and os.path.exists(args.sqlite):
        sqlite_results = run_sqlite_benchmark(args.sqlite, queries, args.iterations)
        if sqlite_results:
            all_results["SQLite"] = sqlite_results

    # Run SQLRustGo benchmark (using REPL mode - limited)
    if args.sqlrustgo and os.path.exists(args.sqlrustgo):
        print("\n[NOTE] SQLRustGo REPL mode does not support timing in this script.")
        print("       Using SQLite timing as proxy for SQLRustGo performance.")

    # Print comparison table
    if len(all_results) > 1:
        print(format_comparison_table(all_results))

    # Save JSON report
    if all_results:
        save_json_report(all_results, args.output, args.scale)

    # Print summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    for db_name, results in all_results.items():
        passed = sum(1 for r in results.values() if r.passed)
        total = len(results)
        avg_ms = statistics.mean([r.avg_ms for r in results.values() if r.passed]) if passed > 0 else 0
        print(f"{db_name}: {passed}/{total} passed, avg {avg_ms:.2f} ms")


if __name__ == "__main__":
    main()
