# SQLRustGo v2.7.0 模块设计文档

**版本**: v2.7.0 (GA)
**发布日期**: 2026-04-22

---

## 概述

本目录包含 SQLRustGo v2.7.0 各核心模块的设计文档。

---

## 模块索引

### 核心引擎模块

| 模块 | 文档 | 说明 |
|------|------|------|
| MVCC | [MVCC_DESIGN.md](./mvcc/MVCC_DESIGN.md) | 多版本并发控制 |
| WAL | [WAL_DESIGN.md](./wal/WAL_DESIGN.md) | 预写日志 (v2.7.0 重点) |
| Storage | [STORAGE_DESIGN.md](./storage/STORAGE_DESIGN.md) | 存储引擎 |
| Executor | [EXECUTOR_DESIGN.md](./executor/EXECUTOR_DESIGN.md) | 执行器 |

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
| Transaction | [TRANSACTION_DESIGN.md](./transaction/TRANSACTION_DESIGN.md) | 事务管理 |
| Bench | [BENCH_DESIGN.md](./bench/BENCH_DESIGN.md) | 基准测试 |

---

## v2.7.0 核心改进

### WAL 崩溃恢复 (T-01)

v2.7.0 实现完整的 WAL 崩溃恢复机制：

- 事务日志预写
- 检查点机制
- 崩溃后重放恢复
- PITR 支持

### FK 稳定性 (T-02)

- 外键约束完整性保证
- 级联删除/更新稳定性
- 死锁检测与避免

### 统一检索 API (T-05)

支持多种检索模式的统一入口：

- lex: 全文检索
- vec: 向量语义检索
- graph: 图关系检索
- hybrid: 混合检索

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [ARCHITECTURE_V2.7.md](../architecture/ARCHITECTURE_V2.7.md) | 整体架构 |
| [PERFORMANCE_ANALYSIS.md](../reports/PERFORMANCE_ANALYSIS.md) | 性能分析 |
| [SQL92_COMPLIANCE.md](../reports/SQL92_COMPLIANCE.md) | SQL 合规报告 |

---

*模块设计文档索引 v2.7.0*
*最后更新: 2026-04-22*
