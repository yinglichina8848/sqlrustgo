# 架构决策记录 (ADR)

> **版本**: v2.7.0
> **最后更新**: 2026-04-22

---

## 概述

本文档记录 SQLRustGo v2.7.0 开发过程中的关键架构决策。

---

## ADR-001: 使用 Rust 作为核心开发语言

**状态**: 已批准

**背景**: 选择数据库引擎的开发语言

**决策**: 使用 Rust 作为核心开发语言

**理由**:
- 内存安全，无需 GC
- 高性能，接近 C/C++
- 丰富的生态 (tokio, serde 等)
- 活跃的社区

**后果**:
- 需要处理所有权和生命周期
- 编译时间长
- 生态系统不如 Java 成熟

---

## ADR-002: 分层架构设计

**状态**: 已批准

**背景**: 确定系统架构

**决策**: 采用分层架构: Parser → Planner → Executor → Storage

**分层说明**:

```
┌─────────────────────────────────────┐
│            Network Layer            │
├─────────────────────────────────────┤
│            Parser Layer             │
├─────────────────────────────────────┤
│            Planner Layer            │
├─────────────────────────────────────┤
│           Executor Layer            │
├─────────────────────────────────────┤
│           Storage Layer             │
└─────────────────────────────────────┘
```

**理由**:
- 清晰的职责分离
- 便于独立测试
- 模块可替换

**后果**:
- 层间通信开销
- 需要定义标准接口

---

## ADR-003: 基于成本的查询优化器

**状态**: 已批准

**背景**: 查询优化策略选择

**决策**: 实现基于成本的查询优化器 (CBO)

**理由**:
- 比规则优化更优
- 可适应不同数据分布
- 业界标准实践

**实现**:
- 统计信息收集
- 代价模型
- 动态规划连接顺序

---

## ADR-004: MVCC 事务隔离

**状态**: 已批准

**背景**: 事务隔离实现

**决策**: 实现 MVCC + SSI 隔离级别

**理由**:
- 读不阻塞写
- 写不阻塞读
- 可序列化隔离

**实现**:
- 快照管理
- 版本链
- 冲突检测

---

## ADR-005: Buffer Pool 内存管理

**状态**: 已批准

**背景**: 存储引擎内存管理

**决策**: 实现 Buffer Pool 管理

**理由**:
- 减少磁盘 I/O
- 提高查询性能
- 内存控制

**实现**:
- LRU 淘汰策略
- 页面预取
- 脏页刷新

---

## ADR-006: MySQL 协议兼容

**状态**: 已批准

**背景**: 网络协议选择

**决策**: 兼容 MySQL 协议

**理由**:
- 现有客户端兼容
- 降低用户迁移成本
- 丰富的生态工具

**实现**:
- MySQL C/S 协议
- 连接 handshake
- SQL 命令解析

---

## ADR-007: 向量化执行

**状态**: 已批准

**背景**: 执行引擎优化

**决策**: 实现向量化执行引擎

**理由**:
- SIMD 加速
- 减少分支预测
- 批处理减少开销

**实现**:
- 列式数据组织
- 批量运算符
- SIMD 指令

---

## ADR-008: 预写日志 (WAL)

**状态**: 已批准

**背景**: 持久化策略

**决策**: 实现 WAL 机制

**理由**:
- 崩溃恢复
- 持久性保证
- 写入优化

**实现**:
- 顺序写入
- 检查点
- 崩溃恢复

---

## ADR-009: 索引结构

**状态**: 已批准

**背景**: 索引实现

**决策**: 实现 B+ Tree 作为主索引

**理由**:
- 范围查询高效
- 磁盘友好
- 业界标准

**同时支持**:
- Hash Index (等值查询)
- Vector Index (向量搜索)

---

## ADR-010: 连接池实现

**状态**: 已批准

**背景**: 并发连接管理

**决策**: 实现内置连接池

**理由**:
- 减少连接开销
- 连接复用
- 资源控制

**实现**:
- 连接队列
- 超时管理
- 健康检查

---

## ADR-011: qmd-bridge 双向数据桥梁 (T-04)

**状态**: 已批准

**背景**: 实现 SQLRustGo 与 QMD (Query Memory Database) 的双向数据桥梁，支持混合检索场景

**决策**: 实现 qmd-bridge 模块作为 SQLRustGo 与 QMD 之间的可插拔桥梁

**架构**:

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

**理由**:
- 支持向量检索、图谱检索、全文检索的融合
- 数据同步到 QMD 实现高性能混合检索
- 可插拔设计，支持降级到纯内部检索
- 统一的检索 API 接口

**核心 Trait**:
```rust
pub trait QmdBridge {
    fn sync_to_qmd(&mut self, data: &QmdData) -> SqlResult<()>;
    fn search_from_qmd(&self, query: &QmdQuery) -> SqlResult<QmdResult>;
    fn hybrid_search(&self, query: &HybridQuery) -> SqlResult<HybridResult>;
    fn sync_status(&self) -> SqlResult<SyncStatus>;
}
```

**SQL 接口扩展**:
```sql
-- 混合检索
SELECT * FROM users 
WHERE HYBRID_SEARCH(
    embedding => query_vector,
    graph_pattern => 'MATCH (a)-[r]->(b)',
    text_query => '关键词',
    weights => [0.4, 0.3, 0.3],
    limit => 10
);

-- 同步到 QMD
SYNC TO QMD FROM users WHERE condition;
```

