# SQLRustGo v2.0.0 Executor 模块详细设计

> **版本**: v2.0.0
> **日期**: 2026-03-29
> **模块**: sqlrustgo-executor

---

## 1. 模块概述

Executor 模块负责查询执行，采用 Volcano Iterator Model 结合 Vectorized Execution。

## 2. 核心 Trait

### 2.1 Executor Trait

```rust
pub trait Executor: Send + Sync {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

### 2.2 ExecutionContext

```rust
pub trait ExecutionContext: Send + Sync {
    fn node_id(&self) -> NodeId;
    fn mode(&self) -> ExecutionMode;
    fn task_scheduler(&self) -> Arc<dyn TaskScheduler>;
    fn batch_size(&self) -> usize;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Local,
    Distributed,
}
```

### 2.3 RecordBatch

```rust
pub struct RecordBatch {
    pub schema: SchemaRef,
    pub num_rows: usize,
    pub columns: Vec<ArrayRef>,
}

pub type ArrayRef = Arc<dyn Array>;

pub trait Array: Send + Sync + std::fmt::Debug {
    fn data_type(&self) -> DataType;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn to廊(&self, chunk: usize) -> ArrayRef;
}
```

---

## 3. 向量化执行器

### 3.1 VectorizedExecutor

```rust
pub struct VectorizedExecutor {
    schema: SchemaRef,
    children: Vec<Arc<dyn Executor>>,
}
```

### 3.2 ScanExecutor

```rust
pub struct ScanExecutor {
    table_ref: TableRef,
    projection: Vec<usize>,
    filter: Option<Expr>,
    batch_size: usize,
}

impl Executor for ScanExecutor {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

### 3.3 FilterExecutor

```rust
pub struct FilterExecutor {
    input: Arc<dyn Executor>,
    predicate: Expr,
}

impl Executor for FilterExecutor {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

### 3.4 ProjectionExecutor

```rust
pub struct ProjectionExecutor {
    input: Arc<dyn Executor>,
    expressions: Vec<Expr>,
    schema: SchemaRef,
}

impl Executor for ProjectionExecutor {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

### 3.5 HashJoinExecutor

```rust
pub struct HashJoinExec {
    left: Arc<dyn Executor>,
    right: Arc<dyn Executor>,
    join_type: JoinType,
    left_key: Expr,
    right_key: Expr,
    build_side: JoinSide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
    Semi,
    Anti,
}
```

### 3.6 AggregateExecutor

```rust
pub struct AggregateExecutor {
    input: Arc<dyn Executor>,
    group_by: Vec<Expr>,
    aggr_exprs: Vec<AggregateFunction>,
    mode: AggregateMode,
}

impl Executor for AggregateExecutor {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

### 3.7 WindowExecutor

```rust
pub struct WindowExecutor {
    input: Arc<dyn Executor>,
    window_functions: Vec<WindowFunction>,
    partition_by: Vec<Expr>,
    order_by: Vec<OrderByExpr>,
    window_frame: Option<WindowFrame>,
}

pub enum WindowFunction {
    RowNumber,
    Rank,
    DenseRank,
    PercentRank,
    CumeDist,
    Ntile(u32),
    FirstValue,
    LastValue,
    Lead,
    Lag,
    Count,
    Sum,
    Avg,
    Min,
    Max,
}
```

---

## 4. 并行执行器

### 4.1 ParallelExecutor

```rust
pub trait TaskScheduler: Send + Sync {
    fn schedule(&self, task: Box<dyn FnOnce() + Send>) -> JoinHandle<()>;
    fn shutdown(&self);
}

pub struct ParallelExecutor {
    scheduler: Arc<dyn TaskScheduler>,
    partition_count: usize,
}

impl ParallelExecutor {
    pub fn new(scheduler: Arc<dyn TaskScheduler>, partition_count: usize) -> Self;
    pub fn execute(&self, plan: Arc<dyn Executor>, ctx: &dyn ExecutionContext) -> Result<Vec<RecordBatch>>;
}
```

### 4.2 ParallelScanExecutor

```rust
pub struct ParallelScanExecutor {
    table_ref: TableRef,
    partition_count: usize,
    projection: Vec<usize>,
    filter: Option<Expr>,
}

impl Executor for ParallelScanExecutor {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

### 4.3 TaskScheduler 实现

```rust
pub struct TokioTaskScheduler {
    runtime: Arc<TokioRuntime>,
}

impl TaskScheduler for TokioTaskScheduler {
    fn schedule(&self, task: Box<dyn FnOnce() + Send>) -> JoinHandle<()>;
    fn shutdown(&self);
}
```

---

## 5. 列式扫描执行器

### 5.1 ColumnarScanExec

```rust
pub struct ColumnarScanExec {
    table_ref: TableRef,
    projection: Vec<ColumnId>,
    filters: Vec<Expr>,
    storage: Arc<dyn ColumnarStorage>,
}

impl Executor for ColumnarScanExec {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}

pub trait ColumnarScan: Send + Sync {
    fn scan(&self, projection: &[ColumnId]) -> Result<Box<dyn RecordBatchReader>>;
    fn scan_with_filter(&self, projection: &[ColumnId], filter: &Expr) -> Result<Box<dyn RecordBatchReader>>;
}
```

---

## 6. SortMergeJoinExecutor

```rust
pub struct SortMergeJoinExec {
    left: Arc<dyn Executor>,
    right: Arc<dyn Executor>,
    left_key: Expr,
    right_key: Expr,
    join_type: JoinType,
}

impl Executor for SortMergeJoinExec {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

---

## 7. SortExecutor

```rust
pub struct SortExec {
    input: Arc<dyn Executor>,
    expr: Vec<OrderByExpr>,
    fetch: Option<usize>,
}

impl Executor for SortExec {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}
```

---

*文档生成日期: 2026-03-29*
*版本: v2.0.0*
