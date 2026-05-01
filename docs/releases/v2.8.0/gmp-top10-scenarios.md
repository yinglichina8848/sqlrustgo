# GMP Top 10 应用场景

> 版本: `v2.8.0`
> 日期: 2026-05-02
> 相关 Issue: T-07

---

## 1. 概述

GMP (Graph Memory Processor) 是 SQLRustGo 的图处理引擎，核心定位为 **GMP（药品生产质量管理规范）可追溯性图引擎**（crates/graph）。

引擎基于属性图模型（Property Graph），原生支持 GMP 领域的 10 个预注册节点标签和 9 个预注册边标签。

本文档定义 v2.8.0 的 Top 10 应用场景，基于 **实际代码能力** 编写，不虚构尚不支持的功能。

### 1.1 核心架构

GMP 引擎提供两种使用方式：

- **Rust API 直接调用**：通过 `GraphStore` trait 提供完整的 CRUD + 遍历接口
- **Cypher 查询**：通过 `MATCH (n)-[r:REL]->(m) WHERE ... RETURN ...` 语法（单跳匹配，不支持 `*` 变长路径）

### 1.2 API 能力对照表

| 能力 | Rust API | Cypher |
|------|----------|--------|
| 节点 CRUD | ✅ create/get/update/delete | ❌ |
| 边 CRUD | ✅ create/get/delete | ❌ |
| 节点标签过滤 | ✅ nodes_by_label() | ✅ MATCH (n:Label) |
| 单跳遍历 | ✅ outgoing/incoming/neighbors_by_label | ✅ (a)-[r:REL]->(b) |
| BFS 遍历 | ✅ 含距离/路径重建 | ❌ |
| DFS 遍历 | ✅ 含时间戳/路径重建 | ❌ |
| 多跳遍历 | ✅ multi_hop(graph_fn, depth) | ❌ |
| 属性条件过滤 | ✅ Rust 闭包 | ✅ WHERE n.prop > val |
| 磁盘持久化 | ✅ DiskGraphStore + WAL | ❌ |
| 分片存储 | ✅ MultiShardGraphStore | ❌ |
| 跨分片遍历 | ✅ CrossShardTraversal | ❌ |

**重要说明**：Cypher 解析器当前仅支持 `(a)-[r:LABEL]->(b)` 单跳模式，不支持 `(a)-[:REL*1..3]->(b)` 变长路径、`COLLECT`/`REDUCE` 聚合、路径变量绑定、分组聚合等功能。多跳查询需通过 Rust API 的 `multi_hop()` 或 `bfs()`/`dfs()` 实现。

### 1.3 Top 10 场景列表

| 排名 | 场景名称 | 场景描述 | 使用方式 | 优先级 |
|------|----------|----------|----------|--------|
| 1 | GMP 批次全链路追溯 | Batch → Device → SOP → Regulation 正向追溯 | Rust API (BFS) | P0 |
| 2 | 物料来源追溯 | 批次所用原料的反向追查 | Rust API (DFS/neighbors) | P0 |
| 3 | 偏差事件影响分析 | Deviation → CAPA → Batch 影响范围评估 | Rust API (BFS + multi_hop) | P0 |
| 4 | 设备校准记录链 | Device → SOP → Calibration 全链路查询 | Rust API / Cypher | P1 |
| 5 | 操作员认证追溯 | Operator → Device → Batch 操作记录追踪 | Rust API / Cypher | P1 |
| 6 | 法规合规性分析 | Regulation → Batch → QA 法规遵从性检查 | Rust API (BFS) | P1 |
| 7 | GMP 图谱持久化与恢复 | 崩溃恢复与数据持久化 | DiskGraphStore + WAL | P1 |
| 8 | 大规模 GMP 分片存储 | 按标签分片的水平扩展 | MultiShardGraphStore | P2 |
| 9 | 质量审计路径重建 | QA 节点间的完整审计轨迹 | Rust API (BFS path重建) | P2 |
| 10 | GMP 图谱随机生成与性能基准 | 测试数据生成与基准测试 | GraphGenerator + Benchmark | P2 |

