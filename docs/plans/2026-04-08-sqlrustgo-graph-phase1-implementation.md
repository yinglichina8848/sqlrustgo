# SQLRustGo Graph Engine Phase-1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 `sqlrustgo-graph` crate 的 Phase-1 核心功能，包括数据模型、LabelRegistry、存储层、遍历 API 和 BFS/DFS 算法。

**Architecture:** 
- 独立 crate，通过 StorageAdapter 复用 StorageEngine
- 使用 u64/u32 ID 系统，核心标签启动时注册，动态标签运行时创建
- AdjacencyIndex 双向邻接表 + LabelIndex O(1) 标签查找
- GraphStore trait 统一 API，Iterator 返回类型避免全量加载

**Tech Stack:** Rust (Edition 2021), std::collections::HashMap, std::collections::VecDeque

---

## 前置准备

### Task 0: 创建 worktree（可选）

如果需要隔离开发：
```bash
cd ~/workspace/dev/yinglichina163/sqlrustgo
git worktree add ../sqlrustgo-graph-worktree feature/graph-phase1
cd ../sqlrustgo-graph-worktree
```

---

## 模块 1：数据模型 (model/)

### Task 1: 创建 model 模块骨架

**Files:**
- Create: `crates/graph/src/model/mod.rs`

**Step 1: 创建目录结构**
```bash
mkdir -p crates/graph/src/model
touch crates/graph/src/model/mod.rs
```

**Step 2: 验证目录创建成功**
```bash
ls -la crates/graph/src/model/
```
Expected: `mod.rs` 文件存在

---

### Task 2: 实现 ID 类型

**Files:**
- Create: `crates/graph/src/model/ids.rs`

**Step 1: 编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_new() {
        let id = NodeId(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_node_id_equality() {
        let id1 = NodeId(1);
        let id2 = NodeId(1);
        let id3 = NodeId(2);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_edge_id_new() {
        let id = EdgeId(100);
        assert_eq!(id.0, 100);
    }

    #[test]
    fn test_label_id_new() {
        let id = LabelId(5);
        assert_eq!(id.0, 5);
    }

    #[test]
    fn test_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NodeId(1));
        set.insert(NodeId(1)); // duplicate
        set.insert(NodeId(2));
        assert_eq!(set.len(), 2); // only 2 unique
    }

    #[test]
    fn test_id_debug() {
        let id = NodeId(42);
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("42"));
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cd ~/workspace/dev/yinglichina163/sqlrustgo
cargo test --package sqlrustgo-graph -- model::ids --nocapture 2>&1
```
Expected: FAIL - `module not found`

**Step 3: 实现 IDs**

```rust
use std::fmt;

/// 节点 ID - 使用 u64 高效存储
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NodeId(pub u64);

/// 边 ID - 使用 u64 高效存储
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EdgeId(pub u64);

/// 标签 ID - 使用 u32（标签数量通常 < 65536）
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LabelId(pub u32);
```

**Step 4: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- model::ids --nocapture
```
Expected: PASS - 6 tests

**Step 5: 提交**
```bash
git add crates/graph/src/model/ids.rs crates/graph/src/model/mod.rs
git commit -m "feat(graph): add NodeId, EdgeId, LabelId types"
```

---

### Task 3: 实现 PropertyMap

**Files:**
- Create: `crates/graph/src/model/property.rs`

**Step 1: 编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_property_map_new() {
        let map = PropertyMap::new();
        assert!(map.is_empty());
    }

    #[test]
    fn test_property_map_insert() {
        let mut map = PropertyMap::new();
        map.insert("name".to_string(), Value::Text("Batch-001".to_string()));
        map.insert("status".to_string(), Value::Integer(1));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_property_map_get() {
        let mut map = PropertyMap::new();
        map.insert("id".to_string(), Value::Integer(42));
        assert_eq!(map.get("id"), Some(&Value::Integer(42)));
        assert_eq!(map.get("nonexistent"), None);
    }

    #[test]
    fn test_property_map_default() {
        let map = PropertyMap::default();
        assert!(map.is_empty());
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- model::property --nocapture
```
Expected: FAIL - `module not found`

**Step 3: 实现 PropertyMap**

```rust
use serde::{Deserialize, Serialize};
use sqlrustgo_types::Value;
use std::collections::HashMap;

/// 属性映射：属性名 → 值
pub type PropertyMap = HashMap<String, Value>;

/// 创建属性映射的辅助宏
#[macro_export]
macro_rules! prop {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = PropertyMap::new();
            $(map.insert($key.to_string(), $value);)*
            map
        }
    };
}
```

**Step 4: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- model::property --nocapture
```
Expected: PASS - 4 tests

**Step 5: 提交**
```bash
git add crates/graph/src/model/property.rs
git commit -m "feat(graph): add PropertyMap type"
```

---

### Task 4: 实现 Node 和 Edge

**Files:**
- Create: `crates/graph/src/model/node.rs`
- Create: `crates/graph/src/model/edge.rs`

**Step 1: 编写 Node 测试**

```rust
// crates/graph/src/model/node.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_new() {
        let node = Node {
            id: NodeId(1),
            label: LabelId(10),
            properties: PropertyMap::new(),
        };
        assert_eq!(node.id, NodeId(1));
        assert_eq!(node.label, LabelId(10));
    }

    #[test]
    fn test_node_with_properties() {
        let props = prop! {
            "batch_id" => Value::Text("B-2026-001".to_string()),
            "quantity" => Value::Integer(1000)
        };
        let node = Node::new(NodeId(1), LabelId(1), props);
        assert_eq!(node.get_property("batch_id"), Some(&Value::Text("B-2026-001".to_string())));
        assert_eq!(node.get_property("quantity"), Some(&Value::Integer(1000)));
        assert_eq!(node.get_property("nonexistent"), None);
    }
}
```

**Step 2: 编写 Edge 测试**

```rust
// crates/graph/src/model/edge.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_new() {
        let edge = Edge {
            id: EdgeId(1),
            from: NodeId(10),
            to: NodeId(20),
            label: LabelId(5),
            properties: PropertyMap::new(),
        };
        assert_eq!(edge.from, NodeId(10));
        assert_eq!(edge.to, NodeId(20));
    }

    #[test]
    fn test_edge_with_properties() {
        let props = prop! {
            "weight" => Value::Float(1.5)
        };
        let edge = Edge::new(EdgeId(1), NodeId(10), NodeId(20), LabelId(5), props);
        assert_eq!(edge.get_property("weight"), Some(&Value::Float(1.5)));
    }
}
```

**Step 3: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- model --nocapture
```
Expected: FAIL - modules not found

**Step 4: 实现 Node**

```rust
// crates/graph/src/model/node.rs

use super::{EdgeId, LabelId, NodeId, PropertyMap};
use sqlrustgo_types::Value;

/// 图节点
#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub label: LabelId,
    pub properties: PropertyMap,
}

