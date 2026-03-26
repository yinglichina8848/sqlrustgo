# SQLRustGo Executor 模块详细设计文档

> **版本**: v1.9.0
> **日期**: 2026-03-26
> **模块**: sqlrustgo-executor

---

## 1. 模块概述

Executor 模块负责物理计划的执行，采用 Volcano 模型和向量化执行。

### 1.1 模块职责

- 物理计划执行
- 算子实现 (Operators)
- 向量化执行 (Vectorized Execution)
- 查询性能分析 (Profiling)
- 查询缓存 (Query Cache)

### 1.2 模块结构

```
crates/executor/
├── src/
│   ├── lib.rs                   # 模块入口
│   ├── executor.rs               # 执行器主逻辑
│   ├── local_executor.rs        # 本地执行器
│   ├── filter.rs                # 过滤算子
│   ├── operator_profile.rs      # 性能分析
│   ├── query_cache.rs           # 查询缓存
│   ├── query_cache_config.rs    # 缓存配置
│   ├── executor_metrics.rs      # 执行指标
│   ├── harness.rs               # 测试工具
│   ├── mock_storage.rs          # 模拟存储
│   ├── pipeline_trace.rs        # 管道追踪
│   ├── session_config.rs        # 会话配置
│   ├── sql_log.rs               # SQL 日志
│   ├── sql_normalizer.rs        # SQL 规范化
│   └── vectorization.rs         # 向量化支持
└── Cargo.toml
```

---

## 2. 核心类设计

### 2.1 执行器架构

```uml
@startuml

class Executor {
  -storage: StorageRef
  -catalog: CatalogRef
  -session: SessionRef
  --
  +execute(plan): ExecutorResult
}

class LocalExecutor {
  -storage: StorageRef
  -batch_size: usize
  --
  +execute_plan(plan): RecordBatch
}

class ExecutionContext {
  -session: SessionRef
  -task_id: TaskId
  -runtime: Runtime
  -variables: HashMap<String, Value>
}

class Task {
  -plan: PhysicalPlan
  -context: ExecutionContext
  --
  +run(): JoinHandle<ExecutorResult>
}

class RecordBatch {
  -num_rows: usize
  -columns: Vec<Array>
  -schema: Schema
}

Executor --> LocalExecutor
LocalExecutor --> ExecutionContext
ExecutionContext --> Task
Task --> RecordBatch

@enduml
```

### 2.2 算子类图

```uml
@startuml

abstract class Operator {
  #children: Vec<Box<dyn Operator>>
  --
  +open(&mut Context)
  +next() -> Option<RecordBatch>
  +close()
}

class SeqScanOperator {
  -table: String
  -filter: Option<Expression>
  -projection: Vec<usize>
}

class IndexScanOperator {
  -table: String
  -index: String
  -key_range: KeyRange
}

class FilterOperator {
  -predicate: Expression
}

class ProjectionOperator {
  -expressions: Vec<Expression>
}

class HashJoinOperator {
  -join_type: JoinType
  -left_keys: Vec<Expression>
  -right_keys: Vec<Expression>
  -build_table: HashMap
}

class MergeJoinOperator {
  -join_type: JoinType
  -left_keys: Vec<Expression>
  -right_keys: Vec<Expression>
}

class AggregateOperator {
  -group_by: Vec<Expression>
  -aggregates: Vec<AggregateExpr>
  -group_states: HashMap
}

class SortOperator {
  -order_by: Vec<OrderExpr>
  -buffer: Vec<RecordBatch>
}

class LimitOperator {
  -limit: u64
  -offset: u64
  -count: u64
}

Operator <|-- SeqScanOperator
Operator <|-- IndexScanOperator
Operator <|-- FilterOperator
Operator <|-- ProjectionOperator
Operator <|-- HashJoinOperator
Operator <|-- MergeJoinOperator
Operator <|-- AggregateOperator
Operator <|-- SortOperator
Operator <|-- LimitOperator

@enduml
```

---

## 3. 执行流程设计

### 3.1 Volcano 模型

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Volcano 模型执行流程                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐                                                      │
│  │   Driver    │                                                      │
│  └──────┬──────┘                                                      │
│         │ call open()                                                 │
│         ▼                                                             │
│  ┌─────────────┐     call next()    ┌─────────────┐                │
│  │  Project    │ ◀───────────────── │    Sort     │                │
│  └──────┬──────┘                    └──────┬──────┘                │
│         │ call next()                     │ call next()           │
│         ▼                                  ▼                        │
│  ┌─────────────┐     call next()    ┌─────────────┐                │
│  │   Filter    │ ◀───────────────── │   HashJoin  │                │
│  └──────┬──────┘                    └──────┬──────┘                │
│         │ call next()                     │ call next()           │
│         ▼                                  ▼                        │
│  ┌─────────────┐                    ┌─────────────┐                │
│  │   SeqScan   │ ──────────────────▶│   SeqScan   │                │
│  └─────────────┘   return batch     └─────────────┘                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 向量化执行

