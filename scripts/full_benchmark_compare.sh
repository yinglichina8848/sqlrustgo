#!/bin/bash
# Comprehensive Benchmark Comparison Script
# SQLRustGo vs SQLite vs PostgreSQL

echo "========================================="
echo "  SQLRustGo v1.6.1 综合性能对比"
echo "========================================="
echo ""

# 配置
ITERATIONS=10
SCALE=1000

echo "测试配置:"
echo "  - 数据规模: $SCALE 行"
echo "  - 迭代次数: $ITERATIONS"
echo ""

# ========================================
# 1. SQLRustGo 测试
# ========================================
echo ">>> 1. SQLRustGo Embedded 模式测试"
echo "----------------------------------------"

# 使用 criterion 进行测试
echo "运行 TPC-H Q1..."
RESULT_Q1=$(cargo bench --bench tpch_bench -- tpch_q1/pricing_summary 2>&1 | grep "time:" | head -1 | sed 's/.*\[//' | sed 's/ ms.*//')
echo "  Q1 延迟: $RESULT_Q1 ms"

echo "运行 TPC-H Q6..."
RESULT_Q6=$(cargo bench --bench tpch_bench -- tpch_q6/revenue_query 2>&1 | grep "time:" | head -1 | sed 's/.*\[//' | sed 's/ ms.*//')
echo "  Q6 延迟: $RESULT_Q6 ms"

echo ""

# ========================================
# 2. SQLite 测试
# ========================================
echo ">>> 2. SQLite 测试 (WAL 模式)"
echo "----------------------------------------"

rm -f /tmp/bench_compare.db
sqlite3 /tmp/bench_compare.db "PRAGMA journal_mode=WAL;"

# 创建表
sqlite3 /tmp/bench_compare.db "
CREATE TABLE lineitem (
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
    
    sqlite3 /tmp/bench_compare.db "INSERT INTO lineitem VALUES ($order_key, $part_key, $supp_key, $quantity, $extended_price, $discount, $tax, '$return_flag', $ship_date);"
done

# 测试 Q1
START=$(date +%s%N)
sqlite3 /tmp/bench_compare.db "SELECT l_returnflag, SUM(l_quantity), SUM(l_extendedprice) FROM lineitem WHERE l_returnflag = 'N' GROUP BY l_returnflag;" > /dev/null
END=$(date +%s%N)
SQLITE_Q1=$(( (END - START) / 1000000 ))
echo "  Q1 延迟: ${SQLITE_Q1}ms"

# 测试 Q6
START=$(date +%s%N)
sqlite3 /tmp/bench_compare.db "SELECT SUM(l_extendedprice * (1 - l_discount / 100.0)) FROM lineitem WHERE l_quantity > 20;" > /dev/null
END=$(date +%s%N)
SQLITE_Q6=$(( (END - START) / 1000000 ))
echo "  Q6 延迟: ${SQLITE_Q6}ms"

echo ""

# ========================================
# 3. PostgreSQL 测试 (如果可用)
# ========================================
echo ">>> 3. PostgreSQL 测试"
echo "----------------------------------------"

if pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    # PostgreSQL 可用
    PGPING="PostgreSQL 运行中"
    
    # 创建表
    psql -h localhost -U postgres -c "DROP TABLE IF EXISTS lineitem;" > /dev/null 2>&1
    psql -h localhost -U postgres -c "CREATE TABLE lineitem (l_orderkey INTEGER, l_partkey INTEGER, l_suppkey INTEGER, l_quantity REAL, l_extendedprice REAL, l_discount REAL, l_tax REAL, l_returnflag TEXT, l_shipdate INTEGER);" > /dev/null 2>&1
    
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
        
        psql -h localhost -U postgres -c "INSERT INTO lineitem VALUES ($order_key, $part_key, $supp_key, $quantity, $extended_price, $discount, $tax, '$return_flag', $ship_date);" > /dev/null 2>&1
    done
    
    # 测试 Q1
    START=$(date +%s%N)
    psql -h localhost -U postgres -t -c "SELECT l_returnflag, SUM(l_quantity), SUM(l_extendedprice) FROM lineitem WHERE l_returnflag = 'N' GROUP BY l_returnflag;" > /dev/null 2>&1
    END=$(date +%s%N)
    PG_Q1=$(( (END - START) / 1000000 ))
    echo "  Q1 延迟: ${PG_Q1}ms"
    
    # 测试 Q6
    START=$(date +%s%N)
    psql -h localhost -U postgres -t -c "SELECT SUM(l_extendedprice * (1 - l_discount / 100.0)) FROM lineitem WHERE l_quantity > 20;" > /dev/null 2>&1
    END=$(date +%s%N)
    PG_Q6=$(( (END - START) / 1000000 ))
    echo "  Q6 延迟: ${PG_Q6}ms"
else
    echo "  PostgreSQL 未运行 (跳过)"
    PG_Q1="N/A"
    PG_Q6="N/A"
fi

echo ""

# ========================================
# 4. 结果汇总
# ========================================
echo "========================================="
echo "  性能对比汇总"
echo "========================================="
echo ""
printf "| 系统      | Q1 (ms) | Q6 (ms) |\n"
printf "|-----------|----------|----------|\n"
printf "| SQLRustGo | %-8s | %-8s |\n" "$RESULT_Q1" "$RESULT_Q6"
printf "| SQLite    | %-8s | %-8s |\n" "$SQLITE_Q1" "$SQLITE_Q6"
printf "| PostgreSQL| %-8s | %-8s |\n" "$PG_Q1" "$PG_Q6"
echo ""

echo "========================================="
echo "  测试完成"
echo "========================================="