**后果**:
- 新增外部依赖 QMD 服务
- 需要维护数据一致性
- 同步延迟需要监控

---

## ADR-012: 统一检索 API (T-05)

**状态**: 已批准

**背景**: 提供统一的检索 API，屏蔽底层存储引擎差异，简化用户使用

**决策**: 实现 Unified Retrieval Layer，提供统一的检索接口

**架构**:

```
┌─────────────────────────────────────────────────────────────┐
│                     Unified Retrieval API                    │
├─────────────────────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐  │
│  │ Vector Search  │  │  Graph Search  │  │ Text Search  │  │
│  │   (HNSW)      │  │    (GMP)       │  │   (FTS)      │  │
│  └────────────────┘  └────────────────┘  └──────────────┘  │
│                            │                                │
│                   ┌────────▼────────┐                       │
│                   │  Query Planner  │                       │
│                   └────────┬────────┘                       │
│                            │                                │
│                   ┌────────▼────────┐                       │
│                   │ Result Merger  │                       │
│                   └─────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

**理由**:
- 统一接口简化开发
- 便于扩展新的检索类型
- 支持混合检索的灵活配置
- 解耦上层应用与底层实现

**核心 API**:
```rust
pub trait UnifiedSearch {
    fn search(&self, request: SearchRequest) -> SqlResult<SearchResponse>;
    fn search_with_rerank(&self, request: SearchRequest, rerank_config: RerankConfig) -> SqlResult<SearchResponse>;
}
```

**SearchRequest 结构**:
```rust
pub struct SearchRequest {
    pub query: Query,
    pub retrieval_types: Vec<RetrievalType>,  // Vector, Graph, Text
    pub weights: Option<Vec<f32>>,
    pub limit: usize,
    pub filters: Vec<Filter>,
}
```

**后果**:
- 新增检索层抽象
- 性能略有开销 (可忽略)
- 需要维护各检索引擎的实现

---

## ADR-013: 混合检索重排序 (T-06)

**状态**: 已批准

**背景**: 混合检索 (向量+图谱+全文) 需要对多路召回结果进行融合排序，提高召回质量

**决策**: 实现混合 rerank 机制，支持基于交叉编码器的重排序

**重排序流程**:

```
┌──────────────────────────────────────────────────────────────┐
│                        混合检索流程                           │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│   Query ──┬── Vector Index (HNSW) ──────┐                   │
│           │                             │                    │
│           ├── Graph Search (GMP) ───────┼──► Result Merger  │
│           │                             │         │          │
│           └── Full-Text Search ─────────┘         │          │
│                                                     ▼          │
│                                            ┌──────────────┐   │
│                                            │ Cross-Encoder │   │
│                                            │   Reranker   │   │
│                                            └──────┬───────┘   │
│                                                   ▼            │
│                                            Final Rankings     │
└──────────────────────────────────────────────────────────────┘
```

**理由**:
- 多路召回结果质量参差不齐，需要统一排序
- 交叉编码器可利用 Query-Document 交互特征
- 提高 Top-K 召回的准确性
- 支持可配置的重排序策略

**实现**:
```rust
/// 重排序配置
pub struct RerankConfig {
    pub model: RerankModel,        // CrossEncoder, BM25, etc.
    pub top_k: usize,              // 重排序候选数量
    pub weights: Vec<f32>,          // 各检索类型权重
}

/// 混合检索结果
pub struct HybridResult {
    pub vector_results: Vec<SearchResult>,
    pub graph_results: Vec<SearchResult>,
    pub reranked_results: Vec<SearchResult>,
    pub scores: Vec<f32>,
}
```

**重排序策略**:
| 策略 | 适用场景 | 性能 |
|------|----------|------|
| BM25 Rerank | 轻量级重排 | 最快 |
| Cross-Encoder | 高精度场景 | 中等 |
| ColBERT | 多向量场景 | 较慢 |
| Learning to Rank | 个性化排序 | 最慢 |

**后果**:
- 增加重排序延迟
- 需要维护重排序模型
- 候选集大小的选择影响效果和性能

---

## ADR-014: 检索结果融合策略

**状态**: 已批准

**背景**: 多路检索召回的结果需要融合

**决策**: 实现多种融合策略，支持可配置

**融合策略**:

| 策略 | 描述 | 适用场景 |
|------|------|----------|
| RRF (Reciprocal Rank Fusion) | 基于排名倒数的融合 | 通用 |
| Score Based | 基于分数加权 | 分数可比较 |
| Compressive Fusion | 基于压缩感知 | 高度稀疏 |
| Learning to Rank | 机器学习融合 | 有标注数据 |

**RRF 公式**:
```
RRF_score(d) = Σ (1 / (k + rank(d))) for each retrieval type
```
其中 k=60 (通常推荐值)

---

## 决策模板

```markdown
## ADR-XXX: 决策标题

**状态**: [已批准/待定/已拒绝]

**背景**: 决策背景

**决策**: 具体决策

**理由**: 决策理由

**后果**: 预期后果
```

---

## 相关文档

- [架构设计](oo/architecture/ARCHITECTURE_V2.7.md)
- [qmd-bridge 设计文档](./qmd-bridge-design.md)
- [性能分析](oo/reports/PERFORMANCE_ANALYSIS.md)
- [API 文档](./API_DOCUMENTATION.md)

---

*本文档由 SQLRustGo Team 维护*
