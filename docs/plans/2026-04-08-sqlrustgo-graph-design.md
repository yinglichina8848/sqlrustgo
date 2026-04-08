# SQLRustGo Graph Engine 设计文档

**版本**: v1.0  
**日期**: 2026-04-08  
**状态**: 架构基线 (Architecture Baseline)  
**关联 Issue**: #1078

---

## 一、Motivation（背景与动机）

### 1.1 为什么需要 Graph Engine

SQLRustGo 的目标是成为 **AI Native Database**，支持多模态查询：

```
SQL Engine      → 结构化数据查询
Vector Engine   → 语义检索 (#1077 RAG)
Graph Engine    → 关系推理 ← 本文档目标
Document Engine → 文档处理
```

当前 SQL + Vector 引擎已就绪，但缺乏**结构化关系推理能力**。

### 1.2 GMP 场景的核心需求

GMP（药品生产质量管理）系统的核心不是普通知识检索，而是**可追溯关系推理**。

典型查询不是：
```
SOP 是什么？
```

而是：
```
这个批次为什么不合规？
```

需要沿关系链推理：
```
Batch → Step → Device → Calibration → Deviation → CAPA → Regulation
```

这是典型 Graph Query，不是普通 SQL 能优雅完成的。

### 1.3 目标定位

```
RAG (semantic retrieval)     ← 已完成 #1077
Graph (structural reasoning)  ← 本文档目标
SQL  (transactional)         ← 已就绪
```

Graph Engine 是 AI Agent 查询数据库的**核心推理引擎**。

---

## 二、Goals & Non-Goals（目标与边界）

### 2.1 Goals（目标）

- [ ] Property Graph 数据模型（节点、边、属性）
- [ ] 双向遍历（incoming/outgoing neighbors）
- [ ] 标签索引（Label Index）支持 O(1) 标签查找
- [ ] BFS/DFS 遍历算法
- [ ] SQL Planner 集成就绪 API
- [ ] 与 StorageEngine 解耦的适配器层
- [ ] GMP 追溯路径查询

### 2.2 Non-Goals（不做）

- [ ] 分布式图存储（当前单节点）
- [ ] Cypher/Gremlin 兼容性
- [ ] RDF Triple Store
- [ ] 属性索引（Phase-2）
- [ ] 最短路径 Dijkstra（Phase-2）
- [ ] 时间维度遍历（Temporal Graph，Phase-2）

### 2.3 设计原则

```
Graph Engine 不拥有持久化能力。
Graph Engine 通过适配器层使用 StorageEngine。
```

这是数据库引擎边界定义的核心原则。

---

## 三、Architecture Overview（架构概览）

### 3.1 SQLRustGo 多引擎架构

```
sqlrustgo-core
sqlrustgo-storage     ← 共享持久化层
sqlrustgo-index
sqlrustgo-vector
sqlrustgo-rag
sqlrustgo-graph       ← 新增
sqlrustgo-executor
sqlrustgo-planner
sqlrustgo-parser
```

### 3.2 Graph Engine 内部结构

```
sqlrustgo-graph
├── Cargo.toml
└── src
    ├── lib.rs                    # 公共 API 导出

    ├── model/
    │   ├── node.rs               # Node/NodeId 定义
    │   ├── edge.rs               # Edge/EdgeId 定义
    │   └── property.rs           # PropertyMap 定义

    ├── registry/
    │   └── label_registry.rs      # LabelRegistry/LabelId

    ├── storage/
    │   ├── node_store.rs         # 节点存储
    │   ├── edge_store.rs         # 边存储
    │   └── storage_adapter.rs    # StorageEngine 适配器

    ├── index/
    │   ├── adjacency_index.rs    # 邻接表索引
    │   └── label_index.rs        # 标签索引

    ├── algorithms/
    │   ├── bfs.rs                # BFS 遍历
    │   └── dfs.rs                # DFS 遍历

    └── traversal/
        └── walker.rs             # 遍历原语
```

### 3.3 模块依赖关系

