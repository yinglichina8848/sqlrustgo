# v1.2.0 API 文档

> **版本**: v1.2.0  
> **更新日期**: 待定  
> **状态**: 🔄 开发中

---

## 一、概述

本文档描述 v1.2.0 版本新增和变更的 API。

### 1.1 API 变更类型

| 类型 | 数量 | 说明 |
|------|------|------|
| 新增 API | 15+ | 向量化、统计信息、CBO |
| 变更 API | 0 | 无破坏性变更 |
| 废弃 API | 0 | 无废弃 API |

### 1.2 兼容性

- ✅ **向后兼容**: v1.1.0 代码无需修改
- ✅ **API 稳定**: 核心 API 保持稳定

---

## 二、向量化执行 API

### 2.1 记录批次

列式内存布局，支持批量数据处理。

```rust
pub struct RecordBatch {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,
    row_count: usize,
}

impl RecordBatch {
    pub fn new(schema: Arc<Schema>, columns: Vec<ArrayRef>) -> Result<Self>;
    pub fn schema(&self) -> &Schema;
    pub fn column(&self, i: usize) -> &ArrayRef;
    pub fn num_columns(&self) -> usize;
    pub fn num_rows(&self) -> usize;
    pub fn slice(&self, offset: usize, length: usize) -> RecordBatch;
}
```

### 2.2 数组特征

列式数组抽象接口。

```rust
pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn is_null(&self, index: usize) -> bool;
    fn null_count(&self) -> usize;
    fn slice(&self, offset: usize, length: usize) -> ArrayRef;
}
```

### 2.3 具体数组类型

#### Int64Array

```rust
pub struct Int64Array {
    data: Vec<i64>,
    null_bitmap: Option<Bitmap>,
}

impl Int64Array {
    pub fn new(data: Vec<i64>) -> Self;
    pub fn value(&self, i: usize) -> i64;
    pub fn values(&self) -> &[i64];
    pub fn iter(&self) -> impl Iterator<Item = Option<i64>>;
}
```

#### Float64Array

```rust
pub struct Float64Array {
    data: Vec<f64>,
    null_bitmap: Option<Bitmap>,
}

impl Float64Array {
    pub fn new(data: Vec<f64>) -> Self;
    pub fn value(&self, i: usize) -> f64;
    pub fn values(&self) -> &[f64];
}
```

#### 字符串数组

```rust
pub struct StringArray {
    data: Vec<String>,
    null_bitmap: Option<Bitmap>,
}

impl StringArray {
    pub fn new(data: Vec<String>) -> Self;
    pub fn value(&self, i: usize) -> &str;
    pub fn iter(&self) -> impl Iterator<Item = Option<&str>>;
}
```

### 2.4 向量化表达式

```rust
pub trait VectorizedExpression: Send + Sync {
    fn evaluate(&self, batch: &RecordBatch) -> Result<ArrayRef>;
    fn data_type(&self) -> &DataType;
}

pub struct VectorizedFilter {
    predicate: Arc<dyn VectorizedExpression>,
}

impl VectorizedFilter {
    pub fn execute(&self, batch: &RecordBatch) -> Result<RecordBatch>;
}

pub struct VectorizedProjection {
    exprs: Vec<Arc<dyn VectorizedExpression>>,
}

impl VectorizedProjection {
    pub fn execute(&self, batch: &RecordBatch) -> Result<RecordBatch>;
}
```

---

## 三、统计信息 API

### 3.1 表格统计

表级统计信息。

```rust
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
    pub column_stats: HashMap<String, ColumnStats>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TableStats {
    pub fn new() -> Self;
    pub fn estimate_cardinality(&self, column: &str) -> usize;
}
```

### 3.2 列统计

列级统计信息。

```rust
pub struct ColumnStats {
    pub distinct_count: usize,
    pub null_count: usize,
    pub min_value: Option<ScalarValue>,
    pub max_value: Option<ScalarValue>,
    pub avg_width: Option<f64>,
    pub most_common_values: Vec<(ScalarValue, usize)>,
    pub histogram: Option<Histogram>,
}

impl ColumnStats {
    pub fn new() -> Self;
    pub fn selectivity(&self, predicate: &Predicate) -> f64;
}
```

### 3.3 统计收集器

