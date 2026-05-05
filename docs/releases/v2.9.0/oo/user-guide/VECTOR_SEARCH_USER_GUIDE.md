# 向量检索用户指南

> **版本**: v2.9.0 (RC)
> **更新日期**: 2026-05-05

---

## 1. 概述

SQLRustGo v2.9.0 提供高性能向量索引，支持 Flat、IVF、HNSW 三种算法，支持 SIMD 加速和 GPU 加速。

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
use sqlrustgo_vector::{HnswIndex, DistanceMetric};

let mut index = HnswIndex::new(DistanceMetric::Cosine, 16, 32, 100);

// 插入向量
index.insert(1, &[0.1, 0.2, 0.3]).unwrap();
index.insert(2, &[0.4, 0.5, 0.6]).unwrap();

// 构建索引
index.build_index().unwrap();

// 搜索
let results = index.search(&[0.1, 0.2, 0.3], 5).unwrap();
```

### 2.3 混合检索

```rust
use sqlrustgo_vector::{HybridSearch, SearchRequest};

let request = SearchRequest {
    query_vector: &[0.1, 0.2, 0.3],
    keyword_query: Some("SELECT * FROM products WHERE category = 'electronics'"),
    top_k: 10,
    alpha: 0.7,  // 0.7 * vector + 0.3 * keyword
};

let results = executor.hybrid_search(request).unwrap();
```

---

## 3. SQL API

### 3.1 创建向量表

```sql
-- 创建带向量列的表
CREATE TABLE embeddings (
    id INTEGER PRIMARY KEY,
    document_id INTEGER,
    embedding FLOAT[128],
    metadata JSON
);

-- 插入向量数据
INSERT INTO embeddings (id, document_id, embedding) VALUES
    (1, 101, '[0.1, 0.2, 0.3, ...]'),
    (2, 102, '[0.4, 0.5, 0.6, ...]');
```

### 3.2 向量索引

```sql
-- 创建 HNSW 向量索引
CREATE VECTOR INDEX idx_embeddings ON embeddings
USING hnsw(embedding, dimension=128, m=16, ef_construction=100);

-- 创建 IVF 向量索引
CREATE VECTOR INDEX idx_embeddings ON embeddings
USING ivf(embedding, dimension=128, nlist=100);
```

### 3.3 向量搜索

```sql
-- 纯向量搜索
SELECT id, document_id, vector_distance(embedding, '[0.1, 0.2, 0.3, ...]', 'cosine') AS distance
FROM embeddings
ORDER BY distance
LIMIT 10;

-- 带过滤的向量搜索
SELECT id, document_id
FROM embeddings
WHERE metadata->>'category' = 'electronics'
ORDER BY vector_distance(embedding, '[0.1, 0.2, 0.3, ...]', 'cosine')
LIMIT 10;
```

---

## 4. 索引管理

### 4.1 查看索引

```sql
-- 查看所有向量索引
SHOW VECTOR INDEXES;

-- 查看索引状态
SELECT * FROM vector_index_stats WHERE index_name = 'idx_embeddings';
```

### 4.2 删除索引

```sql
DROP VECTOR INDEX idx_embeddings;
```

### 4.3 重建索引

```sql
-- 重建索引
ALTER VECTOR INDEX idx_embeddings REBUILD;
```

---

## 5. SIMD 和 GPU 加速

### 5.1 启用 SIMD

```toml
[dependencies]
sqlrustgo-vector = { version = "2.9", features = ["simd"] }
```

### 5.2 启用 GPU

```toml
[dependencies]
sqlrustgo-vector = { version = "2.9", features = ["cuda"] }
```

### 5.3 配置

```sql
-- 设置计算后端
SET vector.backend = 'cuda';  -- 或 'cpu', 'simd'

-- 设置 GPU 设备
SET vector.gpu_device = 0;
```

---

## 6. 性能调优

### 6.1 HNSW 参数

| 参数 | 说明 | 建议值 |
|------|------|--------|
| `m` | 每个节点的最大连接数 | 16-32 |
| `ef_construction` | 构建时的搜索范围 | 100-200 |
| `ef_search` | 搜索时的搜索范围 | 50-200 |

### 6.2 IVF 参数

| 参数 | 说明 | 建议值 |
|------|------|--------|
| `nlist` | 聚类数量 | 100-1000 |
| `nprobe` | 搜索时的聚类数 | 1-100 |

---

## 7. 最佳实践

### 7.1 向量维度

- 建议使用 128-1024 维
- 过高维度会影响搜索性能
- 根据数据特点选择合适维度

### 7.2 索引构建时机

- 批量导入后统一构建
- 增量数据使用后台增量索引
- 定期重建索引保持质量

### 7.3 混合检索

- 合理设置 alpha 参数
- 根据结果质量调整
- 使用 rerank 优化结果

---

## 8. API 参考

| API | 说明 |
|-----|------|
| `FlatIndex::new()` | 创建 Flat 索引 |
| `HnswIndex::new()` | 创建 HNSW 索引 |
| `IvfIndex::new()` | 创建 IVF 索引 |
| `insert()` | 插入向量 |
| `build_index()` | 构建索引 |
| `search()` | 搜索向量 |
| `hybrid_search()` | 混合检索 |

---

*向量检索用户指南 v2.9.0*
*最后更新: 2026-05-05*