---

## 2. 场景详细定义

### 2.1 场景1: GMP 批次全链路追溯

#### 2.1.1 业务描述
从单个批次（Batch）出发，正向追溯其生产过程中涉及的所有设备（Device）、标准操作规程（SOP）、监管法规（Regulation）、物料（Material）、操作员（Operator）和质量检查（QA）记录。

这是 **GMP 引擎的核心设计用例**（参见 `node.rs` 的 GMP_LABELS 和 `edge.rs` 的 GMP_EDGE_LABELS）。

#### 2.1.2 数据模型
```rust
// GMP 预注册节点标签 (src/model/node.rs):
// Batch, Device, SOP, Step, Deviation, CAPA, Regulation, Material, Operator, QA

// GMP 预注册边标签 (src/model/edge.rs):
// produced_by (Batch -> Device)
// calibrated_by (Device -> SOP)
// follows_step (Step -> SOP)
// deviation_from (Deviation -> SOP)
// triggers_cap (Deviation -> CAPA)
// governed_by (Batch -> Regulation)
// uses_material (Batch -> Material)
// operated_by (Device -> Operator)
// inspected_by (Batch -> QA)
```

#### 2.1.3 GMP 查询示例

**Rust API （实际支持的）**：
```rust
use sqlrustgo_graph::*;

let mut store = InMemoryGraphStore::new();

// 创建节点
let batch = store.create_node("Batch", PropertyMapBuilder::new()
    .insert("id", "BATCH-2024-001")
    .insert("product", "Vaccine-A")
    .insert("quantity", 1000i64)
    .build());
let device = store.create_node("Device", PropertyMapBuilder::new()
    .insert("id", "DEVICE-001")
    .insert("status", "operational")
    .build());
let sop = store.create_node("SOP", PropertyMapBuilder::new()
    .insert("id", "SOP-PROD-001")
    .insert("version", "2.1")
    .build());
let regulation = store.create_node("Regulation", PropertyMapBuilder::new()
    .insert("id", "FDA-21CFR-Part11")
    .build());

// 创建追溯边
store.create_edge(batch, device, "produced_by", PropertyMap::new()).unwrap();
store.create_edge(device, sop, "calibrated_by", PropertyMap::new()).unwrap();
store.create_edge(batch, regulation, "governed_by", PropertyMap::new()).unwrap();

// BFS 全链追溯
let mut visited = Vec::new();
store.bfs(batch, |node| {
    visited.push(node);
    true // 继续遍历
});
// visited 包含 batch, device, sop, regulation

// 按边标签查询单跳邻居
let devices = store.neighbors_by_edge_label(batch, "produced_by");
assert_eq!(devices.len(), 1);
```

#### 2.1.4 性能要求
- 单次追溯查询延迟: < 50ms（100万节点规模）
- 支持 50万+ 批次节点

---

### 2.2 场景2: 物料来源追溯

#### 2.2.1 业务描述
从批次出发，反向追溯所使用的全部原材料及其供应商信息，支持质量召回场景。

#### 2.2.2 数据模型
```rust
// 使用预注册标签 Material 和边 uses_material
let material_a = store.create_node("Material", PropertyMapBuilder::new()
    .insert("id", "MAT-001")
    .insert("name", "Active Ingredient A")
    .insert("supplier", "ChemCorp")
    .build());

store.create_edge(batch, material_a, "uses_material", PropertyMap::new()).unwrap();
store.create_edge(batch, material_b, "uses_material", PropertyMap::new()).unwrap();
```

#### 2.2.3 查询示例
```rust
// 查找批次使用的所有物料
let materials = store.neighbors_by_edge_label(batch, "uses_material");

// 遍历所有物料属性
for &mat_id in &materials {
    if let Some(node) = store.get_node(mat_id) {
        let supplier = node.get_property("supplier");
        println!("Material: {:?}, Supplier: {:?}",
            node.get_property("name"), supplier);
    }
}

// 多跳追溯：从批次到物料到更多关联
// 使用 multi_hop 实现 2 跳查询
let get_neighbors = |n: NodeId| -> Vec<NodeId> {
    store.neighbors_by_edge_label(n, "uses_material")
};
let two_hop = multi_hop(get_neighbors, batch, 2);
```

