# Sysbench OLTP 性能对比测试计划

> **版本**: v2.8.0  
> **创建日期**: 2026-04-23  
> **目标**: 全面测试 SQLRustGo、MySQL、PostgreSQL 的 OLTP 性能并生成对比报告

---

## 1. 测试环境要求

### 1.1 硬件环境

| 配置项 | 最低要求 | 推荐配置 |
|--------|----------|----------|
| CPU | 4 核心 | 8+ 核心 |
| 内存 | 8 GB | 16+ GB |
| 磁盘 | SSD | NVMe SSD |
| 操作系统 | macOS/Linux | Linux (Ubuntu 22.04+) |

### 1.2 软件依赖

| 软件 | 版本要求 | 安装命令 |
|------|----------|----------|
| sysbench | 1.0.20+ | `brew install sysbench` (macOS) / `apt install sysbench` (Ubuntu) |
| MySQL Server | 8.0+ | `brew install mysql` (macOS) / `apt install mysql-server` (Ubuntu) |
| PostgreSQL | 14+ | `brew install postgresql` (macOS) / `apt install postgresql` (Ubuntu) |
| Docker | 20.10+ | https://docs.docker.com/get-docker/ |

---

## 2. 数据库安装与启动

### 2.1 SQLRustGo MySQL 服务器

```bash
# 1. 进入项目目录
cd /path/to/sqlrustgo

# 2. 编译发布版本
cargo build --release -p sqlrustgo-mysql-server

# 3. 启动服务器 (默认端口 3306)
./target/release/sqlrustgo-mysql-server --port 3306

# 4. 验证服务器运行
lsof -i :3306
```

### 2.2 MySQL Server

```bash
# macOS
brew services start mysql
# 或手动启动
mysql.server start

# Ubuntu
sudo systemctl start mysql
sudo systemctl status mysql

# 创建测试数据库和用户
mysql -u root -e "
CREATE DATABASE IF NOT EXISTS sbtest;
CREATE USER IF NOT EXISTS 'sbuser'@'localhost' IDENTIFIED BY 'sbpass';
CREATE USER IF NOT EXISTS 'sbuser'@'%' IDENTIFIED BY 'sbpass';
GRANT ALL PRIVILEGES ON *.* TO 'sbuser'@'localhost';
GRANT ALL PRIVILEGES ON *.* TO 'sbuser'@'%';
FLUSH PRIVILEGES;
"
```

### 2.3 PostgreSQL

```bash
# macOS
brew services start postgresql@16
# 或手动启动
pg_ctl -D /opt/homebrew/var/postgresql@16 start

# Ubuntu
sudo systemctl start postgresql
sudo systemctl status postgresql

# 创建测试数据库和用户
sudo -u postgres psql -e "
CREATE USER sbuser WITH PASSWORD 'sbpass';
CREATE DATABASE sbtest OWNER sbuser;
GRANT ALL PRIVILEGES ON DATABASE sbtest TO sbuser;
"
```

---

## 3. Sysbench 表准备

### 3.1 SQLRustGo 准备命令

SQLRustGo 使用 MySQL Wire 协议，sysbench 连接方式与 MySQL 相同：

```bash
# 准备测试表 (SQLRustGo)
sysbench oltp_insert \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306 \
  --mysql-user=sbuser \
  --mysql-password=sbpass \
  --mysql-db=sbtest \
  --table-size=100000 \
  --tables=10 \
  prepare
```

### 3.2 MySQL 准备命令

```bash
# 准备测试表 (MySQL)
sysbench oltp_insert \
  --db-driver=mysql \
  --mysql-host=127.0.0.1 \
  --mysql-port=3306 \
  --mysql-user=sbuser \
  --mysql-password=sbpass \
  --mysql-db=sbtest \
  --table-size=100000 \
  --tables=10 \
  prepare
```

### 3.3 PostgreSQL 准备命令

```bash
# 准备测试表 (PostgreSQL)
sysbench oltp_insert \
  --db-driver=pgsql \
  --pgsql-host=127.0.0.1 \
  --pgsql-port=5432 \
  --pgsql-user=sbuser \
  --pgsql-password=sbpass \
  --pgsql-db=sbtest \
  --table-size=100000 \
  --tables=10 \
  prepare
```

---

## 4. 测试场景

### 4.1 OLTP 测试矩阵

