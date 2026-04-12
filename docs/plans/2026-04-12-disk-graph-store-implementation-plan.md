# DiskGraphStore Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 graph 模块实现 DiskGraphStore，支持 WAL 持久化和崩溃恢复

**Architecture:** 装饰器模式 - DiskGraphStore 包装 InMemoryGraphStore，写操作通过 WAL 持久化，读操作直接走内存

**Tech Stack:** Rust, bincode, WalManager (existing)

---

## Task 1: 创建 disk_graph_store.rs 基础结构

**Files:**
- Create: `crates/graph/src/store/disk_graph_store.rs`
- Modify: `crates/graph/src/store/mod.rs`
- Test: `crates/graph/tests/graph_persistence_tests.rs`

**Step 1: Write the failing test**

```rust
// crates/graph/tests/graph_persistence_tests.rs
#[cfg(test)]
mod basic_persistence_test {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_disk_graph_store_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        
        let store = DiskGraphStore::new(path.clone());
        assert!(store.is_ok());
    }
    
    #[test]
    fn test_in_memory_graph_store_still_works() {
        let store = InMemoryGraphStore::new();
        let node_id = store.create_node("Batch", PropertyMap::new());
        assert!(store.get_node(node_id).is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-graph --test graph_persistence_tests -- --nocapture`
Expected: FAIL - test_disk_graph_store_creation fails because DiskGraphStore doesn't exist

**Step 3: Write minimal structure**

```rust
// crates/graph/src/store/disk_graph_store.rs
use crate::model::*;
use crate::store::*;
use crate::error::GraphError;
use std::path::PathBuf;

pub struct DiskGraphStore {
    inner: InMemoryGraphStore,
    path: PathBuf,
}

impl DiskGraphStore {
    pub fn new(base_path: PathBuf) -> Result<Self, GraphError> {
        Ok(DiskGraphStore {
            inner: InMemoryGraphStore::new(),
            path: base_path,
        })
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-graph --test graph_persistence_tests -- --nocapture`
Expected: test_disk_graph_store_creation PASS

**Step 5: Commit**

```bash
git add crates/graph/src/store/disk_graph_store.rs crates/graph/src/store/mod.rs crates/graph/tests/graph_persistence_tests.rs
git commit -m "feat(graph): add DiskGraphStore skeleton structure"
```

---

## Task 2: 实现构造函数和文件 I/O

**Step 1: Write failing test**

```rust
#[test]
fn test_persist_and_reload_nodes() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    
    // Create and add node
    let mut store = DiskGraphStore::new(path.clone()).unwrap();
    let node_id = store.create_node("Batch", PropertyMap::new());
    
    // Reload
    let store2 = DiskGraphStore::load(path).unwrap();
    assert!(store2.get_node(node_id).is_some());
}
```

**Step 2: Run test - Expected FAIL**

Run: `cargo test -p sqlrustgo-graph test_persist_and_reload_nodes -- --nocapture`
Expected: FAIL - DiskGraphStore::load doesn't exist

**Step 3: Write DiskGraphStore with full structure**

```rust
use crate::model::*;
use crate::store::*;
use crate::error::GraphError;
use std::path::PathBuf;
use std::fs;
use bincode;

const NODES_FILE: &str = "nodes.bin";
const EDGES_FILE: &str = "edges.bin";
const LABELS_FILE: &str = "labels.bin";
const ADJACENCY_FILE: &str = "adjacency.bin";
const META_FILE: &str = "meta.json";

#[derive(Serialize, Deserialize)]
pub struct GraphMeta {
    pub version: String,
    pub next_node_id: u64,
    pub next_edge_id: u64,
}

pub struct DiskGraphStore {
    inner: InMemoryGraphStore,
    path: PathBuf,
    next_node_id: NodeId,
    next_edge_id: EdgeId,
}

impl DiskGraphStore {
    pub fn new(base_path: PathBuf) -> Result<Self, GraphError> {
        // Create directory structure
        fs::create_dir_all(&base_path)?;
        
        let mut store = DiskGraphStore {
            inner: InMemoryGraphStore::new(),
            path: base_path,
            next_node_id: NodeId::MIN,
            next_edge_id: EdgeId::MIN,
        };
        
        // Save initial meta
        store.persist_meta()?;
        Ok(store)
    }
    
    pub fn load(base_path: PathBuf) -> Result<Self, GraphError> {
        // Check if directory exists
        if !base_path.exists() {
            return Self::new(base_path);
        }
        
        // Load meta
        let meta_path = base_path.join(META_FILE);
        let meta: GraphMeta = if meta_path.exists() {
            let data = fs::read(&meta_path)?;
            serde_json::from_slice(&data)?
        } else {
            GraphMeta {
                version: "1.0".to_string(),
                next_node_id: 0,
                next_edge_id: 0,
            }
        };
        
        // Load components (simplified - will be expanded)
        let mut inner = InMemoryGraphStore::new();
        
        Ok(DiskGraphStore {
            inner,
            path: base_path,
            next_node_id: NodeId(meta.next_node_id),
            next_edge_id: EdgeId(meta.next_edge_id),
        })
    }
    
    fn persist_meta(&self) -> Result<(), GraphError> {
        let meta = GraphMeta {
            version: "1.0".to_string(),
            next_node_id: self.next_node_id.0,
            next_edge_id: self.next_edge_id.0,
        };
        let data = serde_json::to_vec(&meta)?;
        fs::write(self.path.join(META_FILE), data)?;
        Ok(())
    }
    
    fn next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id = NodeId(self.next_node_id.0 + 1);
        id
    }
    
    fn next_edge_id(&mut self) -> EdgeId {
        let id = self.next_edge_id;
        self.next_edge_id = EdgeId(self.next_edge_id.0 + 1);
        id
    }
}
```