impl Node {
    pub fn new(id: NodeId, label: LabelId, properties: PropertyMap) -> Self {
        Self { id, label, properties }
    }

    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }
}
```

**Step 5: 实现 Edge**

```rust
// crates/graph/src/model/edge.rs

use super::{EdgeId, LabelId, NodeId, PropertyMap};
use sqlrustgo_types::Value;

/// 图边
#[derive(Clone, Debug)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub label: LabelId,
    pub properties: PropertyMap,
}

impl Edge {
    pub fn new(
        id: EdgeId,
        from: NodeId,
        to: NodeId,
        label: LabelId,
        properties: PropertyMap,
    ) -> Self {
        Self { id, from, to, label, properties }
    }

    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }
}
```

**Step 6: 更新 mod.rs**

```rust
// crates/graph/src/model/mod.rs

pub mod edge;
pub mod ids;
pub mod node;
pub mod property;

pub use edge::Edge;
pub use ids::{EdgeId, LabelId, NodeId};
pub use node::Node;
pub use property::{prop, PropertyMap};
```

**Step 7: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- model --nocapture
```
Expected: PASS - 10 tests

**Step 8: 提交**
```bash
git add crates/graph/src/model/
git commit -m "feat(graph): add Node and Edge models"
```

---

## 模块 2：LabelRegistry (registry/)

### Task 5: 实现 LabelRegistry

**Files:**
- Create: `crates/graph/src/registry/mod.rs`
- Create: `crates/graph/src/registry/label_registry.rs`