#### 2.2.4 性能要求
- 物料追溯延迟: < 30ms
- 支持 10 万+ 物料节点

---

### 2.3 场景3: 偏差事件影响分析

#### 2.3.1 业务描述
当生产过程中发生偏差（Deviation）时，分析该偏差触发的 CAPA（纠正与预防措施）及其影响到的批次范围。

#### 2.3.2 数据模型
```rust
// Deviation -> CAPA 链
let deviation = store.create_node("Deviation", PropertyMapBuilder::new()
    .insert("id", "DEV-2024-001")
    .insert("severity", "critical")
    .insert("description", "Temperature excursion")
    .build());

let capa = store.create_node("CAPA", PropertyMapBuilder::new()
    .insert("id", "CAPA-2024-001")
    .insert("status", "in_progress")
    .build());

store.create_edge(deviation, capa, "triggers_cap", PropertyMap::new()).unwrap();
store.create_edge(deviation, sop, "deviation_from", PropertyMap::new()).unwrap();
```

#### 2.3.3 查询示例
```rust
// DFS 遍历偏差事件影响范围
let mut impact_chain = Vec::new();
store.dfs(deviation, |node| {
    impact_chain.push(node);
    true
});

// 使用 bfs_with_distances 获取距离信息
use sqlrustgo_graph::bfs::bfs_with_distances;
let graph = |n: NodeId| -> Vec<NodeId> {
    let mut neighbors = store.outgoing_neighbors(n);
    neighbors.extend(store.incoming_neighbors(n));
    neighbors
};
let result = bfs_with_distances(&graph, deviation);
for (node_id, dist) in &result.distances {
    if let Some(node) = store.get_node(*node_id) {
        println!("Node(id={}, label={:?}, distance={})",
            node_id, store.label_registry().get_label(node.label), dist);
    }
}

// 使用 multi_hop 查找 3 跳内的所有影响节点
let get_all_neighbors = |n: NodeId| -> Vec<NodeId> {
    let mut out = store.outgoing_neighbors(n);
    out.extend(store.incoming_neighbors(n));
    out
};
let impacts = multi_hop(get_all_neighbors, deviation, 3);
```

---

### 2.4 场景4: 设备校准记录链

#### 2.4.1 业务描述
查询设备（Device）所遵循的标准操作规程（SOP）及校准记录。

#### 2.4.2 数据模型
```rust
let device = store.create_node("Device", PropertyMapBuilder::new()
    .insert("id", "DEV-MILL-001")
    .insert("type", "Tablet Milling Machine")
    .insert("calibration_due", "2024-12-31")
    .build());

let sop = store.create_node("SOP", PropertyMapBuilder::new()
    .insert("id", "SOP-CAL-001")
    .insert("title", "Calibration Procedure A")
    .build());

store.create_edge(device, sop, "calibrated_by", PropertyMap::new()).unwrap();
```

#### 2.4.3 GMP 查询示例
**Cypher 查询（实际支持的单跳模式）**：
```rust
// Cypher 查询：单跳关系匹配
// MATCH (d:Device)-[c:calibrated_by]->(s:SOP)
// WHERE d.id = 'DEV-MILL-001'
// RETURN d.id, s.title
let result = execute_cypher(
    "MATCH (d:Device)-[c:calibrated_by]->(s:SOP) WHERE d.id = 'DEV-MILL-001' RETURN d.id, s.title",
    &store
)?;
for row in &result.rows {
    println!("{:?}", row);
}
```

**Rust API 等价格式**：
```rust
let device_id = store.nodes_by_label("Device")
    .into_iter()
    .find(|&id| {
        store.get_node(id)
            .and_then(|n| n.get_property("id"))
            .and_then(|v| v.as_string())
            == Some(&"DEV-MILL-001".to_string())
    })
    .expect("Device not found");

let sops = store.neighbors_by_edge_label(device_id, "calibrated_by");
```

