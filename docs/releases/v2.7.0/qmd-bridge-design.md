# qmd-bridge 设计文档

> 版本: `v2.7.0`  
> 日期: 2026-05-XX  
> 负责人: TBD  
> 相关 Issue: T-04

---

## 1. 概述

### 1.1 目标
实现 SQLRustGo 与 QMD (Query Memory Database) 的双向数据桥梁，支持：
- 将 SQLRustGo 的图谱/向量数据同步到 QMD
- 通过 QMD 进行混合检索并返回 SQLRustGo 可执行的结果
- 统一的检索 API 接口

### 1.2 架构图
```
┌─────────────────────────────────────────────────────────────┐
│                      SQLRustGo Engine                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │ Parser  │  │Planner  │  │Executor │  │ Transaction │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────┘  │
│       │            │            │               │           │
│  ┌────────────────────────────────────────────────────┐    │
│  │              Unified Retrieval Layer                │    │
│  │  ┌──────────────┐  ┌───────────────────────────┐  │    │
│  │  │ Vector Index  │  │     Graph Index (GMP)    │  │    │
│  │  └──────────────┘  └───────────────────────────┘  │    │
│  └────────────────────────────────────────────────────┘    │
│                           │                                 │
└───────────────────────────│─────────────────────────────────┘
                            │
                    ┌───────▼───────┐
                    │  qmd-bridge   │
                    └───────────────┘
                            │
                    ┌───────▼───────┐
                    │      QMD       │
                    │  (检索引擎)    │
                    └───────────────┘
```

---

## 2. 接口设计

### 2.1 核心 Trait
```rust
/// QMD 检索桥接 trait
pub trait QmdBridge {
    /// 同步数据到 QMD
    fn sync_to_qmd(&mut self, data: &QmdData) -> SqlResult<()>;

    /// 从 QMD 检索
    fn search_from_qmd(&self, query: &QmdQuery) -> SqlResult<QmdResult>;

    /// 混合检索 (向量 + 图谱 + 全文)
    fn hybrid_search(&self, query: &HybridQuery) -> SqlResult<HybridResult>;

    /// 同步状态检查
    fn sync_status(&self) -> SqlResult<SyncStatus>;
}
```

### 2.2 数据类型
```rust
/// QMD 数据格式
pub struct QmdData {
    pub id: String,
    pub data_type: QmdDataType,  // Vector, Graph, Document
    pub content: Vec<f32>,       // 向量数据
    pub metadata: HashMap<String, String>,
    pub timestamp: i64,
}

/// QMD 查询格式
pub struct QmdQuery {
    pub query_type: QueryType,    // Knn, BFS, DFS, Hybrid
    pub vector: Option<Vec<f32>>,
    pub graph_pattern: Option<GraphPattern>,
    pub filters: Vec<Filter>,
    pub limit: usize,
}

/// 混合检索结果
pub struct HybridResult {
    pub vector_results: Vec<SearchResult>,
    pub graph_results: Vec<SearchResult>,
    pub reranked_results: Vec<SearchResult>,
    pub scores: Vec<f32>,
}
```

---

## 3. 功能模块

### 3.1 数据同步模块

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 实时同步 | 数据变更实时同步到 QMD | P0 |
| 批量同步 | 支持全量/增量批量同步 | P0 |
| 冲突处理 | 同步冲突检测与解决 | P1 |
| 断点续传 | 网络中断后从断点恢复 | P2 |

### 3.2 检索模块

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 向量检索 | ANN 检索 (HNSW/IVF-PQ) | P0 |
| 图谱检索 | GMP 模式匹配检索 | P0 |
| 全文检索 | 关键词/语义检索 | P1 |
| 混合检索 | 向量+图谱+全文融合 | P0 |
| 重排序 | 基于交叉编码器的重排序 | P1 |

### 3.3 元数据管理

| 功能 | 描述 | 优先级 |
|------|------|--------|
| Schema 映射 | SQLRustGo Schema → QMD Schema | P0 |
| 索引管理 | QMD 索引创建/删除/更新 | P0 |
| 统计信息 | QMD 索引统计信息收集 | P1 |

---

## 4. SQL 接口扩展

### 4.1 检索语法
```sql
-- 纯向量检索
SELECT * FROM users 
WHERE VECTOR_SEARCH(embedding, query_vector, 'hnsw', limit => 10);

-- 图谱检索
SELECT * FROM users 
WHERE GRAPH_MATCH(pattern, 'MATCH (a)-[r]->(b) WHERE a.age > 30');

-- 混合检索
SELECT * FROM users 
WHERE HYBRID_SEARCH(
    embedding => query_vector,
    graph_pattern => 'MATCH (a)-[r]->(b)',
    text_query => '关键词',
    weights => [0.4, 0.3, 0.3],
    limit => 10
);
```

### 4.2 同步语法
```sql
-- 同步到 QMD
SYNC TO QMD FROM users WHERE condition;

-- 检查同步状态
SELECT * FROM qmd_sync_status();
```

---

## 5. 实现计划

### Phase 1: 基础同步 (2周)
- [ ] 定义 QmdBridge trait
- [ ] 实现基础数据同步
- [ ] 向量检索通道
- [ ] 单元测试

### Phase 2: 图谱集成 (2周)
- [ ] GMP → QMD 图谱同步
- [ ] 图谱检索通道
- [ ] 混合检索基础
- [ ] 集成测试

### Phase 3: 高级特性 (2周)
- [ ] 全文检索集成
- [ ] 重排序机制
- [ ] 性能优化
- [ ] 文档完善

---

## 6. 测试计划

### 6.1 单元测试
- QmdBridge trait 实现测试
- 数据转换测试
- 错误处理测试

### 6.2 集成测试
- QMD 服务集成测试
- 混合检索测试
- 性能基准测试

### 6.3 E2E 测试
```bash
# 同步测试
cargo test --test qmd_sync_e2e

# 检索测试
cargo test --test qmd_search_e2e

# 混合检索测试
cargo test --test qmd_hybrid_e2e
```

---

## 7. 性能目标

| 指标 | 目标 | 测量方法 |
|------|------|----------|
| 同步延迟 | < 100ms | p99 |
| 检索延迟 | < 50ms | p99 (k=10) |
| 混合检索延迟 | < 100ms | p99 |
| 吞吐量 | > 1000 QPS | 并发测试 |

---

## 8. 风险与依赖

### 8.1 风险
| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| QMD 版本兼容 | 高 | 抽象接口隔离 |
| 同步一致性 | 中 | 最终一致性+补偿 |
| 性能瓶颈 | 中 | 异步+批处理 |

### 8.2 依赖
- QMD 服务 (外部依赖)
- SQLRustGo vector 模块
- SQLRustGo graph/gmp 模块

---

## 9. 参考

- QMD 官方文档: TBD
- 现有 vector search 实现: `crates/vector/`
- GMP 模块: `crates/graph/` + `crates/gmp/`
