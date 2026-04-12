# Vector Index Design - v2.2

**Issue**: #1285 (向量索引框架), #1286 (并行 KNN 引擎)  
**Author**: yinglichina163  
**Date**: 2026-04-05  
**Status**: Approved

---

## 概述

实现高性能向量索引框架，支持 IVF、Flat、HNSW 等索引类型，提供并行 KNN 计算能力。

## 技术规格

| 参数 | 选择 |
|------|------|
| 向量规模 | 中规模 (100K ~ 10M) |
| 更新模式 | 实时更新 (增删改) |
| 距离度量 | 余弦、欧几里得、内积、曼哈顿 |
| HNSW M | 16 (默认) |
| HNSW efConstruction | 128 (默认) |
| HNSW efSearch | 64 (默认) |
| IVF 聚类中心 | k = √n |
| GPU 支持 | 第一阶段不含 |

## 架构设计

### 模块结构

```
crates/
├── vector/                    # 新建向量索引模块
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs          # VectorIndex trait
│   │   ├── flat.rs            # Flat 索引 (brute-force)
│   │   ├── ivf.rs             # IVF 索引
│   │   ├── hnsw.rs            # HNSW 索引
│   │   ├── metrics.rs         # 距离度量实现
│   │   ├── build.rs           # 索引构建器
│   │   └── search.rs          # 搜索接口
│   └── Cargo.toml
```

### 接口设计

```rust
pub trait VectorIndex: Send + Sync {
    fn insert(&mut self, id: u64, vector: &[f32]) -> Result<()>;
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(u64, f32)>>;
    fn build_index(&mut self) -> Result<()>;
    fn delete(&mut self, id: u64) -> Result<()>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
    Manhattan,
}
```

## 实现计划

### Phase 1: Flat 索引
- [ ] 实现 4 种距离度量
- [ ] Flat 索引结构 (brute-force)
- [ ] 与 GMP 现有实现对比验证

### Phase 2: IVF 索引
- [ ] k-means 聚类实现
- [ ] IVF 倒排列表
- [ ] 增量插入支持
- [ ] 召回率验证 (>95%)

### Phase 3: HNSW 索引
- [ ] 多层图结构
- [ ] 贪心搜索算法
- [ ] 索引构建算法
- [ ] 性能基准测试 (>2x 提升)

## 与现有 GMP 集成

- 复用 `crates/gmp/src/embedding.rs` 中的 `cosine_similarity` 等函数
- 新增 `crates/vector/` 模块作为独立索引引擎
- GMP 的 `vector_search.rs` 可调用新索引

## 依赖

- Issue #1285 (向量索引框架) - 先决条件
- GMP crate 嵌入模型 (可选复用)