**Step 1: 编写 LabelRegistry 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_registry_new() {
        let registry = LabelRegistry::new();
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_get_or_create_core_label() {
        let mut registry = LabelRegistry::new();
        registry.bootstrap_core_labels(&["Batch", "Device"]);
        
        let batch_id = registry.get_or_create("Batch").unwrap();
        let device_id = registry.get_or_create("Device").unwrap();
        
        assert_eq!(batch_id, LabelId(1));
        assert_eq!(device_id, LabelId(2));
    }

    #[test]
    fn test_get_existing_label() {
        let mut registry = LabelRegistry::new();
        registry.bootstrap_core_labels(&["Batch"]);
        
        let id1 = registry.get_or_create("Batch").unwrap();
        let id2 = registry.get("Batch");
        
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_get_nonexistent_label() {
        let registry = LabelRegistry::new();
        assert!(registry.get("Nonexistent").is_none());
    }

    #[test]
    fn test_resolve_label() {
        let mut registry = LabelRegistry::new();
        registry.bootstrap_core_labels(&["Batch", "Device"]);
        
        assert_eq!(registry.resolve(LabelId(1)), "Batch");
        assert_eq!(registry.resolve(LabelId(2)), "Device");
    }

    #[test]
    fn test_dynamic_label_after_core() {
        let mut registry = LabelRegistry::new();
        registry.bootstrap_core_labels(&["Batch"]);
        
        // Core labels: 1..256, Dynamic: 256+
        let custom = registry.get_or_create("CustomType").unwrap();
        assert_eq!(custom, LabelId(256));
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- registry --nocapture
```
Expected: FAIL - module not found

**Step 3: 实现 LabelRegistry**

```rust
// crates/graph/src/registry/label_registry.rs

use super::{LabelId, PropertyMap};
use std::collections::HashMap;

/// 标签注册表
#[derive(Debug)]
pub struct LabelRegistry {
    name_to_id: HashMap<String, LabelId>,
    id_to_name: Vec<String>,
    next_dynamic_id: LabelId,
}

impl LabelRegistry {
    /// 创建新的注册表（核心标签从 256 开始）
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
            next_dynamic_id: LabelId(256),
        }
    }

    /// 引导核心标签（启动时调用）
    pub fn bootstrap_core_labels(&mut self, labels: &[&str]) {
        for (idx, name) in labels.iter().enumerate() {
            let label_id = LabelId((idx + 1) as u32); // 核心标签从 1 开始
            self.name_to_id.insert(name.to_string(), label_id);
            if self.id_to_name.len() <= idx {
                self.id_to_name.push(name.to_string());
            }
        }
    }

    /// 获取标签 ID（不存在返回 None）
    pub fn get(&self, name: &str) -> Option<LabelId> {
        self.name_to_id.get(name).copied()
    }

    /// 获取或创建标签
    pub fn get_or_create(&mut self, name: &str) -> Option<LabelId> {
        if let Some(&id) = self.name_to_id.get(name) {
            return Some(id);
        }
        
        let id = self.next_dynamic_id;
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.push(name.to_string());
        self.next_dynamic_id = LabelId(self.next_dynamic_id.0 + 1);
        Some(id)
    }

    /// 根据 ID 解析标签名
    pub fn resolve(&self, id: LabelId) -> &str {
        let idx = id.0 as usize;
        if idx > 0 && idx <= self.id_to_name.len() {
            &self.id_to_name[idx - 1]
        } else if idx >= 256 && idx - 256 < self.id_to_name.len() - 255 {
            &self.id_to_name[idx]
        } else {
            "Unknown"
        }
    }

    /// 注册表大小
    pub fn len(&self) -> usize {
        self.name_to_id.len()
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.name_to_id.is_empty()
    }
}

impl Default for LabelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: 更新 mod.rs**

```rust
// crates/graph/src/registry/mod.rs

pub mod label_registry;

pub use label_registry::LabelRegistry;
```

**Step 5: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- registry --nocapture
```
Expected: PASS - 6 tests

**Step 6: 提交**
```bash
git add crates/graph/src/registry/
git commit -m "feat(graph): add LabelRegistry with bootstrap support"
```

---

## 模块 3：存储层 (storage/)

### Task 6: 实现 NodeStore

**Files:**
- Create: `crates/graph/src/storage/mod.rs`
- Create: `crates/graph/src/storage/node_store.rs`

**Step 1: 编写 NodeStore 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LabelId, Node, NodeId, PropertyMap};

    #[test]
    fn test_node_store_insert() {
        let mut store = NodeStore::new();
        let node = Node::new(NodeId(1), LabelId(1), PropertyMap::new());
        
        store.insert(node);
        assert!(store.get(NodeId(1)).is_some());
    }

    #[test]
    fn test_node_store_get() {
        let mut store = NodeStore::new();
        let node = Node::new(NodeId(1), LabelId(1), PropertyMap::new());
        store.insert(node);
        
        let retrieved = store.get(NodeId(1)).unwrap();
        assert_eq!(retrieved.id, NodeId(1));
    }

    #[test]
    fn test_node_store_get_nonexistent() {
        let store = NodeStore::new();
        assert!(store.get(NodeId(999)).is_none());
    }

    #[test]
    fn test_node_store_remove() {
        let mut store = NodeStore::new();
        let node = Node::new(NodeId(1), LabelId(1), PropertyMap::new());
        store.insert(node);
        
        let removed = store.remove(NodeId(1));
        assert!(removed.is_some());
        assert!(store.get(NodeId(1)).is_none());
    }

    #[test]
    fn test_node_store_len() {
        let mut store = NodeStore::new();
        assert_eq!(store.len(), 0);
        
        store.insert(Node::new(NodeId(1), LabelId(1), PropertyMap::new()));
        store.insert(Node::new(NodeId(2), LabelId(1), PropertyMap::new()));
        assert_eq!(store.len(), 2);
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- storage::node_store --nocapture
```
Expected: FAIL

**Step 3: 实现 NodeStore**

```rust
// crates/graph/src/storage/node_store.rs

use crate::{Node, NodeId};
use std::collections::HashMap;

/// 节点存储
#[derive(Debug)]
pub struct NodeStore {
    nodes: HashMap<NodeId, Node>,
    next_id: u64,
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: 1,
        }
    }

    /// 插入节点（自动分配 ID）
    pub fn insert(&mut self, node: Node) -> NodeId {
        let id = node.id;
        self.nodes.insert(id, node);
        if id.0 >= self.next_id {
            self.next_id = id.0 + 1;
        }
        id
    }

    /// 获取节点
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// 获取节点（可变引用）
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// 删除节点
    pub fn remove(&mut self, id: NodeId) -> Option<Node> {
        self.nodes.remove(&id)
    }

    /// 获取下一个可用 ID
    pub fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    /// 存储节点数量
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 迭代所有节点
    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &Node)> {
        self.nodes.iter()
    }
}

impl Default for NodeStore {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: 更新 mod.rs**

```rust
// crates/graph/src/storage/mod.rs

pub mod edge_store;
pub mod node_store;

pub use edge_store::EdgeStore;
pub use node_store::NodeStore;
```

**Step 5: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- storage::node_store --nocapture
```
Expected: PASS - 5 tests

**Step 6: 提交**
```bash
git add crates/graph/src/storage/
git commit -m "feat(graph): add NodeStore"
```

---

### Task 7: 实现 EdgeStore

**Files:**
- Create: `crates/graph/src/storage/edge_store.rs`

**Step 1: 编写 EdgeStore 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, EdgeId, LabelId, NodeId, PropertyMap};

    #[test]
    fn test_edge_store_insert() {
        let mut store = EdgeStore::new();
        let edge = Edge::new(EdgeId(1), NodeId(1), NodeId(2), LabelId(1), PropertyMap::new());
        
        store.insert(edge);
        assert!(store.get(EdgeId(1)).is_some());
    }

    #[test]
    fn test_edge_store_get() {
        let mut store = EdgeStore::new();
        let edge = Edge::new(EdgeId(1), NodeId(1), NodeId(2), LabelId(1), PropertyMap::new());
        store.insert(edge);
        
        let retrieved = store.get(EdgeId(1)).unwrap();
        assert_eq!(retrieved.from, NodeId(1));
        assert_eq!(retrieved.to, NodeId(2));
    }

    #[test]
    fn test_edge_store_remove() {
        let mut store = EdgeStore::new();
        let edge = Edge::new(EdgeId(1), NodeId(1), NodeId(2), LabelId(1), PropertyMap::new());
        store.insert(edge);
        
        store.remove(EdgeId(1));
        assert!(store.get(EdgeId(1)).is_none());
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- storage::edge_store --nocapture
```
Expected: FAIL

**Step 3: 实现 EdgeStore**

```rust
// crates/graph/src/storage/edge_store.rs

use crate::{Edge, EdgeId};
use std::collections::HashMap;

/// 边存储
#[derive(Debug)]
pub struct EdgeStore {
    edges: HashMap<EdgeId, Edge>,
    next_id: u64,
}

impl EdgeStore {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, edge: Edge) -> EdgeId {
        let id = edge.id;
        self.edges.insert(id, edge);
        if id.0 >= self.next_id {
            self.next_id = id.0 + 1;
        }
        id
    }

    pub fn get(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(&id)
    }

    pub fn get_mut(&mut self, id: EdgeId) -> Option<&mut Edge> {
        self.edges.get_mut(&id)
    }

    pub fn remove(&mut self, id: EdgeId) -> Option<Edge> {
        self.edges.remove(&id)
    }

    pub fn next_id(&mut self) -> EdgeId {
        let id = EdgeId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&EdgeId, &Edge)> {
        self.edges.iter()
    }
}

