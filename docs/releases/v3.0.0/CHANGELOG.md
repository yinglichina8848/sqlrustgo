# v3.0.0 变更日志

> **版本**: v3.0.0
> **发布日期**: 2026-05-07

---

## v3.0.0 GA (2026-05-07)

### 新增功能

#### Beta Gate 测试体系

- **B-S1**: concurrency_stress_test - 并发压力测试 (9 tests)
- **B-S2**: crash_recovery_test - 崩溃恢复测试 (8 tests)
- **B-S3**: long_run_stability_test - 长期稳定性测试 (10 tests)
- **B-S4**: wal_integration_test - WAL 集成测试 (16 tests)
- **B-S5**: network_tcp_smoke_test - 网络 TCP 冒烟测试 (6 tests)
- **B-S6**: ssi_stress_test - SSI 隔离级别压力测试

#### 性能基准

- **QPS 基线**: 建立 v3.0.0 性能基线
  - simple_select: 398,353 QPS
  - update: 43,121 QPS (E-09 ≥10,000 ✅)
  - delete: 64,896 QPS (E-09 ≥10,000 ✅)
- **TPC-H 基线**: SF=0.1 全部 22 查询可运行
- **R10/GA-10**: 完整的 check_perf_baseline.sh 脚本

#### 测试基础设施

- **wal_crash_recovery_test**: WAL 崩溃恢复测试 (11 tests)
- **query_cache_test**: 查询缓存测试 (13 tests)

### 改进

- 格式化修复 (cargo fmt)
- EngineConfig 标准化使用
- 移除 deprecated API 使用

### 测试与质量

- 461 测试全部通过
- clippy 零警告
- fmt 检查通过

---

## v3.0.0-alpha (2026-05-06)

### Added

#### 优化器
- `planner::optimizer.rs` 完整重写，桥接至 `crates/optimizer/rules.rs` 真实实现
- ConstantFolding 激活（`SELECT 1+2` → 返回 `3`）
- PredicatePushdown 激活（filter 下推至 TableScan 层）
- ProjectionPruning 激活（冗余列读消除）

#### SQL 兼容性
- IN 子查询支持 (`WHERE c IN (SELECT ...)`)
- EXISTS 子查询支持 (`WHERE EXISTS (SELECT 1)`)
- CASE 表达式完整支持
- COALESCE 函数
- DISTINCT 去重
- INSERT...SELECT

#### 窗口函数
- ROW_NUMBER, RANK, DENSE_RANK
- NTILE, LEAD, LAG
- FIRST_VALUE, LAST_VALUE, NTH_VALUE

#### CTE 执行
- 非递归 WITH 子句
- 递归 CTE（带深度限制）
- 多 CTE 引用链
- CTE 与 JOIN 混用

#### 事务
- Read Committed 隔离级别
- Snapshot Isolation (MVCC)
- Serializable SSI (Proof-026 7/7 通过)

#### 性能组件
- CBO 三规则（PredicatePushdown/ProjectionPruning/ConstantFolding）
- 连接池（max_connections 配置）
- 查询缓存（DML 失效机制）
- Group Commit（WAL 批量 fsync）
- SSL/TLS（rustls + 自签名证书）

#### 信息系统
- EXPLAIN ANALYZE（计划 + 耗时）
- INFORMATION_SCHEMA (TABLES/COLUMNS/STATISTICS)
- SHOW VARIABLES

#### 测试
- SQL Corpus: 100% (485/485)
- TPC-H SF=0.1: 22/22 可运行

---

## v2.9.0 (2026-05-05)

详见 [v2.9.0 CHANGELOG](../v2.9.0/CHANGELOG.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-07*