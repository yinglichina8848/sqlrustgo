# sqlrustgo 2.0 路线图

> 版本：v1.0
> 日期：2026-02-18
> 目标：从"项目代码"升级为"数据库内核架构"

---

## 一、对标 Apache DataFusion

### 1.1 DataFusion 核心设计

| 模块 | 说明 |
|:-----|:-----|
| Arrow 列式内存 | 高效内存布局 |
| LogicalPlan / PhysicalPlan 分离 | 清晰的执行层次 |
| 向量化执行 | 批量处理 |
| CBO（部分实现） | 成本优化 |
| 插件式数据源 | 可扩展存储 |

### 1.2 对比分析

| 模块 | sqlrustgo | DataFusion |
|:-----|:----------|:-----------|
| Parser | ✅ | ✅ |
| LogicalPlan | 部分 | ✅ |
| CBO | ❌ | 部分 |
| 向量化 | ❌ | ✅ |
| 插件 | 基础 | ✅ |
| 分布式 | ❌ | 通过 Ballista |

---

## 二、升级路径

### 第一阶段：核心架构重构

**时间**：1-2 个月

| 任务 | 说明 |
|:-----|:-----|
| LogicalPlan 独立模块 | 清晰的逻辑计划 |
| PhysicalPlan trait 化 | 可扩展的物理计划 |
| Executor 插件化 | 可替换执行器 |
| HashJoin 实现 | 解决 Join 性能 |

**结果**：L3 → L4

### 第二阶段：性能升级

**时间**：2-3 个月

| 任务 | 说明 |
|:-----|:-----|
| 向量化执行 | 批量处理 |
| 批处理表达式 | 列式计算 |
| 基础统计信息 | 表/列统计 |
| 简化 CBO | 成本优化 |

**结果**：可处理 100万行级数据

### 第三阶段：内核级能力

**时间**：3-6 个月

| 任务 | 说明 |
|:-----|:-----|
| 完整 CBO | 成本优化器 |
| Join reorder | Join 重排序 |
| 插件动态加载 | 运行时扩展 |
| Memory pool | 内存管理 |
| Spill to disk | 磁盘溢出 |

**结果**：可商用级内核

### 第四阶段：企业级能力

**时间**：6-12 个月

| 任务 | 说明 |
|:-----|:-----|
| 分布式执行 | 多节点 |
| 任务调度 | 任务分发 |
| 事务支持 | ACID |
| 高可用 | 故障恢复 |

---

## 三、Phase 1 详细计划

### 3.1 LogicalPlan 重构

```rust
pub enum LogicalPlan {
    Projection { input: Box<LogicalPlan>, expr: Vec<Expr> },
    Filter { input: Box<LogicalPlan>, predicate: Expr },
    Join { left: Box<LogicalPlan>, right: Box<LogicalPlan>, on: Vec<(Expr, Expr)> },
    Aggregate { input: Box<LogicalPlan>, group_expr: Vec<Expr>, aggr_expr: Vec<Expr> },
    TableScan { table_name: String, projection: Option<Vec<usize>> },
}
```

### 3.2 PhysicalPlan trait 化

```rust
pub trait PhysicalPlan: Send + Sync {
    fn schema(&self) -> &Schema;
    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>>;
    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>>;
}
```

### 3.3 HashJoin 实现

```rust
pub struct HashJoinExec {
    left: Arc<dyn PhysicalPlan>,
    right: Arc<dyn PhysicalPlan>,
    on: Vec<(Column, Column)>,
    join_type: JoinType,
}
```

---

## 四、Phase 2 详细计划

### 4.1 向量化执行

```rust
pub struct RecordBatch {
    columns: Vec<ArrayRef>,
    row_count: usize,
}

pub trait PhysicalExpr {
    fn evaluate(&self, batch: &RecordBatch) -> Result<ArrayRef>;
}
```

### 4.2 统计信息

```rust
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
}

pub struct ColumnStats {
    pub distinct_count: usize,
    pub null_count: usize,
    pub min_value: Option<ScalarValue>,
    pub max_value: Option<ScalarValue>,
}
```

---

## 五、Phase 3 详细计划

### 5.1 完整 CBO

```rust
pub struct CostBasedOptimizer {
    statistics: Arc<dyn StatisticsCollector>,
    cost_estimator: Arc<dyn CostEstimator>,
    plan_enumerator: Arc<dyn PlanEnumerator>,
}
```

### 5.2 Memory Pool

```rust
pub trait MemoryPool: Send + Sync {
    fn allocate(&self, size: usize) -> Result<MemoryAllocation>;
    fn free(&self, allocation: MemoryAllocation);
    fn available(&self) -> usize;
}
```

### 5.3 Spill to Disk

```rust
pub trait SpillManager: Send + Sync {
    fn spill(&self, batch: &RecordBatch) -> Result<SpillHandle>;
    fn read(&self, handle: &SpillHandle) -> Result<RecordBatch>;
}
```

---

## 六、Phase 4 详细计划

### 6.1 分布式执行

```rust
pub struct DistributedExecutor {
    scheduler: Arc<dyn Scheduler>,
    workers: Vec<WorkerInfo>,
}

pub trait Scheduler: Send + Sync {
    fn schedule(&self, task: Task) -> Result<TaskHandle>;
    fn status(&self, handle: &TaskHandle) -> Result<TaskStatus>;
}
```

### 6.2 事务支持

```rust
pub trait TransactionManager: Send + Sync {
    fn begin(&self) -> Result<Transaction>;
    fn commit(&self, txn: Transaction) -> Result<()>;
    fn rollback(&self, txn: Transaction) -> Result<()>;
}
```

---

## 七、里程碑

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          里程碑规划                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   v1.1.0 (Phase 1)                                                          │
│   ├── LogicalPlan 重构                                                      │
│   ├── PhysicalPlan trait                                                    │
│   ├── HashJoin                                                              │
│   └── 目标：L4 架构                                                         │
│                                                                              │
│   v1.2.0 (Phase 2)                                                          │
│   ├── 向量化执行                                                            │
│   ├── 统计信息                                                              │
│   ├── 简化 CBO                                                              │
│   └── 目标：100万行级                                                       │
│                                                                              │
│   v2.0.0 (Phase 3)                                                          │
│   ├── 完整 CBO                                                              │
│   ├── Memory Pool                                                           │
│   ├── Spill to Disk                                                         │
│   └── 目标：商用级内核                                                      │
│                                                                              │
│   v3.0.0 (Phase 4)                                                          │
│   ├── 分布式执行                                                            │
│   ├── 事务支持                                                              │
│   ├── 高可用                                                                │
│   └── 目标：企业级                                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 八、关键决策

### 8.1 如果不做这些会怎样？

**3 年后**：
- Executor 2000 行
- Join 逻辑混乱
- 性能不可控
- 插件难以插入

### 8.2 如果做完这些

**你将拥有**：
- 可演进内核
- 可融资故事
- 可对标 DataFusion
- 可嵌入产品

---

## 九、总结

```
现在 sqlrustgo 的关键不是"多写功能"。

而是：

把它从"项目代码"升级为"数据库内核架构"。
```

---

*本文档由 TRAE (GLM-5.0) 创建*
