# TPC-H 测试计划

> SQLRustGo TPC-H 基准测试完整计划

**版本**: v1.0
**创建日期**: 2026-05-04
**状态**: 待执行
**执行人**: HP Z6G4 (opencode/hermes)

---

## 1. 测试目标

### 1.1 主要目标

1. **功能验证**: 验证 SQLRustGo 是否支持 TPC-H Q1-Q22 所有查询
2. **性能基准**: 建立 SQLRustGo 在不同 Scale Factor 下的性能基准
3. **对比分析**: 与 SQLite、MySQL、PostgreSQL 进行性能对比
4. **问题发现**: 发现 SQLRustGo 在复杂查询中的性能问题

### 1.2 成功标准

| 指标 | 目标 | 当前状态 |
|------|------|----------|
| 支持的查询数 | 22/22 | 8/22 ✅ |
| SF=0.1 所有查询执行时间 | < 60s | 8 查询 ✅, 14 超时 ❌ |
| SF=1.0 所有查询执行时间 | < 300s | 需测试 |
| JOIN 查询性能 | < 10s (SF=0.1) | > 60s ❌ |

---

## 2. 测试环境

### 2.1 硬件配置

| 组件 | 规格 |
|------|------|
| CPU | Apple M2 Pro / Intel i7+ |
| 内存 | 16GB+ |
| 磁盘 | 50GB+ SSD |
| 操作系统 | macOS 14+ / Ubuntu 22.04+ |

### 2.2 软件配置

| 软件 | 版本要求 |
|------|----------|
| Rust | 1.74+ |
| Cargo | 最新稳定版 |
| MySQL | 8.0+ (可选) |
| PostgreSQL | 15+ (可选) |
| SQLite | 3.40+ (内置) |

---

## 3. 测试矩阵

### 3.1 规模矩阵

| Scale Factor | 数据规模 | 行数 (约) | 内存需求 | 测试时间 | 状态 |
|--------------|----------|-----------|----------|----------|------|
| SF=0.1 | ~100MB | 866,602 | ~1GB | ~5 分钟 | ✅ 可执行 |
| SF=0.3 | ~300MB | 2.6M | ~3GB | ~15 分钟 | ⏳ 待验证 |
| SF=1.0 | ~1GB | 8.6M | ~8GB | ~60 分钟 | ⏳ 待验证 |
| SF=10 | ~10GB | 86M | ~64GB | > 4 小时 | ⏳ 内存限制 |

### 3.2 查询复杂度矩阵

| 类别 | 查询 | 复杂度 | SF=0.1 状态 |
|------|------|--------|--------------|
| 单表聚合 | Q1, Q4, Q6 | 低 | ✅ 通过 |
| 简单 JOIN | Q13, Q14, Q19, Q20, Q22 | 中 | ✅ 通过 |
| 复杂 JOIN | Q2, Q3, Q5, Q7, Q8, Q9, Q10, Q11, Q12, Q15, Q16, Q17, Q18, Q21 | 高 | ❌ 超时 |

### 3.3 对比系统

| 系统 | 版本 | 用途 | 状态 |
|------|------|------|------|
| SQLRustGo | 当前 develop/v2.9.0 | 被测系统 | ✅ |
| SQLite | 3.40+ | 对比基准 | ⏳ 待安装 |
| MySQL | 8.0+ | 对比系统 | ⏳ 待安装 |
| PostgreSQL | 15+ | 对比系统 | ⏳ 待安装 |

---

## 4. 测试执行计划

### 4.1 第一阶段: 环境准备 (预计 30 分钟)

#### 任务 1.1: 克隆最新代码

```bash
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo
git checkout develop/v2.9.0
git pull origin develop/v2.9.0
```

#### 任务 1.2: 构建项目

```bash
cargo build --all-features
```

#### 任务 1.3: 安装 TPC-H dbgen

```bash
cd /tmp
git clone https://github.com/electrum/tpch-dbgen.git
cd tpch-dbgen
make
```

#### 任务 1.4: 生成测试数据

```bash
# SF=0.1
cd /tmp/tpch-dbgen
mkdir -p ~/sqlrustgo-data/tpch-sf01
./dbgen -s 0.1 -f
mv *.tbl ~/sqlrustgo-data/tpch-sf01/

# SF=1.0
cd /tmp/tpch-dbgen
mkdir -p ~/sqlrustgo-data/tpch-sf1
./dbgen -s 1 -f
mv *.tbl ~/sqlrustgo-data/tpch-sf1/
```

### 4.2 第二阶段: SF=0.1 验证 (预计 30 分钟)