| 测试编号 | 测试名称 | 并发线程数 | 测试时长 | 说明 |
|----------|----------|------------|----------|------|
| 1 | point_select | 1, 8, 16, 32, 64 | 60s | 主键点查 |
| 2 | point_select | 128 | 60s | 高并发点查 |
| 3 | read_only | 1, 8, 16, 32 | 60s | 只读混合查询 |
| 4 | read_write | 1, 8, 16, 32 | 60s | 读写混合 (2:1) |
| 5 | write_only | 1, 8, 16 | 60s | 只写 (INSERT/UPDATE/DELETE) |
| 6 | bulk_insert | 1, 8, 16 | 30s | 批量插入 1000 行/事务 |
| 7 | oltp_delete | 8, 32 | 60s | 高频删除 |
| 8 | index_scan | 8, 32 | 60s | 索引范围扫描 |
| 9 | random_points | 32, 64 | 60s | 随机主键查询 |
| 10 | cache_hit | 32 | 60s | 热点数据访问 |

---

## 5. 测试命令模板

### 5.1 Point Select (主键点查)

```bash
#!/bin/bash
# point_select_test.sh

THREADS=(1 8 16 32 64 128)
DB=$1  # mysql, sqlrustgo, pgsql

for threads in "${THREADS[@]}"; do
  echo "=== Point Select, Threads: $threads ==="
  
  if [ "$DB" = "pgsql" ]; then
    sysbench oltp_point_select \
      --db-driver=pgsql \
      --pgsql-host=127.0.0.1 \
      --pgsql-port=5432 \
      --pgsql-user=sbuser \
      --pgsql-password=sbpass \
      --pgsql-db=sbtest \
      --threads=$threads \
      --time=60 \
      --report-interval=5 \
      run
  else
    sysbench oltp_point_select \
      --db-driver=mysql \
      --mysql-host=127.0.0.1 \
      --mysql-port=3306 \
      --mysql-user=sbuser \
      --mysql-password=sbpass \
      --mysql-db=sbtest \
      --threads=$threads \
      --time=60 \
      --report-interval=5 \
      run
  fi
  
  echo ""
done
```

### 5.2 Read Write (读写混合)

```bash
#!/bin/bash
# read_write_test.sh

THREADS=(1 8 16 32)
DB=$1

for threads in "${THREADS[@]}"; do
  echo "=== Read Write, Threads: $threads ==="
  
  if [ "$DB" = "pgsql" ]; then
    sysbench oltp_read_write \
      --db-driver=pgsql \
      --pgsql-host=127.0.0.1 \
      --pgsql-port=5432 \
      --pgsql-user=sbuser \
      --pgsql-password=sbpass \
      --pgsql-db=sbtest \
      --threads=$threads \
      --time=60 \
      --report-interval=5 \
      -- Ols=10 \
      --point-ranges=10 \
      --delete-ranges=10 \
      run
  else
    sysbench oltp_read_write \
      --db-driver=mysql \
      --mysql-host=127.0.0.1 \
      --mysql-port=3306 \
      --mysql-user=sbuser \
      --mysql-password=sbpass \
      --mysql-db=sbtest \
      --threads=$threads \
      --time=60 \
      --report-interval=5 \
      run
  fi
  
  echo ""
done
```

### 5.3 Bulk Insert (批量插入)

```bash
#!/bin/bash
# bulk_insert_test.sh

THREADS=(1 8 16)
DB=$1

for threads in "${THREADS[@]}"; do
  echo "=== Bulk Insert, Threads: $threads ==="
  
  if [ "$DB" = "pgsql" ]; then
    sysbench oltp_insert \
      --db-driver=pgsql \
      --pgsql-host=127.0.0.1 \
      --pgsql-port=5432 \
      --pgsql-user=sbuser \
      --pgsql-password=sbpass \
      --pgsql-db=sbtest \
      --threads=$threads \
      --time=30 \
      --report-interval=5 \
      --bulk-insert=1000 \
      run
  else
    sysbench oltp_insert \
      --db-driver=mysql \
      --mysql-host=127.0.0.1 \
      --mysql-port=3306 \
      --mysql-user=sbuser \
      --mysql-password=sbpass \
      --mysql-db=sbtest \
      --threads=$threads \
      --time=30 \
      --report-interval=5 \
      --bulk-insert=1000 \
      run
  fi
  
  echo ""
done
```

---

## 6. 完整测试执行脚本

