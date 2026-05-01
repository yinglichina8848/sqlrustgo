# Vector 模块设计

**版本**: v2.5.0
**模块**: Vector (向量索引)

---

## 一、What (是什么)

Vector 是 SQLRustGo 的向量索引模块，支持多种向量索引算法（HNSW、IVF、IVFPQ），提供高效的向量相似度搜索能力。

## 二、Why (为什么)

- **AI 应用**: 语义搜索、推荐系统
- **向量检索**: 高维向量数据的最近邻搜索
- **混合查询**: SQL + 向量融合
- **性能优化**: SIMD 加速和量化压缩

## 三、How (如何实现)

### 3.1 索引类型

| 索引类型 | 构建复杂度 | 查询复杂度 | 内存 | 召回率 |
|----------|------------|------------|------|--------|
| Flat | O(n) | O(n) | 100% | 100% |
| HNSW | O(n log n) | O(log n) | ~120% | 95-99% |
| IVF | O(n log k) | O(k log n) | ~110% | 90-95% |
| IVFPQ | O(n log k) | O(log n) | ~10% | 85-90% |

### 3.2 HNSW 实现

```rust
pub struct HNSWIndex {
    // 图结构
    layers: Vec<Layer>,
    // 入口点
    entry_point: Option<NodeId>,
    // 元数据
    m: usize,           // 每层连接数
    ef_construction: usize,  // 构建时搜索范围
    ef_search: usize,   // 查询时搜索范围
    dimension: usize,
}

struct Layer {
    graph: BTreeMap<NodeId, Vec<(NodeId, f32)>>,  // 邻居节点和距离
}

impl HNSWIndex {
    // 构建索引
    pub fn build(&mut self, vectors: &[Vector]) -> Result<()> {
        for (id, vector) in vectors.iter().enumerate() {
            self.insert(id as NodeId, vector.clone());
        }
        Ok(())
    }

    // 插入向量
    pub fn insert(&mut self, id: NodeId, vector: Vector) -> Result<()> {
        // 计算插入层
        let level = self.random_level();

        // 从高层到低层逐层插入
        for l in (0..=level).rev() {
            let neighbors = self.search_layer(vector, l, self.ef_construction);
            self.connect(id, neighbors, l);
        }
        Ok(())
    }

    // 搜索最近邻
    pub fn search(&self, query: &Vector, k: usize) -> Result<Vec<(NodeId, f32)>> {
        // 从顶层开始贪心搜索
        let mut candidates = vec![self.entry_point.unwrap()];

        for layer in self.layers.iter().rev() {
            candidates = self.search_layer_in_layer(query, &candidates, layer, self.ef_search);
        }

        // 取前 k 个
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(candidates.into_iter().take(k).collect())
    }
}
```

### 3.3 IVF-PQ 实现

```rust
pub struct IVFPQIndex {
    // PQ 编码器
    pq: ProductQuantizer,
    // 倒排列表
    inverted_lists: Vec<Vec<VectorId>>,
    // 码书
    codebook: Vec<Vec<f32>>,
    // 质心
    centroids: Vec<Vector>,
}

pub struct ProductQuantizer {
    dimension: usize,
    sub_dim: usize,     // 子维度
    num_sub: usize,     // 子空间数量
    bits: usize,        // 每个子空间的位数
    codebook: Vec<Vec<f32>>,
}

impl IVFPQIndex {
    // 构建
    pub fn build(&mut self, vectors: &[Vector]) -> Result<()> {
        // 1. 聚类得到质心
        self.centroids = self.kmeans(vectors, self.num_clusters)?;

        // 2. 分配向量到倒排列表
        for (id, vector) in vectors.iter().enumerate() {
            let centroid = self.assign_centroid(vector);
            self.inverted_lists[centroid].push(id as VectorId);
        }

        // 3. 训练 PQ 码书
        self.pq.train(vectors);

        // 4. 编码所有向量
        for (id, vector) in vectors.iter().enumerate() {
            let code = self.pq.encode(vector);
            self.codes[id] = code;
        }

        Ok(())
    }

    // 搜索
    pub fn search(&self, query: &Vector, k: usize) -> Result<Vec<(VectorId, f32)>> {
        // 1. 查询向量 PQ 编码
        let query_code = self.pq.encode(query);

        // 2. 计算到各质心的距离
        let centroid_dists = self.centroids
            .iter()
            .enumerate()
            .map(|(i, c)| (i, euclidean(query, c)))
            .collect::<Vec<_>>();

        // 3. 选择最近的 nprobe 个倒排列表
        let selected_lists = self.select_lists(centroid_dists, self.nprobe);

        // 4. 在选中的列表中搜索
        let mut candidates = Vec::new();
        for list_id in selected_lists {
            for &vid in &self.inverted_lists[list_id] {
                let dist = self.pq.decode_distance(&query_code, &self.codes[vid]);
                candidates.push((vid, dist));
            }
        }

        // 5. 取前 k 个
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(candidates.into_iter().take(k).collect())
    }
}
```