**Step 4: Run test - Expected FAIL**

Run: `cargo test -p sqlrustgo-graph test_persist_and_reload_nodes -- --nocapture`
Expected: FAIL - load() doesn't reload node data

**Step 5: Write full load/persist implementation**

```rust
impl DiskGraphStore {
    fn persist_nodes(&self) -> Result<(), GraphError> {
        let data = bincode::serialize(&self.inner.nodes).map_err(|e| GraphError::StorageError(e.to_string()))?;
        fs::write(self.path.join(NODES_FILE), data)?;
        Ok(())
    }
    
    fn load_nodes(&mut self) -> Result<(), GraphError> {
        let path = self.path.join(NODES_FILE);
        if path.exists() {
            let data = fs::read(&path)?;
            let nodes: NodeStore = bincode::deserialize(&data).map_err(|e| GraphError::StorageError(e.to_string()))?;
            self.inner.nodes = nodes;
        }
        Ok(())
    }
}
```

**Step 6: Run test - Expected PASS**

Run: `cargo test -p sqlrustgo-graph test_persist_and_reload_nodes -- --nocapture`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/graph/src/store/disk_graph_store.rs
git commit -m "feat(graph): add DiskGraphStore constructor and file I/O"
```

---

## Task 3: 实现 GraphStore Trait

**Step 1: Write failing test**

```rust
#[test]
fn test_create_and_get_node() {
    let dir = tempdir().unwrap();
    let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();
    
    let mut props = PropertyMap::new();
    props.insert("name", "batch-001");
    
    let node_id = store.create_node("Batch", props);
    assert!(store.get_node(node_id).is_some());
}
```

**Step 2: Run test - Expected FAIL**

Run: `cargo test -p sqlrustgo-graph test_create_and_get_node -- --nocapture`
Expected: FAIL - create_node not implemented

**Step 3: Implement GraphStore trait**

```rust
impl GraphStore for DiskGraphStore {
    fn create_node(&mut self, label: &str, props: PropertyMap) -> NodeId {
        let node_id = self.next_node_id();
        let label_id = self.inner.labels.get_or_register(label);
        let node = Node::new(node_id, label_id, props);
        self.inner.nodes.insert(node);
        self.inner.label_index.add_node(node_id, label_id);
        
        // Persist
        let _ = self.persist_nodes();
        let _ = self.persist_meta();
        
        node_id
    }
    
    fn get_node(&self, id: NodeId) -> Option<Node> {
        self.inner.get_node(id)
    }
    
    fn nodes_by_label(&self, label: &str) -> Vec<NodeId> {
        self.inner.nodes_by_label(label)
    }
    
    fn update_node(&mut self, id: NodeId, props: PropertyMap) -> Result<(), GraphError> {
        self.inner.update_node(id, props)?;
        self.persist_nodes()?;
        Ok(())
    }
    
