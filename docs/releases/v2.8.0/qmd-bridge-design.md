# qmd-bridge 设计文档 (v2.8.0)

> 版本: `v2.8.0`  
> 基线: 基于 `crates/qmd-bridge/` 实际代码  
> 上一版本: `v2.7.0` — [v2.7.0 设计文档](../v2.7.0/qmd-bridge-design.md)  
> 相关 Issue: T-04 (v2.7.0 引入); v2.8.0 无新增功能变更

---

## 1. 概述

### 1.1 目标

qmd-bridge 是 SQLRustGo 与 QMD (Query Memory Database) 之间的双向数据桥梁 crate。它提供统一的检索层，支持：

- 将 SQLRustGo 的向量/图谱/文档数据同步到 QMD
- 通过 QMD 进行向量检索、图谱检索和混合检索
- 可插拔的桥接 trait 设计，便于测试和切换后端
- 混合检索的加权重排机制

### 1.2 架构位置

```
┌─────────────────────────────────────────────────────────────┐
│                      SQLRustGo Engine                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐   │
│  │ Parser  │  │Planner  │  │Executor │  │ Transaction │   │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────┘   │
│       │            │            │               │            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Unified Retrieval Layer                 │   │
│  │  ┌──────────────┐  ┌───────────────────────────┐   │   │
│  │  │ sqlrustgo-   │  │ sqlrustgo-graph + gmp    │   │   │
│  │  │ vector       │  │ (图谱/图模式匹配)        │   │   │
│  │  └──────────────┘  └───────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
│                           │                                  │
└───────────────────────────│──────────────────────────────────┘
                            │
                    ┌───────▼───────┐
                    │  qmd-bridge   │  ← 本 crate
                    │  (桥梁 +      │
                    │   混合检索)    │
                    └───────────────┘
                            │
                    ┌───────▼───────┐
                    │      QMD       │
                    │  (外部检索引擎) │
                    └───────────────┘
```

### 1.3 模块结构

```
crates/qmd-bridge/
├── Cargo.toml        # 依赖声明 (sqlrustgo-types, vector, graph)
├── src/
│   ├── lib.rs        # 模块导出 + 公开 API
│   ├── bridge.rs     # QmdBridge trait + 内存实现 QmdBridgeImpl
│   ├── types.rs      # 数据类型定义 (QmdData, QmdQuery, SearchResult, SyncStatus...)
│   ├── config.rs     # QmdConfig 连接配置
│   ├── sync.rs       # SyncManager 同步状态管理
│   ├── hybrid.rs     # HybridSearcher 混合检索 + 重排
│   └── error.rs      # QmdBridgeError 错误类型
```

### 1.4 依赖关系

| 依赖 | 用途 |
|------|------|
| `sqlrustgo-types` | 公共类型 (SqlResult 等) |
| `sqlrustgo-vector` | 向量数据对接 |
| `sqlrustgo-graph` | 图谱数据对接 |
| `serde` / `serde_json` | 数据序列化 |
| `tokio` | 异步运行时 |
| `thiserror` | 错误类型派生 |
| `tracing` / `tracing-subscriber` | 日志跟踪 |

---

## 2. 核心 Trait 设计

### 2.1 QmdBridge Trait

定义在 `bridge.rs`，是所有 QMD 桥接操作的抽象接口：

```rust
pub trait QmdBridge: Send + Sync {
    /// 同步单条数据到 QMD
    fn sync_to_qmd(&self, data: &QmdData) -> QmdResult<()>;

    /// 批量同步数据到 QMD
    fn sync_batch_to_qmd(&self, data: &[QmdData]) -> QmdResult<()>;

    /// 从 QMD 检索
    fn search_from_qmd(&self, query: &QmdQuery) -> QmdResult<Vec<SearchResult>>;

    /// 混合检索 (向量 + 图谱 + 文本)
    fn hybrid_search(&self, query: &HybridQuery) -> QmdResult<HybridResult>;

    /// 获取同步状态
    fn sync_status(&self) -> QmdResult<SyncStatus>;
}
```

**设计要点**:
- 使用 `Send + Sync` 保证线程安全
- 所有方法返回 `QmdResult<T>`，统一错误处理
- 与 v2.7.0 规划相比，实际增加了 `sync_batch_to_qmd` 批量接口

### 2.2 QmdBridgeImpl — 内存实现