#### 任务 2.1: 快速查询测试 (Q1, Q4, Q6, Q13, Q14, Q19, Q20, Q22)

```bash
cd sqlrustgo

# 快速查询测试
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf01/ \
  --queries Q1,Q4,Q6,Q13,Q14,Q19,Q20,Q22 \
  --iterations 3 \
  --output results-sf01-fast.json

# 保存结果
cat results-sf01-fast.json
```

#### 任务 2.2: 记录结果

| 查询 | Avg (ms) | Min (ms) | Max (ms) | Rows | Status |
|------|----------|----------|----------|------|--------|
| Q1 | | | | | |
| Q4 | | | | | |
| Q6 | | | | | |
| Q13 | | | | | |
| Q14 | | | | | |
| Q19 | | | | | |
| Q20 | | | | | |
| Q22 | | | | | |

### 4.3 第三阶段: SF=1.0 基准测试 (预计 2 小时)

#### 任务 3.1: 快速查询测试 (同 SF=0.1)

```bash
./target/debug/sqlrustgo-bench-cli tpch-bench \
  --ddl scripts/sqlite_tpch_setup.sql \
  --data ~/sqlrustgo-data/tpch-sf1/ \
  --queries Q1,Q4,Q6,Q13,Q14,Q19,Q20,Q22 \
  --iterations 3 \
  --output results-sf1-fast.json
```

#### 任务 3.2: JOIN 查询测试 (带超时)

```bash
# 设置 5 分钟超时
for q in Q2 Q3 Q5 Q7 Q8 Q9 Q10 Q11 Q12 Q15 Q16 Q17 Q18 Q21; do
  echo "Testing $q..."
  timeout 300 ./target/debug/sqlrustgo-bench-cli tpch-bench \
    --ddl scripts/sqlite_tpch_setup.sql \
    --data ~/sqlrustgo-data/tpch-sf1/ \
    --queries $q \
    --iterations 1 \
    --output result-${q}.json || echo "TIMEOUT"
done
```

#### 任务 3.3: 记录结果

| 查询 | SF=0.1 (ms) | SF=1.0 (ms) | 扩展系数 | 状态 |
|------|--------------|-------------|---------|------|
| Q1 | 实测值 | 实测值 | 计算值 | ✅/❌ |
| Q2 | | | | |
| ... | | | | |

### 4.4 第四阶段: 对比测试 (预计 3 小时)

#### 任务 4.1: SQLite 对比

```bash
# 安装 SQLite
brew install sqlite3  # macOS
# apt install sqlite3  # Ubuntu

# 创建数据库
sqlite3 tpch-sf01.db < scripts/sqlite_tpch_setup.sql

# 导入数据
for table in customer nation region supplier part partsupp orders lineitem; do
  sqlite3 tpch-sf01.db ".mode csv" ".import ~/sqlrustgo-data/tpch-sf01/\${table}.tbl \${table}"
done

# 运行查询对比
echo ".mode column
.timer on
SELECT l_returnflag, SUM(l_quantity) FROM lineitem GROUP BY l_returnflag;" | sqlite3 tpch-sf01.db
```

#### 任务 4.2: MySQL 对比

```bash
# 安装 MySQL
brew install mysql  # macOS

# 启动并配置
mysql.server start
mysql -u root -e "ALTER USER 'root'@'localhost' IDENTIFIED WITH mysql_native_password BY 'details';"

# 创建数据库
mysql -u root -p'details' -e "CREATE DATABASE IF NOT EXISTS tpch_sf01;"
mysql -u root -p'details' tpch_sf01 < scripts/mysql_tpch_setup.sql

# 导入数据
for table in customer nation region supplier part partsupp orders lineitem; do
  mysql -u root -p'details' --local-infile=1 tpch_sf01 -e \
    "LOAD DATA LOCAL INFILE '~/sqlrustgo-data/tpch-sf01/\${table}.tbl' INTO TABLE \${table} FIELDS TERMINATED BY '|';"
done

# 运行查询
time mysql -u root -p'details' tpch_sf01 -e "SELECT ..." # Q1
```

#### 任务 4.3: PostgreSQL 对比

```bash
# 安装 PostgreSQL
brew install postgresql  # macOS

# 启动
pg_ctl -D /usr/local/var/postgres start

# 创建数据库
createdb tpch_sf01
psql -d tpch_sf01 -f scripts/pg_tpch_setup.sql

# 导入数据
for table in customer nation region supplier part partsupp orders lineitem; do
  psql -d tpch_sf01 -c "\\COPY \${table} FROM '~/sqlrustgo-data/tpch-sf01/\${table}.tbl' WITH (FORMAT csv, DELIMITER '|');"
done

# 运行查询
time psql -d tpch_sf01 -c "SELECT ..." # Q1
```