impl Default for EdgeStore {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- storage::edge_store --nocapture
```
Expected: PASS - 3 tests

**Step 5: 提交**
```bash
git add crates/graph/src/storage/edge_store.rs
git commit -m "feat(graph): add EdgeStore"
```

---

## 模块 4：索引层 (index/)

### Task 8: 实现 AdjacencyIndex

**Files:**
- Create: `crates/graph/src/index/mod.rs`
- Create: `crates/graph/src/index/adjacency_index.rs`

**Step 1: 编写 AdjacencyIndex 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EdgeId, LabelId, NodeId};

    #[test]
    fn test_adjacency_add_outgoing() {
        let mut index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), EdgeId(10), LabelId(1));
        
        let outgoing = index.outgoing_edges(NodeId(1));
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0], EdgeId(10));
    }

    #[test]
    fn test_adjacency_add_incoming() {
        let mut index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), EdgeId(10), LabelId(1));
        
        let incoming = index.incoming_edges(NodeId(2));
        assert_eq!(incoming.len(), 1);
    }

    #[test]
    fn test_adjacency_remove_edge() {
        let mut index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), EdgeId(10), LabelId(1));
        index.remove_edge(NodeId(1), EdgeId(10));
        
        assert!(index.outgoing_edges(NodeId(1)).is_empty());
    }

    #[test]
    fn test_adjacency_multiple_edges() {
        let mut index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), EdgeId(10), LabelId(1));
        index.add_edge(NodeId(1), NodeId(3), EdgeId(11), LabelId(1));
        
        assert_eq!(index.outgoing_edges(NodeId(1)).len(), 2);
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- index::adjacency --nocapture
```
Expected: FAIL

**Step 3: 实现 AdjacencyIndex**

```rust
// crates/graph/src/index/adjacency_index.rs

use crate::{EdgeId, LabelId, NodeId};
use std::collections::HashMap;

/// 邻接表索引（双向）
#[derive(Debug)]
pub struct AdjacencyIndex {
    outgoing: HashMap<NodeId, Vec<EdgeId>>,
    incoming: HashMap<NodeId, Vec<EdgeId>>,
}

impl AdjacencyIndex {
    pub fn new() -> Self {
        Self {
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        }
    }

    /// 添加边
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, edge_id: EdgeId, _label: LabelId) {
        self.outgoing
            .entry(from)
            .or_insert_with(Vec::new)
            .push(edge_id);
        self.incoming
            .entry(to)
            .or_insert_with(Vec::new)
            .push(edge_id);
    }

    /// 移除边
    pub fn remove_edge(&mut self, from: NodeId, edge_id: EdgeId) {
        if let Some(edges) = self.outgoing.get_mut(&from) {
            edges.retain(|&e| e != edge_id);
        }
        for edges in self.incoming.values_mut() {
            edges.retain(|&e| e != edge_id);
        }
    }

    /// 获取出向边
    pub fn outgoing_edges(&self, node: NodeId) -> Vec<EdgeId> {
        self.outgoing.get(&node).cloned().unwrap_or_default()
    }

    /// 获取入向边
    pub fn incoming_edges(&self, node: NodeId) -> Vec<EdgeId> {
        self.incoming.get(&node).cloned().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.outgoing.is_empty() && self.incoming.is_empty()
    }
}

impl Default for AdjacencyIndex {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: 更新 mod.rs**

```rust
// crates/graph/src/index/mod.rs

pub mod adjacency_index;
pub mod label_index;

pub use adjacency_index::AdjacencyIndex;
pub use label_index::LabelIndex;
```

**Step 5: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- index::adjacency --nocapture
```
Expected: PASS - 4 tests

**Step 6: 提交**
```bash
git add crates/graph/src/index/
git commit -m "feat(graph): add AdjacencyIndex"
```

---

### Task 9: 实现 LabelIndex

**Files:**
- Create: `crates/graph/src/index/label_index.rs`

**Step 1: 编写 LabelIndex 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LabelId, NodeId};

    #[test]
    fn test_label_index_insert() {
        let mut index = LabelIndex::new();
        index.insert(LabelId(1), NodeId(10));
        index.insert(LabelId(1), NodeId(20));
        
        let nodes = index.get_nodes(LabelId(1));
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_label_index_remove() {
        let mut index = LabelIndex::new();
        index.insert(LabelId(1), NodeId(10));
        index.insert(LabelId(1), NodeId(20));
        index.remove(LabelId(1), NodeId(10));
        
        let nodes = index.get_nodes(LabelId(1));
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0], NodeId(20));
    }

    #[test]
    fn test_label_index_get_empty() {
        let index = LabelIndex::new();
        let nodes = index.get_nodes(LabelId(999));
        assert!(nodes.is_empty());
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- index::label_index --nocapture
```
Expected: FAIL

**Step 3: 实现 LabelIndex**

```rust
// crates/graph/src/index/label_index.rs

use crate::{LabelId, NodeId};
use std::collections::HashMap;

/// 标签索引 - O(1) 标签查找
#[derive(Debug)]
pub struct LabelIndex {
    index: HashMap<LabelId, Vec<NodeId>>,
}

impl LabelIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// 插入节点到标签
    pub fn insert(&mut self, label: LabelId, node: NodeId) {
        self.index
            .entry(label)
            .or_insert_with(Vec::new)
            .push(node);
    }

    /// 从标签移除节点
    pub fn remove(&mut self, label: LabelId, node: NodeId) {
        if let Some(nodes) = self.index.get_mut(&label) {
            nodes.retain(|&n| n != node);
        }
    }

    /// 获取标签下的所有节点
    pub fn get_nodes(&self, label: LabelId) -> Vec<NodeId> {
        self.index.get(&label).cloned().unwrap_or_default()
    }

    /// 判断标签是否存在
    pub fn contains_label(&self, label: LabelId) -> bool {
        self.index.contains_key(&label)
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

impl Default for LabelIndex {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- index::label_index --nocapture
```
Expected: PASS - 3 tests

**Step 5: 提交**
```bash
git add crates/graph/src/index/label_index.rs
git commit -m "feat(graph): add LabelIndex"
```

---

## 模块 5：算法 (algorithms/)

### Task 10: 实现 BFS

**Files:**
- Create: `crates/graph/src/algorithms/mod.rs`
- Create: `crates/graph/src/algorithms/bfs.rs`

**Step 1: 编写 BFS 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs_simple() {
        // 构建简单图: 1 -> 2 -> 3
        let mut graph = TestGraph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        
        let mut visited = Vec::new();
        graph.bfs(NodeId(1), |id| visited.push(id));
        
        // BFS 应该按层级访问: 1, 2, 3
        assert!(visited.contains(&NodeId(1)));
        assert!(visited.contains(&NodeId(2)));
        assert!(visited.contains(&NodeId(3)));
    }

    #[test]
    fn test_bfs_early_termination() {
        let mut graph = TestGraph::new();
        graph.add_edge(1, 2);
        graph.add_edge(1, 3);
        graph.add_edge(2, 4);
        graph.add_edge(3, 5);
        
        let mut count = 0;
        graph.bfs(NodeId(1), |id| {
            count += 1;
            if count >= 2 {
                // 手动实现 early exit 需要修改 visitor 签名
            }
        });
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- algorithms::bfs --nocapture
```
Expected: FAIL

**Step 3: 实现 BFS**

```rust
// crates/graph/src/algorithms/bfs.rs

use crate::{AdjacencyIndex, EdgeStore, NodeId};
use std::collections::VecDeque;

/// 广度优先搜索
pub struct Bfs<'a> {
    adjacency: &'a AdjacencyIndex,
    edge_store: &'a EdgeStore,
}

impl<'a> Bfs<'a> {
    pub fn new(adjacency: &'a AdjacencyIndex, edge_store: &'a EdgeStore) -> Self {
        Self {
            adjacency,
            edge_store,
        }
    }

    /// 执行 BFS
    pub fn traverse<F>(&self, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId),
    {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            visitor(node);

            for edge_id in self.adjacency.outgoing_edges(node) {
                if let Some(edge) = self.edge_store.get(edge_id) {
                    if !visited.contains(&edge.to) {
                        visited.insert(edge.to);
                        queue.push_back(edge.to);
                    }
                }
            }
        }
    }
}

use std::collections::HashSet;
```

**Step 4: 更新 mod.rs**

```rust
// crates/graph/src/algorithms/mod.rs

pub mod bfs;
pub mod dfs;

pub use bfs::Bfs;
pub use dfs::Dfs;
```

**Step 5: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- algorithms --nocapture
```
Expected: PASS

**Step 6: 提交**
```bash
git add crates/graph/src/algorithms/
git commit -m "feat(graph): add BFS algorithm"
```

---

### Task 11: 实现 DFS

**Files:**
- Create: `crates/graph/src/algorithms/dfs.rs`

**Step 1: 编写 DFS 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfs_simple() {
        let mut graph = TestGraph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        
        let mut visited = Vec::new();
        graph.dfs(NodeId(1), |id| visited.push(id));
        
        assert!(visited.contains(&NodeId(1)));
        assert!(visited.contains(&NodeId(2)));
        assert!(visited.contains(&NodeId(3)));
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- algorithms::dfs --nocapture
```
Expected: FAIL

**Step 3: 实现 DFS**

```rust
// crates/graph/src/algorithms/dfs.rs

use crate::{AdjacencyIndex, EdgeStore, NodeId};
use std::collections::HashSet;

/// 深度优先搜索
pub struct Dfs<'a> {
    adjacency: &'a AdjacencyIndex,
    edge_store: &'a EdgeStore,
}

impl<'a> Dfs<'a> {
    pub fn new(adjacency: &'a AdjacencyIndex, edge_store: &'a EdgeStore) -> Self {
        Self {
            adjacency,
            edge_store,
        }
    }

    /// 执行 DFS
    pub fn traverse<F>(&self, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId),
    {
        let mut visited = HashSet::new();
        self.dfs_recursive(start, &mut visitor, &mut visited);
    }

    fn dfs_recursive<F>(&self, node: NodeId, visitor: &mut F, visited: &mut HashSet<NodeId>)
    where
        F: FnMut(NodeId),
    {
        if visited.contains(&node) {
            return;
        }
        visited.insert(node);
        visitor(node);

        for edge_id in self.adjacency.outgoing_edges(node) {
            if let Some(edge) = self.edge_store.get(edge_id) {
                self.dfs_recursive(edge.to, visitor, visited);
            }
        }
    }
}
```

**Step 4: 运行测试验证通过**
```bash
cargo test --package sqlrustgo-graph -- algorithms::dfs --nocapture
```
Expected: PASS

**Step 5: 提交**
```bash
git add crates/graph/src/algorithms/dfs.rs
git commit -m "feat(graph): add DFS algorithm"
```

---

## 模块 6：GraphStore Trait + InMemoryGraphStore

### Task 12: 实现 GraphStore Trait

**Files:**
- Create: `crates/graph/src/traversal/mod.rs`
- Create: `crates/graph/src/traversal/walker.rs`
- Modify: `crates/graph/src/lib.rs`

**Step 1: 编写 GraphStore 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_node() {
        let mut graph = InMemoryGraphStore::new();
        let batch_label = graph.label_registry.get_or_create("Batch").unwrap();
        
        let batch_id = graph.create_node(batch_label, prop! {
            "batch_id" => Value::Text("B-2026-001".to_string())
        });
        
        let node = graph.get_node(batch_id);
        assert!(node.is_some());
        assert_eq!(node.unwrap().label, batch_label);
    }

    #[test]
    fn test_create_and_get_edge() {
        let mut graph = InMemoryGraphStore::new();
        let batch_label = graph.label_registry.get_or_create("Batch").unwrap();
        let device_label = graph.label_registry.get_or_create("Device").unwrap();
        
        let batch_id = graph.create_node(batch_label, PropertyMap::new());
        let device_id = graph.create_node(device_label, PropertyMap::new());
        let produced_on = graph.label_registry.get_or_create("produced_by").unwrap();
        
        let edge_id = graph.create_edge(batch_id, device_id, produced_on, PropertyMap::new());
        
        let edge = graph.get_edge(edge_id);
        assert!(edge.is_some());
    }

    #[test]
    fn test_neighbors_by_edge_label() {
        let mut graph = InMemoryGraphStore::new();
        let batch_label = graph.label_registry.get_or_create("Batch").unwrap();
        let device_label = graph.label_registry.get_or_create("Device").unwrap();
        let produced_on = graph.label_registry.get_or_create("produced_by").unwrap();
        
        let batch_id = graph.create_node(batch_label, PropertyMap::new());
        let device_id = graph.create_node(device_label, PropertyMap::new());
        
        graph.create_edge(batch_id, device_id, produced_on, PropertyMap::new());
        
        let neighbors: Vec<NodeId> = graph.neighbors_by_edge_label(batch_id, produced_on).collect();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0], device_id);
    }

    #[test]
    fn test_bfs_traversal() {
        let mut graph = InMemoryGraphStore::new();
        let node_label = graph.label_registry.get_or_create("Node").unwrap();
        
        let n1 = graph.create_node(node_label, PropertyMap::new());
        let n2 = graph.create_node(node_label, PropertyMap::new());
        let n3 = graph.create_node(node_label, PropertyMap::new());
        let edge_label = graph.label_registry.get_or_create("connects").unwrap();
        
        graph.create_edge(n1, n2, edge_label, PropertyMap::new());
        graph.create_edge(n2, n3, edge_label, PropertyMap::new());
        
        let mut visited = Vec::new();
        graph.bfs(n1, |id| visited.push(id));
        
        assert!(visited.contains(&n1));
        assert!(visited.contains(&n2));
        assert!(visited.contains(&n3));
    }

    #[test]
    fn test_gmp_batch_traceability() {
        let mut graph = InMemoryGraphStore::new();
        
        // GMP 标签
        let batch_label = graph.label_registry.get_or_create("Batch").unwrap();
        let device_label = graph.label_registry.get_or_create("Device").unwrap();
        let produced_on = graph.label_registry.get_or_create("produced_by").unwrap();
        
        // 创建节点
        let batch = graph.create_node(batch_label, prop! {
            "batch_id" => Value::Text("B-2026-001".to_string())
        });
        let device = graph.create_node(device_label, prop! {
            "device_id" => Value::Text("D-001".to_string())
        });
        
        // 创建关系
        graph.create_edge(batch, device, produced_on, PropertyMap::new());
        
        // 追溯：批次使用了哪些设备
        let devices: Vec<NodeId> = graph.neighbors_by_edge_label(batch, produced_on).collect();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], device);
        
        // 反向追溯：设备影响了哪些批次
        let batches: Vec<NodeId> = graph.incoming_neighbors(device).collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0], batch);
    }
}
```

**Step 2: 运行测试验证失败**
```bash
cargo test --package sqlrustgo-graph -- --nocapture
```
Expected: FAIL - module not found

**Step 3: 实现 Walker/Traversal**

```rust
// crates/graph/src/traversal/walker.rs

