# SQLRustGo v2.6.0 架构设计

> **版本**: alpha/v2.6.0
> **更新日期**: 2026-04-18

---

## 1. 概述

v2.6.0 是 SQLRustGo 迈向生产就绪的关键版本，专注于 SQL-92 完整支持。本文档描述整体架构设计和核心模块。

---

## 2. 系统架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                     SQLRustGo v2.6.0                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Parser    │───▶│   Planner   │───▶│  Executor   │     │
│  │  (SQL-92)   │    │             │    │             │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                                     │              │
│         │              ┌─────────────┐      │              │
│         └─────────────▶│  Optimizer   │◀─────┘              │
│                        │    (CBO)    │                     │
│                        └─────────────┘                     │
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Storage   │◀───│   Catalog   │◀───│ Transaction │     │
│  │   Engine   │    │             │    │   Manager   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                                                              │
│  ┌─────────────┐    ┌─────────────┐                        │
│  │ MySQL Server│    │    REPL     │                        │
│  │  Protocol   │    │             │                        │
│  └─────────────┘    └─────────────┘                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心模块

| 模块 | 职责 | 关键组件 |
|------|------|----------|
| Parser | SQL 解析 | Tokenizer, AST |
| Planner | 查询规划 | LogicalPlan, PhysicalPlan |
| Optimizer | 优化执行 | CBO, Rules |
| Executor | 执行算子 | Scan, Join, Aggregate |
| Storage | 存储引擎 | BufferPool, BTree, WAL |
| Transaction | 事务管理 | MVCC, Lock |

---

## 3. SQL-92 实现

### 3.1 语法支持

v2.6.0 实现了完整的 SQL-92 核心语法：

| 语法 | 实现位置 | 状态 |
|------|----------|------|
| SELECT | `parser/src/parser.rs` | ✅ |
| INSERT | `parser/src/parser.rs` | ✅ |
| UPDATE | `parser/src/parser.rs` | ✅ |
| DELETE | `parser/src/parser.rs` | ✅ (#1557) |
| CREATE TABLE | `parser/src/parser.rs` | ✅ |
| CREATE INDEX | `parser/src/parser.rs` | ✅ |
| ALTER TABLE | `parser/src/parser.rs` | ✅ |

### 3.2 聚合函数

| 函数 | 实现位置 | 状态 |
|------|----------|------|
| COUNT | `executor/src/aggregate.rs` | ✅ |
| SUM | `executor/src/aggregate.rs` | ✅ |
| AVG | `executor/src/aggregate.rs` | ✅ |
| MIN | `executor/src/aggregate.rs` | ✅ |
| MAX | `executor/src/aggregate.rs` | ✅ |

### 3.3 JOIN 实现

| 类型 | 实现位置 | 状态 |
|------|----------|------|
| INNER JOIN | `executor/src/join.rs` | ✅ |
| LEFT JOIN | `executor/src/join.rs` | ✅ |
| RIGHT JOIN | `executor/src/join.rs` | ✅ |
| CROSS JOIN | `executor/src/join.rs` | ✅ |
| FULL OUTER JOIN | - | ⏳ |

### 3.4 分组查询

| 特性 | 实现位置 | 状态 |
|------|----------|------|
| GROUP BY | `planner/src/group_by.rs` | ✅ |
| HAVING | `planner/src/having.rs` | ✅ (#1567) |

---

## 4. 执行引擎

### 4.1 算子层次

```
ExecutionPlan
├── Scan (SeqScan, IndexScan)
├── Filter
├── Projection
├── Join (HashJoin, NestedLoop)
├── Aggregate (HashAggregate, SortAggregate)
├── Sort
├── Limit
└── Insert/Update/Delete
```

### 4.2 执行模式

| 模式 | 说明 | 状态 |
|------|------|------|
| 向量化执行 | SIMD 加速 | ✅ |
| 并行执行 | 多线程 | ✅ |
| 存储过程 | SP 支持 | ⚠️ 部分 |
| 触发器 | Trigger 支持 | ⚠️ 部分 |

---

## 5. 存储引擎

### 5.1 存储层次

```
Storage Engine
├── MemoryStorage (内存存储)
├── FileStorage (文件存储)
├── ColumnarStorage (列式存储)
└── BufferPool (缓存层)
```

### 5.2 索引类型

| 类型 | 实现 | 状态 |
|------|------|------|
| B+Tree | `storage/src/btree.rs` | ✅ |
| Hash | `storage/src/hash_index.rs` | ✅ |
| Vector | `storage/src/vector_index.rs` | ✅ |

### 5.3 事务支持

| 特性 | 实现 | 状态 |
|------|------|------|
| MVCC | `transaction/src/mvcc.rs` | ✅ (SI) |
| WAL | `storage/src/wal.rs` | ✅ |
| SSI | - | ⏳ |

---

## 6. API 设计

### 6.1 ExecutionEngine API

v2.6.0 引入的高级 API：

```rust
pub struct ExecutionEngine {
    storage: Arc<RwLock<Box<dyn StorageEngine>>>,
    catalog: Arc<Catalog>,
}

impl ExecutionEngine {
    pub fn new(storage: Box<dyn StorageEngine>) -> Self;
    pub fn execute(&self, sql: &str, params: Vec<Value>) -> Result<Vec<Record>, Error>;
    pub fn execute_batch(&self, sql: &str) -> Result<(), Error>;
}
```

---

## 7. 性能优化

### 7.1 CBO 优化器

基于成本的优化器，自动选择最优执行计划：

- 统计信息收集
- 代价模型
- 规则优化

### 7.2 向量化执行

使用 SIMD 指令加速计算：

- 批量数据处理
- 列式计算
- 并行执行

---

## 8. 未来规划

### 8.1 待完成功能

1. **MVCC SSI**: 可串行化快照隔离
2. **FULL OUTER JOIN**: 完整外部连接
3. **覆盖率**: 70%+

### 8.2 路线图

| 阶段 | 日期 | 目标 |
|------|------|------|
| Alpha | 2026-04-21 | P0 功能完成 |
| Beta | 2026-04-28 | P1 功能完成 |
| RC | 2026-05-05 | 候选发布 |
| GA | 2026-05-12 | 生产就绪 |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