```rust
pub struct VectorizedExecutor {
    batch_size: usize,
}

impl VectorizedExecutor {
    pub fn filter_batch(
        &self, 
        batch: &RecordBatch, 
        predicate: &dyn VectorizedExpr
    ) -> RecordBatch {
        // SIMD 并行过滤
        let mask = unsafe {
            predicate.eval_vectorized(batch.columns())
        };
        
        // 根据 mask 选择行
        batch.filter(&mask)
    }
    
    pub fn project_batch(
        &self,
        batch: &RecordBatch,
        projections: &[Expr]
    ) -> RecordBatch {
        let mut columns = Vec::new();
        
        for expr in projections {
            columns.push(expr.eval_vectorized(batch));
        }
        
        RecordBatch::new(columns, batch.num_rows())
    }
}
```

---

## 4. 性能分析设计

### 4.1 算子性能分析

```uml
@startuml

class OperatorProfiler {
  -profiles: HashMap<String, OperatorProfile>
  -enabled: bool
  --
  +start_timer(op_id)
  +stop_timer(op_id, rows)
  +get_profile(op_id): OperatorProfile
  +report(): ProfilingReport
}

class OperatorProfile {
  -operator_name: String
  -total_time_ns: u64
  -execution_count: u64
  -rows_processed: u64
  -avg_time_ns: u64
  -min_time_ns: u64
  -max_time_ns: u64
}

OperatorProfiler --> OperatorProfile

@enduml
```

### 4.2 查询缓存

```uml
@startuml

class QueryCache {
  -cache: LruCache<String, CachedPlan>
  -config: CacheConfig
  --
  +get(sql): Option<PhysicalPlan>
  +put(sql, plan)
  +invalidate(table)
  +clear()
}

class CachedPlan {
  -plan: PhysicalPlan
  -created_at: Timestamp
  -last_accessed: Timestamp
  -access_count: u64
}

QueryCache --> CachedPlan

@enduml
```

---

## 5. 与代码对应检查

### 5.1 模块文件对应

| 设计内容 | 代码文件 | 状态 |
|----------|----------|------|
| 执行器 | `executor.rs` | ✅ 对应 |
| 本地执行器 | `local_executor.rs` | ✅ 对应 |
| 过滤算子 | `filter.rs` | ✅ 对应 |
| 性能分析 | `operator_profile.rs` | ✅ 对应 |
| 查询缓存 | `query_cache.rs` | ✅ 对应 |
| 指标 | `executor_metrics.rs` | ✅ 对应 |
| 向量化 | `vectorization.rs` | ✅ 对应 |

### 5.2 功能覆盖检查

| 功能 | 代码实现 | 状态 |
|------|----------|------|
| SeqScan | ✅ | ✅ |
| IndexScan | ✅ | ✅ |
| HashJoin | ✅ | ✅ |
| MergeJoin | ✅ | ✅ |
| Aggregate | ✅ | ✅ |
| Sort | ✅ | ✅ |
| Filter | ✅ | ✅ |
| Projection | ✅ | ✅ |
| Limit | ✅ | ✅ |
| 向量化执行 | ✅ | ✅ |
| 性能分析 | ✅ | ✅ |
| 查询缓存 | ✅ | ✅ |

---

## 6. 测试设计

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_seq_scan() {
        let mut executor = LocalExecutor::new(storage.clone());
        let plan = PhysicalPlan::SeqScan {
            table: "users".to_string(),
            filter: None,
        };
        
        let result = executor.execute_plan(&plan);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_filter() {
        let filter = Expression::Binary {
            left: Box::new(Column("age".to_string())),
            op: BinaryOperator::Gt,
            right: Box::new(Literal(Value::Integer(18))),
        };
        
        let op = FilterOperator::new(filter);
        // 测试过滤逻辑
    }
    
    #[test]
    fn test_hash_join() {
        let join = HashJoinOperator::new(
            JoinType::Inner,
            vec![col("id")],
            vec![col("user_id")],
        );
        
        // 测试 Join 逻辑
    }
}
```

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-03-26 | OpenCode | 初始版本 |

**文档状态**: ✅ 已完成