    fn delete_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        self.inner.delete_node(id)?;
        self.persist_nodes()?;
        Ok(())
    }
    
    fn create_edge(&mut self, from: NodeId, to: NodeId, label: &str, props: PropertyMap) -> Result<EdgeId, GraphError> {
        let edge_id = self.next_edge_id();
        let label_id = self.inner.labels.get_or_register(label);
        
        if !self.inner.nodes.contains(from) {
            return Err(GraphError::NodeNotFound(from));
        }
        if !self.inner.nodes.contains(to) {
            return Err(GraphError::NodeNotFound(to));
        }
        
        let edge = Edge::new(edge_id, from, to, label_id, props);
        self.inner.edges.insert(edge.clone());
        self.inner.adjacency.add_edge(from, to, label_id, edge_id);
        self.inner.label_index.add_edge(edge_id, label_id);
        
        self.persist_edges()?;
        self.persist_meta()?;
        
        Ok(edge_id)
    }
    
    fn get_edge(&self, id: EdgeId) -> Option<Edge> {
        self.inner.get_edge(id)
    }
    
    fn edges_by_label(&self, label: &str) -> Vec<EdgeId> {
        self.inner.edges_by_label(label)
    }
    
    fn delete_edge(&mut self, id: EdgeId) -> Result<(), GraphError> {
        self.inner.delete_edge(id)?;
        self.persist_edges()?;
        Ok(())
    }
    
    fn outgoing_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.inner.outgoing_neighbors(node)
    }
    
    fn incoming_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.inner.incoming_neighbors(node)
    }
    
    fn neighbors_by_edge_label(&self, node: NodeId, edge_label: &str) -> Vec<NodeId> {
        self.inner.neighbors_by_edge_label(node, edge_label)
    }
    
    fn bfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId) -> bool,
    {
        self.inner.bfs(start, visitor)
    }
    
    fn dfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId) -> bool,
    {
        self.inner.dfs(start, visitor)
    }
    
    fn node_count(&self) -> usize {
        self.inner.node_count()
    }
    
    fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }
    
    fn label_registry(&self) -> &LabelRegistry {
        self.inner.label_registry()
    }
}
```

**Step 4: Add missing persist methods**

```rust
impl DiskGraphStore {
    fn persist_edges(&self) -> Result<(), GraphError> {
        let data = bincode::serialize(&self.inner.edges).map_err(|e| GraphError::StorageError(e.to_string()))?;
        fs::write(self.path.join(EDGES_FILE), data)?;
        Ok(())
    }
    
    fn load_edges(&mut self) -> Result<(), GraphError> {
        let path = self.path.join(EDGES_FILE);
        if path.exists() {
            let data = fs::read(&path)?;
            let edges: EdgeStore = bincode::deserialize(&data).map_err(|e| GraphError::StorageError(e.to_string()))?;
            self.inner.edges = edges;
        }
        Ok(())
    }
}
```

**Step 5: Run test - Expected PASS**

Run: `cargo test -p sqlrustgo-graph test_create_and_get_node -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/graph/src/store/disk_graph_store.rs
git commit -m "feat(graph): implement GraphStore trait for DiskGraphStore"
```

---

## Task 4: 实现 WAL 集成

**Step 1: Write failing test**

```rust
#[test]
fn test_wal_logging() {
    let dir = tempdir().unwrap();
    let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();
    
    // Enable WAL
    store.set_wal_enabled(true);
    
    // Create node (should be logged)
    let node_id = store.create_node("Batch", PropertyMap::new());
    
    // Simulate crash and recover
    let recovered = DiskGraphStore::load_recovery(dir.path().to_path_buf()).unwrap();
    assert!(recovered.get_node(node_id).is_some());
}
```

**Step 2: Define GraphWalEntry**

```rust
#[derive(Serialize, Deserialize, Clone)]
pub enum GraphWalEntry {
    CreateNode { node_id: NodeId, label_id: LabelId, props: PropertyMap },
    UpdateNode { node_id: NodeId, props: PropertyMap },
    DeleteNode { node_id: NodeId },
    CreateEdge { edge_id: EdgeId, from: NodeId, to: NodeId, label_id: LabelId, props: PropertyMap },
    DeleteEdge { edge_id: EdgeId },
}
```

**Step 3: Add WAL fields and integration**

```rust
use crate::wal::{WalManager, WalEntry, WalEntryType};

pub struct DiskGraphStore {
    inner: InMemoryGraphStore,
    path: PathBuf,
    wal: WalManager,
    wal_enabled: bool,
    next_node_id: NodeId,
    next_edge_id: EdgeId,
}

impl DiskGraphStore {
    pub fn new(base_path: PathBuf) -> Result<Self, GraphError> {
        fs::create_dir_all(&base_path)?;
        
        let wal_path = base_path.join("wal");
        let wal = WalManager::new(wal_path);
        
        let mut store = DiskGraphStore {
            inner: InMemoryGraphStore::new(),
            path: base_path,
            wal,
            wal_enabled: true,
            next_node_id: NodeId::MIN,
            next_edge_id: EdgeId::MIN,
        };
        
        store.persist_meta()?;
        Ok(store)
    }
    
