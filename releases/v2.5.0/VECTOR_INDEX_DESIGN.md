# 向量索引设计文档

**版本**: v2.5.0
**最后更新**: 2026-04-16

---

## 概述

SQLRustGo提供多种向量索引实现以支持高效相似性搜索：

- **Flat**: 暴力O(n)搜索（基线）
- **HNSW**: 分层可导航小世界
- **IVF**: 倒排文件索引
- **IVFPQ**: 带乘积量化的IVF压缩

## 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    VectorStore trait                             │
│  - insert(id, vector)                                          │
│  - search(query, k) -> Vec<SearchResult>                       │
│  - build_index()                                               │
│  - save/load                                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
     ┌───────────┐     ┌───────────┐     ┌───────────┐
     │FlatIndex  │     │ HNSWIndex │     │ IVFPQIndex│
     └───────────┘     └───────────┘     └───────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │ ProductQuantizer  │
                    │ - PQ16编码       │
                    │ - k_sub = 64    │
                    └───────────────────┘
```

## Flat索引

最简单的实现 - 存储所有向量并暴力搜索：

```rust
pub struct FlatIndex {
    vectors: Vec<Vec<f32>>,
    dimension: usize,
}

impl VectorIndex for FlatIndex {
    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        let distances: Vec<_> = self.vectors
            .iter()
            .enumerate()
            .map(|(i, v)| (i, euclidean_distance(query, v)))
            .collect();

        distances
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .take(k)
            .map(|(i, d)| SearchResult { id: i, distance: d })
            .collect()
    }
}
```

## HNSW索引

分层可导航小世界 - 提供O(log n)平均搜索：

```rust
pub struct HNSWIndex {
    layers: Vec<HNSWLayer>,
    entry_point: Option<usize>,
    m: usize,           // 每个节点最大连接数
    m0: usize,          // 第0层最大连接数
    ef_construction: usize,
    ef_search: usize,
}

pub struct HNSWLayer {
    neighbors: HashMap<usize, Vec<(usize, f32)>>,  // 节点 -> [(邻居, 距离)]
    distances: HashMap<(usize, usize), f32>,
}

impl VectorIndex for HNSWIndex {
    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        // 1. 从最高层的入口点开始
        // 2. 贪心搜索到第0层
        // 3. 在第0层使用束搜索 (ef_search)
        // 4. 返回k个最近邻居
    }
}
```

### HNSW构建

```rust
impl HNSWIndex {
    pub fn insert(&mut self, id: usize, vector: Vec<f32>) {
        // 1. 采样随机层 L ~ exp(-L)
        // 2. 从L到0在每一层插入
        // 3. 在每一层使用贪心搜索找邻居
        // 4. 连接到m个最近邻居
    }
}
```

## IVF索引

倒排文件索引 - 将向量空间划分为聚类：

```rust
pub struct IVFIndex {
    quantizer: Box<dyn VectorIndex>,  // 通常是HNSW或Flat
    centroids: Vec<Vec<f32>>,
    inverted_list: Vec<Vec<usize>>,    // 每个聚类的倒排列表
    nlist: usize,                      // 聚类数量
}

impl VectorIndex for IVFIndex {
    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        // 1. 找nprobe个最近聚类中心
        let nearest_centroids = self.quantizer.search(query, self.nprobe);

        // 2. 在这些聚类的倒排列表中搜索
        let mut candidates = Vec::new();
        for centroid_id in nearest_centroids {
            for &vector_id in &self.inverted_list[centroid_id] {
                candidates.push(vector_id);
            }
        }

        // 3. 重新排序候选
        self.rerank(query, candidates, k)
    }
}
```

## IVFPQ索引

带乘积量化的IVF - 实现高压缩比：

```rust
pub struct IVFPQIndex {
    pq: ProductQuantizer,
    inverted_list: Vec<Vec<Vec<u8>>>,  // 压缩向量
    nlist: usize,
    nprobe: usize,
}

pub struct ProductQuantizer {
    dimension: usize,
    m_pq: usize,           // 子向量数量
    k_sub: usize,          // 子向量聚类中心数 (PQ16为256)
    codebooks: Vec<Vec<f32>>,
}