### 3.4 SIMD 加速

```rust
#[cfg(target_arch = "x86_64")]
pub fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    unsafe {
        let mut sum = _mm512_setzero_ps();
        let mut square_sum = _mm512_setzero_ps();

        for chunk in a.chunks(16) {
            let va = _mm512_loadu_ps(chunk.as_ptr());
            let vb = _mm512_loadu_ps(b.as_ptr());

            let diff = _mm512_sub_ps(va, vb);
            square_sum = _mm512_fmadd_ps(diff, diff, square_sum);
        }

        let result = _mm512_reduce_add_ps(square_sum);
        result.sqrt()
    }
}
```

## 四、接口设计

### 4.1 公开 API

```rust
pub trait VectorIndex {
    // 插入向量
    fn insert(&mut self, id: VectorId, vector: Vector) -> Result<()>;

    // 搜索最近邻
    fn search(&self, query: &Vector, k: usize) -> Result<Vec<(VectorId, f32)>>;

    // 批量搜索
    fn search_batch(&self, queries: &[Vector], k: usize) -> Result<Vec<Vec<(VectorId, f32)>>>;

    // 删除向量
    fn delete(&mut self, id: VectorId) -> Result<()>;

    // 获取向量
    fn get(&self, id: VectorId) -> Result<Option<Vector>>;
}
```

### 4.2 索引工厂

```rust
pub enum VectorIndexType {
    Flat,
    HNSW { m: usize, ef_construction: usize, ef_search: usize },
    IVF { nlist: usize, nprobe: usize },
    IVFPQ { nlist: usize, nprobe: usize, sub_dim: usize, bits: usize },
}

pub struct VectorIndexFactory;

impl VectorIndexFactory {
    pub fn create(
        index_type: VectorIndexType,
        dimension: usize,
    ) -> Result<Box<dyn VectorIndex>> {
        match index_type {
            VectorIndexType::Flat => Ok(Box::new(FlatIndex::new(dimension))),
            VectorIndexType::HNSW { .. } => Ok(Box::new(HNSWIndex::new(dimension, m, ef))),
            VectorIndexType::IVF { .. } => Ok(Box::new(IVFIndex::new(dimension, nlist))),
            VectorIndexType::IVFPQ { .. } => Ok(Box::new(IVFPQIndex::new(dimension, nlist, sub_dim, bits))),
        }
    }
}
```

## 五、性能考虑

### 5.1 性能指标

| 索引类型 | 100K 向量 | 1M 向量 | 10M 向量 |
|----------|-----------|---------|----------|
| Flat | < 10ms | < 100ms | < 1s |
| HNSW | < 5ms | < 10ms | < 50ms |
| IVFPQ | < 2ms | < 5ms | < 20ms |

### 5.2 内存使用

| 索引类型 | 100K 向量 (128维) |
|----------|-------------------|
| Flat | ~50MB |
| HNSW | ~60MB |
| IVFPQ | ~5MB |

## 六、相关文档

- [VECTOR_INDEX_DESIGN.md](../../../VECTOR_INDEX_DESIGN.md) - 详细设计
- *(已归档 - 统一查询文档不存在)*

---

*Vector 模块设计 v2.5.0*