```bash
#!/bin/bash
# full_sysbench_comparison.sh
# 完整 Sysbench 对比测试脚本

set -e

RESULTS_DIR="./sysbench_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "========================================"
echo "Sysbench OLTP Performance Comparison"
echo "Results Directory: $RESULTS_DIR"
echo "Date: $(date)"
echo "========================================"

# 测试数据库列表
DATABASES=("sqlrustgo" "mysql" "pgsql")

# 测试场景
SCENARIOS=(
  "oltp_point_select:threads=1,8,16,32,64:time=60"
  "oltp_read_only:threads=1,8,16,32:time=60"
  "oltp_read_write:threads=1,8,16,32:time=60"
  "oltp_write_only:threads=1,8,16:time=60"
  "oltp_insert:threads=1,8,16:time=30"
)

run_test() {
  local db=$1
  local scenario=$2
  local threads=$3
  local time=$4
  
  local name=$(echo $scenario | cut -d: -f1)
  local outfile="$RESULTS_DIR/${db}_${name}_t${threads}.log"
  
  echo "Testing $db $name threads=$threads..."
  
  if [ "$db" = "pgsql" ]; then
    sysbench $scenario \
      --db-driver=pgsql \
      --pgsql-host=127.0.0.1 \
      --pgsql-port=5432 \
      --pgsql-user=sbuser \
      --pgsql-password=sbpass \
      --pgsql-db=sbtest \
      --threads=$threads \
      --time=$time \
      --report-interval=5 \
      run 2>&1 | tee "$outfile"
  else
    sysbench $scenario \
      --db-driver=mysql \
      --mysql-host=127.0.0.1 \
      --mysql-port=3306 \
      --mysql-user=sbuser \
      --mysql-password=sbpass \
      --mysql-db=sbtest \
      --threads=$threads \
      --time=$time \
      --report-interval=5 \
      run 2>&1 | tee "$outfile"
  fi
  
  echo "Results saved to $outfile"
}

# 执行测试
for db in "${DATABASES[@]}"; do
  echo ""
  echo "========================================"
  echo "Testing Database: $db"
  echo "========================================"
  
  for entry in "${SCENARIOS[@]}"; do
    scenario=$(echo $entry | cut -d: -f1)
    threads_list=$(echo $entry | cut -d: -f2)
    time=$(echo $entry | cut -d: -f3)
    
    IFS=',' read -ra THREADS <<< "$threads_list"
    for t in "${THREADS[@]}"; do
      threads=$(echo $t | cut -d'=' -f2)
      run_test "$db" "$scenario" "$threads" "$time"
      sleep 5  # 冷却间隔
    done
  done
done

echo ""
echo "========================================"
echo "All tests completed!"
echo "Results saved to: $RESULTS_DIR"
echo "========================================"
```

---

## 7. 结果收集与报告生成

### 7.1 结果提取脚本