```
GraphStore API
      ↑
      │
Traversal ←→ LabelRegistry ←→ AdjacencyIndex
      ↑              ↑              ↑
      │              │              │
      └──────────────┴──────────────┘
                        ↓
               StorageAdapter
                        ↓
               StorageEngine
```

---

## 四、Property Graph Data Model（属性图数据模型）

### 4.1 核心结构

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EdgeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LabelId(pub u32);

pub struct Node {
    pub id: NodeId,
    pub label: LabelId,
    pub properties: PropertyMap,
}

pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub label: LabelId,
    pub properties: PropertyMap,
}

pub type PropertyMap = HashMap<String, Value>;
```

### 4.2 GMP 核心标签

Phase-1 预定义核心标签：

```rust
const GMP_CORE_LABELS: &[&str] = &[
    "Batch",      // 批次
    "Device",     // 设备
    "SOP",        // 标准操作规程
    "Step",       // 生产步骤
    "Deviation",  // 偏差
    "CAPA",       // 纠正预防措施
    "Regulation", // 法规
    "Material",   // 物料
    "Operator",    // 操作员
    "QA",         // 质量保证
];
```

### 4.3 GMP 典型边类型

```rust
const GMP_EDGE_TYPES: &[&str] = &[
    "produced_by",      // 批次 → 操作员
    "produced_on",      // 批次 → 设备
    "uses",             // 批次 → 物料
    "defined_by",       // 步骤 → SOP
    "calibrated_by",    // 设备 → 校准记录
    "related_to",       // 偏差 → 批次
    "caused_by",        // 偏差 → 设备
    "mitigated_by",     // 偏差 → CAPA
    "resolves",         // CAPA → 偏差
    "references",       // SOP → 法规
    "verified_by",     // 校准 → QA
];
```

---

## 五、ID System Design（ID 系统设计）

### 5.1 ID 类型定义

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EdgeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LabelId(pub u32);
```

### 5.2 ID 分配策略

```
LabelId 分配：
0        保留
1..255   核心标签（编译时确定）
256+     动态标签（运行时创建）
```

### 5.3 设计原则

| 原则 | 说明 |
|------|------|
| 内部 ID = u64/u32 | 紧凑、高效、cache-friendly |
| 外部 ID = UUID | 作为 node 属性，支持跨系统同步 |
| Snowflake 兼容 | 未来可扩展成分布式 ID |

---

## 六、Label System Design（标签系统设计）

### 6.1 LabelRegistry

```rust
pub struct LabelRegistry {
    name_to_id: HashMap<String, LabelId>,
    id_to_name: Vec<String>,
    next_dynamic_id: LabelId,
}

impl LabelRegistry {
    /// 获取标签 ID（不存在返回 None）
    pub fn get(&self, name: &str) -> Option<LabelId>;

    /// 获取或创建标签
    pub fn get_or_create(&mut self, name: &str) -> LabelId;

    /// 根据 ID 解析标签名
    pub fn resolve(&self, id: LabelId) -> &str;

    /// 引导核心标签（启动时调用）
    pub fn bootstrap_core_labels(&mut self, labels: &[&str]);
}
```

### 6.2 标签 bootstrap

```rust
impl LabelRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
            next_dynamic_id: LabelId(256), // 动态标签从 256 开始
        };
        registry.bootstrap_core_labels(GMP_CORE_LABELS);
        registry
    }
}
```

### 6.3 LabelIndex

```rust
pub struct LabelIndex {
    index: HashMap<LabelId, Vec<NodeId>>,
}

impl LabelIndex {
    pub fn insert(&mut self, label: LabelId, node: NodeId);
    pub fn remove(&mut self, label: LabelId, node: NodeId);
    pub fn get_nodes(&self, label: LabelId) -> &[NodeId];
}
```

时间复杂度：O(1) 标签查找

---

## 七、Storage Layout（存储布局）

### 7.1 NodeStore

