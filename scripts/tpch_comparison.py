#!/usr/bin/env python3
"""
TPC-H Benchmark Comparison Script
Compares SQLRustGo against MySQL, PostgreSQL, and SQLite

Usage:
    python3 scripts/tpch_comparison.py [--iterations N] [--scale SF]
"""

import argparse
import time
import statistics
import sys
import os
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from typing import Dict, List, Optional

# Database drivers (install as needed)
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

try:
    import sqlite3

    SQLITE_AVAILABLE = True
except ImportError:
    SQLITE_AVAILABLE = False


@dataclass
class QueryResult:
    name: str
    times: List[float]
    rows: int
    passed: bool
    error: Optional[str] = None

    @property
    def avg_ms(self) -> float:
        return statistics.mean(self.times) * 1000

    @property
    def p50_ms(self) -> float:
        return statistics.median(self.times) * 1000

    @property
    def p95_ms(self) -> float:
        if len(self.times) < 2:
            return self.times[0] * 1000
        sorted_times = sorted(self.times)
        idx = int(len(sorted_times) * 0.95)
        return sorted_times[idx] * 1000

    @property
    def p99_ms(self) -> float:
        if len(self.times) < 2:
            return self.times[0] * 1000
        sorted_times = sorted(self.times)
        idx = int(len(sorted_times) * 0.99)
        return sorted_times[idx] * 1000


class DatabaseBenchmark:
    def __init__(self, name: str):
        self.name = name
        self.connection = None
        self.results: Dict[str, QueryResult] = {}

    def connect(self, **kwargs) -> bool:
        """Override in subclass"""
        raise NotImplementedError

    def disconnect(self):
        if self.connection:
            self.connection.close()

    def execute_query(self, sql: str) -> tuple:
        """Returns (rows, elapsed_time)"""
        raise NotImplementedError

    def run_benchmark(
        self, queries: Dict[str, str], iterations: int = 3
    ) -> Dict[str, QueryResult]:
        for name, sql in queries.items():
            times = []
            rows = 0
            passed = True
            error = None

            for i in range(iterations):
                try:
                    rows, elapsed = self.execute_query(sql)
                    times.append(elapsed)
                except Exception as e:
                    passed = False
                    error = str(e)
                    break

            self.results[name] = QueryResult(
                name=name, times=times, rows=rows, passed=passed, error=error
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
        start = time.perf_counter()
        cursor = self.connection.cursor()
        cursor.execute(sql)
        rows = cursor.fetchall()
        self.connection.commit()
        elapsed = time.perf_counter() - start
        return len(rows), elapsed


class MySQLBenchmark(DatabaseBenchmark):
    def __init__(
        self, host="localhost", user="root", password="", database="tpch_test"
    ):
        super().__init__("MySQL")
        self.host = host
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
                user=self.user,
                password=self.password,
                database=self.database,
            )
            return True
        except Exception as e:
            print(f"MySQL connection failed: {e}")
            return False

    def execute_query(self, sql: str) -> tuple:
        start = time.perf_counter()
        cursor = self.connection.cursor()
        cursor.execute(sql)
        rows = cursor.fetchall()
        self.connection.commit()
        elapsed = time.perf_counter() - start
        return len(rows), elapsed


class PostgreSQLBenchmark(DatabaseBenchmark):
    def __init__(
        self, host="localhost", user="postgres", password="", database="tpch_test"
    ):
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
        start = time.perf_counter()
        cursor = self.connection.cursor()
        cursor.execute(sql)
        rows = cursor.fetchall()
        self.connection.commit()
        elapsed = time.perf_counter() - start
        return len(rows), elapsed


# TPC-H Queries
TPCH_QUERIES = {
    "Q1": "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_shipdate <= '1995-12-01' GROUP BY l_returnflag",
    "Q2": "SELECT s_acctbal, s_name, n_name, p_partkey FROM part, supplier, partsupp, nation, region WHERE p_partkey = ps_partkey AND s_suppkey = ps_suppkey AND p_size = 15 AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'EUROPE' ORDER BY s_acctbal DESC LIMIT 10",
    "Q3": "SELECT o_orderkey, SUM(l_extendedprice) FROM orders JOIN lineitem ON o_orderkey = l_orderkey WHERE o_orderdate < '1995-03-15' GROUP BY o_orderkey",
    "Q4": "SELECT o_orderpriority, COUNT(*) FROM orders WHERE o_orderdate >= '1993-07-01' AND o_orderdate < '1993-10-01' GROUP BY o_orderpriority",
    "Q5": "SELECT n_name, SUM(l_extendedprice) FROM customer, orders, lineitem, supplier, nation, region WHERE c_custkey = o_custkey AND l_orderkey = o_orderkey AND l_suppkey = s_suppkey AND c_nationkey = s_nationkey AND s_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'ASIA' GROUP BY n_name",
    "Q6": "SELECT SUM(l_extendedprice) FROM lineitem WHERE l_quantity < 24 AND l_shipdate >= '1994-01-01'",
    "Q10": "SELECT c_custkey, COUNT(*) FROM customer GROUP BY c_custkey",
    "Q13": "SELECT c_mktsegment, COUNT(*) FROM customer GROUP BY c_mktsegment",
    "Q14": "SELECT p_type, COUNT(*) FROM part GROUP BY p_type",
    "Q19": "SELECT p_brand, SUM(p_retailprice) FROM part GROUP BY p_brand",
    "Q20": "SELECT s_nationkey, COUNT(*) FROM supplier GROUP BY s_nationkey",
    "Q22": "SELECT c_nationkey, COUNT(*) FROM customer WHERE c_acctbal > 0 GROUP BY c_nationkey",
}


