# 向量化并行执行引擎集成实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将向量化执行引擎、列式存储、并行执行统一集成为完整的向量化并行执行流水线

**Architecture:** 
- 使用独立的 VectorStorage trait 扩展存储接口
- DataChunk 作为向量化执行核心数据结构
- SIMD 聚合使用 std::simd 实现
- Partition + SIMD + Reduce 并行模式

**Tech Stack:** Rust std::simd, Rayon, ColumnarStorage, DataChunk

---

## 实施阶段

### Phase 0: 数据结构统一 (ColumnArray ↔ ChunkColumn)

#### Task 0.1: 添加 ColumnArray 到 ChunkColumn 转换

**Files:**
- Create: `crates/storage/src/columnar/convert.rs`
- Modify: `crates/executor/src/vectorization.rs` (import)
- Test: `crates/executor/tests/test_vector_convert.rs`

**Step 1: 创建转换模块**

```rust
// crates/storage/src/columnar/convert.rs

use crate::columnar::chunk::ChunkColumn;
use sqlrustgo_executor::vectorization::ColumnArray;

/// 从 ChunkColumn 转换到 ColumnArray
impl From<ChunkColumn> for ColumnArray {
    fn from(col: ChunkColumn) -> Self {
        match col {
            ChunkColumn::Int64(v) => ColumnArray::Int64(v),
            ChunkColumn::Float64(v) => ColumnArray::Float64(v),
            ChunkColumn::Boolean(v) => ColumnArray::Boolean(v),
            ChunkColumn::Text(v) => ColumnArray::Text(v),
            ChunkColumn::Null => ColumnArray::Null,
        }
    }
}
```

**Step 2: 添加 ChunkColumn enum 到 storage crate**

检查 `crates/storage/src/columnar/chunk.rs` 是否已有 ChunkColumn enum，如没有则添加。

**Step 3: 验证编译**

```bash
cargo build -p sqlrustgo-storage
cargo build -p sqlrustgo-executor
```

---

#### Task 0.2: 添加 DataChunk 到 ColumnarStorage 扫描

**Files:**
- Modify: `crates/storage/src/columnar/storage.rs`
- Test: `crates/executor/tests/test_columnar_vector_scan.rs`

**Step 1: 添加 scan_chunk 方法到 ColumnarStorage**

在 `ColumnarStorage` 结构体实现中添加:

```rust
impl ColumnarStorage {
    /// 扫描整个表，返回 DataChunk
    pub fn scan_chunk(&self, table: &str) -> ColumnarResult<DataChunk> {
        use crate::columnar::convert::IntoColumnArray;
        
        let table_store = self.tables.get(table)
            .ok_or_else(|| ColumnarError::TableNotFound(table.to_string()))?;
        
        let mut chunk = DataChunk::new(table_store.row_count());
        
        for (_, col_chunk) in &table_store.columns {
            let col_array: ColumnArray = col_chunk.data.clone().into();
            chunk.add_column(col_array);
        }
        
        Ok(chunk)
    }
}
```

**Step 2: 添加 DataChunk 导入**

```rust
use sqlrustgo_executor::vectorization::DataChunk;
```

**Step 3: 运行测试**

```bash
cargo test -p sqlrustgo-storage -- columnar
```

---

### Phase 1: VectorStorage Trait

#### Task 1.1: 定义 VectorStorage trait

**Files:**
- Modify: `crates/storage/src/engine.rs`
- Test: `crates/executor/tests/test_vector_storage_trait.rs`

**Step 1: 添加 VectorStorage trait**

在 `engine.rs` 文件末尾添加:

```rust
/// 向量化存储接口 - 用于向量化执行引擎
pub trait VectorStorage: Send + Sync {
    /// 扫描整个表，返回 DataChunk
    fn scan_chunk(&self, table: &str) -> SqlResult<DataChunk>;
    
    /// 按列扫描，返回 DataChunk
    fn scan_columns(&self, table: &str, columns: &[usize]) -> SqlResult<DataChunk>;
    
    /// 范围扫描，返回 DataChunk
    fn scan_range(&self, table: &str, start: usize, end: usize) -> SqlResult<DataChunk>;
    
    /// 获取表 schema (列名列表)
    fn table_schema(&self, table: &str) -> SqlResult<Vec<String>>;
}
```

**Step 2: 为 ColumnarStorage 实现 VectorStorage**

创建 `crates/storage/src/columnar/vector_storage.rs`:

```rust
use super::{ColumnarResult, ColumnarStorage};
use crate::engine::{SqlResult, VectorStorage};
use sqlrustgo_executor::vectorization::DataChunk;

impl VectorStorage for ColumnarStorage {
    fn scan_chunk(&self, table: &str) -> SqlResult<DataChunk> {
        self.scan_chunk(table)
            .map_err(|e| SqlError::ExecutionError(e.to_string()))
    }
    
    fn scan_columns(&self, table: &str, columns: &[usize]) -> SqlResult<DataChunk> {
        // 实现按列扫描
    }
    
    fn scan_range(&self, table: &str, start: usize, end: usize) -> SqlResult<DataChunk> {
        // 实现范围扫描
    }
    
    fn table_schema(&self, table: &str) -> SqlResult<Vec<String>> {
        let info = self.get_table_info(table)?;
        Ok(info.columns.iter().map(|c| c.name.clone()).collect())
    }
}
```

**Step 3: 运行测试验证**

```bash
cargo test -p sqlrustgo-storage -- vector_storage
```

---

### Phase 2: VectorizedExecutor

#### Task 2.1: 创建 VectorizedSeqScanExecutor

**Files:**
- Create: `crates/executor/src/vector_executor.rs`
- Modify: `crates/executor/src/lib.rs` (export)
- Test: `crates/executor/tests/test_vector_executor.rs`

**Step 1: 创建向量化扫描执行器**

```rust
// crates/executor/src/vector_executor.rs

use crate::{ExecutorResult, SqlResult};
use sqlrustgo_planner::{PhysicalPlan, Schema};
use sqlrustgo_storage::engine::VectorStorage;
use sqlrustgo_types::Value;
use std::sync::Arc;

/// 向量化顺序扫描执行器
pub struct VectorizedSeqScanExecutor {
    storage: Arc<dyn VectorStorage>,
    table_name: String,
    schema: Schema,
    batch_size: usize,
}

impl VectorizedSeqScanExecutor {
    pub fn new(
        storage: Arc<dyn VectorStorage>,
        table_name: String,
        schema: Schema,
    ) -> Self {
        Self {
            storage,
            table_name,
            schema,
            batch_size: 4096,
        }
    }
    
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
    
    /// 执行向量化扫描
    pub fn execute(&self) -> SqlResult<ExecutorResult> {
        let chunk = self.storage.scan_chunk(&self.table_name)?;
        let rows = chunk.to_rows();
        Ok(ExecutorResult::new(rows, 0))
    }
}
```

**Step 2: 更新 lib.rs 导出**

```rust
pub mod vector_executor;
pub use vector_executor::VectorizedSeqScanExecutor;
```

**Step 3: 编写测试**

```rust
#[test]
fn test_vectorized_seq_scan() {
    use sqlrustgo_executor::VectorizedSeqScanExecutor;
    use sqlrustgo_storage::{ColumnarStorage, VectorStorage};
    
    let storage = Arc::new(ColumnarStorage::new());
    // 创建测试表和数据...
    
    let executor = VectorizedSeqScanExecutor::new(
        storage.clone(),
        "test_table".to_string(),
        schema,
    );
    
    let result = executor.execute().unwrap();
    assert_eq!(result.rows.len(), 1000);
}
```

**Step 4: 运行测试**

```bash
cargo test -p sqlrustgo-executor -- test_vectorized_seq_scan
```

---

#### Task 2.2: 向量化过滤

**Files:**
- Modify: `crates/executor/src/vector_executor.rs`
- Test: `crates/executor/tests/test_vector_filter.rs`

**Step 1: 添加向量化过滤执行器**

```rust
/// 向量化过滤执行器
pub struct VectorizedFilterExecutor {
    input: Box<dyn VectorizedScan>,
    predicate: Expr,
}

impl VectorizedFilterExecutor {
    pub fn execute(&self) -> SqlResult<ExecutorResult> {
        use crate::vectorization::{apply_filter, DataChunk};
        
        let chunk = self.input.execute()?;
        let predicate_col = crate::vectorization::eval_expr(
            &self.predicate, 
            &chunk, 
            &self.input.schema()
        );
        
        let filtered = apply_filter(&chunk, &predicate_col);
        let rows = filtered.to_rows();
        
        Ok(ExecutorResult::new(rows, 0))
    }
}
```

---

### Phase 3: SIMD 聚合

#### Task 3.1: 实现 std::simd 聚合函数

**Files:**
- Create: `crates/executor/src/simd_vectorized.rs`
- Modify: `crates/executor/src/vectorization.rs` (use)
- Test: `crates/executor/tests/test_simd_agg.rs`

**Step 1: 创建 SIMD 聚合模块**

