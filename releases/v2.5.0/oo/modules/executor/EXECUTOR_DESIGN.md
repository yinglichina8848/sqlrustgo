# Executor 模块设计

**版本**: v2.5.0
**模块**: Executor (查询执行器)

---

## 一、What (是什么)

Executor 是 SQLRustGo 的查询执行引擎，负责执行由优化器生成的物理执行计划，支持向量化执行和并行处理。

## 二、Why (为什么)

- **高性能**: 向量化执行减少函数调用开销
- **并行处理**: 多线程并行处理数据
- **SIMD 加速**: 利用 CPU 指令集加速计算
- **灵活执行**: 支持多种执行策略

## 三、How (如何实现)

### 3.1 执行器架构

```
┌─────────────────────────────────────────┐
│           ExecutorManager                 │
├─────────────────────────────────────────┤
│  - 创建执行上下文                       │
│  - 管理执行资源                         │
│  - 处理执行结果                         │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         ExecutionContext                 │
├─────────────────────────────────────────┤
│  - Session 状态                         │
│  - 事务上下文                           │
│  - 统计信息                             │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│           PhysicalPlan                   │
├─────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐│
│  │  Scan   │→ │ Filter  │→ │ Project ││
│  └─────────┘  └─────────┘  └─────────┘│
└─────────────────────────────────────────┘
```

### 3.2 向量化执行

```rust
// 批处理结构
struct Batch {
    num_rows: usize,
    columns: Vec<Column>,
}

struct Column {
    data: ColumnData,
    nulls: Bitmap,
}

enum ColumnData {
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    String(Vec<String>),
    // ...
}
```

### 3.3 SIMD 加速

```rust
#[cfg(target_arch = "x86_64")]
pub fn filter_avx512(predicate: &[u8], data: &[i32]) -> Vec<i32> {
    unsafe {
        let mut result = Vec::new();
        for chunk in data.chunks(16) {
            let mask = _mm256_loadu_si256(predicate.as_ptr() as *const _);
            let values = _mm256_loadu_si256(chunk.as_ptr() as *const _);
            let filtered = _mm256_and_si256(mask, values);
            // ... 保存结果
        }
        result
    }
}
```

### 3.4 并行执行

```rust
pub struct ParallelExecutor {
    thread_pool: RayonThreadPool,
    max_parallelism: usize,
}

impl ParallelExecutor {
    pub fn execute_parallel(&self, plan: &PhysicalPlan) -> Result<Batch> {
        // 数据分区
        let partitions = self.partition_data();

        // 并行执行
        let results = partitions
            .par_iter()
            .map(|partition| self.execute_partition(plan, partition))
            .collect::<Result<Vec<_>>>()?;

        // 合并结果
        self.merge_results(results)
    }
}
```

## 四、接口设计

### 4.1 公开 API

```rust
impl Executor {
    // 创建执行器
    pub fn new(context: ExecutionContext) -> Self;

    // 执行物理计划
    pub fn execute(&mut self, plan: PhysicalPlan) -> Result<BatchIterator>;

    // 执行单条查询
    pub fn execute_one(&mut self, plan: PhysicalPlan) -> Result<Row>;

    // 执行批处理
    pub fn execute_batch(&mut self, plan: PhysicalPlan) -> Result<Vec<Batch>>;

    // 关闭执行器
    pub fn close(&mut self) -> Result<()>;
}

// 批量迭代器
pub trait BatchIterator {
    fn next_batch(&mut self) -> Result<Option<Batch>>;
    fn schema(&self) -> Schema;
}
```

### 4.2 执行器类型

| 执行器 | 说明 |
|--------|------|
| SeqExecutor | 顺序执行 |
| VectorExecutor | 向量化执行 |
| ParallelExecutor | 并行执行 |
| TransactionalExecutor | 事务执行 |

## 五、性能优化

### 5.1 优化技术

| 技术 | 作用 | 效果 |
|------|------|------|
| 向量化 | 批量处理 | 减少函数调用 10-100x |
| SIMD | 数据级并行 | 加速计算 4-16x |
| 缓存友好 | 内存布局优化 | 减少缓存未命中 |
| 预取 | 提前加载数据 | 减少 IO 等待 |

### 5.2 性能指标

| 查询类型 | 目标延迟 | 当前状态 |
|----------|----------|----------|
| 点查 | < 1ms | ✅ |
| 范围扫描 | < 10ms | ✅ |
| 聚合 | < 50ms | ✅ |
| JOIN | < 100ms | ✅ |

## 六、相关文档

- [ARCHITECTURE_V2.5.md](../../architecture/ARCHITECTURE_V2.5.md) - 整体架构
- [OPTIMIZER_DESIGN.md](../optimizer/OPTIMIZER_DESIGN.md) - 优化器

---

*Executor 模块设计 v2.5.0*
