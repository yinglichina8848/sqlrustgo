# 向量检索性能基准测试报告

**Issue**: #1343 向量检索性能基准 - 10万/100万向量KNN  
**日期**: 2026-04-10  
**分支**: develop/v2.5.0

## 性能目标

| 指标 | 目标 |
|------|------|
| 10K vectors KNN | < 5ms |
| 100K vectors KNN | < 10ms |
| 1M vectors KNN | < 100ms |

## 测试环境

- **平台**: macOS (Apple Silicon)
- **向量维度**: 128
- **距离度量**: Cosine Similarity
- **并行化**: Rayon (多线程)
- **HNSW 参数**: m=16, ef_construction=200, ef_search=256

## 测试结果汇总

### 1. HNSW (Hierarchical Navigable Small World)

#### 增量构建 (Incremental Build)

| 规模 | 构建时间 | 搜索时间 | vs 目标 | 状态 |
|------|----------|----------|---------|------|
| 1K | 0.2s | 0.33ms | - | ✅ |
| 10K | 22s | 4.4ms | < 5ms | ✅ |
| 100K | 2977s (~50分钟) | 64ms | < 10ms | ❌ |

**问题**: 增量构建 O(n²) 复杂度导致 100K 规模需要约 50 分钟。

#### 批量构建 + NN-descent 优化 (Batch Build + NN-descent)

| 规模 | 构建时间 | 搜索时间 | vs 目标 | 状态 |
|------|----------|----------|---------|------|
| 10K | 22s | 4.5ms | < 5ms | ✅ |
| 100K | 306s | 64ms | < 10ms | ❌ |

**问题**: 批量构建无法正确构建层次结构的"高速公路"连接，导致搜索时无法找到几何捷径。

### 2. ParallelKnn (备选方案 - Brute Force + Rayon)

| 规模 | 构建时间 | 搜索时间 | vs 目标 | 状态 |
|------|----------|----------|---------|------|
| 10K | ~0s | 8ms | < 5ms | ⚠️ 接近 |
| 100K | ~0s | 86ms | < 10ms | ❌ |
| 1M | ~0s | 987ms | < 100ms | ❌ |

## 性能分析

### HNSW 增量构建瓶颈

```
100K 向量增量构建耗时分解:
- 每次插入搜索已有图: O(n) 
- 总复杂度: O(n²) = 100K × 50K = 5B 次操作
- 实际耗时: ~2977s (50分钟)
```

### HNSW 批量构建问题

**根因**: HNSW 的层次结构需要增量插入才能正确构建"高速公路"连接。

- Layer 0 可以通过并行 k-NN 计算
- 高层的"高速公路"效果需要逐个插入向量才能正确建立跳层连接
- 批量构建时，高层使用随机连接，没有几何捷径效果

### ParallelKnn 瓶颈

```
1M 向量搜索耗时分解:
- 距离计算: ~800ms (128M 次 cos similarity)
- Top-K 排序: ~150ms (1M elements)  
- 线程同步: ~40ms
- 总计: ~987ms
```

## 代码优化

### 已完成的优化

1. **Generational visited tracking**: 使用 `AtomicU32` + `RwLock<Vec<u32>>` 实现 O(1) 访问标记
2. **search_layer 重构**: 使用 `BinaryHeap` 替代 `Vec + remove(0)`，提高优先队列效率
3. **移除不必要的数据结构克隆**: 减少内存分配
4. **批量构建方法**: `build_from_vectors` 支持并行 k-NN
5. **NN-descent 风格迭代优化**: 迭代优化邻居连接

### 未使用的代码清理

移除了以下未使用的方法:
- `distance()` - 未使用
- `build_layer0_graph()` - 被 `build_from_vectors` 替代

## 回归测试

已集成以下测试到 `cargo test`:

```bash
# 运行所有向量测试
cargo test -p sqlrustgo-vector

# 运行 HNSW 测试 (包括 1K 快速测试)
cargo test -p sqlrustgo-vector hnsw -- --nocapture

# 运行 10K/100K/1M 性能测试 (需要 --ignored)
cargo test -p sqlrustgo-vector -- --ignored --nocapture
```

### 测试列表

| 测试 | 规模 | 状态 |
|------|------|------|
| `test_hnsw_insert_and_search` | 1K | ✅ 通过 |
| `test_hnsw_1k_build_and_search` | 1K | ✅ 通过 |
| `test_hnsw_10k_build_and_search` | 10K | ⚠️ 被忽略 (22s 构建) |
| `test_hnsw_100k_search_performance` | 100K | ⚠️ 被忽略 (50分钟构建) |
| `test_hnsw_100k_batch_build` | 100K | ⚠️ 被忽略 (306s 构建) |
| `test_hnsw_1m_search_performance` | 1M | ⚠️ 被忽略 |

## 结论

### ✅ 已达标
- 10K vectors KNN < 5ms ✅ (HNSW 实测 4.4ms)

### ❌ 未达标
- 100K vectors KNN < 10ms ❌ (HNSW 批量构建搜索 64ms)
- 1M vectors KNN < 100ms ❌ (ParallelKnn 实测 987ms)

### 建议

1. **短期**: 
   - HNSW 增量构建 100K 需 50 分钟，不适合生产环境
   - 需要研究如何加速 HNSW 构建 (如 IVF-PQ 预处理)

2. **中期**: 
   - 实现 IVF-PQ (倒排索引 + 产品量化) 以支持更大规模数据
   - 可在有限内存下处理更大规模数据

3. **长期**: 
   - GPU 加速 OpenCL 实现
   - 分布式向量索引

## 附录: 基准测试命令

```bash
# 运行所有基准测试
cargo bench -p sqlrustgo-vector

# 运行特定基准测试
cargo bench -p sqlrustgo-vector --bench vector_benchmark -- flat_search
cargo bench -p sqlrustgo-vector --bench vector_benchmark -- parallel_knn
cargo bench -p sqlrustgo-vector --bench vector_benchmark -- hnsw_search

# 运行性能测试 (实际数据量)
cargo test -p sqlrustgo-vector -- --ignored --nocapture
```

## 提交历史

| Commit | 描述 |
|--------|------|
| `e4dc513` | perf(vector): optimize HNSW with generational visited tracking |
| `1f40449` | perf(vector): HNSW batch build implementation |
| `703f43d` | perf(vector): HNSW batch build implementation - hierarchical structure incomplete |
| `8d0d3ff` | perf(vector): NN-descent refinement attempted - still not meeting targets |