def run_comparison(args):
    results = {}

    # SQLite Benchmark
    if args.sqlite:
        sqlite_bench = SQLiteBenchmark(args.sqlite)
        if sqlite_bench.connect():
            print("Running SQLite benchmark...")
            sqlite_bench.run_benchmark(TPCH_QUERIES, iterations=args.iterations)
            results["SQLite"] = sqlite_bench.results
            sqlite_bench.disconnect()

    # MySQL Benchmark
    if args.mysql and MYSQL_AVAILABLE:
        mysql_bench = MySQLBenchmark(
            host=args.mysql_host,
            user=args.mysql_user,
            password=args.mysql_password,
            database=args.mysql_db,
        )
        if mysql_bench.connect():
            print("Running MySQL benchmark...")
            mysql_bench.run_benchmark(TPCH_QUERIES, iterations=args.iterations)
            results["MySQL"] = mysql_bench.results
            mysql_bench.disconnect()

    # PostgreSQL Benchmark
    if args.pg and PG_AVAILABLE:
        pg_bench = PostgreSQLBenchmark(
            host=args.pg_host,
            user=args.pg_user,
            password=args.pg_password,
            database=args.pg_db,
        )
        if pg_bench.connect():
            print("Running PostgreSQL benchmark...")
            pg_bench.run_benchmark(TPCH_QUERIES, iterations=args.iterations)
            results["PostgreSQL"] = pg_bench.results
            pg_bench.disconnect()

    # Print comparison table
    print("\n" + "=" * 80)
    print("TPC-H BENCHMARK COMPARISON RESULTS")
    print("=" * 80)

    queries = list(TPCH_QUERIES.keys())

    # Header
    header = f"{'Query':<8}"
    for db_name in results.keys():
        header += f"{db_name + ' (ms)':<20}"
    print(header)
    print("-" * 80)

    # Results per query
    for query in queries:
        row = f"{query:<8}"
        for db_name, db_results in results.items():
            if query in db_results:
                r = db_results[query]
                if r.passed:
                    row += f"{r.avg_ms:<20.2f}"
                else:
                    row += f"{'ERROR':<20}"
            else:
                row += f"{'N/A':<20}"
        print(row)

    print("=" * 80)

    # Save to file
    if args.output:
        with open(args.output, "w") as f:
            f.write("# TPC-H Benchmark Comparison\n\n")
            f.write(f"## Configuration\n")
            f.write(f"- Iterations: {args.iterations}\n")
            f.write(f"- Scale Factor: {args.scale}\n\n")
            f.write("## Results\n\n")
            f.write("| Query | " + " | ".join(results.keys()) + " |\n")
            f.write("|-------|" + "|-----".join(["---" for _ in results]) + "|\n")
            for query in queries:
                row = f"| {query} |"
                for db_name, db_results in results.items():
                    if query in db_results:
                        r = db_results[query]
                        if r.passed:
                            row += f" {r.avg_ms:.2f} |"
                        else:
                            row += " ERROR |"
                    else:
                        row += " N/A |"
                f.write(row + "\n")

        print(f"\nResults saved to: {args.output}")


def main():
    parser = argparse.ArgumentParser(description="TPC-H Benchmark Comparison")
    parser.add_argument(
        "--iterations", type=int, default=3, help="Number of iterations per query"
    )
    parser.add_argument("--scale", type=str, default="SF1", help="Scale factor")

    # SQLite options
    parser.add_argument("--sqlite", type=str, help="SQLite database path")

    # MySQL options
    parser.add_argument("--mysql", action="store_true", help="Run MySQL benchmark")
    parser.add_argument("--mysql-host", default="localhost")
    parser.add_argument("--mysql-user", default="root")
    parser.add_argument("--mysql-password", default="")
    parser.add_argument("--mysql-db", default="tpch_test")

    # PostgreSQL options
    parser.add_argument("--pg", action="store_true", help="Run PostgreSQL benchmark")
    parser.add_argument("--pg-host", default="localhost")
    parser.add_argument("--pg-user", default="postgres")
    parser.add_argument("--pg-password", default="")
    parser.add_argument("--pg-db", default="tpch_test")

    parser.add_argument("--output", type=str, help="Output file for results")

    args = parser.parse_args()

    if not any([args.sqlite, args.mysql, args.pg]):
        print("No database specified. Use --sqlite, --mysql, or --pg")
        print("Example: python3 scripts/tpch_comparison.py --sqlite /tmp/tpch.db")
        sys.exit(1)

    run_comparison(args)


if __name__ == "__main__":
    main()