---

## 5. 结果收集模板

### 5.1 SQLRustGo 结果

```json
{
  "system": "sqlrustgo",
  "version": "develop/v2.9.0",
  "date": "2026-05-04",
  "scale_factor": "0.1",
  "queries": [
    {
      "name": "Q1",
      "avg_ms": 1337.98,
      "min_ms": 1300.00,
      "max_ms": 1400.00,
      "p50_ms": 1330.00,
      "p95_ms": 1390.00,
      "p99_ms": 1400.00,
      "rows": 3,
      "iterations": 3,
      "status": "PASS"
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

### 5.2 对比结果表

| 查询 | SQLRustGo (ms) | SQLite (ms) | MySQL (ms) | PostgreSQL (ms) | 最快系统 |
|------|-----------------|-------------|------------|-----------------|----------|
| Q1 | 1338 | | | | |
| Q4 | 435 | | | | |
| Q6 | 1024 | | | | |
| Q13 | 308 | | | | |
| Q14 | 257 | | | | |
| Q19 | 256 | | | | |
| Q20 | 255 | | | | |
| Q22 | 267 | | | | |

---

## 6. 问题跟踪

### 6.1 已知问题

| Issue | 描述 | 影响查询 | 优先级 |
|-------|------|---------|--------|
| JOIN-001 | 多表 JOIN 超时 | Q2-Q22 (除简单查询) | P0 - 阻塞 |
| HASH-JOIN-001 | 未实现 Hash Join | 复杂 JOIN | P1 - 重要 |
| CBO-001 | Join Order 选择不佳 | 复杂 JOIN | P1 - 重要 |

### 6.2 待优化项

1. **P0**: 实现 Hash Join 替代 Nest Loop Join
2. **P1**: 优化 CBO (Cost-Based Optimizer) 的 Join Order 选择
3. **P1**: 添加索引支持以加速等值连接
4. **P2**: 实现查询并行化
5. **P2**: 添加向量化执行

---

## 7. 报告模板

### 7.1 执行摘要

```markdown
# TPC-H 测试报告

**日期**: YYYY-MM-DD
**执行人**: HP Z6G4
**系统**: SQLRustGo develop/v2.9.0

## 执行摘要

| 指标 | 数值 |
|------|------|
| 测试规模 | SF=0.1, SF=1.0 |
| 通过查询 | X/22 |
| 失败查询 | Y/22 |
| 超时查询 | Z/22 |

## 性能摘要

| SF | 总时间 | 平均查询时间 | 最快查询 | 最慢查询 |
|----|--------|--------------|----------|----------|
| 0.1 | X ms | Y ms | Qx (Z ms) | Qy (Z ms) |
| 1.0 | X ms | Y ms | Qx (Z ms) | Qy (Z ms) |

## 与其他系统对比

| 系统 | SF=0.1 总时间 | SF=1.0 总时间 |
|------|---------------|---------------|
| SQLRustGo | | |
| SQLite | | |
| MySQL | | |
| PostgreSQL | | |

## 问题与建议

### 阻塞问题
1. ...

### 优化建议
1. ...
```

---

## 8. 验收标准

### 8.1 最小验收 (必须完成)

- [ ] SF=0.1 快速查询 (Q1, Q4, Q6, Q13, Q14, Q19, Q20, Q22) 执行成功
- [ ] SF=1.0 快速查询执行成功
- [ ] 生成完整测试报告

### 8.2 目标验收 (期望完成)

- [ ] SF=0.1 所有 22 个查询执行完成
- [ ] SF=1.0 所有 22 个查询执行完成
- [ ] 与 SQLite 对比测试完成
- [ ] 与 MySQL 对比测试完成
- [ ] 与 PostgreSQL 对比测试完成

### 8.3 扩展验收 (可选)

- [ ] SF=10 压力测试
- [ ] JOIN 算法优化后重新测试
- [ ] 并行查询优化后重新测试

---

## 9. 资源链接

- [TPC-H 官方规范](http://www.tpc.org/tpch/)
- [TPC-H dbgen GitHub](https://github.com/electrum/tpch-dbgen)
- [SQLRustGo 项目](http://192.168.0.252:3000/openclaw/sqlrustgo)
- [TPC-H 测试指南](./TPC-H-TEST-GUIDE-FULL.md)

---

## 10. 维护记录

| 日期 | 版本 | 修改内容 | 作者 |
|------|------|----------|------|
| 2026-05-04 | v1.0 | 初始版本 | yinglichina8848 |