```rust
pub struct NodeStore {
    nodes: HashMap<NodeId, Node>,
}

impl NodeStore {
    pub fn insert(&mut self, node: Node) -> NodeId;
    pub fn get(&self, id: NodeId) -> Option<&Node>;
    pub fn remove(&mut self, id: NodeId) -> Option<Node>;
}
```

### 7.2 EdgeStore

```rust
pub struct EdgeStore {
    edges: HashMap<EdgeId, Edge>,
}

impl EdgeStore {
    pub fn insert(&mut self, edge: Edge) -> EdgeId;
    pub fn get(&self, id: EdgeId) -> Option<&Edge>;
    pub fn remove(&mut self, id: EdgeId) -> Option<Edge>;
}
```

### 7.3 AdjacencyIndex（核心索引）

```rust
pub struct AdjacencyIndex {
    /// outgoing[node] = [edge_ids]
    outgoing: HashMap<NodeId, Vec<EdgeId>>,
    /// incoming[node] = [edge_ids]
    incoming: HashMap<NodeId, Vec<EdgeId>>,
}

impl AdjacencyIndex {
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, edge_id: EdgeId);
    pub fn remove_edge(&mut self, from: NodeId, edge_id: EdgeId);
    pub fn outgoing_edges(&self, node: NodeId) -> &[EdgeId];
    pub fn incoming_edges(&self, node: NodeId) -> &[EdgeId];
}
```

### 7.4 StorageAdapter（StorageEngine 适配器）

```rust
/// Graph 节点表结构
const GRAPH_NODES_TABLE: &str = "graph_nodes";
const GRAPH_EDGES_TABLE: &str = "graph_edges";

/// 节点表列
const NODE_COLUMNS: &[&str] = &["id", "label", "properties"];
/// 边表列
const EDGE_COLUMNS: &[&str] = &["id", "from_node", "to_node", "label", "properties"];
```

---

## 八、GraphStore Trait（核心 API）

### 8.1 完整 Trait 定义

```rust
/// 图存储引擎核心 trait
pub trait GraphStore {
    // ====================
    // 节点操作
    // ====================

    /// 创建节点
    fn create_node(
        &mut self,
        label: LabelId,
        props: PropertyMap,
    ) -> NodeId;

    /// 获取节点
    fn get_node(&self, id: NodeId) -> Option<&Node>;

    /// 按标签迭代节点（返回 Iterator，支持流式处理）
    fn nodes_by_label(
        &self,
        label: LabelId,
    ) -> Box<dyn Iterator<Item = NodeId> + '_>;

    // ====================
    // 边操作
    // ====================

    /// 创建边
    fn create_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: LabelId,
        props: PropertyMap,
    ) -> EdgeId;

    /// 获取边
    fn get_edge(&self, id: EdgeId) -> Option<&Edge>;

    // ====================
    // 遍历原语
    // ====================

    /// 获取出向邻居（通过边标签过滤）
    fn neighbors_by_edge_label(
        &self,
        node: NodeId,
        edge_label: LabelId,
    ) -> Box<dyn Iterator<Item = NodeId> + '_>;

    /// 获取所有出向邻居
    fn outgoing_neighbors(
        &self,
        node: NodeId,
    ) -> Box<dyn Iterator<Item = NodeId> + '_>;

    /// 获取所有入向邻居
    fn incoming_neighbors(
        &self,
        node: NodeId,
    ) -> Box<dyn Iterator<Item = NodeId> + '_>;

    // ====================
    // 算法（Phase-1）
    // ====================

    /// 广度优先搜索
    fn bfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId);

    /// 深度优先搜索
    fn dfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId);
}
```

### 8.2 设计理由

| API | 理由 |
|-----|------|
| `Iterator` 返回类型 | 支持 lazy evaluation，避免一次性加载全部到内存 |
| `LabelId` 参数 | 避免运行时字符串查找，planner 优化友好 |
| `visitor` 模式 | 支持 early termination，性能关键 |
| `Box<dyn Iterator>` | 抽象迭代器实现，支持多种存储后端 |

---

## 九、Traversal API（遍历 API）

### 9.1 基础遍历

