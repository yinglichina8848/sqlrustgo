# TPC-H 测试完整指南

> 本指南详细说明如何执行 TPC-H 基准测试，包括环境准备、数据生成、测试执行、结果收集和对比分析。

**版本**: v1.0
**更新日期**: 2026-05-04
**维护者**: SQLRustGo Team

---

## 目录

1. [概述](#概述)
2. [环境准备](#环境准备)
3. [测试数据集生成](#测试数据集生成)
4. [数据导入](#数据导入)
5. [测试执行](#测试执行)
6. [结果收集](#结果收集)
7. [对比测试](#对比测试)
8. [常见问题](#常见问题)

---

## 概述

### TPC-H 简介

TPC-H 是一个决策支持基准测试，包含 22 个查询，模拟商业数据仓库场景。

### 测试规模

| Scale Factor | 数据规模 | 行数 (约) | 典型用途 |
|--------------|----------|-----------|----------|
| SF=0.1 | ~100MB | 866,602 | 快速功能验证 |
| SF=0.3 | ~300MB | 2.6M | 开发测试 |
| SF=1.0 | ~1GB | 8.6M | 标准基准测试 |
| SF=10 | ~10GB | 86M | 压力测试 |

### 测试查询

TPC-H 包含 22 个查询 (Q1-Q22)：
- Q1: 定价汇总报告 (简单聚合)
- Q2-Q22: 复杂 JOIN 查询 (多表关联、嵌套查询)

---

## 环境准备

### 1.1 系统要求

- **操作系统**: macOS / Linux
- **内存**: 建议 16GB+ (SF=1.0 需要约 8GB)
- **磁盘**: 建议 50GB+ 可用空间
- **Rust**: 1.74+ (已安装)

### 1.2 构建项目

```bash
# 克隆项目
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo

# 切换到开发分支
git checkout develop/v2.9.0

# 构建 (启用所有特性)
cargo build --all-features

# 验证构建
./target/debug/sqlrustgo-bench-cli --help
```

### 1.3 验证构建产物

```bash
# 应该能看到以下命令
SQLRustGo Benchmark CLI

Commands:
  tpch         # TPC-H 查询测试
  tpch-import  # TPC-H 数据导入
  tpch-bench   # TPC-H 基准测试
  oltp         # OLTP 基准测试
  help         # 帮助
```

---

## 测试数据集生成

### 2.1 安装 TPC-H dbgen

TPC-H dbgen 是 TPC 组织提供的官方数据生成工具。

```bash
# 进入临时目录
cd /tmp

# 克隆 dbgen 仓库
git clone https://github.com/electrum/tpch-dbgen.git
cd tpch-dbgen

# 编译 (macOS/Linux)
make
```

**注意**: 编译会有警告，但不影响使用。

### 2.2 生成测试数据

#### SF=0.1 数据 (快速测试用)

```bash
cd /tmp/tpch-dbgen

# 创建数据目录
mkdir -p ~/sqlrustgo-data/tpch-sf01

# 生成 SF=0.1 数据
./dbgen -s 0.1 -f

# 移动到目标目录
mv *.tbl ~/sqlrustgo-data/tpch-sf01/

# 验证
ls -la ~/sqlrustgo-data/tpch-sf01/
```

#### SF=1.0 数据 (标准基准)

```bash
cd /tmp/tpch-dbgen

# 创建数据目录
mkdir -p ~/sqlrustgo-data/tpch-sf1

# 生成 SF=1.0 数据 (约 5-10 分钟)
./dbgen -s 1 -f

# 移动到目标目录
mv *.tbl ~/sqlrustgo-data/tpch-sf1/

# 验证
ls -la ~/sqlrustgo-data/tpch-sf1/
```

#### SF=10 数据 (压力测试)

```bash
cd /tmp/tpch-dbgen

# 创建数据目录
mkdir -p ~/sqlrustgo-data/tpch-sf10

# 生成 SF=10 数据 (约 30-60 分钟)
./dbgen -s 10 -f

# 移动到目标目录
mv *.tbl ~/sqlrustgo-data/tpch-sf10/

# 验证
ls -la ~/sqlrustgo-data/tpch-sf10/
```

### 2.3 数据文件说明

| 文件名 | 表名 | SF=0.1 行数 | SF=1.0 行数 | SF=10 行数 |
|--------|------|-------------|-------------|-------------|
| customer.tbl | customer | 15,000 | 150,000 | 1,500,000 |
| orders.tbl | orders | 150,000 | 1,500,000 | 15,000,000 |
| lineitem.tbl | lineitem | 600,572 | 6,001,215 | 60,012,150 |
| part.tbl | part | 20,000 | 200,000 | 2,000,000 |
| partsupp.tbl | partsupp | 80,000 | 800,000 | 8,000,000 |
| supplier.tbl | supplier | 1,000 | 10,000 | 100,000 |
| nation.tbl | nation | 25 | 25 | 25 |
| region.tbl | region | 5 | 5 | 5 |

### 2.4 数据验证

```bash
# 验证 SF=0.1 数据
wc -l ~/sqlrustgo-data/tpch-sf01/*.tbl

# 预期输出:
#    15000 customer.tbl
#      1000 supplier.tbl
#     25000 nation.tbl
#        25 region.tbl
#    200000 part.tbl
#    800000 partsupp.tbl
#   1500000 orders.tbl
#    600572 lineitem.tbl
```

---

## 数据导入

### 3.1 使用 tpch-import 命令

```bash
# 导入 SF=0.1 数据
./target/debug/sqlrustgo-bench-cli tpch-import \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --output /tmp/tpch-sf01-storage
```

### 3.2 导入输出说明

```
=== TPC-H Import ===
DDL: scripts/sqlite_tpch_setup.sql
Data: /Users/xxx/sqlrustgo-data/tpch-sf01/
Output: /tmp/tpch-sf01-storage

Found 8 tables:
  - customer (8 columns)
  - nation (4 columns)
  - region (3 columns)
  - supplier (7 columns)
  - part (9 columns)
  - partsupp (5 columns)
  - orders (9 columns)
  - lineitem (16 columns)

=== Importing Data ===
  customer: imported 15000 rows in 0.02s
  nation: imported 25 rows in 0.00s
  ...
  lineitem: imported 600572 rows in 0.67s

Total import time: 0.89s

=== Verifying Data ===
  customer: 15000 rows [OK]
  ...

==============================================
  Import Complete!
==============================================
```

### 3.3 不同存储引擎对比

| 存储引擎 | 优点 | 缺点 | 适用场景 |
|----------|------|------|----------|
| MemoryStorage | 最快 | 内存限制 | SF≤1, 功能测试 |
| FileStorage | 支持大数据 | 较慢 | SF≥1, 生产 |
| ColumnarStorage | 分析快 | 导入慢 | OLAP 场景 |

---

## 测试执行

### 4.1 基本测试命令

```bash
# 测试单个查询
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries Q1

# 测试多个查询 (逗号分隔)
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries Q1,Q4,Q6

# 测试所有查询
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries all

# 指定迭代次数
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries Q1,Q4,Q6 \
  --iterations 5

# 输出到 JSON 文件
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries all \
  --output results.json
```

### 4.2 完整测试流程

#### SF=0.1 快速验证 (约 5-10 分钟)

```bash
# 只运行快速查询 (单表聚合，无 JOIN)
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries Q1,Q4,Q6,Q13,Q14,Q19,Q20,Q22 \
  --iterations 3
```

#### SF=1.0 标准测试 (约 30-60 分钟)

```bash
# 运行所有查询 (可能部分超时)
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf1/ \
  --queries all \
  --iterations 3
```

### 4.3 测试参数说明

| 参数 | 说明 | 默认值 |
|------|------|---------|
| `--ddl` | DDL 文件路径 | 必需 |
| `--data` | TPC-H 数据目录 | 必需 |
| `--queries` | 查询列表 (Q1,Q2... 或 all) | all |
| `--iterations` | 每个查询运行次数 | 3 |
| `--batch-size` | 批量插入大小 | 10000 |
| `--output` | 结果输出 JSON 文件 | (无) |

### 4.4 测试输出格式

```
==============================================
  TPC-H Benchmark with Real Data
==============================================
DDL: scripts/sqlite_tpch_setup.sql
Data: /Users/xxx/sqlrustgo-data/tpch-sf01/
Queries: Q1,Q4,Q6
Iterations: 3

=== Parsing DDL ===
...

=== Importing data ===
  customer: 15000 rows in 0.03s
  ...

Total import: 866602 rows in 2.35s

=== Running 3 TPC-H Queries ===

  Q1: 1337.98 ms (3 rows, 3 iters)
  Q4: 435.28 ms (5 rows, 3 iters)
  Q6: 1024.46 ms (1 rows, 3 iters)

==============================================
  Benchmark Complete
==============================================

Query      Avg (ms)       Rows
------------------------------
Q1          1337.98          3
Q4           435.28          5
Q6          1024.46          1
------------------------------
TOTAL       2797.72
```

---

## 结果收集

### 5.1 JSON 输出格式

```bash
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries all \
  --output tpch-results-sf01.json
```

生成的 JSON 格式:

```json
{
  "timestamp": "2026-05-04T12:00:00Z",
  "scale_factor": "0.1",
  "system": "sqlrustgo",
  "queries": [
    {
      "name": "Q1",
      "avg_latency_ms": 1337.98,
      "min_ms": 1300.00,
      "max_ms": 1400.00,
      "rows": 3,
      "iterations": 3
    }
  ],
  "summary": {
    "total_ms": 2797.72,
    "queries_passed": 8,
    "queries_failed": 14,
    "queries_timeout": 14
  }
}
```

### 5.2 收集性能指标

| 指标 | 说明 |
|------|------|
| avg_latency_ms | 平均延迟 (毫秒) |
| min_ms | 最小延迟 |
| max_ms | 最大延迟 |
| p50_ms | P50 延迟 |
| p95_ms | P95 延迟 |
| p99_ms | P99 延迟 |
| rows | 返回行数 |
| iterations | 运行次数 |

### 5.3 性能基准对比表

| 查询 | SF=0.1 (ms) | SF=1.0 (ms) | 预期比例 | 状态 |
|------|--------------|---------------|----------|------|
| Q1 | ~1,300 | ~21,000 | ~16x | ✅/❌ |
| Q4 | ~400 | ~6,500 | ~16x | ✅/❌ |
| ... | ... | ... | ... | ... |

---

## 对比测试

### 6.1 SQLRustGo vs SQLite

```bash
# 1. 准备 SQLite 测试
sqlite3 tpch-sf01.db < scripts/sqlite_tpch_setup.sql

# 2. 导入数据到 SQLite
for table in customer nation region supplier part partsupp orders lineitem; do
  sqlite3 tpch-sf01.db << EOF
  .mode csv
  .import ~/sqlrustgo-data/tpch-sf01/\${table}.tbl \${table}
EOF
done

# 3. 运行查询对比
sqlite3 tpch-sf01.db "SELECT ... FROM ..." # Q1
```

### 6.2 SQLRustGo vs MySQL

```bash
# 1. 安装 MySQL
brew install mysql  # macOS
# apt install mysql-server  # Ubuntu

# 2. 启动 MySQL
mysql.server start  # macOS
# service mysql start  # Ubuntu

# 3. 创建数据库
mysql -u root -p -e "CREATE DATABASE IF NOT EXISTS tpch_sf01;"

# 4. 创建表
mysql -u root -p tpch_sf01 < scripts/mysql_tpch_setup.sql

# 5. 导入数据
for table in customer nation region supplier part partsupp orders lineitem; do
  mysql -u root -p tpch_sf01 -e \
    "LOAD DATA LOCAL INFILE '~/sqlrustgo-data/tpch-sf01/\${table}.tbl' INTO TABLE \${table} FIELDS TERMINATED BY '|';"
done

# 6. 运行查询
mysql -u root -p tpch_sf01 -e "SELECT ..." # Q1
```

### 6.3 SQLRustGo vs PostgreSQL

```bash
# 1. 安装 PostgreSQL
brew install postgresql  # macOS
# apt install postgresql  # Ubuntu

# 2. 启动 PostgreSQL
pg_ctl -D /usr/local/var/postgres start  # macOS
# service postgresql start  # Ubuntu

# 3. 创建数据库
createdb tpch_sf01

# 4. 创建表
psql -d tpch_sf01 -f scripts/pg_tpch_setup.sql

# 5. 导入数据
for table in customer nation region supplier part partsupp orders lineitem; do
  psql -d tpch_sf01 -c "\\COPY \${table} FROM '~/sqlrustgo-data/tpch-sf01/\${table}.tbl' WITH (FORMAT csv, DELIMITER '|');"
done

# 6. 运行查询
psql -d tpch_sf01 -c "SELECT ..." # Q1
```

### 6.4 自动化对比脚本

```python
#!/usr/bin/env python3
"""
TPC-H 对比测试脚本
对比 SQLRustGo、SQLite、MySQL、PostgreSQL 的性能
"""

import subprocess
import json
import time
from datetime import datetime

class TPC-H-Comparator:
    def __init__(self, data_dir):
        self.data_dir = data_dir
        self.results = {}

    def run_sqlrustgo(self, queries, iterations=3):
        """运行 SQLRustGo 测试"""
        cmd = [
            "./target/debug/sqlrustgo-bench-cli",
            "tpch-bench",
            "--ddl", "scripts/sqlite_tpch_setup.sql",
            "--data", f"{self.data_dir}",
            "--queries", ",".join(queries),
            "--iterations", str(iterations),
            "--output", "/tmp/sqlrustgo-results.json"
        ]
        start = time.time()
        subprocess.run(cmd)
        elapsed = time.time() - start

        with open("/tmp/sqlrustgo-results.json") as f:
            return json.load(f), elapsed

    def run_sqlite(self, queries):
        """运行 SQLite 测试"""
        # 实现...
        pass

    def run_mysql(self, queries):
        """运行 MySQL 测试"""
        # 实现...
        pass

    def compare(self, queries):
        """执行对比测试"""
        print("Running comparison...")

        # SQLRustGo
        sqlrustgo_results, sqlrustgo_time = self.run_sqlrustgo(queries)

        # SQLite
        # ...

        # MySQL
        # ...

        # PostgreSQL
        # ...

        return self.generate_report()

    def generate_report(self):
        """生成对比报告"""
        report = {
            "timestamp": datetime.now().isoformat(),
            "system_results": self.results,
            "comparisons": []
        }

        for query in queries:
            row = {
                "query": query,
                "sqlrustgo_ms": self.results["sqlrustgo"][query],
                "sqlite_ms": self.results["sqlite"][query],
                "mysql_ms": self.results["mysql"][query],
                "postgres_ms": self.results["postgres"][query],
            }
            row["speedup_vs_sqlite"] = row["sqlite_ms"] / row["sqlrustgo_ms"]
            row["speedup_vs_mysql"] = row["mysql_ms"] / row["sqlrustgo_ms"]
            row["speedup_vs_postgres"] = row["postgres_ms"] / row["sqlrustgo_ms"]
            report["comparisons"].append(row)

        return report

if __name__ == "__main__":
    comparator = TPC-H-Comparator("~/sqlrustgo-data/tpch-sf01")
    report = comparator.compare(["Q1", "Q4", "Q6", "Q13"])
    print(json.dumps(report, indent=2))
```

---

## 常见问题

### Q1: 数据导入失败

**症状**: `Error: Failed to open file`

**解决方案**:
```bash
# 检查文件是否存在
ls -la ~/sqlrustgo-data/tpch-sf01/*.tbl

# 检查文件权限
chmod 644 ~/sqlrustgo-data/tpch-sf01/*.tbl
```

### Q2: 查询超时

**症状**: 查询运行超过 60 秒

**解决方案**:
- 使用 SF=0.1 进行快速测试
- 优化 join 算法 (TODO)
- 添加索引 (TODO)

### Q3: 内存不足

**症状**: `MemoryError` 或系统变慢

**解决方案**:
```bash
# 使用 FileStorage 而不是 MemoryStorage
# SF=1.0 需要约 8GB 内存
# SF=0.1 需要约 1GB 内存
```

### Q4: 编译失败

**症状**: `cargo build` 报错

**解决方案**:
```bash
# 清理并重新构建
cargo clean
cargo build --all-features

# 如果还是失败，检查 Rust 版本
rustc --version  # 需要 1.74+
```

### Q5: dbgen 编译失败

**症状**: gcc 编译报错

**解决方案**:
```bash
# macOS 可能需要指定 SDK
cd /tmp/tpch-dbgen
make CC=gcc CFLAGS="-I."  # 显式指定
```

---

## 附录

### A. TPC-H 查询列表

| 查询 | 描述 | 复杂度 |
|------|------|--------|
| Q1 | Pricing Summary Report | 单表聚合 |
| Q2 | Minimum Cost Supplier | 2 表 JOIN |
| Q3 | Shipping Priority | 3 表 JOIN |
| Q4 | Order Priority Checking | 单表过滤 |
| Q5 | Local Supplier Volume | 5 表 JOIN |
| Q6 | Forecast Revenue Change | 单表聚合 |
| Q7 | Volume Shipping | 4 表 JOIN |
| Q8 | National Market Share | 6 表 JOIN |
| Q9 | Product Type Profit | 5 表 JOIN |
| Q10 | Returned Item Reporting | 4 表 JOIN |
| Q11 | Important Stock | 3 表 JOIN |
| Q12 | Shipping Modes | 3 表 JOIN |
| Q13 | Customer Distribution | 外连接 |
| Q14 | Promotion Effect | 2 表 JOIN |
| Q15 | Top Supplier | 视图 |
| Q16 | Parts/Supplier | 4 表 JOIN |
| Q17 | Small Quantity | 2 表 JOIN |
| Q18 | Large Volume | 3 表 JOIN |
| Q19 | Discounted Revenue | 3 表 JOIN |
| Q20 | Potential Promotion | 3 表 JOIN |
| Q21 | Waiting Suppliers | 4 表 JOIN |
| Q22 | Global Sales | 2 表 JOIN |

### B. 参考文档

- [TPC-H 官方规范](http://www.tpc.org/tpch/)
- [TPC-H dbgen GitHub](https://github.com/electrum/tpch-dbgen)
- [SQLRustGo 文档](../README.md)

### C. 维护记录

| 日期 | 版本 | 修改内容 |
|------|------|----------|
| 2026-05-04 | v1.0 | 初始版本 |
