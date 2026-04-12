# SQLRustGo v2.1.0 测试手册

**版本**: v2.1.0
**更新日期**: 2026-04-02

---

## 一、测试概述

### 1.1 测试目标

- 确保代码质量符合发布标准
- 验证所有功能正常工作
- 保持代码覆盖率 ≥80%

### 1.2 测试分类

| 测试类型 | 说明 | 覆盖模块 |
|----------|------|----------|
| 单元测试 | 单个模块功能测试 | parser, planner, executor, storage |
| 集成测试 | 多模块协作测试 | 跨模块功能 |
| 回归测试 | 防止功能退化 | 全量测试 |
| 性能测试 | 性能指标验证 | 基准测试 |

---

## 二、快速运行

### 2.1 运行所有测试

```bash
# 完整测试
cargo test --workspace

# 带覆盖率
cargo tarpaulin -v
```

### 2.2 分类测试

```bash
# 单元测试
cargo test --lib

# 集成测试
cargo test --test '*_test'

# 回归测试
cargo test --test regression_test

# 性能测试
cargo bench
```

---

## 三、单元测试

### 3.1 模块测试命令

```bash
# Parser 模块
cargo test -p sqlrustgo-parser

# Planner 模块
cargo test -p sqlrustgo-planner

# Executor 模块
cargo test -p sqlrustgo-executor

# Storage 模块
cargo test -p sqlrustgo-storage

# Types 模块
cargo test -p sqlrustgo-types

# Common 模块
cargo test -p sqlrustgo-common

# Server 模块
cargo test -p sqlrustgo-server

# Tools 模块
cargo test -p sqlrustgo-tools
```

### 3.2 特定测试文件

```bash
# 聚合类型测试
cargo test aggregate_type_test

# 备份功能测试
cargo test backup_test

# B+树测试
cargo test bplus_tree_test

# 缓冲区池测试
cargo test buffer_pool_test

# 文件存储测试
cargo test file_storage_test

# 本地执行器测试
cargo test local_executor_test

# mysqldump 测试
cargo test mysqldump_test

# 优化器成本测试
cargo test optimizer_cost_test

# 优化器规则测试
cargo test optimizer_rules_test

# 解析器标记测试
cargo test parser_token_test

# Prometheus 测试
cargo test prometheus_test

# 查询缓存测试
cargo test query_cache_test

# 健康检查端点测试
cargo test server_health_test

# 慢查询日志测试
cargo test slow_query_log_test

# 类型值测试
cargo test types_value_test

# 向量化测试
cargo test vectorization_test
```

---

## 四、集成测试

### 4.1 集成测试命令

```bash
# 自增测试
cargo test --test autoinc_test

# 批量插入测试
cargo test --test batch_insert_test

# 二进制格式测试
cargo test --test binary_format_test

# 边界测试
cargo test --test boundary_test

# 目录一致性测试
cargo test --test catalog_consistency_test

# 校验和损坏测试
cargo test --test checksum_corruption_test

# 列式存储测试
cargo test --test columnar_storage_test

# 日期时间类型测试
cargo test --test datetime_type_test

# 分布式事务测试
cargo test --test distributed_transaction_test

# 错误处理测试
cargo test --test error_handling_test

# 执行器测试
cargo test --test executor_test

# FK 动作测试
cargo test --test fk_actions_test

# 健康端点测试
cargo test --test health_endpoint_test

# 索引集成测试
cargo test --test index_integration_test

# 索引扫描测试
cargo test --test index_scan_test

# 联接测试
cargo test --test join_test

# MySQL 兼容性测试
cargo test --test mysql_compatibility_test

# Parquet 测试
cargo test --test parquet_test

# 物理备份测试
cargo test --test physical_backup_test

# 回归测试
cargo test --test regression_test

# 保存点测试
cargo test --test savepoint_test

# 会话配置测试
cargo test --test session_config_test

# 集合操作测试
cargo test --test set_operations_test

# 存储集成测试
cargo test --test storage_integration_test

# 存储过程测试
cargo test --test stored_proc_test

# 表扫描测试
cargo test --test table_scan_test

# 教学场景测试
cargo test --test teaching_scenario_test

# TPC-H 测试
cargo test --test tpch_test
cargo test --test tpch_benchmark
cargo test --test tpch_full_test

# 触发器测试
cargo test --test trigger_test

# 视图测试
cargo test --test view_test

# WAL 集成测试
cargo test --test wal_integration_test

# 窗口函数测试
cargo test --test window_function_test
```

### 4.1.1 v2.1.0 实际测试结果

| 测试类型 | 测试数 | 通过数 | 通过率 | 状态 |
|----------|--------|--------|--------|------|
| 回归测试 | 1029 | 1022 | 99.3% | ✅ |
| TPC-H 测试 | 11 | 11 | 100% | ✅ |
| TPC-H 基准 | 12 | 12 | 100% | ✅ |
| TPC-H 完整 | 34 | 34 | 100% | ✅ |
| AgentSQL 测试 | 32 | 32 | 100% | ✅ |

**回归测试执行命令：**
```bash
cargo test --test regression_test
```