`QmdBridgeImpl` 是 trait 的内存实现，使用 `std::sync::Mutex<Vec<QmdData>>` 作为存储后端，主要用于：

- **单元测试**: 无需外部 QMD 服务即可验证桥接逻辑
- **原型验证**: 在无 QMD 环境运行时提供占位实现
- **集成过渡**: 作为正式 QMD 客户端实现的模板

```rust
pub struct QmdBridgeImpl {
    config: QmdConfig,
    storage: std::sync::Mutex<Vec<QmdData>>,
}
```

**搜索实现**:
- 基于元数据过滤 (支持 `Eq` 和 `Contains` 运算符)
- 向量检索使用余弦相似度 (`cosine_similarity` 函数)
- 结果按分数降序排列并截取 `limit` 条
- 图谱/文本结果当前返回空集合 (占位)

---

## 3. 数据类型

### 3.1 QmdDataType — 数据类型枚举

```rust
pub enum QmdDataType {
    Vector,    // 向量数据
    Graph,     // 图谱数据
    Document,  // 文档/文本数据
    Mixed,     // 混合类型
}
```

### 3.2 QmdData — 桥接数据格式

```rust
pub struct QmdData {
    pub id: String,
    pub data_type: QmdDataType,
    pub vector: Option<Vec<f32>>,       // 向量
    pub graph: Option<GraphData>,       // 图谱结构
    pub text: Option<String>,           // 文本内容
    pub metadata: HashMap<String, String>,  // 元数据
    pub timestamp: i64,                 // 时间戳
}
```

提供便捷构造方法：
- `QmdData::new_vector(id, vector)` — 创建向量数据
- `QmdData::new_document(id, text)` — 创建文档数据

### 3.3 GraphData — 图谱结构

```rust
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, String>,
}

pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub properties: HashMap<String, String>,
}
```

### 3.4 QueryType — 查询类型

```rust
pub enum QueryType {
    Knn,     // K 近邻搜索
    Bfs,     // 广度优先搜索
    Dfs,     // 深度优先搜索
    Range,   // 范围搜索
    Hybrid,  // 混合搜索
}
```

### 3.5 QmdQuery — 查询参数

```rust
pub struct QmdQuery {
    pub query_type: QueryType,
    pub vector: Option<Vec<f32>>,       // 查询向量
    pub graph_pattern: Option<String>,  // 图谱模式
    pub text: Option<String>,           // 文本查询
    pub filters: Vec<Filter>,           // 过滤条件
    pub limit: usize,                   // 最大结果数
    pub threshold: Option<f32>,         // 距离阈值
}
```

### 3.6 Filter — 过滤条件

```rust
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,  // Eq, Ne, Gt, Gte, Lt, Lte, In, Contains
    pub value: String,
}
```

**当前实现状态**: `QmdBridgeImpl` 仅支持 `Eq` 和 `Contains` 两种运算符，其他运算符为预留。

### 3.7 SearchResult / SyncStatus

```rust
pub struct SearchResult {
    pub id: String,
    pub score: f32,         // 相似度分数
    pub data: QmdData,      // 结果数据
}

pub struct SyncStatus {
    pub last_sync: i64,
    pub items_synced: u64,
    pub state: SyncState,   // Idle | Syncing | Completed | Failed
    pub error: Option<String>,
}
```

---

## 4. 配置

### 4.1 QmdConfig

```rust
pub struct QmdConfig {
    pub server_addr: String,    // 默认 "127.0.0.1"
    pub server_port: u16,       // 默认 8080
    pub timeout_secs: u64,      // 默认 30
    pub batch_size: usize,      // 默认 1000
    pub compression: bool,      // 默认 false
    pub retry_attempts: u32,    // 默认 3
}
```

提供 `server_url()` 方法生成完整 URL: `http://{addr}:{port}`

---

## 5. 同步管理

### 5.1 SyncManager

`SyncManager` 独立管理数据同步的生命周期状态，与 `QmdBridgeImpl` 中的实际存储操作解耦。

**状态机**:

```
Idle ──start_sync()──> Syncing ──complete_sync()──> Completed
                              ──fail_sync()──────> Failed
                              ──reset()──────────> Idle
```

**关键方法**:

