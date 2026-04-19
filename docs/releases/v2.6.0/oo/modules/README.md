# SQLRustGo v2.6.0 模块设计文档

**版本**: v2.6.0 (生产就绪版本)
**发布日期**: 2026-04-17

---

## 概述

本目录包含 SQLRustGo v2.6.0 各核心模块的设计文档。

---

## 模块索引

### 核心引擎模块

| 模块 | 文档 | 说明 |
|------|------|------|
| MVCC | [MVCC_DESIGN.md](./mvcc/MVCC_DESIGN.md) | 多版本并发控制 (SSI) |
| WAL | [WAL_DESIGN.md](./wal/WAL_DESIGN.md) | 预写日志 |
| Storage | [STORAGE_DESIGN.md](./storage/STORAGE_DESIGN.md) | 存储引擎 |
| Executor | [EXECUTOR_DESIGN.md](./executor/EXECUTOR_DESIGN.md) | 向量化执行器 |

### 查询处理模块

| 模块 | 文档 | 说明 |
|------|------|------|
| Parser | [PARSER_DESIGN.md](./parser/PARSER_DESIGN.md) | SQL 解析器 |
| Optimizer | [OPTIMIZER_DESIGN.md](./optimizer/OPTIMIZER_DESIGN.md) | 查询优化器 |
| Planner | [PLANNER_DESIGN.md](./planner/PLANNER_DESIGN.md) | 查询规划器 |
| Catalog | [CATALOG_DESIGN.md](./catalog/CATALOG_DESIGN.md) | 元数据管理 |

### 高级功能模块

| 模块 | 文档 | 说明 |
|------|------|------|
| Graph | [GRAPH_DESIGN.md](./graph/GRAPH_DESIGN.md) | 图引擎 |
| Vector | [VECTOR_DESIGN.md](./vector/VECTOR_DESIGN.md) | 向量索引 |
| Unified Query | [UNIFIED_QUERY_DESIGN.md](./unified-query/UNIFIED_QUERY_DESIGN.md) | 统一查询 |

### 工具与服务模块

| 模块 | 文档 | 说明 |
|------|------|------|
| Server | [SERVER_DESIGN.md](./server/SERVER_DESIGN.md) | 服务器 |
| Bench | [BENCH_DESIGN.md](./bench/BENCH_DESIGN.md) | 基准测试 |

---

## v2.6.0 核心改进

### SIMD 加速

v2.6.0 全面支持 SIMD 加速:

- AVX-512 指令集
- 向量化聚合函数
- SIMD 过滤
- SIMD HASH JOIN

### MVCC SSI

v2.6.0 实现 Serializable Snapshot Isolation:

- 乐观并发控制
- 冲突检测
- 事务回滚

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [ARCHITECTURE_V2.6.md](../architecture/ARCHITECTURE_V2.6.md) | 整体架构 |
| [PERFORMANCE_ANALYSIS.md](../reports/PERFORMANCE_ANALYSIS.md) | 性能分析 |
| [SQL92_COMPLIANCE.md](../reports/SQL92_COMPLIANCE.md) | SQL 合规报告 |

---

*模块设计文档索引 v2.6.0*
*最后更新: 2026-04-17*