**回归测试结果摘要：**
- 总测试数：1029
- 通过：1022
- 失败：7
- 通过率：99.3%

### 4.2 教学场景测试

```bash
# 客户端/服务器测试
cargo test --test teaching_scenario_client_server_test
```

### 4.3 安全测试

```bash
# RBAC 权限测试
cargo test --test auth_rbac_test

# 日志配置测试
cargo test --test logging_test
```

### 4.4 SQL CLI 测试

```bash
# SQL CLI UPDATE/DELETE 测试
cargo test --test sql_cli_test
```

---

## 五、TPC-H 测试

### 5.1 运行 TPC-H 测试

```bash
# 基本 TPC-H 测试 (11个查询)
cargo test --test tpch_test

# TPC-H 基准测试 (12个查询)
cargo test --test tpch_benchmark

# TPC-H 完整测试 (34个查询)
cargo test --test tpch_full_test
```

### 5.2 TPC-H Q1-Q22 支持状态

| 查询 | 状态 | 说明 |
|------|------|------|
| Q1-Q22 | Phase 1 | BETWEEN/DATE/IN 已实现 |
| Q1-Q22 | Phase 2+3 | 完整支持进行中 |

---

## 六、回归测试

### 6.1 运行回归测试

```bash
# 运行完整回归测试套件
cargo test --test regression_test

# 运行特定类别
cargo test --test regression_test -- --nocapture

# 查看测试分类
cargo test --test regression_test -- --list
```

### 6.2 回归测试分类

| 分类 | 测试文件数 | 说明 |
|------|-----------|------|
| SQL 解析器 | 15+ | Parser 功能 |
| 执行器 | 20+ | Executor 功能 |
| 优化器 | 10+ | Optimizer 功能 |
| 存储 | 15+ | Storage 功能 |
| 事务 | 10+ | Transaction 功能 |
| 性能 | 10+ | Benchmark |
| 安全 | 5+ | RBAC/Firewall |

---

## 七、性能测试

### 7.1 基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench tpch_benchmark
cargo bench --bench query_cache_benchmark
```

### 7.2 性能目标

| 指标 | 目标 | 测试命令 |
|------|------|----------|
| QPS (50并发) | ≥1000 | `cargo bench --bench tpch_benchmark` |
| P50 延迟 | <50ms | 基准测试输出 |
| P99 延迟 | <100ms | 基准测试输出 |
| 并发连接数 | ≥50 | 压力测试 |

---

## 八、覆盖率

### 8.1 生成覆盖率报告

```bash
# 安装 tarpaulin (如果未安装)
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin -v --out html

# 查看覆盖率
cargo tarpaulin -v --fail-under 80
```

### 8.2 覆盖率目标

| 模块 | 目标覆盖率 |
|------|-----------|
| parser | ≥80% |
| planner | ≥80% |
| executor | ≥80% |
| storage | ≥80% |
| types | ≥80% |
| **总体** | **≥80%** |

---

## 九、持续集成

### 9.1 CI 测试流程

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build
        run: cargo build --release
      - name: Test
        run: cargo test --workspace
      - name: Coverage
        run: cargo tarpaulin --fail-under 80
      - name: Clippy
        run: cargo clippy -- -D warnings
```

### 9.2 回归测试流程

```yaml
# .github/workflows/regression.yml
name: Regression
on:
  schedule:
    - cron: '0 2 * * *'  # 每天凌晨2点
jobs:
  regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Regression
        run: cargo test --test regression_test
      - name: Report
        run: cargo tarpaulin -o html
```

---

## 十、测试数据

### 10.1 测试数据库

```sql
-- TPC-H 测试数据
-- SF=1 (1GB)
-- SF=10 (10GB)

-- 创建 TPC-H 表
CREATE TABLE lineitem (...);
CREATE TABLE orders (...);
CREATE TABLE customer (...);
CREATE TABLE supplier (...);
CREATE TABLE part (...);
CREATE TABLE partsupp (...);
CREATE TABLE nation (...);
CREATE TABLE region (...);
```

### 10.2 测试脚本

```bash
# 生成 TPC-H 数据
cargo run --bin tpch_gen --scale-factor 1

# 加载测试数据
cargo run --bin tpch_load --database tpch
```

---

## 附录 A: 测试命令速查

| 测试类型 | 命令 |
|----------|------|
| 所有测试 | `cargo test --workspace` |
| 单元测试 | `cargo test --lib` |
| 集成测试 | `cargo test --test '*_test'` |
| 回归测试 | `cargo test --test regression_test` |
| 性能测试 | `cargo bench` |
| 覆盖率 | `cargo tarpaulin --fail-under 80` |
| 代码规范 | `cargo clippy -- -D warnings` |
| 格式化 | `cargo fmt --check` |

---

## 附录 B: 常见问题

### Q: 测试失败怎么办？
A: 查看详细输出 `cargo test -- --nocapture`

### Q: 覆盖率不足怎么办？
A: 增加测试用例，参考 `tests/integration/`

### Q: 如何跳过慢测试？
A: `cargo test --test '*_test' -- --skip slow_test`

---

*测试手册 v2.1.0*
