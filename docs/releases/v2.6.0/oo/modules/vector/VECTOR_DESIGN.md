# Vector 模块设计

**版本**: v2.6.0
**模块**: Vector (向量索引)

---

## 一、What (是什么)

Vector 是 SQLRustGo 的向量索引模块，支持 HNSW、IVF、IVFPQ 等多种索引算法。

## 二、Why (为什么)

- **AI 应用**: 语义搜索、推荐系统
- **向量检索**: 高维向量数据的最近邻搜索
- **混合查询**: SQL + 向量融合

## 三、支持的索引类型

| 索引类型 | 构建复杂度 | 查询复杂度 | 召回率 |
|----------|------------|------------|--------|
| Flat | O(n) | O(n) | 100% |
| HNSW | O(n log n) | O(log n) | 95-99% |
| IVF | O(n log k) | O(k log n) | 90-95% |
| IVFPQ | O(n log k) | O(log n) | 85-90% |

## 四、v2.6.0 优化

- HNSW 内存优化，内存降低 30%
- 支持 AVX-512 SIMD 加速
- 改进的 PQ 编码

## 五、相关文档

- [ARCHITECTURE_V2.6.md](../architecture/ARCHITECTURE_V2.6.md)
- [PERFORMANCE_ANALYSIS.md](../reports/PERFORMANCE_ANALYSIS.md)

---

*Vector 模块设计 v2.6.0*
