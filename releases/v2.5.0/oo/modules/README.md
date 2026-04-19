# SQLRustGo v2.5.0 模块设计文档

**版本**: v2.5.0 (里程碑版本)
**发布日期**: 2026-04-16

---

## 概述

本目录包含 SQLRustGo v2.5.0 各核心模块的设计文档。每个模块都遵循统一的设计规范，包括 What/Why/How 三段式分析。

---

## 模块索引

### 核心引擎模块

| 模块 | 文档 | 说明 |
|------|------|------|
| MVCC | [MVCC_DESIGN.md](./mvcc/MVCC_DESIGN.md) | 多版本并发控制 |
| WAL | [WAL_DESIGN.md](./wal/WAL_DESIGN.md) | 预写日志 |
| Storage | [STORAGE_DESIGN.md](./storage/STORAGE_DESIGN.md) | 存储引擎 |
| Executor | [EXECUTOR_DESIGN.md](./executor/EXECUTOR_DESIGN.md) | 查询执行器 |

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
| Transaction | [TRANSACTION_DESIGN.md](./transaction/TRANSACTION_DESIGN.md) | 事务管理 |

### 工具与服务模块

| 模块 | 文档 | 说明 |
|------|------|------|
| Agent (OpenClaw) | [AGENT_DESIGN.md](./openclaw/AGENT_DESIGN.md) | Agent 框架 |
| Server | [SERVER_DESIGN.md](./server/SERVER_DESIGN.md) | 服务器 |
| Bench | [BENCH_DESIGN.md](./bench/BENCH_DESIGN.md) | 基准测试 |

---

## 模块依赖关系

```
                    ┌─────────────┐
                    │   Server    │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│    Bench      │  │  Unified Query │  │    Agent      │
└───────┬───────┘  └───────┬───────┘  └───────────────┘
        │                  │
        └────────┬─────────┘
                 ▼
    ┌───────────────────────────────┐
    │         Executor              │
    └───────────────┬───────────────┘
                    │
    ┌───────────────┼───────────────┐
    │               │               │
    ▼               ▼               ▼
┌────────┐    ┌──────────┐    ┌──────────┐
│ Parser │    │ Optimizer │    │ Planner  │
└───┬────┘    └────┬─────┘    └────┬─────┘
    │              │                │
    └──────────────┼────────────────┘
                   ▼
    ┌───────────────────────────────┐
    │         Storage               │
    └───────────────┬───────────────┘
                    │
    ┌───────────────┼───────────────┐
    │               │               │
    ▼               ▼               ▼
┌────────┐    ┌──────────┐    ┌──────────┐
│ MVCC   │    │    WAL   │    │ Catalog  │
└────────┘    └──────────┘    └──────────┘
```

---

## 设计规范

每个模块设计文档应包含:

### 1. What (是什么)
- 模块的定义和职责
- 在系统中的角色

### 2. Why (为什么)
- 存在的目的和价值
- 解决的问题域

### 3. How (如何实现)
- 核心数据结构
- 关键算法
- 交互流程

### 4. 接口设计
- 公开 API
- 参数说明
- 返回值

### 5. 错误处理
- 错误类型
- 处理策略

### 6. 性能考虑
- 时间复杂度
- 空间复杂度
- 优化策略

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [ARCHITECTURE_V2.5.md](../architecture/ARCHITECTURE_V2.5.md) | 整体架构 |
| [MVCC_DESIGN.md](../../MVCC_DESIGN.md) | MVCC 设计 (根目录) |
| [GRAPH_ENGINE_DESIGN.md](../../GRAPH_ENGINE_DESIGN.md) | 图引擎设计 (根目录) |
| [VECTOR_INDEX_DESIGN.md](../../VECTOR_INDEX_DESIGN.md) | 向量索引设计 (根目录) |

---

*模块设计文档索引 v2.5.0*
*最后更新: 2026-04-16*