```python
#!/usr/bin/env python3
# parse_sysbench_results.py

import re
import os
import sys
import json
from pathlib import Path

def parse_sysbench_log(filepath):
    """解析 sysbench 输出日志"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    # 提取关键指标
    patterns = {
        'transactions': r'transactions:\s+(\d+\.?\d*)\s+\((\d+\.?\d*)\s+per\s+s\)',
        'queries': r'queries:\s+(\d+\.?\d*)\s+\((\d+\.?\d*)\s+per\s+s\)',
        'latency_avg': r'avg:\s+(\d+\.?\d*)',
        'latency_p50': r'\s+50th percentile:\s+(\d+\.?\d*)',
        'latency_p95': r'\s+95th percentile:\s+(\d+\.?\d*)',
        'latency_p99': r'\s+99th percentile:\s+(\d+\.?\d*)',
        'latency_min': r'min:\s+(\d+\.?\d*)',
        'latency_max': r'max:\s+(\d+\.?\d*)',
    }
    
    result = {}
    for key, pattern in patterns.items():
        match = re.search(pattern, content)
        if match:
            result[key] = float(match.group(1))
    
    return result

def generate_report(results_dir, output_file):
    """生成对比报告"""
    
    databases = ['sqlrustgo', 'mysql', 'pgsql']
    scenarios = ['oltp_point_select', 'oltp_read_only', 'oltp_read_write', 'oltp_insert']
    threads = [1, 8, 16, 32]
    
    report_lines = []
    report_lines.append("# Sysbench OLTP 性能对比报告")
    report_lines.append(f"\n**生成日期**: 2026-04-23\n")
    report_lines.append("## 1. 测试环境\n")
    report_lines.append("| 配置项 | 值 |")
    report_lines.append("|--------|---|")
    report_lines.append("| CPU | TODO |")
    report_lines.append("| 内存 | TODO |")
    report_lines.append("| 磁盘 | TODO |")
    report_lines.append("| 操作系统 | TODO |")
    report_lines.append("| sysbench | 1.0.20 |")
    report_lines.append("")
    
    # 生成每个场景的对比表
    for scenario in scenarios:
        report_lines.append(f"\n## {scenario} 对比\n")
        report_lines.append("| 数据库 | 线程数 | QPS | TPS | 平均延迟(ms) | p95延迟(ms) | p99延迟(ms) |")
        report_lines.append("|--------|--------|-----|-----|--------------|-------------|-------------|")
        
        for t in threads:
            row = f"| {scenario} | {t} threads | - | - | - | - | - |"
            
            for db in databases:
                log_file = f"{results_dir}/{db}_{scenario}_t{t}.log"
                if os.path.exists(log_file):
                    data = parse_sysbench_log(log_file)
                    if data:
                        tps = data.get('transactions', 0)
                        qps = data.get('queries', 0)
                        avg_lat = data.get('latency_avg', 0)
                        p95_lat = data.get('latency_p95', 0)
                        p99_lat = data.get('latency_p99', 0)
                        row = f"| **{db}** | {t} | {qps:.0f} | {tps:.0f} | {avg_lat:.2f} | {p95_lat:.2f} | {p99_lat:.2f} |"
                else:
                    row = f"| {db} | {t} | N/A | N/A | N/A | N/A | N/A |"
            
            report_lines.append(row)
    
    # 生成汇总表
    report_lines.append("\n## 2. 性能汇总\n")
    report_lines.append("\n### 2.1 Point Select (32 线程)\n")
    report_lines.append("| 数据库 | QPS | TPS | p99延迟 |")
    report_lines.append("|--------|-----|-----|---------|")
    
    for db in databases:
        log_file = f"{results_dir}/{db}_oltp_point_select_t32.log"
        if os.path.exists(log_file):
            data = parse_sysbench_log(log_file)
            if data:
                tps = data.get('transactions', 0)
                qps = data.get('queries', 0)
                p99_lat = data.get('latency_p99', 0)
                report_lines.append(f"| {db} | {qps:.0f} | {tps:.0f} | {p99_lat:.2f}ms |")
        else:
            report_lines.append(f"| {db} | N/A | N/A | N/A |")
    
    report_lines.append("\n### 2.2 Read Write (32 线程)\n")
    report_lines.append("| 数据库 | QPS | TPS | p99延迟 |")
    report_lines.append("|--------|-----|-----|---------|")
    
    for db in databases:
        log_file = f"{results_dir}/{db}_oltp_read_write_t32.log"
        if os.path.exists(log_file):
            data = parse_sysbench_log(log_file)
            if data:
                tps = data.get('transactions', 0)
                qps = data.get('queries', 0)
                p99_lat = data.get('latency_p99', 0)
                report_lines.append(f"| {db} | {qps:.0f} | {tps:.0f} | {p99_lat:.2f}ms |")
        else:
            report_lines.append(f"| {db} | N/A | N/A | N/A |")
    
    report_lines.append("\n## 3. 分析结论\n")
    report_lines.append("\nTODO: 根据测试结果填写分析结论\n")
    
    report_lines.append("\n---\n")
    report_lines.append("*本报告由 Sysbench 自动化测试生成*\n")
    
    with open(output_file, 'w') as f:
        f.write('\n'.join(report_lines))
    
    print(f"Report saved to: {output_file}")

if __name__ == "__main__":
    results_dir = sys.argv[1] if len(sys.argv) > 1 else "./sysbench_results"
    output_file = sys.argv[2] if len(sys.argv) > 2 else "./SYSBENCH_COMPARISON_REPORT.md"
    generate_report(results_dir, output_file)
```

---

## 8. 测试执行检查清单

### 8.1 测试前检查

```bash
# 1. 确认 sysbench 已安装
sysbench --version

# 2. 确认所有数据库服务运行中
lsof -i :3306  # SQLRustGo / MySQL
lsof -i :5432  # PostgreSQL

# 3. 确认测试数据库和用户已创建
# MySQL:
mysql -u sbuser -psbpass -e "SELECT 1"

# PostgreSQL:
PGPASSWORD=sbpass psql -h 127.0.0.1 -U sbuser -d sbtest -c "SELECT 1"

# 4. 清理旧数据
mysql -u root -e "DROP DATABASE IF EXISTS sbtest; CREATE DATABASE sbtest;"
psql -U postgres -c "DROP DATABASE IF EXISTS sbtest; CREATE DATABASE sbtest;"
```

### 8.2 测试执行顺序

```
1. [ ] SQLRustGo 单独测试
2. [ ] MySQL 对比测试
3. [ ] PostgreSQL 对比测试
4. [ ] 收集所有结果日志
5. [ ] 生成对比报告
```

---

## 9. 预期测试时间