use crate::{
    AdjacencyIndex, EdgeStore, GraphStore, LabelId, LabelRegistry, LabelIndex, Node, NodeId,
    PropertyMap,
};
use std::collections::{HashSet, VecDeque};

/// 遍历原语实现
pub struct Walker<'a> {
    adjacency: &'a AdjacencyIndex,
    edge_store: &'a EdgeStore,
}

impl<'a> Walker<'a> {
    pub fn new(adjacency: &'a AdjacencyIndex, edge_store: &'a EdgeStore) -> Self {
        Self {
            adjacency,
            edge_store,
        }
    }

    pub fn outgoing_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.adjacency
            .outgoing_edges(node)
            .iter()
            .filter_map(|edge_id| self.edge_store.get(*edge_id).map(|e| e.to))
            .collect()
    }

    pub fn incoming_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.adjacency
            .incoming_edges(node)
            .iter()
            .filter_map(|edge_id| self.edge_store.get(*edge_id).map(|e| e.from))
            .collect()
    }

    pub fn neighbors_by_edge_label(&self, node: NodeId, edge_label: LabelId) -> Vec<NodeId> {
        self.adjacency
            .outgoing_edges(node)
            .iter()
            .filter_map(|edge_id| self.edge_store.get(*edge_id))
            .filter(|e| e.label == edge_label)
            .map(|e| e.to)
            .collect()
    }
}
```

**Step 4: 实现 InMemoryGraphStore**

```rust
// crates/graph/src/lib.rs

