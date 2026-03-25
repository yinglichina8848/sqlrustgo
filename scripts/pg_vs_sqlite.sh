#!/bin/bash
# Simple PostgreSQL vs SQLite benchmark
# 不依赖 SQLRustGo 的编译

echo "=== PostgreSQL vs SQLite Benchmark ==="
echo ""

SCALE=1000

# SQLite 测试
echo ">>> SQLite 测试"
rm -f /tmp/simple_bench.db
sqlite3 /tmp/simple_bench.db "PRAGMA journal_mode=WAL;"

sqlite3 /tmp/simple_bench.db "CREATE TABLE lineitem (
    l_orderkey INTEGER,
    l_partkey INTEGER,
    l_suppkey INTEGER,
    l_quantity REAL,
    l_extendedprice REAL,
    l_discount REAL,
    l_tax REAL,
    l_returnflag TEXT,
    l_shipdate INTEGER
);"

for i in $(seq 1 $SCALE); do
    order_key=$((i % 10000 + 1))
    part_key=$((i % 200000 + 1))
    supp_key=$((i % 100 + 1))
    quantity=$((i % 50 + 1))
    extended_price=$((i % 10000 + 1))
    discount=$((i % 10))
    tax=$((i % 8 + 1))
    return_flag=$( [ $((i % 3)) -eq 0 ] && echo "R" || echo "N" )
    ship_date=$((87600 + i % 2000))
    sqlite3 /tmp/simple_bench.db "INSERT INTO lineitem VALUES ($order_key, $part_key, $supp_key, $quantity, $extended_price, $discount, $tax, '$return_flag', $ship_date);"
done

START=$(date +%s%N)
sqlite3 /tmp/simple_bench.db "SELECT l_returnflag, SUM(l_quantity), SUM(l_extendedprice) FROM lineitem WHERE l_returnflag = 'N' GROUP BY l_returnflag;" > /dev/null 2>&1
END=$(date +%s%N)
SQLITE_Q1=$(( (END - START) / 1000000 ))

START=$(date +%s%N)
sqlite3 /tmp/simple_bench.db "SELECT SUM(l_extendedprice * (1 - l_discount / 100.0)) FROM lineitem WHERE l_quantity > 20;" > /dev/null 2>&1
END=$(date +%s%N)
SQLITE_Q6=$(( (END - START) / 1000000 ))

echo "  SQLite Q1: ${SQLITE_Q1}ms"
echo "  SQLite Q6: ${SQLITE_Q6}ms"

# PostgreSQL 测试
echo ""
echo ">>> PostgreSQL 测试"

# 创建表
psql -U benchmark -d sqlrustgo_benchmark -h localhost -c "DROP TABLE IF EXISTS lineitem;" > /dev/null 2>&1
psql -U benchmark -d sqlrustgo_benchmark -h localhost -c "CREATE TABLE lineitem (
    l_orderkey INTEGER,
    l_partkey INTEGER,
    l_suppkey INTEGER,
    l_quantity REAL,
    l_extendedprice REAL,
    l_discount REAL,
    l_tax REAL,
    l_returnflag TEXT,
    l_shipdate INTEGER
);" > /dev/null 2>&1

# 插入数据
for i in $(seq 1 $SCALE); do
    order_key=$((i % 10000 + 1))
    part_key=$((i % 200000 + 1))
    supp_key=$((i % 100 + 1))
    quantity=$((i % 50 + 1))
    extended_price=$((i % 10000 + 1))
    discount=$((i % 10))
    tax=$((i % 8 + 1))
    return_flag=$( [ $((i % 3)) -eq 0 ] && echo "R" || echo "N" )
    ship_date=$((87600 + i % 2000))
    psql -U benchmark -d sqlrustgo_benchmark -h localhost -c "INSERT INTO lineitem VALUES ($order_key, $part_key, $supp_key, $quantity, $extended_price, $discount, $tax, '$return_flag', $ship_date);" > /dev/null 2>&1
done

START=$(date +%s%N)
psql -U benchmark -d sqlrustgo_benchmark -h localhost -t -c "SELECT l_returnflag, SUM(l_quantity), SUM(l_extendedprice) FROM lineitem WHERE l_returnflag = 'N' GROUP BY l_returnflag;" > /dev/null 2>&1
END=$(date +%s%N)
PG_Q1=$(( (END - START) / 1000000 ))

START=$(date +%s%N)
psql -U benchmark -d sqlrustgo_benchmark -h localhost -t -c "SELECT SUM(l_extendedprice * (1 - l_discount / 100.0)) FROM lineitem WHERE l_quantity > 20;" > /dev/null 2>&1
END=$(date +%s%N)
PG_Q6=$(( (END - START) / 1000000 ))

echo "  PostgreSQL Q1: ${PG_Q1}ms"
echo "  PostgreSQL Q6: ${PG_Q6}ms"

# 汇总
echo ""
echo "========================================="
echo "  性能对比汇总"
echo "========================================="
echo ""
printf "| 系统      | Q1 (ms) | Q6 (ms) |\n"
printf "|-----------|----------|----------|\n"
printf "| SQLite    | %-8s | %-8s |\n" "$SQLITE_Q1" "$SQLITE_Q6"
printf "| PostgreSQL| %-8s | %-8s |\n" "$PG_Q1" "$PG_Q6"
echo ""

# 性能比较
if [ "$PG_Q1" -gt 0 ] && [ "$SQLITE_Q1" -gt 0 ]; then
    RATIO_Q1=$(echo "scale=2; $PG_Q1 / $SQLITE_Q1" | bc)
    RATIO_Q6=$(echo "scale=2; $PG_Q6 / $SQLITE_Q6" | bc)
    echo "PostgreSQL vs SQLite:"
    echo "  Q1: PostgreSQL 慢 ${RATIO_Q1}x"
    echo "  Q6: PostgreSQL 慢 ${RATIO_Q6}x"
fi

echo ""
echo "=== 完成 ==="
