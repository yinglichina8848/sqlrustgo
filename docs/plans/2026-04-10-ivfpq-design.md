# IVFPQ 向量索引设计

**Issue**: #1343 向量检索性能优化  
**日期**: 2026-04-10  
**分支**: develop/v2.5.0

## 目标

将 Product Quantization (PQ) 集成到现有 IVF 实现，形成 IVFPQ 索引，以达成：

| 指标 | 目标 | 当前 |
|------|------|------|
| 10K vectors KNN | < 5ms | ✅ 4.4ms |
| 100K vectors KNN | < 10ms | ❌ 64ms |
| 1M vectors KNN | < 100ms | ❌ 987ms |

## 架构

### 数据结构

```rust
struct IVFPQIndex {
    dimension: usize,           // 向量维度
    metric: DistanceMetric,     // 距离度量
    nlist: usize,               // 聚类数 (默认 256)
    m_pq: usize,                // PQ 子向量数 (默认 16)
    k_sub: usize,               // 子向量聚类数 (默认 256)
    
    // 原始向量（构建后可能释放）
    vectors: Vec<(u64, Vec<f32>)>,
    
    // 聚类
    clusters: Vec<Cluster>,
    
    // PQ 编码器
    pq: ProductQuantizer,
    
    built: bool,
}

struct Cluster {
    center: Vec<f32>,           // 聚类中心
    vector_ids: Vec<u64>,       // 原始 ID 列表
    codes: Vec<Vec<u8>>,        // PQ 编码列表
}

struct ProductQuantizer {
    dimension: usize,
    m_pq: usize,                // 子向量数
    k_sub: usize,               // 每个子空间的聚类数
    centroids: Vec<Vec<f32>>,   // 码本: [m_pq][k_sub][sub_dim]
}
```

### 压缩比

- 原始: 1M vectors × 128 dim × 4 bytes = **512 MB**
- IVFPQ: 1M × 16 bytes (PQ code) + 256 × 128 × 4 bytes (聚类中心) ≈ **16.5 MB**
- **压缩比: ~31x**

## 算法

### 1. PQ 训练 (Product Quantization)

对每个聚类内的向量独立训练 PQ：

```
1. 将向量分成 m_pq 个子向量 (每个 sub_dim = 128 / 16 = 8)
2. 对每个子空间执行 k-means (k = k_sub = 256)
3. 存储码本: centroids[sub_idx][centroid_idx][sub_dim]
```

### 2. 编码 (Encode)

```
encode(vector):
    codes = []
    for sub_idx in 0..m_pq:
        sub_vec = vector[sub_idx * sub_dim : (sub_idx + 1) * sub_dim]
        // 找到最近的 centroids
        min_dist = INF
        min_idx = 0
        for c_idx in 0..k_sub:
            dist = euclidean(sub_vec, centroids[sub_idx][c_idx])
            if dist < min_dist:
                min_dist = dist
                min_idx = c_idx
        codes.push(min_idx as u8)
    return codes  // m_pq 字节
```

### 3. 搜索 (ADC - Asymmetric Distance Computation)

```
search(query, k):
    // 1. 找到最近的 nprobe 个聚类
    scores = [(cluster_id, compute_similarity(query, center)) for center]
    top_clusters = scores.sort(desc).take(nprobe)
    
    // 2. 对每个候选聚类，用 ADC 计算距离
    candidates = []
    for cluster in top_clusters:
        for (id, code) in cluster.codes:
            dist = adc_distance(query, code)  // 不解码！
            candidates.push((id, dist))
    
    // 3. 返回 top-k
    return candidates.sort(desc).take(k)

adc_distance(query, code):
    total = 0
    for sub_idx in 0..m_pq:
        // 找到 query 子向量最近的 centroids
        sub_vec = query[sub_idx * sub_dim : ...]
        c_idx = code[sub_idx]
        total += euclidean(sub_vec, centroids[sub_idx][c_idx])
    return total
```

## 实现计划

### 阶段 1: ProductQuantizer 模块

```rust
// crates/vector/src/pq.rs (新文件)

impl ProductQuantizer {
    pub fn new(dimension: usize, m_pq: usize, k_sub: usize) -> Self;
    pub fn train(&mut self, vectors: &[Vec<f32>]) -> VectorResult<()>;
    pub fn encode(&self, vector: &[f32]) -> Vec<u8>;
    pub fn decode(&self, code: &[u8]) -> Vec<f32>;
    pub fn adc_distance(&self, query: &[f32], code: &[u8]) -> f32;
}
```

### 阶段 2: IVFPQ 索引

```rust
// crates/vector/src/ivfpq.rs (新文件)

pub struct IvfpqIndex {
    // 复用部分 IVF 字段
    dimension: usize,
    metric: DistanceMetric,
    nlist: usize,
    vectors: Vec<(u64, Vec<f32>)>,
    clusters: Vec<Cluster>,
    pq: ProductQuantizer,
    built: bool,
}

impl IvfpqIndex {
    pub fn new(metric: DistanceMetric, nlist: usize, m_pq: usize) -> Self;
    pub fn with_params(nlist: usize, m_pq: usize, k_sub: usize, metric: DistanceMetric) -> Self;
}
```

### 阶段 3: 集成到 lib.rs

```rust
// crates/vector/src/lib.rs

pub mod ivfpq;
pub use ivfpq::IvfpqIndex;
```

## 性能目标

| 规模 | 构建时间 | 搜索时间 | 内存使用 | vs 目标 |
|------|----------|----------|----------|---------|
| 10K | < 1s | < 2ms | ~1 MB | ✅ |
| 100K | < 5s | < 5ms | ~10 MB | ✅ |
| 1M | < 30s | < 50ms | ~65 MB | ✅ |

## 测试计划

```bash
# 运行 IVFPQ 测试
cargo test -p sqlrustgo-vector ivfpq

# 运行性能测试 (需要 --ignored)
cargo test -p sqlrustgo-vector -- --ignored --nocapture
```

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| PQ 精度损失 | 使用 m_pq=16, k_sub=256 保持高精度 |
| 码本训练慢 | 使用 Rayon 并行训练各子空间 |
| Apple Silicon SIMD | 使用 rayon 并行化替代 |

## 参考资料

- Jégou et al., "Product Quantization for Nearest Neighbor Search"
- Faiss IVFPQ 实现