pub mod algorithms;
pub mod index;
pub mod model;
pub mod registry;
pub mod storage;
pub mod traversal;

pub use algorithms::{Bfs, Dfs};
pub use index::{AdjacencyIndex, LabelIndex};
pub use model::{prop, Edge, EdgeId, LabelId, Node, NodeId, PropertyMap};
pub use registry::LabelRegistry;
pub use storage::{EdgeStore, NodeStore};
pub use traversal::walker::Walker;

/// 图存储引擎核心 trait
pub trait GraphStore {
    // 节点操作
    fn create_node(&mut self, label: LabelId, props: PropertyMap) -> NodeId;
    fn get_node(&self, id: NodeId) -> Option<&Node>;
    fn nodes_by_label(&self, label: LabelId) -> Box<dyn Iterator<Item = NodeId> + '_>;

    // 边操作
    fn create_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: LabelId,
        props: PropertyMap,
    ) -> EdgeId;
    fn get_edge(&self, id: EdgeId) -> Option<&Edge>;

    // 遍历原语
    fn neighbors_by_edge_label(
        &self,
        node: NodeId,
        edge_label: LabelId,
    ) -> Box<dyn Iterator<Item = NodeId> + '_>;
    fn outgoing_neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;
    fn incoming_neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;

    // 算法
    fn bfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId);
    fn dfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId);
}

