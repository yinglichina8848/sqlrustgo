#!/bin/bash
# SQLite Benchmark Comparison Script

echo "=== SQLRustGo vs SQLite Benchmark ==="
echo ""

SCALE=1
ITERATIONS=10

# 创建 SQLite 数据库
echo "创建 SQLite 数据库..."
rm -f /tmp/bench.db
sqlite3 /tmp/bench.db <<EOF
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
);

-- 插入测试数据
EOF

# 生成测试数据并插入
echo "插入测试数据..."
for i in $(seq 1 1000); do
    order_key=$((i % 10000 + 1))
    part_key=$((i % 200000 + 1))
    supp_key=$((i % 100 + 1))
    quantity=$((i % 50 + 1))
    extended_price=$((i % 10000 + 1))
    discount=$((i % 10))
    tax=$((i % 8 + 1))
    return_flag=$( [ $((i % 3)) -eq 0 ] && echo "R" || echo "N" )
    ship_date=$((87600 + i % 2000))
    
    sqlite3 /tmp/bench.db "INSERT INTO lineitem VALUES ($order_key, $part_key, $supp_key, $quantity, $extended_price, $discount, $tax, '$return_flag', $ship_date);"
done

echo "测试数据已插入: $(sqlite3 /tmp/bench.db 'SELECT COUNT(*) FROM lineitem') 行"
echo ""

# 测试 Q1 - 聚合查询
echo "=== SQLite Q1 测试 ==="
time sqlite3 /tmp/bench.db "SELECT l_returnflag, SUM(l_quantity) as sum_qty, SUM(l_extendedprice) as sum_base_price FROM lineitem WHERE l_returnflag = 'N' GROUP BY l_returnflag;"

echo ""
echo "=== SQLite Q6 测试 ==="
time sqlite3 /tmp/bench.db "SELECT SUM(l_extendedprice * (1 - l_discount / 100.0)) as revenue FROM lineitem WHERE l_quantity > 20;"

echo ""
echo "=== 完成 ==="