```rust
/// 出向邻居遍历
fn outgoing_neighbors(&self, node: NodeId) -> Vec<NodeId> {
    let edges = self.adjacency.outgoing_edges(node);
    edges.iter()
        .filter_map(|edge_id| self.edge_store.get(*edge_id))
        .map(|edge| edge.to)
        .collect()
}

/// 入向邻居遍历
fn incoming_neighbors(&self, node: NodeId) -> Vec<NodeId> {
    let edges = self.adjacency.incoming_edges(node);
    edges.iter()
        .filter_map(|edge_id| self.edge_store.get(*edge_id))
        .map(|edge| edge.from)
        .collect()
}
```

### 9.2 带标签过滤的遍历

```rust
/// 按边标签过滤的遍历（GMP 核心查询）
fn neighbors_by_edge_label(
    &self,
    node: NodeId,
    edge_label: LabelId,
) -> Vec<NodeId> {
    let edges = self.adjacency.outgoing_edges(node);
    edges.iter()
        .filter_map(|edge_id| self.edge_store.get(*edge_id))
        .filter(|edge| edge.label == edge_label)
        .map(|edge| edge.to)
        .collect()
}
```

### 9.3 GMP 典型查询示例

```rust
// 查询 1: Batch → Device（批次使用哪些设备）
let devices = graph.neighbors_by_edge_label(batch_id, PRODUCED_ON_LABEL);

// 查询 2: Device → impacted Batch（设备影响哪些批次）
let batches = graph.incoming_neighbors(device_id);

// 查询 3: Batch → SOP → Regulation（追溯合规路径）
fn trace_regulation_path(&self, batch_id: NodeId) -> Vec<NodeId> {
    let mut path = Vec::new();
    self.dfs(batch_id, |node_id| {
        if let Some(node) = self.get_node(node_id) {
            if node.label == REGULATION_LABEL {
                return; // 找到法规节点，停止
            }
            path.push(node_id);
        }
    });
    path
}
```

---

## 十、Adjacency Index（邻接表索引）

### 10.1 双向邻接表

```rust
pub struct AdjacencyIndex {
    outgoing: HashMap<NodeId, Vec<EdgeId>>,
    incoming: HashMap<NodeId, Vec<EdgeId>>,
}
```

### 10.2 操作复杂度

| 操作 | 复杂度 |
|------|--------|
| 添加边 | O(1) |
| 删除边 | O(degree) |
| 获取出向边 | O(1) |
| 获取入向边 | O(1) |

### 10.3 BFS 实现

```rust
pub fn bfs<F>(&self, start: NodeId, mut visitor: F)
where
    F: FnMut(NodeId),
{
    use std::collections::VecDeque;

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(node) = queue.pop_front() {
        visitor(node);

        for edge_id in self.outgoing_edges(node) {
            if let Some(edge) = self.edge_store.get(*edge_id) {
                if !visited.contains(&edge.to) {
                    visited.insert(edge.to);
                    queue.push_back(edge.to);
                }
            }
        }
    }
}
```

---

## 十一、Phase-1 Scope（Phase-1 实现范围）

### 11.1 必须实现

| 模块 | 功能 |
|------|------|
| `model/` | Node, Edge, PropertyMap, NodeId, EdgeId, LabelId |
| `registry/` | LabelRegistry, bootstrap, get_or_create |
| `storage/` | NodeStore, EdgeStore |
| `index/` | AdjacencyIndex, LabelIndex |
| `algorithms/` | BFS, DFS |
| `traversal/` | outgoing_neighbors, incoming_neighbors, neighbors_by_edge_label |
| `lib.rs` | GraphStore trait, InMemoryGraphStore 实现 |

### 11.2 API 清单