---

### 2.5 场景5: 操作员认证追溯

#### 2.5.1 业务描述
查询操作员（Operator）操作过的设备（Device）及生产的批次（Batch），用于认证合规性审计。

#### 2.5.2 查询示例
```rust
// 创建操作员和关联
let operator = store.create_node("Operator", PropertyMapBuilder::new()
    .insert("id", "OP-001")
    .insert("name", "John Smith")
    .insert("certification", "GMP-2024")
    .build());

// Device operated by Operator
store.create_edge(operator, device, "operated_by", PropertyMap::new()).unwrap();

// Batch inspected by QA
let qa = store.create_node("QA", PropertyMapBuilder::new()
    .insert("id", "QA-2024-001")
    .insert("result", "approved")
    .build());
store.create_edge(batch, qa, "inspected_by", PropertyMap::new()).unwrap();

// 查询操作员涉及的所有设备
let operated_devices = store.neighbors_by_edge_label(operator, "operated_by");

// 组合查询：使用 BFS 从操作员出发找到所有关联批次
let mut all_related = Vec::new();
store.bfs(operator, |node| {
    all_related.push(node);
    true
});
```

**Cypher 查询**：
```sql
MATCH (o:Operator)-[r:operated_by]->(d:Device)
WHERE o.id = 'OP-001'
RETURN o.name, d.id, d.type
```

---

### 2.6 场景6: 法规合规性分析

#### 2.6.1 业务描述
分析特定法规（Regulation）所管辖的批次范围及其 QA 检查结果，确保 GMP 合规性。

#### 2.6.2 查询示例
```rust
// 创建法规节点
let regulation = store.create_node("Regulation", PropertyMapBuilder::new()
    .insert("id", "FDA-21CFR-Part11")
    .insert("title", "Electronic Records; Electronic Signatures")
    .build());

// 建立关联
store.create_edge(batch, regulation, "governed_by", PropertyMap::new()).unwrap();

// BFS 遍历：从法规出发查找所有受管辖的批次和 QA 记录
let mut compliance_chain = Vec::new();
// 注意：BFS 只遍历 outgoing，所以需要从 batch 反向查
// 或者使用 incoming_neighbors
let regulated_batches = store.incoming_neighbors(regulation);
for &batch_id in &regulated_batches {
    let qa_records = store.neighbors_by_edge_label(batch_id, "inspected_by");
    compliance_chain.push((batch_id, qa_records));
}

// 使用 dfs_with_timing 获得拓扑排序
use sqlrustgo_graph::dfs::dfs_with_timing;
let graph = |n: NodeId| -> Vec<NodeId> {
    let mut neighbors = store.outgoing_neighbors(n);
    neighbors.extend(store.incoming_neighbors(n));
    neighbors
};
let dfs_result = dfs_with_timing(&graph, regulation);
```

---

### 2.7 场景7: GMP 图谱持久化与恢复

#### 2.7.1 业务描述
将 GMP 图谱持久化到磁盘，支持 Write-Ahead Log (WAL) 的崩溃恢复能力。

#### 2.7.2 实际代码能力
`DiskGraphStore` 支持：
- 基于 `serde_json` 的序列化存储（nodes.json, edges.json, labels.json, meta.json）
- WAL 日志（graph.wal）记录所有写操作（CreateNode, UpdateNode, DeleteNode, CreateEdge, DeleteEdge）
- `fsync` 保证写入持久化
- 启动时自动回放 WAL 恢复数据

```rust
use std::path::PathBuf;
use sqlrustgo_graph::store::DiskGraphStore;

// 创建带 WAL 的持久化存储
let mut store = DiskGraphStore::new(PathBuf::from("/data/gmp_graph"))?;

// 所有写操作自动写入 WAL
let batch = store.create_node("Batch", props);
store.create_edge(batch, device, "produced_by", PropertyMap::new())?;

// 关闭后可重新加载（自动回放 WAL）
let store = DiskGraphStore::load(PathBuf::from("/data/gmp_graph"))?;
assert_eq!(store.node_count(), 1);
```