统计信息收集器。

```rust
pub struct StatisticsCollector {
    storage: Arc<dyn Storage>,
}

impl StatisticsCollector {
    pub fn new(storage: Arc<dyn Storage>) -> Self;
    pub fn collect_table_stats(&self, table_name: &str) -> Result<TableStats>;
    pub fn collect_column_stats(&self, table: &str, column: &str) -> Result<ColumnStats>;
    pub fn analyze_table(&self, table_name: &str) -> Result<()>;
}
```

---

## 四、CBO API

### 4.1 成本模型

成本模型接口。

```rust
pub struct CostModel {
    pub seq_scan_cost: f64,
    pub idx_scan_cost: f64,
    pub filter_cost: f64,
    pub join_cost: f64,
    pub aggregate_cost: f64,
}

impl CostModel {
    pub fn new() -> Self;
    pub fn estimate(&self, plan: &LogicalPlan, stats: &TableStats) -> Cost;
    pub fn estimate_scan(&self, rows: usize) -> Cost;
    pub fn estimate_filter(&self, rows: usize, selectivity: f64) -> Cost;
    pub fn estimate_join(&self, left_rows: usize, right_rows: usize) -> Cost;
}
```

### 4.2 Cost

成本表示。

```rust
pub struct Cost {
    pub cpu: f64,
    pub io: f64,
    pub memory: f64,
}

impl Cost {
    pub fn new(cpu: f64, io: f64, memory: f64) -> Self;
    pub fn total(&self) -> f64;
    pub fn add(&self, other: &Cost) -> Cost;
}
```

### 4.3 优化器

查询优化器。

```rust
pub struct Optimizer {
    cost_model: CostModel,
    stats: Arc<StatisticsStore>,
}

impl Optimizer {
    pub fn new(cost_model: CostModel, stats: Arc<StatisticsStore>) -> Self;
    pub fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan>;
    pub fn choose_join_order(&self, plan: &LogicalPlan) -> Result<LogicalPlan>;
    pub fn choose_index(&self, plan: &LogicalPlan) -> Result<LogicalPlan>;
}
```

---

## 五、网络层 API

### 5.1 服务器

异步服务器。

```rust
pub struct Server {
    config: ServerConfig,
    handler: Arc<dyn QueryHandler>,
}

impl Server {
    pub fn new(config: ServerConfig, handler: Arc<dyn QueryHandler>) -> Self;
    pub async fn start(&self) -> Result<()>;
    pub async fn shutdown(&self) -> Result<()>;
}
```

### 5.2 会议

会话管理。

```rust
pub struct Session {
    id: Uuid,
    user: String,
    database: String,
    created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user: String, database: String) -> Self;
    pub fn id(&self) -> Uuid;
    pub fn user(&self) -> &str;
    pub fn database(&self) -> &str;
}
```

---

## 六、使用示例

### 6.1 向量化执行

```rust
use sqlrustgo::vectorized::{RecordBatch, Int64Array, VectorizedFilter};

// 创建 RecordBatch
let schema = Arc::new(Schema::new(vec![
    Field::new("id", DataType::Int64, false),
    Field::new("value", DataType::Int64, true),
]));

let columns: Vec<ArrayRef> = vec![
    Arc::new(Int64Array::new(vec![1, 2, 3, 4, 5])),
    Arc::new(Int64Array::new(vec![10, 20, 30, 40, 50])),
];

let batch = RecordBatch::new(schema, columns)?;

// 向量化过滤
let filter = VectorizedFilter::new(predicate);
let filtered = filter.execute(&batch)?;
```

### 6.2 统计信息收集

```rust
use sqlrustgo::stats::{StatisticsCollector, TableStats};

let collector = StatisticsCollector::new(storage);

// 收集表统计信息
let stats = collector.collect_table_stats("users")?;
println!("Row count: {}", stats.row_count);

// 执行 ANALYZE
collector.analyze_table("users")?;
```

### 6.3 CBO 优化

```rust
use sqlrustgo::optimizer::{Optimizer, CostModel};

let cost_model = CostModel::new();
let optimizer = Optimizer::new(cost_model, stats_store);

// 优化查询计划
let optimized_plan = optimizer.optimize(logical_plan)?;
```

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-04 | 初始版本 |