```rust
// crates/executor/src/simd_vectorized.rs

#![allow(unused)]

use std::simd::{f64x4, i64x4, u64x4, SimdOrd, SimdPartialOrd};

/// SIMD 加速的 i64 求和
#[inline]
pub fn sum_i64(values: &[i64]) -> i64 {
    if values.is_empty() {
        return 0;
    }
    
    let mut sum = 0i64;
    let chunks = values.chunks(4);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let v = i64x4::from_slice(chunk);
        sum += v.reduce_sum();
    }
    
    for &v in remainder {
        sum += v;
    }
    
    sum
}

/// SIMD 加速的 f64 求和 (Kahan 算法)
#[inline]
pub fn sum_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    
    let mut sum = 0.0;
    let mut c = 0.0; // Kahan compensation
    
    for &v in values {
        let y = v - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    
    sum
}

/// SIMD 加速的 i64 MIN
#[inline]
pub fn min_i64(values: &[i64]) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    
    let mut min = i64::MAX;
    let chunks = values.chunks(4);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let v = i64x4::from_slice(chunk);
        min = min.min(v.reduce_min());
    }
    
    for &v in remainder {
        min = min.min(v);
    }
    
    Some(min)
}

/// SIMD 加速的 i64 MAX
#[inline]
pub fn max_i64(values: &[i64]) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    
    let mut max = i64::MIN;
    let chunks = values.chunks(4);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let v = i64x4::from_slice(chunk);
        max = max.max(v.reduce_max());
    }
    
    for &v in remainder {
        max = max.max(v);
    }
    
    Some(max)
}
```

**Step 2: 在 vectorization.rs 中使用**

修改 `crates/executor/src/vectorization.rs` 中的 simd_agg 模块:

```rust
pub mod simd_agg {
    pub use crate::simd_vectorized::{sum_i64, sum_f64, min_i64, max_i64};
    // ... 其他函数
}
```

**Step 3: 编写测试**

```rust
#[test]
fn test_simd_sum_i64() {
    use crate::simd_vectorized::sum_i64;
    
    let values: Vec<i64> = (1..=1000).collect();
    let result = sum_i64(&values);
    assert_eq!(result, 500500);
}
```

**Step 4: 运行性能测试对比**

```bash
cargo test -p sqlrustgo-executor -- test_simd
```

---

### Phase 4: 并行向量化执行

#### Task 4.1: 创建 PartitionAgent

**Files:**
- Create: `crates/executor/src/partition_agent.rs`
- Test: `crates/executor/tests/test_partition_agent.rs`

**Step 1: 创建分区代理**

```rust
// crates/executor/src/partition_agent.rs

use crate::vectorization::DataChunk;

/// 分区描述符
#[derive(Debug, Clone)]
pub struct Partition {
    pub id: usize,
    pub start: usize,
    pub end: usize,
}

/// PartitionAgent - 管理数据分区
pub struct PartitionAgent {
    partition_count: usize,
}

impl PartitionAgent {
    pub fn new(partition_count: usize) -> Self {
        Self { partition_count }
    }
    
    /// 将 DataChunk 分区
    pub fn partition(&self, chunk: &DataChunk) -> Vec<Partition> {
        let total_rows = chunk.num_rows();
        let partition_size = total_rows / self.partition_count;
        
        (0..self.partition_count)
            .map(|i| {
                let start = i * partition_size;
                let end = if i == self.partition_count - 1 {
                    total_rows
                } else {
                    start + partition_size
                };
                Partition { id: i, start, end }
            })
            .collect()
    }
}
```

---

#### Task 4.2: 创建 ParallelVectorExecutor

**Files:**
- Create: `crates/executor/src/parallel_vector_executor.rs`
- Modify: `crates/executor/src/lib.rs` (export)
- Test: `crates/executor/tests/test_parallel_vector.rs`

**Step 1: 创建并行向量化执行器**