    pub fn set_wal_enabled(&mut self, enabled: bool) {
        self.wal_enabled = enabled;
    }
    
    fn log_wal_entry(&self, entry: GraphWalEntry) -> Result<(), GraphError> {
        if !self.wal_enabled {
            return Ok(());
        }
        let bytes = bincode::serialize(&entry).map_err(|e| GraphError::StorageError(e.to_string()))?;
        self.wal.log(WalEntryType::Insert, 0, bytes)?;
        self.wal.sync()?;
        Ok(())
    }
}
```

**Step 4: Integrate WAL into create_node**

```rust
fn create_node(&mut self, label: &str, props: PropertyMap) -> NodeId {
    let node_id = self.next_node_id();
    let label_id = self.inner.labels.get_or_register(label);
    let node = Node::new(node_id, label_id, props.clone());
    
    // Log to WAL before applying
    if self.wal_enabled {
        let entry = GraphWalEntry::CreateNode {
            node_id,
            label_id,
            props,
        };
        let _ = self.log_wal_entry(entry);
    }
    
    // Apply to memory
    self.inner.nodes.insert(node);
    self.inner.label_index.add_node(node_id, label_id);
    
    // Persist
    let _ = self.persist_nodes();
    let _ = self.persist_meta();
    
    node_id
}
```

**Step 5: Implement recover method**

```rust
pub fn load_recovery(base_path: PathBuf) -> Result<Self, GraphError> {
    let mut store = Self::new(base_path.clone())?;
    
    // Load components
    store.load_nodes()?;
    store.load_edges()?;
    
    // Replay WAL
    let wal_path = base_path.join("wal");
    if wal_path.exists() {
        let entries = store.wal.recover()?;
        for entry in entries {
            store.replay_wal_entry(entry)?;
        }
    }
    
    Ok(store)
}

fn replay_wal_entry(&mut self, bytes: Vec<u8>) -> Result<(), GraphError> {
    let entry: GraphWalEntry = bincode::deserialize(&bytes)
        .map_err(|e| GraphError::StorageError(e.to_string()))?;
    
    match entry {
        GraphWalEntry::CreateNode { node_id, label_id, props } => {
            let node = Node::new(node_id, label_id, props);
            self.inner.nodes.insert(node);
            self.inner.label_index.add_node(node_id, label_id);
        }
        GraphWalEntry::DeleteNode { node_id } => {
            self.inner.delete_node(node_id)?;
        }
        // ... handle other cases
    }
    Ok(())
}
```

**Step 6: Run test - Expected PASS**

**Step 7: Commit**

```bash
git add crates/graph/src/store/disk_graph_store.rs
git commit -m "feat(graph): add WAL integration to DiskGraphStore"
```

---

## Task 5: 完善 crash recovery 和测试

**Step 1: Write comprehensive tests**

```rust
#[test]
fn test_full_crud_persistence() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    
    // Create graph
    let mut store = DiskGraphStore::new(path.clone()).unwrap();
    let batch = store.create_node("Batch", PropertyMap::new());
    let device = store.create_node("Device", PropertyMap::new());
    store.create_edge(batch, device, "produced_by", PropertyMap::new()).unwrap();
    
    // Reload
    let store2 = DiskGraphStore::load(path).unwrap();
    assert_eq!(store2.node_count(), 2);
    assert_eq!(store2.edge_count(), 1);
}

#[test]
fn test_label_registry_persistence() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    
    let mut store = DiskGraphStore::new(path.clone()).unwrap();
    store.create_node("Batch", PropertyMap::new());
    store.create_node("Device", PropertyMap::new());
    
    let store2 = DiskGraphStore::load(path).unwrap();
    assert!(store2.label_registry().get("Batch").is_some());
    assert!(store2.label_registry().get("Device").is_some());
}
```

**Step 2: Run all tests**

Run: `cargo test -p sqlrustgo-graph -- --nocapture`

**Step 3: Run clippy**

Run: `cargo clippy -p sqlrustgo-graph -- -D warnings`

**Step 4: Commit**

```bash
git add -A
git commit -m "feat(graph): add DiskGraphStore with WAL persistence and crash recovery

- Implement DiskGraphStore wrapping InMemoryGraphStore
- Add WAL logging for all write operations
- Add crash recovery with WAL replay
- Add comprehensive persistence tests

Closes #1378"
```

---

## 执行选项

**1. Subagent-Driven (本会话)** - 每个任务派生子代理，快速迭代
**2. Parallel Session (新会话)** - 在新会话中使用 executing-plans，批量执行带检查点

选择哪个方式？