```rust
// Phase-1 可用 API
trait GraphStore {
    // 节点
    fn create_node(&mut self, label: LabelId, props: PropertyMap) -> NodeId;
    fn get_node(&self, id: NodeId) -> Option<&Node>;
    fn nodes_by_label(&self, label: LabelId) -> Box<dyn Iterator<Item = NodeId> + '_>;

    // 边
    fn create_edge(&mut self, from: NodeId, to: NodeId, label: LabelId, props: PropertyMap) -> EdgeId;
    fn get_edge(&self, id: EdgeId) -> Option<&Edge>;

    // 遍历
    fn outgoing_neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;
    fn incoming_neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;
    fn neighbors_by_edge_label(&self, node: NodeId, edge_label: LabelId) -> Box<dyn Iterator<Item = NodeId> + '_>;

    // 算法
    fn bfs<F>(&self, start: NodeId, visitor: F) where F: FnMut(NodeId);
    fn dfs<F>(&self, start: NodeId, visitor: F) where F: FnMut(NodeId);
}
```

### 11.3 测试场景

```rust
#[test]
fn test_gmp_batch_traceability() {
    let mut graph = InMemoryGraphStore::new();

    // 创建节点
    let batch = graph.create_node(LABEL_BATCH, prop!{"batch_id": "B-2026-001"});
    let device = graph.create_node(LABEL_DEVICE, prop!{"device_id": "D-001"});
    let calibration = graph.create_node(LABEL_CALIBRATION, prop!{"status": "valid"});

    // 创建边
    graph.create_edge(batch, device, EDGE_PRODUCED_ON, PropertyMap::new());
    graph.create_edge(device, calibration, EDGE_CALIBRATED_BY, PropertyMap::new());

    // 验证：追溯设备影响路径
    let impacted_batches: Vec<NodeId> = graph.incoming_neighbors(device).collect();
    assert!( impacted_batches.contains(&batch));
}
```

---

## 十二、Phase-2 Roadmap（后续规划）

### 12.1 Phase-2 功能

| 功能 | 说明 |
|------|------|
| Dijkstra 最短路径 | 带权重的最短路径算法 |
| Property Index | 属性索引，支持属性过滤查询 |
| Constraint Traversal | 只经过特定类型节点的遍历 |
| StorageAdapter | 持久化到 StorageEngine |

### 12.2 Phase-3 功能

| 功能 | 说明 |
|------|------|
| SQL Integration | SQL planner 支持 GRAPH_MATCH 表达式 |
| Graph + SQL Hybrid | 联合查询优化 |
| Graph Cost Model | 图操作成本估算 |

---

## 十三、SQL Integration Plan（SQL 集成计划）

### 13.1 Planner 集成

```rust
// 未来 SQL Planner 调用
fn plan_graph_match(&self, pattern: GraphPattern) -> PlanNode {
    // 解析标签
    let label_id = self.label_registry.get_or_create(&pattern.label);

    // 使用 LabelIndex 快速查找起始节点
    let start_nodes = self.label_index.get_nodes(label_id);

    // 生成遍历计划
    PlanNode::GraphTraversal {
        start: start_nodes,
        edge_filter: pattern.edge_label,
        depth_limit: pattern.max_depth,
    }
}
```

### 13.2 示例 SQL

```sql
-- 未来支持：查找使用过期设备的所有批次
SELECT b.batch_id, b.production_date
FROM batch b
WHERE EXISTS (
    SELECT 1
    FROM graph_edges e
    JOIN graph_nodes d ON e.to_node = d.id
    WHERE e.from_node = b.node_id
      AND d.label = 'Device'
      AND d.properties->>'calibration_status' = 'expired'
);
```

---

## 十四、RAG / Vector Integration Plan（RAG 集成计划）

### 14.1 Graph + RAG 协同

```
Document (RAG)
    ↓ embeds
EmbeddingChunk
    ↓ linked_to
Entity (Graph)
    ↓ belongs_to
Batch/Device/SOP (Graph)
```

### 14.2 集成点

| 组件 | 集成方式 |
|------|----------|
| RAG | 实体链接：Embedding → Graph Node |
| Vector | 相似实体查找：Vector Search → Graph Traversal |
| SQL | 联合查询：SQL Filter + Graph Path |

---

## 十五、GMP Traceability Examples（GMP 追溯示例）

### 15.1 批次追溯路径