/// 内存图存储实现（Phase-1）
pub struct InMemoryGraphStore {
    pub node_store: NodeStore,
    pub edge_store: EdgeStore,
    pub label_registry: LabelRegistry,
    pub label_index: LabelIndex,
    pub adjacency: AdjacencyIndex,
}

impl InMemoryGraphStore {
    pub fn new() -> Self {
        let mut registry = LabelRegistry::new();
        registry.bootstrap_core_labels(&[
            "Batch", "Device", "SOP", "Step", "Deviation", "CAPA", "Regulation", "Material",
        ]);
        
        Self {
            node_store: NodeStore::new(),
            edge_store: EdgeStore::new(),
            label_registry: registry,
            label_index: LabelIndex::new(),
            adjacency: AdjacencyIndex::new(),
        }
    }
}

impl GraphStore for InMemoryGraphStore {
    fn create_node(&mut self, label: LabelId, props: PropertyMap) -> NodeId {
        let id = self.node_store.next_id();
        let node = Node::new(id, label, props);
        self.node_store.insert(node);
        self.label_index.insert(label, id);
        id
    }

    fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.node_store.get(id)
    }

    fn nodes_by_label(&self, label: LabelId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        Box::new(self.label_index.get_nodes(label).into_iter())
    }

    fn create_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: LabelId,
        props: PropertyMap,
    ) -> EdgeId {
        let id = self.edge_store.next_id();
        let edge = Edge::new(id, from, to, label, props);
        self.edge_store.insert(edge);
        self.adjacency.add_edge(from, to, id, label);
        id
    }

    fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edge_store.get(id)
    }

    fn neighbors_by_edge_label(
        &self,
        node: NodeId,
        edge_label: LabelId,
    ) -> Box<dyn Iterator<Item = NodeId> + '_> {
        let walker = Walker::new(&self.adjacency, &self.edge_store);
        Box::new(walker.neighbors_by_edge_label(node, edge_label).into_iter())
    }

    fn outgoing_neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        let walker = Walker::new(&self.adjacency, &self.edge_store);
        Box::new(walker.outgoing_neighbors(node).into_iter())
    }

    fn incoming_neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        let walker = Walker::new(&self.adjacency, &self.edge_store);
        Box::new(walker.incoming_neighbors(node).into_iter())
    }

    fn bfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId),
    {
        let bfs = Bfs::new(&self.adjacency, &self.edge_store);
        bfs.traverse(start, visitor);
    }

    fn dfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId),
    {
        let dfs = Dfs::new(&self.adjacency, &self.edge_store);
        dfs.traverse(start, visitor);
    }
}