| 测试场景 | SQLRustGo | MySQL | PostgreSQL | 合计 |
|----------|-----------|-------|------------|------|
| Point Select (5 种并发) | ~5min | ~5min | ~5min | ~15min |
| Read Only (4 种并发) | ~4min | ~4min | ~4min | ~12min |
| Read Write (4 种并发) | ~4min | ~4min | ~4min | ~12min |
| Write Only (3 种并发) | ~3min | ~3min | ~3min | ~9min |
| Bulk Insert (3 种并发) | ~1.5min | ~1.5min | ~1.5min | ~4.5min |
| **总计** | ~17.5min | ~17.5min | ~17.5min | **~52.5min** |

---

## 10. 输出文件清单

测试完成后，应生成以下文件：

| 文件名 | 说明 |
|--------|------|
| `SYSBENCH_COMPARISON_REPORT.md` | 最终对比报告 (Markdown) |
| `sysbench_results_YYYYMMDD_HHMMSS/` | 原始日志目录 |
| `*_oltp_point_select_t*.log` | Point Select 测试日志 |
| `*_oltp_read_write_t*.log` | Read Write 测试日志 |
| `*_oltp_read_only_t*.log` | Read Only 测试日志 |
| `*_oltp_insert_t*.log` | Insert 测试日志 |
| `*_oltp_write_only_t*.log` | Write Only 测试日志 |
| `test_environment.json` | 测试环境信息 |

---

## 11. 报告模板

生成报告后，请填写以下信息：

```markdown
# Sysbench OLTP 性能对比报告

## 1. 测试环境

| 配置项 | 值 |
|--------|---|
| CPU | [填写] |
| 核心数 | [填写] |
| 内存 | [填写] |
| 磁盘类型 | [填写] |
| 操作系统 | [填写] |
| 内核版本 | [填写] |
| MySQL 版本 | [填写] |
| PostgreSQL 版本 | [填写] |
| SQLRustGo 版本 | [填写] |

## 2. 测试结果汇总

### 2.1 Point Select (主键点查)

| 数据库 | 32线程 QPS | 32线程 TPS | p99延迟 |
|--------|------------|------------|---------|
| SQLRustGo | [值] | [值] | [值]ms |
| MySQL | [值] | [值] | [值]ms |
| PostgreSQL | [值] | [值] | [值]ms |

### 2.2 Read Write (读写混合)

| 数据库 | 32线程 QPS | 32线程 TPS | p99延迟 |
|--------|------------|------------|---------|
| SQLRustGo | [值] | [值] | [值]ms |
| MySQL | [值] | [值] | [值]ms |
| PostgreSQL | [值] | [值] | [值]ms |

### 2.3 Insert (插入)

| 数据库 | 16线程 QPS | 16线程 TPS | p99延迟 |
|--------|------------|------------|---------|
| SQLRustGo | [值] | [值] | [值]ms |
| MySQL | [值] | [值] | [值]ms |
| PostgreSQL | [值] | [值] | [值]ms |

## 3. 性能对比分析

### 3.1 SQLRustGo vs MySQL

- Point Select: [SQLRustGo 是 MySQL 的 X%]
- Read Write: [SQLRustGo 是 MySQL 的 X%]
- Insert: [SQLRustGo 是 MySQL 的 X%]

### 3.2 SQLRustGo vs PostgreSQL

- Point Select: [SQLRustGo 是 PostgreSQL 的 X%]
- Read Write: [SQLRustGo 是 PostgreSQL 的 X%]
- Insert: [SQLRustGo 是 PostgreSQL 的 X%]

## 4. 结论

[根据测试结果填写分析结论]

## 5. 附录

### 5.1 原始测试数据

[链接到原始日志]

### 5.2 测试命令

[使用的完整 sysbench 命令]
```

---

## 12. 已知限制

### 12.1 SQLRustGo 当前限制

1. **索引支持**: MemoryStorage 部分索引操作可能不完全
2. **事务支持**: 基础事务支持，高级特性可能有限
3. **并发控制**: 最大并发连接数取决于配置
4. **持久化**: MemoryStorage 为内存存储，重启后数据丢失

### 12.2 测试建议

1. 首次测试建议使用较小数据量 (--table-size=10000)
2. 确认 SQLRustGo 稳定后再运行完整测试
3. 每轮测试间留有冷却时间 (建议 5-10 秒)
4. 每个配置建议运行 2-3 次取平均值

---

*本文档由 SQLRustGo Team 生成*
*如有问题请提交 Issue: https://github.com/minzuuniversity/sqlrustgo/issues*
