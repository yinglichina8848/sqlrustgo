# SQLRustGo v2.7.0 OO 架构文档

> **版本**: v2.7.0 (GA)
> **代号**: Enterprise Resilience
> **更新日期**: 2026-04-22

---

## 概述

本文档目录包含 SQLRustGo v2.7.0 的面向对象架构设计文档、模块设计、报告和用户指南。

---

## 目录结构

```
oo/
├── architecture/          # 架构设计文档
│   └── ARCHITECTURE_V2.7.md
├── modules/             # 模块设计文档
│   ├── mvcc/
│   ├── wal/
│   ├── executor/
│   ├── parser/
│   ├── storage/
│   ├── optimizer/
│   ├── catalog/
│   ├── planner/
│   ├── transaction/
│   ├── server/
│   ├── graph/
│   ├── vector/
│   ├── unified-query/
│   └── bench/
├── reports/             # 分析报告
│   ├── SQL92_COMPLIANCE.md
│   └── PERFORMANCE_ANALYSIS.md
└── user-guide/          # 用户指南
    └── USER_MANUAL.md
```

---

## 文档索引

### 架构文档

| 文档 | 说明 |
|------|------|
| [ARCHITECTURE_V2.7.md](./architecture/ARCHITECTURE_V2.7.md) | v2.7.0 整体架构设计 |

### 模块文档

| 模块 | 文档 | 说明 |
|------|------|------|
| MVCC | [MVCC_DESIGN.md](./modules/mvcc/MVCC_DESIGN.md) | 多版本并发控制 |
| WAL | [WAL_DESIGN.md](./modules/wal/WAL_DESIGN.md) | 预写日志 |
| Executor | [EXECUTOR_DESIGN.md](./modules/executor/EXECUTOR_DESIGN.md) | 执行器 |
| Parser | [PARSER_DESIGN.md](./modules/parser/PARSER_DESIGN.md) | SQL 解析器 |
| Storage | [STORAGE_DESIGN.md](./modules/storage/STORAGE_DESIGN.md) | 存储引擎 |
| Optimizer | [OPTIMIZER_DESIGN.md](./modules/optimizer/OPTIMIZER_DESIGN.md) | 查询优化器 |
| Catalog | [CATALOG_DESIGN.md](./modules/catalog/CATALOG_DESIGN.md) | 元数据管理 |
| Planner | [PLANNER_DESIGN.md](./modules/planner/PLANNER_DESIGN.md) | 查询规划器 |
| Transaction | [TRANSACTION_DESIGN.md](./modules/transaction/TRANSACTION_DESIGN.md) | 事务管理 |
| Server | [SERVER_DESIGN.md](./modules/server/SERVER_DESIGN.md) | 服务器 |
| Graph | [GRAPH_DESIGN.md](./modules/graph/GRAPH_DESIGN.md) | 图引擎 |
| Vector | [VECTOR_DESIGN.md](./modules/vector/VECTOR_DESIGN.md) | 向量索引 |
| Unified Query | [UNIFIED_QUERY_DESIGN.md](./modules/unified-query/UNIFIED_QUERY_DESIGN.md) | 统一查询 |
| Bench | [BENCH_DESIGN.md](./modules/bench/BENCH_DESIGN.md) | 基准测试 |

### 报告

| 文档 | 说明 |
|------|------|
| [SQL92_COMPLIANCE.md](./reports/SQL92_COMPLIANCE.md) | SQL-92 合规性报告 |
| [PERFORMANCE_ANALYSIS.md](./reports/PERFORMANCE_ANALYSIS.md) | 性能分析报告 |

### 用户指南

| 文档 | 说明 |
|------|------|
| [USER_MANUAL.md](./user-guide/USER_MANUAL.md) | 用户手册 |

---

## 版本特性

v2.7.0 重点实现企业级韧性：

- WAL 崩溃恢复
- FK 稳定性增强
- 备份恢复机制
- 审计证据链
- 统一检索 API (lex/vec/graph/hybrid)
- 混合检索重排

---

## 相关文档

- [../VERSION_PLAN.md](../VERSION_PLAN.md) - 版本计划
- [../RELEASE_NOTES.md](../RELEASE_NOTES.md) - 发布说明
- [../FEATURE_MATRIX.md](../FEATURE_MATRIX.md) - 功能矩阵

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-22*
