# 向量检索用户指南

> **版本**: v2.7.0 (GA)
> **更新日期**: 2026-04-22

---

## 1. 概述

SQLRustGo v2.7.0 提供高性能向量索引，支持 Flat、IVF、HNSW 三种算法，支持 SIMD 加速和 GPU 加速。

### 1.1 支持的索引类型

| 类型 | 说明 | 适用场景 |
|------|------|----------|
| **Flat** | 暴力搜索 O(n) | 小数据集、精确搜索 |
| **IVF** | 倒排文件索引 | 中等规模、平衡性能 |
| **HNSW** | 分层可导航小世界图 | 大规模、高召回 |

### 1.2 支持的距离度量

| 度量 | 说明 |
|------|------|
| **Cosine** | 余弦相似度 |
| **Euclidean** | 欧几里得距离 |
| **DotProduct** | 点积 |
| **Manhattan** | 曼哈顿距离 |

---

## 2. 快速开始

### 2.1 Flat 索引

```rust
use sqlrustgo_vector::{FlatIndex, DistanceMetric, VectorIndex};

let mut index = FlatIndex::new(DistanceMetric::Cosine);

// 插入向量
index.insert(1, &[0.1, 0.2, 0.3]).unwrap();
index.insert(2, &[0.4, 0.5, 0.6]).unwrap();

// 构建索引
index.build_index().unwrap();

// 搜索
let results = index.search(&[0.1, 0.2, 0.3], 1).unwrap();
```

### 2.2 HNSW 索引

```rust
use sqlrustgo_vector::HnswIndex;

let mut index = HnswIndex::new(
    DistanceMetric::Cosine,
    16,    // m
    200,   // ef_construction
    100,   // max_elements
);

index.insert(1, &[0.1, 0.2, 0.3]).unwrap();
index.build_index().unwrap();

let results = index.search(&[0.1, 0.2, 0.3], 5).unwrap();
```

### 2.3 IVF 索引

```rust
use sqlrustgo_vector::IvfIndex;

let mut index = IvfIndex::new(
    DistanceMetric::Euclidean,
    100,   // nlist (聚类数)
    10,    // nprobe
);

index.insert(1, &[0.1, 0.2, 0.3]).unwrap();
index.train(&train_data).unwrap();
index.build_index().unwrap();
```

---

## 3. 高级特性

### 3.1 SIMD 加速

```rust
use sqlrustgo_vector::simd_explicit::{
    compute_similarity_simd,
    batch_compute_distances,
};

let similarity = compute_similarity_simd(
    &[0.1, 0.2, 0.3],
    &[0.4, 0.5, 0.6],
).unwrap();

let distances = batch_compute_distances(
    &query,
    &vectors,
    DistanceMetric::Cosine,
).unwrap();
```

### 3.2 GPU 加速

```rust
use sqlrustgo_vector::gpu_accel::{GpuAccelerator, GpuConfig};

let config = GpuConfig {
    device_id: 0,
    use_fp16: true,
};

let gpu = GpuAccelerator::new(config).unwrap();
let results = gpu.search(&query, &index, 10).unwrap();
```

### 3.3 分片索引

```rust
use sqlrustgo_vector::sharded_index::{ShardedVectorIndex, HashPartitioner};

let partitioner = HashPartitioner::new(4);  // 4 个分片
let sharded = ShardedVectorIndex::new(partitioner);

sharded.insert(1, &[0.1, 0.2, 0.3]).unwrap();
sharded.insert(2, &[0.4, 0.5, 0.6]).unwrap();

let results = sharded.search(&[0.1, 0.2, 0.3], 5).unwrap();
```

---

## 4. 混合检索

### 4.1 SQL 混合搜索

```sql
-- 混合搜索 (RRF)
SELECT * FROM documents
WHERE HYBRID_SEARCH(
    content,
    embedding,
    strategy = 'RRF',
    weights = [0.3, 0.7]
)
LIMIT 10;
```

### 4.2 混合检索配置

```rust
use sqlrustgo_vector::sql_vector_hybrid::{HybridSearcher, HybridSearchConfig};

let config = HybridSearchConfig {
    vector_weight: 0.7,
    keyword_weight: 0.3,
    rrf_k: 60,
};

let searcher = HybridSearcher::new(config);
let results = searcher.hybrid_search("关键词", &embedding, 10).unwrap();
```

---

## 5. 性能调优

### 5.1 HNSW 参数

| 参数 | 说明 | 推荐值 |
|------|------|--------|
| `m` | 每个节点的最大连接数 | 16-64 |
| `ef_construction` | 构建时的搜索范围 | 100-400 |
| `ef_search` | 搜索时的搜索范围 | 50-1000 |

### 5.2 IVF 参数

| 参数 | 说明 | 推荐值 |
|------|------|--------|
| `nlist` | 聚类数量 | 数据量的 1/10 |
| `nprobe` | 查询探查的聚类数 | 1-50 |

### 5.3 批量写入

```rust
use sqlrustgo_vector::batch_writer::{BatchVectorWriter, BatchWriteConfig};

let config = BatchWriteConfig {
    batch_size: 1000,
    flush_interval_ms: 100,
};

let writer = BatchVectorWriter::new(index, config);
writer.write_many(&vectors).unwrap();
writer.flush().unwrap();
```

---

## 6. 向量存储设计

### 6.1 表结构

```sql
CREATE TABLE document_embeddings (
    id BIGINT PRIMARY KEY,
    document_id BIGINT,
    embedding VECTOR(384),
    created_at TIMESTAMP
);

CREATE INDEX idx_embedding ON document_embeddings USING hnsw(embedding);
```

### 6.2 嵌入生成

```rust
use sqlrustgo_gmp::semantic_embedding::{ProviderFactory, EmbeddingProviderConfig, OpenAIConfig};

let config = EmbeddingProviderConfig::OpenAI(OpenAIConfig {
    api_key: "sk-xxx".to_string(),
    model: "text-embedding-3-small".to_string(),
});

let provider = ProviderFactory::create(config);
let embedding = provider.embed("要嵌入的文本").unwrap();
```

---

## 7. 最佳实践

### 7.1 索引选择

- **小数据集 (< 10k)**: Flat 索引，精确结果
- **中等规模 (10k - 1M)**: IVF 或 HNSW
- **大规模 (> 1M)**: HNSW，内存充足时启用 GPU

### 7.2 维度选择

- OpenAI text-embedding-3-small: 1536 维
- OpenAI text-embedding-3-large: 3072 维
- BGE-large: 1024 维

### 7.3 性能优化

- 启用 SIMD 加速
- 大数据集启用批量写入
- 合理设置 HNSW 参数

---

## 8. API 参考

| API | 说明 |
|-----|------|
| `FlatIndex::new()` | 创建 Flat 索引 |
| `HnswIndex::new()` | 创建 HNSW 索引 |
| `IvfIndex::new()` | 创建 IVF 索引 |
| `VectorIndex::insert()` | 插入向量 |
| `VectorIndex::search()` | 搜索向量 |
| `HybridSearcher` | 混合搜索 |
| `GpuAccelerator` | GPU 加速 |

---

## 9. 故障排查

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 搜索结果为空 | 索引未构建 | 调用 `build_index()` |
| 内存占用过高 | 向量维度太高 | 降低维度或启用量化 |
| GPU 未找到 | CUDA 未安装 | 检查 CUDA 环境 |

---

*向量检索用户指南 v2.7.0*
*最后更新: 2026-04-22*