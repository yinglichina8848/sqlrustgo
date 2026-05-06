# v3.0.0 Alpha Release Notes

> **版本**: v3.0.0-alpha
> **日期**: 2026-05-06
> **代号**: Performance Core
> **从**: v2.9.0

---

## 变更概述

v3.0.0-alpha 是进入 **Alpha 阶段** 的第一个版本，聚焦于优化器激活、SQL 兼容性补全和事务隔离级别正确性验证。

---

## 主要新增功能

### 优化器激活 (A-OPT)

| 功能 | 描述 | PR/Issue |
|------|------|----------|
| PredicatePushdown 真实调用 | `planner::optimizer.rs` 桥接到 `crates/optimizer/rules.rs` | #358 |
| ProjectionPruning 桥接 | 列裁剪下推至 TableScan | #358 |
| ConstantFolding 桥接 | 常量折叠端到端验证 `1+2→3` | #358 |

### SQL 兼容性 (A-SQL)

| 功能 | 描述 | PR/Issue |
|------|------|----------|
| IN 子查询 | `SELECT * FROM t WHERE c IN (SELECT ...)` | cf4d733e |
| EXISTS 子查询 | `SELECT * FROM t WHERE EXISTS (SELECT 1)` | cf4d733e |
| CASE 表达式 | `CASE WHEN ... THEN ... ELSE ... END` | #368 |
| COALESCE | `COALESCE(NULL, NULL, c)` | #368 |
| DISTINCT | `SELECT DISTINCT c FROM t` | #368 |
| INSERT...SELECT | SELECT 结果直接插入目标表 | #368 |

### 窗口函数 (A-WINDOW)

| 函数 | 状态 | 备注 |
|------|------|------|
| ROW_NUMBER | ✅ | |
| RANK | ✅ | |
| DENSE_RANK | ✅ | |
| NTILE | ✅ | |
| LEAD | ✅ | |
| LAG | ✅ | |
| FIRST_VALUE | ✅ | |
| LAST_VALUE | ✅ | |
| NTH_VALUE | ✅ | |

### CTE 执行

| 功能 | 状态 | 备注 |
|------|------|------|
| WITH 子句 | ✅ | 非递归 CTE |
| 递归 CTE | ✅ | 深度限制已设置 |
| 多 CTE 引用 | ✅ | |
| CTE 与 JOIN 混用 | ✅ | |

### 事务隔离 (A-TX)

| 隔离级别 | 状态 | 备注 |
|----------|------|------|
| Read Committed | ✅ | |
| Snapshot Isolation | ✅ | MVCC 实现 |
| Serializable (SSI) | ✅ | Proof-026 7/7 通过 |

### EXPLAIN 与信息schema

| 功能 | 状态 | 备注 |
|------|------|------|
| EXPLAIN ANALYZE | ✅ | 计划+耗时 |
| INFORMATION_SCHEMA | ✅ | TABLES/COLUMNS/STATISTICS |
| SHOW VARIABLES | ✅ | |

### 性能组件

| 功能 | 状态 | 备注 |
|------|------|------|
| CBO 规则（3条） | ✅ | PredicatePushdown/ProjectionPruning/ConstantFolding |
| CBO 代价模型 | ⚠️ | SimpleCostModel 存在，未接入 planner |
| 连接池 | ✅ | max_connections 配置 |
| 查询缓存 | ✅ | DML 失效 |
| Group Commit | ✅ | WAL 批量 fsync |
| SSL/TLS | ✅ | rustls + 自签名证书 |

---

## SQL Corpus 状态

| 指标 | 结果 |
|------|------|
| 通过率 | **100%** (485/485) |
| 覆盖率 | SQL-92 标准语法 |

---

## TPC-H 状态

| SF | 状态 | 耗时 | 备注 |
|----|------|------|------|
| 0.1 | ✅ 22/22 | ~10.9s | Q1~Q22 全部可运行 |
| 1 | ⚠️ 待验证 | — | 曾有 OOM 记录 |

---

## 代码质量

| 检查项 | 状态 |
|--------|------|
| `cargo build --all-features` | ✅ |
| `cargo clippy --all-features` | ✅ 零警告 |
| `cargo fmt --all -- --check` | ✅ |
| `cargo test --workspace` | ✅ |
| `cargo audit` | ✅ |

---

## 已知问题 (Alpha 待解决)

1. **CBO 代价模型未集成**: SimpleCostModel 存在但未接入 planner 的 plan 选择逻辑
2. **TPC-H SF=1 OOM**: SF=1 全量查询曾发生 OOM，内存治理待完成
3. **覆盖率缺口**: optimizer、executor 模块覆盖率未达到 GA 目标

---

## 升级路径

详见 [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)

---

## 下一步

- **Beta 阶段**: 性能优化 (TPC-H p99<2s)、CBO 代价模型集成、内存治理
- **GA 阶段**: 覆盖率 ≥85%、TPC-H SF=1 稳定、混沌工程验证