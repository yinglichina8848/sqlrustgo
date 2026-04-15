# 图引擎设计文档

**版本**: v2.5.0
**最后更新**: 2026-04-16
**Issue**: #1381

---

## 概述

SQLRustGo Graph提供属性图支持和Cypher查询语言，用于图遍历和分析。

## 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    GraphEngine                                  │
│  - query(cypher: &str) -> GraphResult                        │
│  - execute_plan(plan: GraphPlan) -> GraphResult               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CypherParser                                 │
│  - parse(matches: &[MatchClause]) -> GraphPlan                │
│  - 支持: MATCH, WHERE, RETURN, ORDER BY, LIMIT                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    GraphPlanner                                 │
│  - LogicalPlan -> PhysicalPlan                                  │
│  - 遍历规划 (BFS/DFS)                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    GraphExecutor                               │
│  - BFSExecutor                                                 │
│  - DFSExecutor                                                 │
│  - MultiHopExecutor                                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    DiskGraphStore                              │
│  - 带WAL的图持久化                                             │
│  - 属性存储                                                    │
│  - 索引支持                                                    │
└─────────────────────────────────────────────────────────────────┘
```

## 属性图模型

```rust
// Node 表示图的顶点
pub struct Node {
    pub id: NodeId,
    pub labels: Vec<String>,      // 例如: ["Person", "Developer"]
    pub properties: HashMap<String, Value>,
}

// Edge 表示图的关系
pub struct Edge {
    pub id: EdgeId,
    pub src: NodeId,
    pub dst: NodeId,
    pub rel_type: String,         // 例如: "KNOWS", "WORKS_WITH"
    pub properties: HashMap<String, Value>,
}

pub struct Graph {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
    pub index: GraphIndex,
}
```

## Cypher支持 (Phase-1)

### MATCH

```cypher
MATCH (p:Person {name: 'Alice'})-[:KNOWS]->(friend)
WHERE friend.age > 30
RETURN friend.name, friend.age
ORDER BY friend.age
LIMIT 10
```

### 实现

```rust
// 解析MATCH子句
pub fn parse_match(pattern: &str) -> MatchClause {
    MatchClause {
        node_patterns: parse_node_patterns(pattern),
        edge_patterns: parse_edge_patterns(pattern),
        where_clause: None,
    }
}

// 执行MATCH
pub fn execute_match(
    graph: &Graph,
    match_clause: &MatchClause,
) -> Vec<MatchResult> {
    let start_nodes = find_start_nodes(graph, &match_clause.node_patterns[0]);
    let mut results = Vec::new();

    for node in start_nodes {
        let matched = traverse_from_node(graph, node, &match_clause.edge_patterns);
        results.extend(matched);
    }

    results
}
```

### 遍历策略

**BFS (广度优先搜索)**:
- 用于最短路径查询
- 逐层扩展

**DFS (深度优先搜索)**:
- 用于模式匹配
- 回溯前的深度遍历

**多跳**:
- 可配置跳数
- 每跳聚合

## 图存储 (DiskGraphStore)

PR: #1413

```rust
pub struct DiskGraphStore {
    wal_manager: WalManager,
    node_store: BTreeMap<NodeId, Node>,
    edge_store: BTreeMap<EdgeId, Edge>,
    adjacency_list: HashMap<NodeId, Vec<EdgeId>>,
    property_index: HashMap<String, NodeId>,
}

impl GraphStorage for DiskGraphStore {
    fn insert_node(&self, node: Node) -> Result<()> {
        // 1. 写入WAL条目
        self.wal_manager.log_node_insert(node.id)?;

        // 2. 插入节点存储
        self.node_store.insert(node.id, node.clone())?;

        // 3. 更新属性索引
        self.update_property_index(&node)?;

        Ok(())
    }

    fn insert_edge(&self, edge: Edge) -> Result<()> {
        // 1. 写入WAL条目
        self.wal_manager.log_edge_insert(edge.id)?;

        // 2. 插入边存储
        self.edge_store.insert(edge.id, edge.clone())?;

        // 3. 更新邻接表
        self.adjacency_list
            .entry(edge.src)
            .or_default()
            .push(edge.id);

        Ok(())
    }
}
```

## WAL集成

图操作记录到WAL用于崩溃恢复：

```rust
pub enum GraphWalEntry {
    NodeInsert { node_id: NodeId },
    NodeDelete { node_id: NodeId },
    EdgeInsert { edge_id: EdgeId },
    EdgeDelete { edge_id: EdgeId },
    NodeUpdate { node_id: NodeId, properties: HashMap<String, Value> },
}
```

## Barabási-Albert 图生成器

用于基准测试和测试：

```rust
pub struct GraphGenerator {
    rng: StdRng,
    initial_nodes: usize,
    edges_per_node: usize,
}

impl GraphGenerator {
    // Barabási-Albert 模型生成无标度图
    pub fn generate_scale_free(&mut self, n: usize) -> Graph {
        let mut graph = Graph::new();

        // 从初始节点的完全图开始
        // 通过优先附加添加剩余节点
        for new_node in self.initial_nodes..n {
            let targets = self.select_preferential_targets(new_node);
            for target in targets {
                graph.add_edge(new_node, target);
            }
        }

        graph
    }
}
```

## Cypher查询示例

### 查找朋友的朋友
```cypher
MATCH (p:Person {name: 'Alice'})-[:KNOWS*2]->(fof)
RETURN DISTINCT fof.name
LIMIT 20
```

### 共同朋友
```cypher
MATCH (a:Person)-[:KNOWS]->(b:Person)
WHERE a.name = 'Alice' AND b.name = 'Bob'
MATCH path = (a)-[:KNOWS*2]->(b)
WHERE ALL(r IN RELATIONSHIPS(path) WHERE r.since > 2000)
RETURN path
```

### 社区发现
```cypher
MATCH (p:Person)-[:KNOWS]->(friend)
WITH p, friend, COUNT(*) AS weight
WHERE weight > 5
RETURN p.name, COLLECT(friend.name) AS community
```

## 测试覆盖

| 测试 | 位置 | 状态 |
|------|------|------|
| Cypher解析 | `graph/cypher_parser_test.rs` | ✅ |
| BFS遍历 | `graph/traversal_test.rs` | ✅ |
| DFS遍历 | `graph/traversal_test.rs` | ✅ |
| 多跳 | `graph/multi_hop_test.rs` | ✅ |
| 图持久化 | `graph/persistence_test.rs` | ✅ |
| 无标度生成 | `graph/generator_test.rs` | ✅ |

## 基准测试结果

| 场景 | 节点数 | 边数 | 查询时间 |
|------|--------|------|----------|
| 小规模 | 1K | 5K | ~5ms |
| 中规模 | 10K | 50K | ~45ms |
| 大规模 | 100K | 500K | ~380ms |
| 超大规模 | 1M | 5M | ~3.5s |

## 未来工作

### Phase 2 (v2.6.0)
- 路径模式匹配
- 可变长度路径查询
- 最短路径算法

### Phase 3 (v2.7.0)
- 图算法 (PageRank, 连通分量)
- 图可视化
- Cypher UPDATE/DELETE支持