| 方法 | 功能 |
|------|------|
| `new(config)` | 创建管理器，初始状态为 Idle |
| `status()` | 返回当前 SyncStatus |
| `start_sync()` | 启动同步 (若已在 Syncing 则返回错误) |
| `complete_sync(n)` | 标记同步完成，记录同步条目数 |
| `fail_sync(err)` | 标记同步失败，记录错误信息 |
| `reset()` | 重置为 Idle 状态 |
| `is_sync_needed(interval)` | 检查是否超过指定时间间隔需要重新同步 |

---

## 6. 混合检索

### 6.1 HybridQuery

```rust
pub struct HybridQuery {
    pub vector: Option<Vec<f32>>,       // 向量查询
    pub graph_pattern: Option<String>,  // 图谱模式
    pub text_query: Option<String>,     // 文本查询
    pub limit: usize,                   // 结果数上限
    pub filters: Vec<Filter>,           // 过滤条件
}
```

提供 builder 风格构造方法：
- `HybridQuery::with_vector(vec, limit)` — 从向量开始构建
- `.with_graph_pattern(pattern)` — 添加图谱模式
- `.with_text_query(text)` — 添加文本查询

### 6.2 HybridSearchConfig — 权重配置

```rust
pub struct HybridSearchConfig {
    pub vector_weight: f32,    // 默认 0.4
    pub graph_weight: f32,     // 默认 0.3
    pub text_weight: f32,      // 默认 0.3
    pub limit: usize,          // 默认 10
    pub enable_rerank: bool,   // 默认 true
}
```

### 6.3 HybridSearcher — 重排引擎

`HybridSearcher` 实现三类结果的重排融合算法：

```
算法:
1. 向量结果 → combined(item, score = vector_score * vector_weight)
2. 图谱结果 → 若 ID 已存在则累加 score += graph_score * graph_weight
             否则新建条目
3. 文本结果 → 若 ID 已存在则累加 score += text_score * text_weight
             否则新建条目
4. 按 combined score 降序排列
5. 截取 limit 条
```

### 6.4 HybridResult

```rust
pub struct HybridResult {
    pub results: Vec<HybridSearchResultItem>,  // 重排后的最终结果
    pub vector_results: Vec<SearchResult>,     // 原始向量结果
    pub graph_results: Vec<SearchResult>,      // 原始图谱结果
    pub text_results: Vec<SearchResult>,       // 原始文本结果
}

pub struct HybridSearchResultItem {
    pub id: String,
    pub score: f32,                 // 加权组合分数
    pub vector_score: Option<f32>,  // 向量分数组件
    pub graph_score: Option<f32>,   // 图谱分数组件
    pub text_score: Option<f32>,    // 文本分数组件
    pub data: QmdData,
}
```

---

## 7. 错误处理

### 7.1 QmdBridgeError

```rust
pub enum QmdBridgeError {
    Connection(String),          // 连接错误
    Sync(String),                // 同步错误
    Search(String),              // 搜索错误
    Serialization(serde_json::Error),  // 序列化错误
    NotAvailable(String),        // QMD 不可用
    InvalidConfig(String),       // 配置无效
    Timeout(String),             // 超时
    DimensionMismatch { expected: usize, actual: usize },  // 向量维度不匹配
}
```

定义 `QmdResult<T>` 为 `Result<T, QmdBridgeError>` 的别名。

---

## 8. 与 v2.7.0 的对比

| 维度 | v2.7.0 规划 | v2.8.0 实际代码 | 差异 |
|------|-------------|-----------------|------|
| QmdBridge trait | sync + search + hybrid + status | 同上 + `sync_batch_to_qmd` | 新增批量同步 |
| 实现方式 | 规划中 | `QmdBridgeImpl` (内存实现) | 实际已有实现 |
| 数据类型 | `content: Vec<f32>` | `vector: Option<Vec<f32>>` + GraphData + text | 更丰富 |
| 过滤 | 规划中 | `Filter` + `FilterOperator` (8种) | 已实现但仅 Eq/Contains 有效 |
| 混合检索 | 规划 HybridResult | `HybridSearcher` + 加权重排 | 已实现 |
| SyncManager | 规划中 | 完整状态机实现 | 已实现 |
| cosine_similarity | 未提及 | 已实现用于向量比较 | 已实现 |
| 外部 QMD 连接 | QMD 服务集成 | `QmdConfig` + server_url | 配置层已就绪 |
| 集成 | - | server 模块 `qmd_bridge_available` 健康检查 | 部分集成 |
| 测试 | 规划 E2E | 单元测试: bridge(3) + sync(4) + hybrid(2) | 9个单元测试 |

