# Issue #1343: 向量检索性能基准

## 概述
验证向量检索性能达到目标，为 v2.5.0 性能优化提供基准数据。

## Issue 要求

### 性能目标
| 指标 | 目标 |
|------|------|
| 10K vectors KNN | < 5ms |
| 100K vectors KNN | < 10ms |
| 1M vectors KNN | < 100ms |

### 功能点
- [x] KNN 检索性能
  - [ ] 10万向量 KNN 检索 < 10ms
  - [ ] 100万向量 KNN 检索 < 100ms
  - [x] HNSW/IVF 参数调优
- [ ] 混合查询性能
  - [ ] SQL WHERE 预过滤 + Vector Top-K
  - [ ] 混合评分算法优化

## 实现计划

### 1. 扩展 vector_benchmark.rs

添加以下基准测试：

#### KNN 性能基准
- `bench_10k_knn_cosine` - 10K 向量，余弦相似度，目标 < 5ms
- `bench_100k_knn_cosine` - 100K 向量，余弦相似度，目标 < 10ms
- `bench_1m_knn_cosine` - 1M 向量，余弦相似度，目标 < 100ms
- `bench_1m_knn_euclidean` - 1M 向量，欧氏距离

#### HNSW 参数调优基准
- `bench_hnsw_ef_search` - ef_search 参数 (16, 32, 64, 128, 256)
- `bench_hnsw_m_param` - m 参数 (8, 16, 32, 64)
- `bench_hnsw_100k_search` - 100K 向量 HNSW 搜索

#### IVF 参数调优基准
- `bench_ivf_nlists` - nlists 参数 (50, 100, 200, 500)
- `bench_ivf_100k_search` - 100K 向量 IVF 搜索

#### 混合查询基准
- `bench_hybrid_filtered_search` - SQL WHERE 预过滤 + Vector Top-K
- `bench_hybrid_weighted_scoring` - 混合评分算法

### 2. 集成测试

在 `tests/integration/vector_storage_integration_test.rs` 中添加：
- 大规模向量插入和搜索测试
- HNSW/IVF 索引类型切换测试
- 混合查询功能测试

## 验收标准

1. `cargo bench` 运行成功，所有基准测试通过
2. 性能目标达标：
   - 10K KNN < 5ms
   - 100K KNN < 10ms  
   - 1M KNN < 100ms
3. `cargo test` 所有测试通过

## 依赖

- Issue #1326 (父任务) - 向量存储层

## 状态

- [ ] P0: KNN 性能基准完成
- [ ] P0: HNSW/IVF 参数调优完成
- [ ] P1: 混合查询基准完成
