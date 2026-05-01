# TPC-H 测试指南

本文档说明如何运行 TPC-H SF=1 和 SF=10 基准测试。

## 目录

- [环境准备](#环境准备)
- [SF=1 测试](#sf1-测试)
- [SF=10 测试](#sf10-测试)
- [快速导入工具](#快速导入工具)
- [对比测试](#对比测试)
- [常见问题](#常见问题)

---

## 环境准备

### 1. 安装 MySQL (可选，用于对比测试)

```bash
# 安装 MySQL
sudo apt install mysql-server

# 启动 MySQL
sudo service mysql start

# 配置 root 密码
mysql -u root -e "ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY 'details';"
```

### 2. 生成 TPC-H 数据

使用 `dbgen` 工具生成 SF=1 或 SF=10 数据:

```bash
# 克隆 TPC-H dbgen
cd /tmp
git clone https://github.com/electrum/tpch-dbgen.git
cd tpch-dbgen

# 编译
make

# 生成 SF=1 数据
mkdir -p sf1
./dbgen -s 1 -f -d
mv *.tbl sf1/

# 生成 SF=10 数据
mkdir -p sf10
./dbgen -s 10 -f -d
mv *.tbl sf10/
```

### 3. 加载数据到 MySQL (可选)

```bash
# 创建数据库
mysql -u root -p'details' -e "CREATE DATABASE IF NOT EXISTS tpch_sf1;"

# 创建表
mysql -u root -p'details' tpch_sf1 < scripts/mysql_tpch_setup.sql

# 加载数据
for table in nation region supplier part partsupp customer orders lineitem; do
  mysql -u root -p'details' --local-infile=1 tpch_sf1 -e \
    "LOAD DATA LOCAL INFILE '/tmp/tpch-dbgen/sf1/${table}.tbl' INTO TABLE ${table} FIELDS TERMINATED BY '|';"
done

# 验证
mysql -u root -p'details' tpch_sf1 -e "SELECT COUNT(*) FROM lineitem; SELECT COUNT(*) FROM orders;"
```

---

## SF=1 测试

SF=1 数据规模:
- Lineitem: 6,000,000 行
- Orders: 1,500,000 行
- Customer: 150,000 行
- Part: 200,000 行
- Supplier: 10,000 行
- PartSupp: 800,000 行

### 方法 1: 使用环境变量指定数据路径

```bash
export TPCH_DATA_DIR=/tmp/tpch-dbgen/sf1
cargo test --test tpch_sf1_benchmark -- --nocapture
```

### 方法 2: 使用已有的 SQLite 数据库

```bash
# SF1 数据已预加载到 MySQL，导出为 CSV
python3 << 'EOF'
import mysql.connector
import csv

conn = mysql.connector.connect(host="localhost", user="root", password="details", database="tpch_sf1")
cursor = conn.cursor()

tables = ['nation', 'region', 'supplier', 'part', 'partsupp', 'customer', 'orders', 'lineitem']
for table in tables:
    print(f"Exporting {table}...")
    cursor.execute(f"SELECT * FROM {table}")
    rows = cursor.fetchall()
    with open(f'/tmp/tpch-sf1/{table}.csv', 'w', newline='') as f:
        writer = csv.writer(f, delimiter='|')
        writer.writerows(rows)
cursor.close()
conn.close()
EOF
```

### 预期结果

```
=== SF=1 Benchmark Results ===
P99 Target: 1000ms
All Passed: YES ✅
Q4: P99=132.26ms avg=131.60ms
Q10: P99=183.36ms avg=181.09ms
Q13: P99=146.13ms avg=145.95ms
Q14: P99=162.37ms avg=161.04ms
Q19: P99=157.14ms avg=156.83ms
Q20: P99=128.22ms avg=127.32ms
Q22: P99=152.26ms avg=152.04ms
```

---

## SF=10 测试

SF=10 数据规模:
- Lineitem: 60,000,000 行 (~7.3 GB)
- Orders: 15,000,000 行 (~1.7 GB)
- 其他表按比例放大

### 注意事项

SF=10 测试需要:
- 内存: ~50 GB
- 磁盘空间: ~100 GB
- 运行时间: 2-4 小时 (取决于硬件)

### ⚠️ 已知问题

当前 SF10 测试 (`tpch_sf10_benchmark.rs`) 的数据生成器存在 bug，会产生重复的 partsupp 键值，导致测试失败。

**推荐方案**: 使用 `tpch_binary_import` 从真实 .tbl 文件导入数据:

```bash
# 1. 先生成 SF10 .tbl 文件
cd /tmp/tpch-dbgen
./dbgen -s 10 -f -d

# 2. 使用二进制导入工具
cargo run --example tpch_binary_import -p sqlrustgo-storage -- /tmp/tpch-dbgen

# 3. 导入完成后，修改测试以使用导入的数据
```

### 方法 1: 使用忽略测试运行

```bash
cargo test --test tpch_sf10_benchmark -- --ignored --nocapture
```

### 方法 2: 使用 Python 对比脚本

```bash
# 安装依赖
pip3 install mysql-connector-python psycopg2

# 运行对比测试
python3 scripts/tpch_comparison.py \
  --mysql \
  --mysql-host localhost \
  --mysql-user root \
  --mysql-password details \
  --mysql-db tpch_sf1 \
  --sqlite /tmp/tpch_sf01.db \
  --iterations 3
```

### 方法 3: 使用 Python 对比脚本

```bash
# 安装依赖
pip3 install mysql-connector-python psycopg2

# 运行对比测试
python3 scripts/tpch_comparison.py \
  --mysql \
  --mysql-host localhost \
  --mysql-user root \
  --mysql-password details \
  --mysql-db tpch_sf1 \
  --sqlite /tmp/tpch_sf01.db \
  --iterations 3
```

---

## 快速导入工具

### tpch_binary_import

直接从 `.tbl` 文件导入到 BinaryTableStorage，速度最快。

```bash
cargo run --example tpch_binary_import -p sqlrustgo-storage -- /path/to/tpch-sf1
```

输出示例:
```
==============================================
  TPC-H Fast Binary Import
==============================================
Source: "/tmp/tpch-dbgen/sf1"
Binary dir: "/tmp/tpch_binary"
Importing nation...
  nation: 25 rows in 0.00s (67374 rows/s)
Importing region...
  region: 5 rows in 0.00s (48037 rows/s)
...
Importing lineitem...
  lineitem: 6001215 rows in 205.30s (29232 rows/s)

==============================================
  Import Complete!
  Total time: 272.05s
  Binary files in: "/tmp/tpch_binary"
==============================================
```

### tpch_import

导入到 FileStorage，用于兼容测试。

```bash
TPCH_DATA_DIR=/tmp/tpch-dbgen/sf1 cargo run --example tpch_import -p sqlrustgo-storage
```

---

## 对比测试

### SQLRustGo vs MySQL

```bash
# 1. 启动 MySQL 并加载 SF1 数据
mysql -u root -p'details' -e "CREATE DATABASE IF NOT EXISTS tpch_sf1;"
mysql -u root -p'details' tpch_sf1 < scripts/mysql_tpch_setup.sql
# ... 加载数据 ...

# 2. 运行对比
python3 scripts/tpch_comparison.py \
  --mysql \
  --mysql-host localhost \
  --mysql-user root \
  --mysql-password details \
  --mysql-db tpch_sf1 \
  --sqlite /tmp/tpch_sf01.db \
  --iterations 3
```

### SQLRustGo vs PostgreSQL

```bash
# 1. 启动 PostgreSQL
sudo service postgresql start

# 2. 创建数据库
sudo -u postgres psql -c "CREATE DATABASE tpch_sf1;"

# 3. 运行对比
python3 scripts/tpch_comparison.py \
  --pg \
  --pg-host /var/run/postgresql \
  --pg-user postgres \
  --pg-db tpch_sf1 \
  --sqlite /tmp/tpch_sf01.db \
  --iterations 3
```

---

## 常见问题

### Q1: 测试提示 "file not found"

确保 `TPCH_DATA_DIR` 环境变量指向包含 `.tbl` 文件的目录:

```bash
export TPCH_DATA_DIR=/tmp/tpch-dbgen/sf1
cargo test --test tpch_sf1_benchmark -- --nocapture
```

### Q2: MySQL 连接失败

检查 MySQL 是否运行并配置正确:

```bash
# 检查状态
sudo service mysql status

# 测试连接
mysql -u root -p'details' -e "SELECT 1;"

# 设置环境变量
export MYSQL_HOST=localhost
export MYSQL_USER=root
export MYSQL_PASSWORD=details
export MYSQL_DATABASE=tpch_sf1
```

### Q3: SF10 测试超时

SF=10 生成 60M 行数据需要较长时间。推荐使用:
1. 预先生成数据并保存
2. 使用 `tpch_binary_import` 加快导入
3. 考虑只在必要时运行完整 SF=10 测试

### Q4: 内存不足

SF=10 需要约 50GB 内存。如果内存不足:
1. 减少测试规模 (使用 SF=1)
2. 增加 swap 空间
3. 关闭其他占用内存的程序

---

## 性能基准

### SF=1 测试结果 (2026-04-16)

| Query | P99 (ms) | Avg (ms) | 状态 |
|-------|-----------|----------|------|
| Q4 | 132.26 | 131.60 | ✅ |
| Q10 | 183.36 | 181.09 | ✅ |
| Q13 | 146.13 | 145.95 | ✅ |
| Q14 | 162.37 | 161.04 | ✅ |
| Q19 | 157.14 | 156.83 | ✅ |
| Q20 | 128.22 | 127.32 | ✅ |
| Q22 | 152.26 | 152.04 | ✅ |

**目标**: P99 < 1000ms
**结果**: 所有查询通过 ✅

---

## 相关文件

- 测试代码: `tests/integration/tpch_sf1_benchmark.rs`
- SF10 测试: `tests/integration/tpch_sf10_benchmark.rs`
- 导入工具: `crates/storage/examples/tpch_binary_import.rs`
- 对比脚本: `scripts/tpch_comparison.py`
- SQL 查询: `queries/q1.sql` - `q22.sql`