---

## 9. v2.8.0 状态评估

### 9.1 已完成

- QmdBridge trait 定义 (5 个方法)
- QmdBridgeImpl 内存实现 (支持同步/搜索/混合检索/状态查询)
- 完整数据类型体系 (QmdData/GraphData/Filter/SearchResult/SyncStatus)
- SyncManager 同步状态机 (Idle/Syncing/Completed/Failed)
- HybridSearcher 加权重排引擎 (vector/graph/text 三模态融合)
- QmdConfig 连接配置 (server_addr/port/timeout/batch_size/retry)
- QmdBridgeError 错误体系 (8 种错误变体)
- 9 个单元测试覆盖核心路径

### 9.2 规划中 / 未实现 (与实际代码一致)

以下功能在 v2.7.0 设计文档中规划但 **当前代码未实现**：

| 功能 | 状态 | 说明 |
|------|------|------|
| 外部 QMD 服务 HTTP 客户端 | 未实现 | `QmdConfig` 已就绪，但无实际的 HTTP 连接代码 |
| 实时数据变更同步 | 未实现 | SyncManager 仅管理状态，无实际变更监听 |
| 断点续传 | 未实现 | - |
| Schema 映射 | 未实现 | - |
| QMD 索引管理 | 未实现 | - |
| SQL 语法扩展 (SYNC TO QMD, HYBRID_SEARCH) | 未实现 | 规划在解析器/执行器层，qmd-bridge 暂未关联 |
| 图谱检索 (graph_pattern) | 占位 | `GraphData` 类型已定义，但搜索返回空 |
| 文本检索 | 占位 | 类型已定义，搜索通道未实现 |
| 向量 ANN 索引 (HNSW/IVF-PQ) | 未实现 | 当前仅线性扫描余弦相似度 |

### 9.3 已知局限

- **QmdBridgeImpl** 使用 `std::sync::Mutex`，高并发下锁竞争明显
- **FilterOperator** 定义 8 种运算符，当前仅 `Eq`/`Contains` 生效
- **cosine_similarity** 没有针对空向量/零向量的特殊处理外的数值防护
- **无异步 API**: `QmdBridge` trait 方法为同步调用，未使用 `async`
- **server 集成**: 仅在 `hybrid_endpoints.rs` 中作为健康检查字段存在，未实际调用

---

## 10. 测试覆盖

### 10.1 bridge.rs 单元测试

```
test_bridge_sync       → 同步单条数据，验证 items_synced == 1
test_bridge_search     → 同步后检索，验证结果数 == 1
test_cosine_similarity → 完全相同向量 score=1，正交向量 score=0
```

### 10.2 sync.rs 单元测试

```
test_sync_manager_status → 新建 SyncManager 状态为 Idle
test_sync_lifecycle     → start → complete 完整生命周期
test_sync_failure       → start → fail，验证 state=Failed
test_is_sync_needed     → 未同步需同步；刚同步后不需要
```

### 10.3 hybrid.rs 单元测试

```
test_hybrid_rerank      → 向量结果排序优先于图谱结果
test_hybrid_combined_id → 同一 ID 在多个模态出现时分数加权合并
```

### 10.4 运行方式

```bash
cargo test -p sqlrustgo-qmd-bridge --lib
```

---

## 11. 风险与依赖

### 11.1 风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| QMD 服务未就绪 | 混合检索不可用 | QmdBridgeImpl 内存实现可离线测试 |
| 外部 QMD 版本兼容 | API 不匹配 | 抽象 trait 隔离实现细节 |
| 同步一致性 | 数据不一致 | SyncManager 状态机跟踪 |

### 11.2 关键依赖

- `sqlrustgo-types` — 基础类型定义
- `sqlrustgo-vector` — 向量数据源
- `sqlrustgo-graph` — 图谱数据源
- `serde` / `serde_json` — 跨进程数据交换

---

## 12. 参考

- [v2.7.0 qmd-bridge Design](../v2.7.0/qmd-bridge-design.md) — 原始规划文档
- [v2.7.0 Architecture Decisions (ADR-011)](../v2.7.0/ARCHITECTURE_DECISIONS.md) — 架构决策记录
- `crates/qmd-bridge/src/` — 实际代码源
- `crates/server/src/hybrid_endpoints.rs` — server 端集成
