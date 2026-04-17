# v2.6.0 集成性能测试计划

> **版本**: v2.6.0
> **创建日期**: 2026-04-17
> **维护人**: yinglichina8848

---

## 一、测试目标

### 1.1 核心指标

| 指标 | 当前值 | v2.6.0 目标 |
|------|---------|--------------|
| 单元测试覆盖率 | 49% | ≥70% |
| SQL Corpus 通过率 | 93.2% | ≥95% |
| TPC-H SF1 通过率 | 95%+ | 100% |
| Sysbench QPS | - | ≥1000 |

### 1.2 测试类型覆盖

| 测试类型 | 优先级 | 说明 |
|----------|--------|------|
| 单元测试 | P0 | 模块级功能验证 |
| SQL Corpus | P0 | SQL 语法回归测试 |
| TPC-H | P0 | OLAP 性能测试 |
| Sysbench | P0 | OLTP 压测 |
| 集成测试 | P1 | 模块间协作 |
| 压力测试 | P1 | 并发/长时间运行 |
| 协议测试 | P1 | MySQL 协议兼容 |

---

## 二、测试套件

### 2.1 单元测试

```bash
# 运行所有单元测试
cargo test --lib

# 按模块测试
cargo test -p sqlrustgo-parser --lib
cargo test -p sqlrustgo-planner --lib
cargo test -p sqlrustgo-executor --lib
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-optimizer --lib
```

### 2.2 SQL Corpus 回归测试

```bash
# 运行所有 SQL 测试
cargo test -p sqlrustgo-sql-corpus

# 按类别运行
cargo test sql_corpus -- SELECT
cargo test sql_corpus -- JOIN
cargo test sql_corpus -- AGGREGATE
cargo test sql_corpus -- DELETE
cargo test sql_corpus -- UPDATE
```

**目标**: ≥95% 通过率

### 2.3 TPC-H 性能测试

```bash
# SF=1 完整测试
cargo test -p sqlrustgo-tpch --test tpch_sf1

# 单个查询测试
cargo test -p sqlrustgo-tpch --test tpch_q1
cargo test -p sqlrustgo-tpch --test tpch_q2

# 性能基准
cargo bench --bench tpch_sf1
```

**目标**: 100% 通过，SF1 总时间 < 5s

### 2.4 Sysbench 压测

```bash
# 启动 SQLRustGo
cargo run --bin sqlrustgo -- --port 3307

# 安装 sysbench (macOS)
brew install sysbench

# 数据准备
sysbench oltp_read_write \
    --db-driver=mysql \
    --mysql-host=127.0.0.1 \
    --mysql-port=3307 \
    --mysql-user=root \
    --threads=50 \
    --tables=1 \
    --table-size=100000 \
    prepare

# 运行测试
sysbench oltp_read_write \
    --db-driver=mysql \
    --mysql-host=127.0.0.1 \
    --mysql-port=3307 \
    --mysql-user=root \
    --threads=50 \
    --time=60 \
    run
```

**目标**:
| 指标 | 阈值 |
|------|-------|
| oltp_point_select QPS | ≥ 1000 |
| oltp_read_only QPS | ≥ 800 |
| oltp_read_write QPS | ≥ 500 |
| P99 Latency | < 200ms |

---

## 三、门禁检查清单

### 3.1 发布前必须通过

| # | 检查项 | 阈值 | 验证命令 |
|---|--------|------|-----------|
| 1 | 单元测试 | 100% 通过 | `cargo test --lib` |
| 2 | SQL Corpus | ≥95% 通过 | `cargo test -p sqlrustgo-sql-corpus` |
| 3 | TPC-H SF1 | 100% 通过 | `cargo test -p sqlrustgo-tpch --test tpch_sf1` |
| 4 | Sysbench QPS | ≥1000 | sysbench run |
| 5 | 覆盖率 | ≥70% | `cargo tarpaulin` |
| 6 | Clippy | 0 警告 | `cargo clippy` |
| 7 | Format | 无问题 | `cargo fmt --check` |
| 8 | Build | 成功 | `cargo build --release` |

