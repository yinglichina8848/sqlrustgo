---
entity_type: tool
confidence: 90
domains: [sqlrustgo, executor, sql]
last_updated: 2026-04-17
---

# Executor 模块

> 执行引擎 - 将物理执行计划转化为查询结果

## 概述

Executor 模块负责执行 Planner 生成的物理计划，返回查询结果。

## 源码位置

```
crates/executor/
├── src/
│   ├── lib.rs
│   ├── executor.rs       # 主执行器
│   ├── local_executor.rs # 本地执行
│   ├── filter.rs        # 过滤算子
│   ├── join.rs          # JOIN 执行
│   ├── aggregate.rs      # 聚合算子
│   ├── stored_proc.rs   # 存储过程
│   ├── trigger.rs       # 触发器 (未集成)
│   └── ...
└── Cargo.toml
```

## 执行模型

```
SQL → Parser → Planner → Optimizer → Executor → Results
```

## 核心结构

```rust
// 物理计划 trait
pub trait PhysicalPlan {
    fn execute(&self) -> Result<RecordBatch, ExecutorError>;
}

// 执行器
pub struct LocalExecutor {
    catalog: CatalogRef,
    storage: StorageRef,
}
```

## 算子支持

| 算子 | 状态 | 说明 |
|------|------|------|
| SeqScan | ✅ | 全表扫描 |
| IndexScan | ⚠️ | 索引扫描 (未启用) |
| Filter | ✅ | 条件过滤 |
| Projection | ✅ | 列投影 |
| HashJoin | ✅ | Hash 连接 |
| SortMergeJoin | ✅ | 排序合并连接 |
| Aggregate | ✅ | 聚合操作 |
| Sort | ✅ | 排序 |
| Limit | ✅ | 限制行数 |
| Window | ✅ | 窗口函数 |

## JOIN 类型支持

| 类型 | 状态 |
|------|------|
| INNER JOIN | ✅ |
| LEFT JOIN | ✅ |
| RIGHT JOIN | ✅ |
| FULL OUTER JOIN | ⚠️ 部分 |
| LEFT SEMI JOIN | ✅ |
| LEFT ANTI JOIN | ✅ |

## 约束验证状态

| 约束 | Parser | Executor |
|------|--------|----------|
| PRIMARY KEY | ✅ | ✅ 已验证 |
| UNIQUE | ✅ | ✅ 已验证 |
| FOREIGN KEY | ✅ | ❌ **未验证** (Issue #1497) |
| CHECK | ⚠️ | ❌ 未实现 |
| NOT NULL | ✅ | ✅ 已验证 |

## 触发器状态

- **解析器**: ✅ 支持
- **数据结构**: ✅ 定义完整
- **执行器**: ⚠️ 框架存在但未集成

## 存储过程状态

- **解析器**: ✅ 支持
- **执行器**: ⚠️ 框架存在但未集成

## 测试

```bash
cargo test --package sqlrustgo-executor
cargo test --test executor_test
```

## 已知未集成功能 (Issue #1497)

1. 外键约束验证 - `table_foreign_keys: None`
2. 触发器执行 - 未连接到 DML
3. 存储过程调用 - 未连接到执行链
4. 并行执行 - `ParallelExecutor` 未启用
5. SIMD 向量化 - 未启用

## 相关文件

- [Parser 模块](./Parser.md) - 输入来源
- [Storage 模块](./Storage.md) - 数据访问
- [Transaction 模块](./Transaction.md) - 事务管理

---

*最后更新: 2026-04-17*
