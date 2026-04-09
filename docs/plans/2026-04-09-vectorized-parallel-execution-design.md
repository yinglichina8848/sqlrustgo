# 向量化并行执行引擎集成设计

**日期**: 2026-04-09
**状态**: 已批准
**目标**: 将向量化、列式存储、并行执行统一集成

---

## 1. 架构总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Query Execution Layer                        │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐    ┌──────────────────┐                     │
│  │  VolcanoExecutor │    │ VectorizedExecutor│  (共存)           │
│  │  (legacy row)    │    │  (batch vector)    │                     │
│  └────────┬─────────┘    └────────┬─────────┘                     │
│           │                        │                                  │
│           │              ┌─────────┴─────────┐                       │
│           │              │  PartitionAgent   │                       │
│           │              │  (并行分区管理)    │                       │
│           │              └─────────┬─────────┘                       │
│           │                        │                                  │
├───────────┴────────────────────────┴─────────────────────────────────┤
│                         SIMD Layer (std::simd)                      │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ sum_i64  │  │ avg_f64  │  │ min_i64  │  │ max_i64  │  ...    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘          │
├─────────────────────────────────────────────────────────────────────┤
│                         Storage Layer                                │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐    ┌──────────────────┐                     │
│  │   StorageEngine   │    │  VectorStorage    │  (独立 trait)       │
│  │   (legacy)       │    │  (新接口)         │                     │
│  └────────┬─────────┘    └────────┬─────────┘                     │
│           │                        │                                  │
│           │              ┌─────────┴─────────┐                       │
│           │              │  ColumnarStorage  │                       │
│           │              │  (已有,增强)       │                       │
│           │              └──────────────────┘                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. 核心数据结构

### 2.1 DataChunk (已有)

```rust
// crates/executor/src/vectorization.rs
pub struct DataChunk {
    columns: Vec<ColumnArray>,  // 列式数据
    num_rows: usize,
    schema: Vec<String>,
}

pub enum ColumnArray {
    Int64(Vec<i64>),
    Float64(Vec<f64>),
    Boolean(Vec<bool>),
    Text(Vec<String>),
    Null,
}
```

### 2.2 ColumnChunk (已有)

```rust
// crates/storage/src/columnar/chunk.rs
pub struct ColumnChunk {
    pub data: ChunkColumn,
    pub stats: ColumnStats,
    pub compression: CompressionType,
}

pub enum ChunkColumn {
    Int64(Vec<i64>),
    Float64(Vec<f64>),
    Boolean(Vec<bool>),
    Text(Vec<String>),
    Null,
}
```

### 2.3 新增: VectorStorage Trait

```rust
// crates/storage/src/engine.rs
pub trait VectorStorage: Send + Sync {
    fn scan_chunk(&self, table: &str) -> SqlResult<DataChunk>;
    fn scan_columns(&self, table: &str, columns: &[usize]) -> SqlResult<DataChunk>;
    fn scan_range(&self, table: &str, start: usize, end: usize) -> SqlResult<DataChunk>;
    fn table_schema(&self, table: &str) -> SqlResult<Vec<String>>;
}
```

---

## 3. 实现顺序

### Phase 0: 数据结构统一
- [ ] ColumnArray ↔ ChunkColumn 互相转换
- [ ] DataChunk 与 ColumnarStorage 对接

### Phase 1: VectorStorage Trait
- [ ] 定义 VectorStorage trait
- [ ] ColumnarStorage 实现 VectorStorage
- [ ] MemoryStorage/FilStorage 默认实现

### Phase 2: VectorizedExecutor
- [ ] VectorizedSeqScanExecutor
- [ ] VectorizedFilter
- [ ] VectorizedProjection

### Phase 3: SIMD 聚合
- [ ] 使用 std::simd 重写 simd_agg
- [ ] SIMD SUM/COUNT/AVG/MIN/MAX

### Phase 4: 并行向量化
- [ ] PartitionAgent 分区管理
- [ ] ParallelVectorExecutor
- [ ] Partition + SIMD + Reduce 模式

### Phase 5: 集成测试
- [ ] 向量化执行测试
- [ ] SIMD 性能基准测试
- [ ] 并行加速比测试

---

## 4. 关键设计决策

### 4.1 SIMD 策略
- 使用 `std::simd` (Rust 稳定版便携 SIMD)
- 兼容性好，支持 AVX2/AVX-512 自动选择

### 4.2 StorageEngine 关系
- VectorStorage 作为独立 trait
- 不破坏现有 StorageEngine 接口
- 实现者可选择实现或不实现

### 4.3 执行器关系
- VectorizedExecutor 与 VolcanoExecutor 共存
- Planner 根据查询类型选择使用哪个
- 保持向后兼容

### 4.4 并行策略
- Partition + SIMD + Reduce 混合模式
- 每线程处理完整 DataChunk 分区
- 最后 Reduce 合并结果

---

## 5. 文件变更清单

| 文件 | 变更类型 |
|------|---------|
| `crates/executor/src/vectorization.rs` | 增强 |
| `crates/executor/src/vector_executor.rs` | 新增 |
| `crates/executor/src/parallel_vector_executor.rs` | 新增 |
| `crates/executor/src/simd_vectorized.rs` | 新增 |
| `crates/storage/src/engine.rs` | 增强 |
| `crates/storage/src/columnar/storage.rs` | 增强 |

---

## 6. 预期收益

- **向量化扫描**: 减少函数调用开销，提高 CPU 缓存命中率
- **SIMD 聚合**: 批量处理数据，利用 CPU 向量指令
- **并行执行**: 多线程并行处理，充分利用多核
- **列式存储**: 只读取需要的列，减少 IO

---

## 7. 风险与缓解

| 风险 | 缓解措施 |
|------|---------|
| std::simd 稳定性 | 使用稳定版，有 scalar fallback |
| 并行开销 | 小数据集不并行，直接返回 |
| 内存占用 | Batch size 可配置 |