### 3.2 门禁执行流程

```bash
#!/bin/bash
# gate-check.sh - 发布前门禁检查

set -e

echo "=== Gate Check Start ==="

# 1. 编译检查
echo "[1/8] Build check..."
cargo build --release
echo "✓ Build passed"

# 2. Format 检查
echo "[2/8] Format check..."
cargo fmt --check
echo "✓ Format passed"

# 3. Clippy 检查
echo "[3/8] Clippy check..."
cargo clippy -- -D warnings
echo "✓ Clippy passed"

# 4. 单元测试
echo "[4/8] Unit tests..."
cargo test --lib
echo "✓ Unit tests passed"

# 5. SQL Corpus
echo "[5/8] SQL Corpus..."
cargo test -p sqlrustgo-sql-corpus || (echo "⚠ SQL Corpus failed"; exit 1)
echo "✓ SQL Corpus passed"

# 6. TPC-H
echo "[6/8] TPC-H SF1..."
cargo test -p sqlrustgo-tpch --test tpch_sf1 || (echo "⚠ TPC-H failed"; exit 1)
echo "✓ TPC-H passed"

# 7. 覆盖率 (仅 Beta+)
echo "[7/8] Coverage..."
cargo tarpaulin --output-dir ./target/tarpaulin || true
echo "✓ Coverage checked"

# 8. Sysbench (仅 RC+)
echo "[8/8] Sysbench..."
# 需要先启动数据库
# sysbench oltp_read_write run || (echo "⚠ Sysbench failed"; exit 1)
echo "✓ Sysbench checked (manual)"

echo "=== Gate Check Complete ==="
```

---

## 四、测试时间线

### 4.1 Alpha 阶段 (2026-04-21)

| 周 | 测试类型 | 目标 |
|----|----------|------|
| 1 | 单元测试 | 通过率 ≥80% |
| 2 | SQL Corpus | 通过率 ≥90% |

### 4.2 Beta 阶段 (2026-04-28)

| 周 | 测试类型 | 目标 |
|----|----------|------|
| 3 | TPC-H | 100% 通过 |
| 4 | Sysbench | QPS ≥ 1000 |

### 4.3 RC 阶段 (2026-05-05)

| 周 | 测试类型 | 目标 |
|----|----------|------|
| 5 | 压力测试 | 72h 稳定运行 |
| 6 | 完整门禁 | 全部通过 |

### 4.4 GA 阶段 (2026-05-12)

| 检查项 | 目标 |
|--------|------|
| 单元测试 | 100% |
| SQL Corpus | ≥95% |
| TPC-H | 100% |
| Sysbench | ≥1000 QPS |
| 覆盖率 | ≥70% |

---

## 五、CI 集成

### 5.1 GitHub Actions Workflow

```yaml
name: Release Gate
on:
  pull_request:
    branches: [main, develop/v2.6.0]
  workflow_dispatch:

jobs:
  gate-check:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: cargo build --release

      - name: Format
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Unit Tests
        run: cargo test --lib

      - name: SQL Corpus
        run: cargo test -p sqlrustgo-sql-corpus

      - name: TPC-H
        run: cargo test -p sqlrustgo-tpch --test tpch_sf1

      - name: Coverage
        run: cargo tarpaulin --output-dir ./target/tarpaulin
```

### 5.2 每日性能测试

```yaml
name: Daily Performance
on:
  schedule:
    - cron: '0 0 * * *'  # 每天 UTC 0:00
  workflow_dispatch:

jobs:
  performance:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v4

      - name: Run Sysbench
        run: |
          cargo run --bin sqlrustgo -- --port 3307 &
          sleep 5
          sysbench oltp_read_write --threads=50 --time=60 run
```

---

## 六、相关 Issue

- Issue #1521: SQLRustGo Sysbench 兼容性测试计划
- Issue #1498: SQL-92 语法支持
- Issue #1497: 功能集成
- Issue #1480: 覆盖率提升计划
- Issue #1560: SQL-92 功能缺失 (4个测试失败)

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |

---

*维护人: yinglichina8848*
*更新频率: 每次版本发布*