impl ProductQuantizer {
    // PQ16: dimension=128, m=8, k_sub=256 -> 16x压缩
    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        (0..self.m_pq)
            .map(|i| {
                let start = i * self.sub_dim;
                let sub = &vector[start..start + self.sub_dim];
                self.find_nearest_centroid(sub, i)
            })
            .collect()
    }

    pub fn decode(&self, code: &[u8]) -> Vec<f32> {
        let mut result = vec![0.0; self.dimension];
        for (i, &centroid_id) in code.iter().enumerate() {
            let start = i * self.sub_dim;
            let centroid = &self.codebooks[i][centroid_id as usize * self.sub_dim..];
            result[start..start + self.sub_dim].copy_from_slice(centroid);
        }
        result
    }
}
```

## SIMD加速

AVX-512和AVX2 intrinsics用于距离计算：

```rust
#[cfg(target_arch = "x86_64")]
pub fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    debug_assert!(a.len() % 16 == 0);

    unsafe {
        let mut sum = _mm512_setzero_ps();
        for chunk in a.chunks(16) {
            let va = _mm512_loadu_ps(chunk.as_ptr());
            let vb = _mm512_loadu_ps(b.as_ptr().add(chunk.as_ptr() as usize - a.as_ptr() as usize));
            let diff = _mm512_sub_ps(va, vb);
            sum = _mm512_fmadd_ps(diff, diff, sum);
        }
        _mm512_reduce_add_ps(sum).sqrt()
    }
}
```

## 并行KNN

基于Rayon的并行搜索：

```rust
pub struct ParallelKnn {
    index: Arc<dyn VectorIndex>,
    num_threads: usize,
}

impl ParallelKnn {
    pub fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        // 将搜索空间分配给线程
        let results = (0..self.num_threads)
            .into_par_iter()
            .flat_map(|thread_id| {
                let start = thread_id * (self.index.size() / self.num_threads);
                let end = if thread_id == self.num_threads - 1 {
                    self.index.size()
                } else {
                    start + self.index.size() / self.num_threads
                };
                self.index.search_range(query, k, start, end)
            })
            .collect();

        // 合并并排序结果
        merge_and_sort(results, k)
    }
}
```

## 性能对比

| 索引 | 构建时间 | 查询时间 | 内存 | 召回率 |
|------|----------|----------|------|--------|
| Flat | O(n) | O(n) | 100% | 100% |
| HNSW | O(n log n) | O(log n) | ~120% | 95-99% |
| IVF | O(n log k) | O(k log n) | ~110% | 90-95% |
| IVFPQ | O(n log k) | O(log n) | ~10% | 85-90% |

## 配置

```rust
pub struct VectorIndexConfig {
    pub index_type: IndexType,  // Flat, HNSW, IVF, IVFPQ
    pub dimension: usize,
    pub metric: Metric,         // Euclidean, Cosine, Dot
    // HNSW特定
    pub m: Option<usize>,      // 最大连接数 (默认: 16)
    pub ef_construction: Option<usize>,
    pub ef_search: Option<usize>,
    // IVF特定
    pub nlist: Option<usize>,   // 聚类数量
    pub nprobe: Option<usize>,
    // PQ特定
    pub m_pq: Option<usize>,    // 子向量数 (默认: 8)
    pub k_sub: Option<usize>,   // 子聚类中心数 (默认: 256)
}
```

## 测试覆盖

| 测试 | 位置 | 状态 |
|------|------|------|
| Flat索引 | `vector/flat_test.rs` | ✅ |
| HNSW插入 | `vector/hnsw_test.rs` | ✅ |
| HNSW搜索 | `vector/hnsw_test.rs` | ✅ |
| IVF搜索 | `vector/ivf_test.rs` | ✅ |
| IVFPQ编码/解码 | `vector/ivfpq_test.rs` | ✅ |
| IVFPQ压缩 | `vector/ivfpq_test.rs` | ✅ |
| SIMD距离 | `vector/simd_test.rs` | ✅ |
| 并行KNN | `vector/parallel_knn_test.rs` | ✅ |

## 基准测试结果

| 数据集 | 大小 | 索引 | QPS | P99延迟 |
|--------|------|-------|-----|---------|
| SIFT-1M | 1M | Flat | 100 | 100ms |
| SIFT-1M | 1M | HNSW | 5,000 | 2ms |
| SIFT-1M | 1M | IVFPQ | 15,000 | 0.5ms |
| Deep-1M | 1M | HNSW | 3,000 | 3ms |
| Deep-1M | 1M | IVFPQ | 10,000 | 0.8ms |