```rust
// crates/executor/src/parallel_vector_executor.rs

use crate::{ExecutorResult, SqlResult};
use crate::partition_agent::PartitionAgent;
use crate::task_scheduler::{RayonTaskScheduler, TaskScheduler};
use crate::vectorization::DataChunk;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlrustgo_planner::PhysicalPlan;
use sqlrustgo_storage::engine::VectorStorage;
use std::sync::Arc;

/// 并行向量化执行器
pub struct ParallelVectorExecutor {
    storage: Arc<dyn VectorStorage>,
    scheduler: Arc<RayonTaskScheduler>,
    partition_agent: PartitionAgent,
}

impl ParallelVectorExecutor {
    pub fn new(
        storage: Arc<dyn VectorStorage>,
        partition_count: usize,
    ) -> Self {
        let scheduler = RayonTaskScheduler::new(partition_count);
        Self {
            storage,
            scheduler: Arc::new(scheduler),
            partition_agent: PartitionAgent::new(partition_count),
        }
    }
    
    /// 执行并行向量化扫描
    pub fn execute_scan(&self, table: &str) -> SqlResult<ExecutorResult> {
        // 1. 获取完整 DataChunk
        let chunk = self.storage.scan_chunk(table)?;
        let total_rows = chunk.num_rows();
        
        // 2. 分区
        let partitions = self.partition_agent.partition(&chunk);
        
        // 3. 并行处理每个分区
        let results: Vec<DataChunk> = partitions
            .into_par_iter()
            .map(|partition| {
                chunk.slice(partition.start, partition.end)
            })
            .collect();
        
        // 4. 合并结果
        let merged = DataChunk::concatenate(results);
        let rows = merged.to_rows();
        
        Ok(ExecutorResult::new(rows, 0))
    }
    
    /// 执行并行向量化聚合
    pub fn execute_aggregate(
        &self, 
        table: &str, 
        agg_funcs: &[AggFunction]
    ) -> SqlResult<ExecutorResult> {
        let chunk = self.storage.scan_chunk(table)?;
        let partitions = self.partition_agent.partition(&chunk);
        
        // 并行计算每个分区的聚合
        let partial_results: Vec<Vec<Value>> = partitions
            .into_par_iter()
            .map(|partition| {
                let part_chunk = chunk.slice(partition.start, partition.end);
                compute_aggregates(&part_chunk, agg_funcs)
            })
            .collect();
        
        // 合并部分结果
        let final_result = merge_aggregate_results(partial_results, agg_funcs);
        
        Ok(ExecutorResult::new(vec![final_result], 0))
    }
}
```

---

### Phase 5: 集成测试

#### Task 5.1: 向量化执行集成测试

**Files:**
- Create: `crates/executor/tests/test_vectorized_integration.rs`

**Step 1: 编写集成测试**

```rust
#[test]
fn test_vectorized_scan_integration() {
    // 创建 ColumnarStorage 并插入数据
    let storage = ColumnarStorage::new();
    storage.create_table(&table_info).unwrap();
    insert_test_data(&storage, "test_table", 10000);
    
    let vector_storage: Arc<dyn VectorStorage> = Arc::new(storage);
    
    // 测试向量化扫描
    let executor = VectorizedSeqScanExecutor::new(
        vector_storage.clone(),
        "test_table".to_string(),
        schema,
    );
    
    let result = executor.execute().unwrap();
    assert_eq!(result.rows.len(), 10000);
}

#[test]
fn test_parallel_vector_scan_speedup() {
    let storage = ColumnarStorage::new();
    insert_test_data(&storage, "speedup_test", 100000);
    
    let vector_storage: Arc<dyn VectorStorage> = Arc::new(storage);
    
    // 单线程基准
    let single_executor = VectorizedSeqScanExecutor::new(
        vector_storage.clone(),
        "speedup_test".to_string(),
        schema.clone(),
    );
    let start = Instant::now();
    single_executor.execute().unwrap();
    let single_time = start.elapsed();
    
    // 4线程并行
    let parallel_executor = ParallelVectorExecutor::new(
        vector_storage,
        4,
    );
    let start = Instant::now();
    parallel_executor.execute_scan("speedup_test").unwrap();
    let parallel_time = start.elapsed();
    
    let speedup = single_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);
    assert!(speedup > 1.0, "Parallel should be faster");
}
```

**Step 2: 运行集成测试**

```bash
cargo test -p sqlrustgo-executor -- test_vectorized_integration
```

---

## 验证检查清单

在完成每个任务后验证:

- [ ] `cargo build -p sqlrustgo-executor` 编译通过
- [ ] `cargo test -p sqlrustgo-executor -- <test_name>` 测试通过
- [ ] `cargo check --workspace` 无警告
- [ ] 代码符合项目风格 (使用 `cargo fmt`)

---

## 提交信息格式

```
feat(executor): add vectorized execution engine

- Add VectorStorage trait for columnar scan
- Implement VectorizedSeqScanExecutor  
- Add SIMD聚合 functions using std::simd
- Add ParallelVectorExecutor with partition+reduce pattern

Closes: #1310
```

---

## 执行选项

**1. Subagent-Driven (当前会话)** - 每任务派发新的 subagent，任务间 review，快速迭代

**2. Parallel Session (新会话)** - 在新会话中使用 executing-plans，批量执行带检查点

选择哪种方式？