```
Batch(B-2026-001)
    ├── produced_by → Operator(Zhang)
    ├── produced_on → Device(D-001)
    │                   └── calibrated_by → Calibration(C-001)
    │                                       └── verified_by → QA(Wang)
    ├── uses → Material(M-001)
    └── related_to → Deviation(DV-001)
                        └── caused_by → Device(D-002)
                        └── mitigated_by → CAPA(CAPA-001)
                                            └── resolves → Deviation(DV-001)
```

### 15.2 合规查询示例

```rust
// 查询：某批次的所有合规路径
fn trace_compliance_path(&self, batch_id: NodeId) -> Vec<Path> {
    let mut paths = Vec::new();

    // BFS 查找所有到 Regulation 的路径
    self.bfs_with_path(batch_id, |node, path| {
        if let Some(n) = self.get_node(node) {
            if n.label == LABEL_REGULATION {
                paths.push(path.clone());
            }
        }
    });

    paths
}
```

### 15.3 影响范围分析

```rust
// 查询：某设备故障影响的所有批次
fn impacted_batches(&self, device_id: NodeId) -> Vec<NodeId> {
    let mut affected = Vec::new();

    // 反向遍历：找到所有指向该设备的批次
    for batch_id in self.incoming_neighbors(device_id) {
        affected.push(batch_id);
    }

    affected
}
```

---

## 十六、Future Extensions（未来扩展）

### 16.1 Temporal Graph（时间维度遍历）

```rust
// 未来支持：时间有效性检查
struct TemporalEdge {
    edge: Edge,
    valid_from: DateTime,
    valid_to: Option<DateTime>,
}

fn neighbors_at_time(&self, node: NodeId, at: DateTime) -> Vec<NodeId>;
```

### 16.2 Constraint Traversal（约束遍历）

```rust
// 未来支持：只经过特定类型
fn constrained_dfs<F>(
    &self,
    start: NodeId,
    allowed_labels: &[LabelId],
    visitor: F
);
```

### 16.3 分布式扩展

```rust
// 未来：分布式图存储
trait DistributedGraphStore {
    fn partition(&self) -> Vec<Partition>;
    fn remote_neighbors(&self, node: NodeId) -> RemoteIterator;
}
```

---

## 十七、设计决策汇总

| # | 决策项 | 选择 |
|---|--------|------|
| 1 | Crate 结构 | 独立 `sqlrustgo-graph` crate |
| 2 | 图模型 | Property Graph（不是 RDF） |
| 3 | ID 系统 | `NodeId(u64)`, `EdgeId(u64)`, `LabelId(u32)` |
| 4 | 外部 ID | UUID 作为 node 属性，不嵌入 ID |
| 5 | Label 系统 | LabelRegistry + CoreLabelBootstrap + LabelIndex |
| 6 | API 风格 | Iterator + visitor 模式，避免 Vec 全量加载 |
| 7 | 遍历 | 双向（incoming/outgoing）+ edge-label 过滤 |
| 8 | 持久化 | Graph 不拥有存储，通过 StorageAdapter 复用 StorageEngine |
| 9 | Phase-1 | BFS/DFS + 基础遍历，暂无 Dijkstra |
| 10 | SQL 集成 | Planner 接口预留，Phase-2 实现 |

---

## 附录 A：Crate 初始化模板

```
sqlrustgo-graph/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── model/
│   │   ├── mod.rs
│   │   ├── node.rs
│   │   ├── edge.rs
│   │   └── property.rs
│   ├── registry/
│   │   ├── mod.rs
│   │   └── label_registry.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── node_store.rs
│   │   ├── edge_store.rs
│   │   └── storage_adapter.rs
│   ├── index/
│   │   ├── mod.rs
│   │   ├── adjacency_index.rs
│   │   └── label_index.rs
│   ├── algorithms/
│   │   ├── mod.rs
│   │   ├── bfs.rs
│   │   └── dfs.rs
│   └── traversal/
│       ├── mod.rs
│       └── walker.rs
└── tests/
    └── graph_tests.rs
```
