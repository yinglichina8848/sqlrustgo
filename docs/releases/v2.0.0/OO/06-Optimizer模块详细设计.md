# SQLRustGo v2.0.0 Optimizer 模块详细设计

> **版本**: v2.0.0
> **日期**: 2026-03-29
> **模块**: sqlrustgo-optimizer

---

## 1. 模块概述

Optimizer 模块负责查询优化，包括基于规则的优化 (RBO) 和基于成本的优化 (CBO)。

## 2. 核心组件

### 2.1 CboOptimizer

```rust
pub struct CboOptimizer {
    rules: Vec<OptimizerRule>,
    cost_estimator: Arc<dyn CostEstimator>,
    statistics: Arc<StatisticsProvider>,
}

impl CboOptimizer {
    pub fn new(
        rules: Vec<OptimizerRule>,
        cost_estimator: Arc<dyn CostEstimator>,
        statistics: Arc<StatisticsProvider>,
    ) -> Self;
    pub fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan>;
}
```

### 2.2 CostEstimator

```rust
pub trait CostEstimator: Send + Sync {
    fn estimate_scan(&self, table: &TableRef, filter: &Expr) -> Cost;
    fn estimate_join(&self, left: &LogicalPlan, right: &LogicalPlan, join_type: JoinType) -> Cost;
    fn estimate_aggregate(&self, input: &LogicalPlan, group_by: &[Expr]) -> Cost;
    fn estimate_sort(&self, input: &LogicalPlan) -> Cost;
}

#[derive(Clone)]
pub struct Cost {
    pub cpu_cost: f64,
    pub io_cost: f64,
    pub memory_cost: f64,
}

impl Cost {
    pub fn zero() -> Self;
    pub fn new(cpu: f64, io: f64, memory: f64) -> Self;
    pub fn add(&self, other: &Cost) -> Cost;
    pub fn multiply(&self, factor: f64) -> Cost;
}
```

### 2.3 StatisticsProvider

```rust
pub trait StatisticsProvider: Send + Sync {
    fn table_statistics(&self, table: &str) -> Result<TableStatistics>;
    fn column_statistics(&self, table: &str, column: &str) -> Result<ColumnStatistics>;
}

pub struct TableStatistics {
    pub row_count: u64,
    pub data_size_bytes: u64,
}

pub struct ColumnStatistics {
    pub null_count: u64,
    pub distinct_count: u64,
    pub min_value: Option<ScalarValue>,
    pub max_value: Option<ScalarValue>,
    pub ndv: f64,
    pub histogram: Option<Histogram>,
}
```

---

## 3. 优化规则

### 3.1 OptimizerRule 枚举

```rust
pub enum OptimizerRule {
    PredicatePushdown,
    ProjectionPruning,
    ConstantFolding,
    CommonSubexpressionElimination,
    Decorrelate,
    RewriteDistinct,
    SimplifyExpressions,
    WindowFunctionRewrite,
    JoinReordering,
    ReduceCrossJoin,
    EliminateLimit,
    CombineFilter,
    SimplifyCase,
}
```

### 3.2 PredicatePushdown

```rust
pub struct PredicatePushdown;

impl OptimizerRule for PredicatePushdown {
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        // 将过滤条件下推到扫描层
    }
}
```

### 3.3 ProjectionPruning

```rust
pub struct ProjectionPruning;

impl OptimizerRule for ProjectionPruning {
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        // 消除冗余投影列
    }
}
```

### 3.4 JoinReordering

```rust
pub struct JoinReordering {
    cost_estimator: Arc<dyn CostEstimator>,
}

impl OptimizerRule for JoinReordering {
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        // 根据成本选择最优 join 顺序
    }
}
```

---

## 4. Cascades 优化器

### 4.1 Cascades 结构

```rust
pub struct CascadesOptimizer {
    cost_estimator: Arc<dyn CostEstimator>,
    rules: Vec<OptimizerRule>,
    memo: Memo,
}

pub struct Memo {
    groups: HashMap<GroupId, Group>,
}

pub struct Group {
    pub id: GroupId,
    pub logical_expressions: Vec<LogicalExpression>,
    pub physical_expressions: Vec<PhysicalExpression>,
    pub cost: Option<Cost>,
    pub best_plan: Option<PhysicalPlan>,
}
```

### 4.2 Memo 结构

```rust
pub struct Memo {
    pub groups: Vec<Group>,
}

pub struct GroupId(pub usize);

pub struct LogicalExpression {
    pub id: ExprId,
    pub expr: LogicalExpr,
    pub cost: Option<Cost>,
}
```

---

## 5. 物理计划

### 5.1 PhysicalPlan 枚举

```rust
pub enum PhysicalPlan {
    TableScan(TableScanExec),
    Projection(ProjectionExec),
    Filter(FilterExec),
    HashJoin(HashJoinExec),
    SortMergeJoin(SortMergeJoinExec),
    NestedLoopJoin(NestedLoopJoinExec),
    Aggregate(AggregateExec),
    Window(WindowExec),
    Sort(SortExec),
    Limit(LimitExec),
    Union(UnionExec),
    ColumnarScan(ColumnarScanExec),
    ParallelScan(ParallelScanExec),
}
```

### 5.2 物理算子实现

```rust
pub struct TableScanExec {
    table_name: String,
    projection: Vec<usize>,
    filters: Vec<Expr>,
}

pub struct HashJoinExec {
    left: Arc<dyn Executor>,
    right: Arc<dyn Executor>,
    join_type: JoinType,
    left_key: Expr,
    right_key: Expr,
    build_side: JoinSide,
}

pub struct AggregateExec {
    input: Arc<dyn Executor>,
    group_by: Vec<Expr>,
    aggr_exprs: Vec<AggregateFunction>,
    mode: AggregateMode,
}

pub enum AggregateMode {
    Partial,
    Final,
    FinalPartitioned,
}
```

---

## 6. 计划缓存

### 6.1 PlanCache

```rust
pub struct PlanCache {
    cache: RwLock<LruCache<String, Arc<LogicalPlan>>>,
    capacity: usize,
}

impl PlanCache {
    pub fn new(capacity: usize) -> Self;
    pub fn get(&self, key: &str) -> Option<Arc<LogicalPlan>>;
    pub fn insert(&self, key: String, plan: Arc<LogicalPlan>);
    pub fn clear(&self);
}
```

---

*文档生成日期: 2026-03-29*
*版本: v2.0.0*
