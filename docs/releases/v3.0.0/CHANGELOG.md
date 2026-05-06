# v3.0.0 Changelog

> **版本**: v3.0.0-alpha
> **日期**: 2026-05-06

---

## [3.0.0-alpha] (2026-05-06)

### Added

#### 优化器
- `planner::optimizer.rs` 完整重写，桥接至 `crates/optimizer/rules.rs` 真实实现
- ConstantFolding 激活（`SELECT 1+2` → 返回 `3`）
- PredicatePushdown 激活（filter 下推至 TableScan 层）
- ProjectionPruning 激活（冗余列读取消除）

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

### Changed

- 优化器调用链：从占位符实现 → 真实 CBO 规则
- `cargo test --all-features`: 86 测试全部通过

### Known Issues

- SimpleCostModel 未接入 planner
- TPC-H SF=1 存在 OOM 风险
- optimizer/executor 模块覆盖率未达 GA 标准

---

## [2.9.0] (2026-04-??)

详见 [v2.9.0 CHANGELOG](../v2.9.0/CHANGELOG.md)