#### 2.7.3 WAL 条目类型
| 条目类型 | 说明 |
|----------|------|
| CreateNode | 记录 node_id, label_id, props |
| UpdateNode | 记录 node_id, props |
| DeleteNode | 记录 node_id |
| CreateEdge | 记录 edge_id, from, to, label_id, props |
| DeleteEdge | 记录 edge_id |

---

### 2.8 场景8: 大规模 GMP 分片存储

#### 2.8.1 业务描述
当 GMP 数据规模超过单机内存容量时，使用标签级分片实现水平扩展。

#### 2.8.2 实际代码能力
`MultiShardGraphStore` 支持：
- 基于 `LabelBasedGraphPartitioner` 的标签级分片路由
- 节点按标签路由到不同分片
- 同一分片内的边创建（跨分片边创建会返回 `InvalidEdge` 错误）
- 跨分片 BFS 遍历（`CrossShardTraversal::distributed_bfs`）

```rust
use sqlrustgo_graph::*;

let mut store = MultiShardGraphStore::new();

// 按标签分片：Batch 和 Material 放到不同分片
store.register_label_sharding("Batch", GraphShardId(0));
store.register_label_sharding("Material", GraphShardId(1));
store.set_default_shard(GraphShardId(0));

// 节点自动路由到对应分片
let batch = store.create_node("Batch", props);  // 分片 0
let material = store.create_node("Material", props);  // 分片 1

// 跨分片边创建失败
let result = store.create_edge(batch, material, "uses_material", PropertyMap::new());
assert!(result.is_err()); // 跨分片边不支持

// 跨分片 BFS 遍历
let traversal = CrossShardTraversal::new(store);
let result = traversal.distributed_bfs(batch, 5); // 最多 5 层
```

#### 2.8.3 性能目标
- 支持 100万+ 节点，`节点/分片 > 10万` 时性能显著
- 跨分片 BFS 时间复杂度 O(V+E)

---

### 2.9 场景9: 质量审计路径重建

#### 2.9.1 业务描述
在 GMP 质量审计中，需要在任意两个相关节点之间重建完整的审计轨迹路径。

#### 2.9.2 实际代码能力
`BfsResult::reconstruct_path()` 和 `DfsResult::reconstruct_path()` 支持基于 parent 映射的路径重建。

```rust
use sqlrustgo_graph::bfs::bfs_with_distances;

// 构建一个包含完整 GMP 链的图谱
// (QA) <- inspected_by - (Batch) - produced_by -> (Device)

// BFS 并记录距离和父节点
let graph = |n: NodeId| -> Vec<NodeId> {
    let mut neighbors = store.outgoing_neighbors(n);
    neighbors.extend(store.incoming_neighbors(n));
    neighbors
};
let result = bfs_with_distances(&graph, qa_node);

// 重建从 QA 到 Device 的路径
if let Some(path) = result.reconstruct_path(qa_node, device_node) {
    // path = [QA, Batch, Device]
    for &node_id in &path {
        if let Some(node) = store.get_node(node_id) {
            let label = store.label_registry().get_label(node.label)
                .unwrap_or("Unknown");
            println!("-> {} (id={})", label, node_id);
        }
    }
}

// 获取两节点间距离
let distance = result.distances.get(&device_node);
println!("Audit trail length: {:?} hops", distance);
```

---

### 2.10 场景10: GMP 图谱随机生成与性能基准

#### 2.10.1 业务描述
使用 `GraphGenerator` 生成具有幂律度分布的测试图数据，用于功能验证和性能基准测试。

#### 2.10.2 实际代码能力
`GraphGenerator` 基于 Barabási-Albert 优先连接模型，生成具有真实网络特性的测试图。

```rust
use sqlrustgo_graph::graph_generator::GraphGenerator;

// 生成 100 节点、平均度 3 的图
let gen = GraphGenerator::new(42); // 确定性种子
let store = gen.generate(100, 3);

assert_eq!(store.node_count(), 100);
assert!(store.edge_count() > 0);

// 确定性：相同种子生成相同图
let gen2 = GraphGenerator::new(42);
let store2 = gen2.generate(100, 3);
assert_eq!(store1.edge_count(), store2.edge_count());
```

