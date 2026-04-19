# Graph 模块设计

**版本**: v2.5.0
**模块**: Graph (图引擎)

---

## 一、What (是什么)

Graph 是 SQLRustGo 的图查询引擎，支持属性图存储和 Cypher 查询语言，提供 BFS/DFS 遍历和多跳查询能力。

## 二、Why (为什么)

- **关系数据处理**: 社交网络、推荐系统等场景
- **高效遍历**: 图结构数据的路径查询
- **Cypher 支持**: 友好的图查询语法
- **统一查询**: 与 SQL 融合

## 三、How (如何实现)

### 3.1 图存储架构

```
┌─────────────────────────────────────────┐
│          DiskGraphStore                   │
├─────────────────────────────────────────┤
│  - 节点存储                             │
│  - 边存储                               │
│  - 属性存储                             │
│  - 索引                                 │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│         GraphMetadata                     │
├─────────────────────────────────────────┤
│  - Schema 管理                         │
│  - 标签索引                            │
│  - 类型系统                             │
└─────────────────────────────────────────┘
```

### 3.2 Cypher 解析

```rust
// Cypher AST
pub enum CypherStatement {
    Match(MatchStmt),
    Create(CreateStmt),
    Merge(MergeStmt),
    Update(UpdateStmt),
    Delete(DeleteStmt),
}

pub struct MatchStmt {
    pub pattern: Pattern,
    pub where: Option<Expr>,
    pub returns: Vec<ReturnItem>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u64>,
}

pub struct Pattern {
    pub nodes: Vec<PatternNode>,
    pub edges: Vec<PatternEdge>,
}
```

### 3.3 图遍历

```rust
pub enum TraversalType {
    BFS,
    DFS,
    MultiHop { min_hops: usize, max_hops: usize },
}

pub struct GraphTraversal {
    traversal_type: TraversalType,
    start_nodes: Vec<NodeId>,
    edge_filters: Vec<EdgeFilter>,
    node_filters: Vec<NodeFilter>,
}

impl GraphTraversal {
    pub fn execute(&self) -> Result<TraversalResult> {
        match self.traversal_type {
            TraversalType::BFS => self.execute_bfs(),
            TraversalType::DFS => self.execute_dfs(),
            TraversalType::MultiHop { .. } => self.execute_multi_hop(),
        }
    }
}
```

### 3.4 存储结构

```rust
// 节点存储
struct NodeStore {
    nodes: BTreeMap<NodeId, Node>,
    label_index: HashMap<Label, Vec<NodeId>>,
    property_index: HashMap<PropertyKey, BTreeMap<Value, Vec<NodeId>>>,
}

// 边存储
struct EdgeStore {
    edges: BTreeMap<EdgeId, Edge>,
    src_index: HashMap<NodeId, Vec<EdgeId>>,
    dst_index: HashMap<NodeId, Vec<EdgeId>>,
    type_index: HashMap<EdgeType, Vec<EdgeId>>,
}
```

## 四、接口设计

### 4.1 公开 API

```rust
impl GraphEngine {
    // 创建图
    pub fn create_graph(&self, name: &str) -> Result<GraphId>;

    // 插入节点
    pub fn insert_node(&self, graph: GraphId, node: Node) -> Result<NodeId>;

    // 插入边
    pub fn insert_edge(&self, graph: GraphId, edge: Edge) -> Result<EdgeId>;

    // Cypher 查询
    pub fn execute_cypher(&self, graph: GraphId, query: &str) -> Result<CypherResult>;

    // BFS 遍历
    pub fn bfs(&self, start: NodeId, edge_type: EdgeType, max_depth: usize) -> Result<Vec<NodeId>>;

    // DFS 遍历
    pub fn dfs(&self, start: NodeId, edge_type: EdgeType, max_depth: usize) -> Result<Vec<NodeId>>;
}
```

### 4.2 Cypher 支持

| 语法 | 状态 |
|------|------|
| MATCH | ✅ |
| WHERE | ✅ |
| RETURN | ✅ |
| WITH | ⏳ |
| CREATE | ✅ |
| MERGE | ⏳ |
| SET | ⏳ |
| DELETE | ⏳ |

## 五、性能考虑

| 操作 | 时间复杂度 | 说明 |
|------|------------|------|
| 点查询 | O(1) | HashMap |
| 边查询 | O(degree) | 邻接表 |
| BFS | O(V+E) | 图遍历 |
| DFS | O(V+E) | 图遍历 |
| 多跳 | O(k*(V+E)) | k 跳查询 |

### 优化策略

1. **索引优化**: 标签索引、属性索引
2. **缓存**: 热点节点缓存
3. **并行遍历**: 多线程 BFS/DFS
4. **剪枝**: 提前终止不必要的遍历

## 六、相关文档

- [GRAPH_ENGINE_DESIGN.md](../../../GRAPH_ENGINE_DESIGN.md) - 详细设计
- *(已归档 - 统一查询文档不存在)*

---

*Graph 模块设计 v2.5.0*