impl Default for InMemoryGraphStore {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 5: 添加缺失导入**

在 `Cargo.toml` 中确保有：
```toml
sqlrustgo-types = { path = "../types" }
```

**Step 6: 运行测试验证**
```bash
cargo test --package sqlrustgo-graph -- --nocapture
```
Expected: PASS - 所有集成测试

**Step 7: 提交**
```bash
git add crates/graph/src/
git commit -m "feat(graph): add GraphStore trait and InMemoryGraphStore"
```

---

## 最终验证

### Task 13: 运行所有测试

**Step 1: 运行完整测试套件**
```bash
cargo test --package sqlrustgo-graph
```

**Step 2: 运行 clippy 检查**
```bash
cargo clippy --package sqlrustgo-graph -- -D warnings
```

**Step 3: 提交所有更改**
```bash
git add crates/graph/
git commit -m "feat(graph): complete Phase-1 implementation with tests"
```

---

## 附录：完整文件结构

```
crates/graph/
├── Cargo.toml
├── src/
│   ├── lib.rs              # GraphStore trait + InMemoryGraphStore
│   ├── model/
│   │   ├── mod.rs
│   │   ├── ids.rs          # NodeId, EdgeId, LabelId
│   │   ├── node.rs         # Node
│   │   ├── edge.rs         # Edge
│   │   └── property.rs     # PropertyMap
│   ├── registry/
│   │   ├── mod.rs
│   │   └── label_registry.rs  # LabelRegistry
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── node_store.rs   # NodeStore
│   │   └── edge_store.rs   # EdgeStore
│   ├── index/
│   │   ├── mod.rs
│   │   ├── adjacency_index.rs  # AdjacencyIndex
│   │   └── label_index.rs   # LabelIndex
│   ├── algorithms/
│   │   ├── mod.rs
│   │   ├── bfs.rs           # BFS
│   │   └── dfs.rs           # DFS
│   └── traversal/
│       ├── mod.rs
│       └── walker.rs        # Traversal primitives
└── tests/
    └── graph_tests.rs       # Integration tests
```