**性能基准测试**（crates/graph/benches/）：
- graph_benchmark.rs: BFS/DFS/边查询基准
- graph_benchmark_extended.rs: 多跳/大规模图基准

```
// 基准测试参考数据（硬件：Mac Mini M4）
// graph_benchmark_20260416_021851.json
// - BFS 1000节点: ~50μs
// - DFS 1000节点: ~30μs
// - 2-hop 1000节点: ~200μs
// - 3-hop 1000节点: ~500μs
```

---

## 3. 场景优先级与交付计划

### Phase 1 (v2.8.0-alpha): P0 场景
- [x] 场景1: GMP 批次全链路追溯
- [x] 场景2: 物料来源追溯
- [x] 场景3: 偏差事件影响分析

### Phase 2 (v2.8.0-beta): P1 场景
- [x] 场景4: 设备校准记录链
- [x] 场景5: 操作员认证追溯
- [x] 场景6: 法规合规性分析
- [x] 场景7: GMP 图谱持久化与恢复

### Phase 3 (v2.8.0-RC): P2 场景
- [x] 场景8: 大规模 GMP 分片存储
- [x] 场景9: 质量审计路径重建
- [x] 场景10: GMP 图谱随机生成与性能基准

---

## 4. 代码位置对照

| 功能 | 文件路径 |
|------|----------|
| GraphStore trait | `crates/graph/src/store/graph_store.rs` |
| InMemoryGraphStore | `crates/graph/src/store/graph_store.rs` |
| DiskGraphStore + WAL | `crates/graph/src/store/disk_graph_store.rs` |
| MultiShardGraphStore | `crates/graph/src/sharded_graph.rs` |
| Cypher 解析器 | `crates/graph/src/cypher/parser.rs` |
| Cypher 执行器 | `crates/graph/src/cypher/executor.rs` |
| BFS 遍历 | `crates/graph/src/traversal/bfs.rs` |
| DFS 遍历 | `crates/graph/src/traversal/dfs.rs` |
| 多跳遍历 | `crates/graph/src/traversal/multi_hop.rs` |
| 图生成器 | `crates/graph/src/graph_generator.rs` |
| 节点模型 + GMP_LABELS | `crates/graph/src/model/node.rs` |
| 边模型 + GMP_EDGE_LABELS | `crates/graph/src/model/edge.rs` |
| 属性模型 | `crates/graph/src/model/property.rs` |
| 集成测试 | `crates/graph/tests/graph_tests.rs` |
| 性能基准 | `crates/graph/benches/graph_benchmark.rs` |

---

## 5. 与 v2.7.0 的差异说明

v2.7.0 的文档中描述了一些 **尚未实现** 的特性，本版本基于实际代码做了修正：

| v2.7.0 描述 | v2.8.0 实际情况 |
|-------------|----------------|
| `GRAPH MATCH` SQL 扩展语法 | 使用独立 Cypher 解析器（`crates/graph/src/cypher/`），并非 SQL 扩展 |
| `(a)-[:REL*1..3]->(b)` 变长路径 | Cypher 解析器仅支持单跳 `(a)-[r:REL]->(b)` |
| `COLLECT`, `REDUCE`, `COUNT` 聚合函数 | Cypher 执行器不支持聚合 |
| `ALL(r IN relationships WHERE ...)` | 不支持 |
| `LENGTH(path)` | 不支持 |
| SQL 表 + GRAPH 扩展 | GMP 引擎是独立图引擎，不基于 SQL 表 |
| 社交网络/知识图谱/欺诈检测 | 核心用例是 GMP 可追溯性，非通用图数据库 |
| 最短路径算法 | 支持路径重建但不支持加权最短路径 |

GMP 引擎的核心设计初衷是 **制药行业 GMP 可追溯性**，建议基于 GMP 领域构建应用，而非泛化的图数据库场景。
