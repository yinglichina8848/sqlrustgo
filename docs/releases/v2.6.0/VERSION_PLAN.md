# v2.6.0 版本计划

> **版本**: v2.6.0
> **创建日期**: 2026-04-17
> **目标发布日期**: 2026-05-12 (GA)
> **维护人**: yinglichina8848

---

## 一、版本概述

v2.6.0 是 SQLRustGo 迈向 **生产就绪 (Production Ready)** 的关键版本。

**目标**: 达到替代 MySQL 用于简单生产环境的能力。

### 继承 Issue

| Issue | 标题 | 说明 |
|-------|------|------|
| #1501 | v2.6.0 详细开发计划 | 主计划 |
| #1497 | 功能已实现但未集成 | P0-1 功能集成 |
| #1498 | SQL-92 语法扩展支持计划 | P0-2 SQL 语法 |
| #1389 | MVCC 并发控制 - 快照隔离 | P0-3 MVCC SSI |
| #1480 | 覆盖率提升计划: 49% → 70% | P2 覆盖率 |
| #1379 | FOREIGN KEY 约束 | 外键完善 |

---

## 二、功能范围

### P0 - 必须完成 (生产就绪)

#### P0-1: 功能集成 (继承 Issue #1497)

| 功能模块 | 问题 | 开发任务 |
|----------|------|----------|
| 索引扫描 | 已实现但未启用 | 集成到查询计划，实现 should_use_index 逻辑 |
| CBO 优化器 | 已实现但未调用 | 实现统计信息驱动，默认启用优化 |
| 存储过程 | 已实现但未集成 | 集成到执行器流程 |
| 触发器 | 已实现但未集成 | 集成到执行器流程 |
| 外键约束 | Parser 完成，Executor 未验证 | 启用外键约束验证 |
| WAL 日志 | 已实现但未默认启用 | 默认启用 WAL |

#### P0-2: SQL 语法扩展 (继承 Issue #1498)

| 语法 | 失败 case 数 | 开发任务 |
|------|-------------|----------|
| 聚合函数 (COUNT, SUM, AVG, MIN, MAX) | 11 | Parser + Executor 实现 |
| JOIN 语法 | 14 | Parser + Executor 实现 |
| GROUP BY / HAVING | 8 | Parser + Executor 实现 |

#### P0-3: MVCC SSI (继承 Issue #1389)

| 任务 | 说明 |
|------|------|
| SSI (Serializable Snapshot Isolation) | 可串行化快照隔离 |
| SSI 事务冲突检测 | 检测并解决写冲突 |
| SSI 事务回滚机制 | 事务回滚逻辑 |
| SSI 与 MVCC 索引集成 | 索引支持 SSI |

### P1 - 重要功能

| 功能 | 说明 |
|------|------|
| DELETE 语句 | Parser + Executor 实现，4 个失败 case |
| FULL OUTER JOIN | HashJoin/SortMergeJoin 完整支持 |
| 外键约束完善 | RESTRICT、NO ACTION 动作实现 |

### P2 - 增强功能

| 功能 | 说明 |
|------|------|
| CREATE INDEX | Parser + Executor 实现，3 个失败 case |
| COUNT(DISTINCT) | Executor 实现，1 个失败 case |
| 覆盖率提升 | 49% → 55% → 62% → 70% |

---

## 三、开发时间线

| 版本 | 日期 | 目标 |
|------|------|------|
| v2.6.0-alpha | 2026-04-21 | P0 功能开发完成 |
| v2.6.0-beta | 2026-04-28 | P1 功能开发完成 |
| v2.6.0-rc1 | 2026-05-05 | RC 候选 |
| v2.6.0-GA | 2026-05-12 | 正式发布 |

---

## 四、验收标准

### P0 验收

```bash
# CBO 优化器测试
cargo test -p sqlrustgo-planner --lib

# 存储过程/触发器测试
cargo test -p sqlrustgo-executor --lib

# 索引扫描测试
cargo test -p sqlrustgo-storage --lib

# MVCC SSI 测试
cargo test mvcc_serializable
cargo test mvcc_index

# SQL 语法测试
cargo test sql_corpus  # 聚合函数 90%+
cargo test sql_corpus  # JOIN 90%+
cargo test sql_corpus  # GROUP BY 90%+
```

### P1 验收

```bash
# DELETE 语句测试
cargo test sql_corpus  # DELETE 90%+

# FULL OUTER JOIN 测试
cargo test sql_corpus  # FULL OUTER JOIN 100%

# 外键约束测试
cargo test fk_constraint
```

### P2 验收

```bash
# CREATE INDEX 测试
cargo test sql_corpus  # CREATE INDEX 90%+

# 覆盖率
cargo tarpaulin  # ≥ 70%
```

### 性能验收

```bash
# TPC-H SF=1
cargo run --bin tpch-benchmark -- --sf 1  # < 5s

# 并发压力测试
cargo test --test concurrency_stress_test  # 100% 通过
```

---

## 五、分支结构

```
main (稳定版本)
├── release/v2.5.0 (锁定)
├── release/v2.6.0 (待创建)
└── develop/v2.6.0 (开发分支)
```

---

## 六、测试策略

### SQL Regression Corpus 扩展

| 阶段 | 目标用例数 |
|------|-----------|
| v2.6.0 | +5000 条 |
| v2.6.1 | +2000 条 |
| v2.6.2 | +5000 条 |

### 目录结构

```
sql_corpus/
├── DML/
│   ├── SELECT/      # 1500+ 用例
│   ├── INSERT/      # 500+ 用例
│   ├── UPDATE/      # 500+ 用例
│   └── DELETE/      # 500+ 用例
├── DDL/
│   ├── CREATE_TABLE/
│   ├── ALTER_TABLE/
│   └── FOREIGN_KEY/
├── Transactions/
│   └── isolation_levels.sql
└── Special/
    └── NULL_semantics.sql
```

---

## 七、风险记录

| 风险 | 影响 | 缓解措施 | 状态 |
|------|------|----------|------|
| Catalog 类型未定义 | stored_proc 阻塞 | 优先定义 Catalog 类型 | ⚠️ |
| Parser 类型导出缺失 | trigger 阻塞 | 已通过 PR #1508 解决 | ✅ |
| 外键 Executor 验证 | 影响数据完整性 | 实现约束检查 | 🔴 |
| WAL 默认启用 | 影响兼容性 | 提供配置开关 | 🔴 |

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
