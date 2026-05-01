//! DiskGraphStore - 持久化图存储，支持 WAL 崩溃恢复

use crate::error::GraphError;
use crate::model::*;
use crate::store::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

const WAL_FILE: &str = "graph.wal";
const NODES_FILE: &str = "nodes.json";
const EDGES_FILE: &str = "edges.json";
const LABELS_FILE: &str = "labels.json";
const META_FILE: &str = "meta.json";

/// Graph metadata for persistence
#[derive(Serialize, Deserialize, Debug)]
pub struct GraphMeta {
    /// Schema version
    pub version: String,
    /// Next available node ID
    pub next_node_id: u64,
    /// Next available edge ID
    pub next_edge_id: u64,
}

impl Default for GraphMeta {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            next_node_id: 0,
            next_edge_id: 0,
        }
    }
}

/// WAL entry types for graph operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GraphWalEntry {
    CreateNode {
        node_id: NodeId,
        label_id: LabelId,
        props: HashMap<String, PropertyValue>,
    },
    UpdateNode {
        node_id: NodeId,
        props: HashMap<String, PropertyValue>,
    },
    DeleteNode {
        node_id: NodeId,
    },
    CreateEdge {
        edge_id: EdgeId,
        from: NodeId,
        to: NodeId,
        label_id: LabelId,
        props: HashMap<String, PropertyValue>,
    },
    DeleteEdge {
        edge_id: EdgeId,
    },
}

/// Disk-based graph store with WAL recovery
pub struct DiskGraphStore {
    inner: InMemoryGraphStore,
    path: PathBuf,
    next_node_id: NodeId,
    next_edge_id: EdgeId,
    wal_enabled: bool,
    wal_path: PathBuf,
}

impl DiskGraphStore {
    /// Create a new disk-based graph store with WAL enabled
    pub fn new(base_path: PathBuf) -> Result<Self, GraphError> {
        fs::create_dir_all(&base_path).map_err(|e| GraphError::StorageError(e.to_string()))?;

        let wal_path = base_path.join(WAL_FILE);

        let store = Self {
            inner: InMemoryGraphStore::new(),
            path: base_path,
            next_node_id: NodeId::MIN,
            next_edge_id: EdgeId::MIN,
            wal_enabled: true,
            wal_path,
        };

        store.persist_meta()?;
        Ok(store)
    }

    /// Create a new disk-based graph store without WAL
    pub fn new_without_wal(base_path: PathBuf) -> Result<Self, GraphError> {
        fs::create_dir_all(&base_path).map_err(|e| GraphError::StorageError(e.to_string()))?;

        let wal_path = base_path.join(WAL_FILE);

        Ok(Self {
            inner: InMemoryGraphStore::new(),
            path: base_path,
            next_node_id: NodeId::MIN,
            next_edge_id: EdgeId::MIN,
            wal_enabled: false,
            wal_path,
        })
    }

    pub fn load(base_path: PathBuf) -> Result<Self, GraphError> {
        if !base_path.exists() {
            return Self::new(base_path);
        }

        let wal_path = base_path.join(WAL_FILE);

        let (next_node_id, next_edge_id) = Self::load_meta(&base_path)?;

        let mut inner = InMemoryGraphStore::new();
        Self::load_nodes(&base_path, &mut inner)?;
        Self::load_edges(&base_path, &mut inner)?;
        Self::load_labels(&base_path, &mut inner)?;

        let mut store = Self {
            inner,
            path: base_path,
            next_node_id: NodeId(next_node_id),
            next_edge_id: EdgeId(next_edge_id),
            wal_enabled: true,
            wal_path,
        };

        store.replay_wal()?;

        Ok(store)
    }

    pub fn set_wal_enabled(&mut self, enabled: bool) {
        self.wal_enabled = enabled;
    }

    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
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

    fn load_meta(base_path: &Path) -> Result<(u64, u64), GraphError> {
        let meta_path = base_path.join(META_FILE);
        if meta_path.exists() {
            let data = fs::read(&meta_path).map_err(|e| GraphError::StorageError(e.to_string()))?;
            let meta: GraphMeta = serde_json::from_slice(&data)
                .map_err(|e| GraphError::StorageError(e.to_string()))?;
            Ok((meta.next_node_id, meta.next_edge_id))
        } else {
            Ok((0, 0))
        }
    }

    fn persist_meta(&self) -> Result<(), GraphError> {
        let meta = GraphMeta {
            version: "1.0".to_string(),
            next_node_id: self.next_node_id.0,
            next_edge_id: self.next_edge_id.0,
        };
        let data = serde_json::to_vec_pretty(&meta)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        fs::write(self.path.join(META_FILE), data)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn load_nodes(base_path: &Path, inner: &mut InMemoryGraphStore) -> Result<(), GraphError> {
        let nodes_path = base_path.join(NODES_FILE);
        if nodes_path.exists() {
            let data =
                fs::read(&nodes_path).map_err(|e| GraphError::StorageError(e.to_string()))?;
            #[derive(Deserialize)]
            struct NodeData {
                id: u64,
                label_id: u32,
                props: HashMap<String, PropertyValue>,
            }

            let nodes: Vec<NodeData> = serde_json::from_slice(&data)
                .map_err(|e| GraphError::StorageError(e.to_string()))?;
            for node_data in nodes {
                let mut props = PropertyMap::new();
                for (k, v) in node_data.props {
                    props.insert(k, v);
                }
                let node = Node::new(NodeId(node_data.id), LabelId(node_data.label_id), props);
                let node_id = node.id;
                let node_label = node.label;
                inner.nodes.insert(node);
                inner.label_index.add_node(node_id, node_label);
            }
        }
        Ok(())
    }

    fn persist_nodes(&self) -> Result<(), GraphError> {
        #[derive(Serialize)]
        struct NodeData {
            id: u64,
            label_id: u32,
            props: HashMap<String, PropertyValue>,
        }

        let nodes: Vec<NodeData> = self
            .inner
            .nodes
            .ids()
            .iter()
            .filter_map(|id| self.inner.nodes.get(*id))
            .map(|node| {
                let mut props = HashMap::new();
                for (k, v) in node.properties.iter() {
                    props.insert(k.clone(), v.clone());
                }
                NodeData {
                    id: node.id.0,
                    label_id: node.label.0,
                    props,
                }
            })
            .collect();

        let data = serde_json::to_vec_pretty(&nodes)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        fs::write(self.path.join(NODES_FILE), data)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn load_edges(base_path: &Path, inner: &mut InMemoryGraphStore) -> Result<(), GraphError> {
        let edges_path = base_path.join(EDGES_FILE);
        if edges_path.exists() {
            let data =
                fs::read(&edges_path).map_err(|e| GraphError::StorageError(e.to_string()))?;
            #[derive(Deserialize)]
            struct EdgeData {
                id: u64,
                from: u64,
                to: u64,
                label_id: u32,
                props: HashMap<String, PropertyValue>,
                direction: Direction,
            }

            let edges: Vec<EdgeData> = serde_json::from_slice(&data)
                .map_err(|e| GraphError::StorageError(e.to_string()))?;
            for edge_data in edges {
                let mut props = PropertyMap::new();
                for (k, v) in edge_data.props {
                    props.insert(k, v);
                }
                let edge = Edge {
                    id: EdgeId(edge_data.id),
                    from: NodeId(edge_data.from),
                    to: NodeId(edge_data.to),
                    label: LabelId(edge_data.label_id),
                    properties: props,
                    direction: edge_data.direction,
                };
                inner.edges.insert(edge.clone());
                inner
                    .adjacency
                    .add_edge(edge.from, edge.to, edge.label, edge.id);
                inner.label_index.add_edge(edge.id, edge.label);
            }
        }
        Ok(())
    }

    fn persist_edges(&self) -> Result<(), GraphError> {
        #[derive(Serialize)]
        struct EdgeData {
            id: u64,
            from: u64,
            to: u64,
            label_id: u32,
            props: HashMap<String, PropertyValue>,
            direction: Direction,
        }

        let edges: Vec<EdgeData> = self
            .inner
            .edges
            .ids()
            .iter()
            .filter_map(|id| self.inner.edges.get(*id))
            .map(|edge| {
                let mut props = HashMap::new();
                for (k, v) in edge.properties.iter() {
                    props.insert(k.clone(), v.clone());
                }
                EdgeData {
                    id: edge.id.0,
                    from: edge.from.0,
                    to: edge.to.0,
                    label_id: edge.label.0,
                    props,
                    direction: edge.direction,
                }
            })
            .collect();

        let data = serde_json::to_vec_pretty(&edges)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        fs::write(self.path.join(EDGES_FILE), data)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn load_labels(base_path: &Path, inner: &mut InMemoryGraphStore) -> Result<(), GraphError> {
        let labels_path = base_path.join(LABELS_FILE);
        if labels_path.exists() {
            let data =
                fs::read(&labels_path).map_err(|e| GraphError::StorageError(e.to_string()))?;
            #[derive(Deserialize)]
            struct LabelEntry {
                label: String,
                id: u32,
            }

            let labels: Vec<LabelEntry> = serde_json::from_slice(&data)
                .map_err(|e| GraphError::StorageError(e.to_string()))?;
            for entry in labels {
                inner
                    .labels
                    .string_to_id
                    .insert(entry.label.clone(), LabelId(entry.id));
                inner
                    .labels
                    .id_to_string
                    .insert(LabelId(entry.id), entry.label);
                let next = entry.id + 1;
                if next > inner.labels.next_id.0 {
                    inner.labels.next_id = LabelId(next);
                }
            }
        }
        Ok(())
    }

    fn persist_labels(&self) -> Result<(), GraphError> {
        #[derive(Serialize)]
        struct LabelEntry {
            label: String,
            id: u32,
        }

        let labels: Vec<LabelEntry> = self
            .inner
            .labels
            .iter()
            .map(|(id, label)| LabelEntry {
                label: label.clone(),
                id: id.0,
            })
            .collect();

        let data = serde_json::to_vec_pretty(&labels)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        fs::write(self.path.join(LABELS_FILE), data)
            .map_err(|e| GraphError::StorageError(e.to_string()))?;
        Ok(())
    }

    fn log_wal_entry(&self, entry: &GraphWalEntry) -> Result<(), GraphError> {
        if !self.wal_enabled {
            return Ok(());
        }

        let data = serde_json::to_vec(entry)
            .map_err(|e| GraphError::StorageError(format!("JSON serialization error: {}", e)))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)
            .map_err(|e| GraphError::StorageError(format!("File open error: {}", e)))?;

        use std::io::Write;
        writeln!(file, "{}", data.len())
            .map_err(|e| GraphError::StorageError(format!("Write error: {}", e)))?;
        file.write_all(&data)
            .map_err(|e| GraphError::StorageError(format!("Write error: {}", e)))?;

        file.sync_all()
            .map_err(|e| GraphError::StorageError(format!("Sync error: {}", e)))?;

        Ok(())
    }

    fn replay_wal(&mut self) -> Result<(), GraphError> {
        if !self.wal_path.exists() {
            return Ok(());
        }

        let data = fs::read(&self.wal_path).map_err(|e| GraphError::StorageError(e.to_string()))?;

        for line in data.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }

            if String::from_utf8_lossy(line).parse::<usize>().is_ok() {
                continue;
            }

            if let Ok(entry) = serde_json::from_slice::<GraphWalEntry>(line) {
                self.apply_wal_entry(&entry)?;
            }
        }

        Ok(())
    }

    fn apply_wal_entry(&mut self, entry: &GraphWalEntry) -> Result<(), GraphError> {
        match entry {
            GraphWalEntry::CreateNode {
                node_id,
                label_id,
                props,
            } => {
                let mut p = PropertyMap::new();
                for (k, v) in props {
                    p.insert(k.clone(), v.clone());
                }
                let node = Node::new(*node_id, *label_id, p);
                self.inner.nodes.insert(node);
                self.inner.label_index.add_node(*node_id, *label_id);
            }
            GraphWalEntry::UpdateNode { node_id, props } => {
                if let Some(node) = self.inner.nodes.get(*node_id) {
                    let mut node_mut = node.clone();
                    let mut p = PropertyMap::new();
                    for (k, v) in props {
                        p.insert(k.clone(), v.clone());
                    }
                    node_mut.properties.extend(p);
                    self.inner.nodes.insert(node_mut);
                }
            }
            GraphWalEntry::DeleteNode { node_id } => {
                let _ = self.inner.delete_node(*node_id);
            }
            GraphWalEntry::CreateEdge {
                edge_id,
                from,
                to,
                label_id,
                props,
            } => {
                let mut p = PropertyMap::new();
                for (k, v) in props {
                    p.insert(k.clone(), v.clone());
                }
                let edge = Edge::new(*edge_id, *from, *to, *label_id, p);
                self.inner.edges.insert(edge.clone());
                self.inner
                    .adjacency
                    .add_edge(*from, *to, *label_id, *edge_id);
                self.inner.label_index.add_edge(*edge_id, *label_id);
            }
            GraphWalEntry::DeleteEdge { edge_id } => {
                let _ = self.inner.delete_edge(*edge_id);
            }
        }
        Ok(())
    }
}

impl GraphStore for DiskGraphStore {
    fn create_node(&mut self, label: &str, props: PropertyMap) -> NodeId {
        let node_id = self.next_node_id();
        let label_id = self.inner.labels.get_or_register(label);
        let node = Node::new(node_id, label_id, props.clone());

        let mut props_map = HashMap::new();
        for (k, v) in props.iter() {
            props_map.insert(k.clone(), v.clone());
        }
        let entry = GraphWalEntry::CreateNode {
            node_id,
            label_id,
            props: props_map,
        };
        let _ = self.log_wal_entry(&entry);

        self.inner.nodes.insert(node);
        self.inner.label_index.add_node(node_id, label_id);

        let _ = self.persist_nodes();
        let _ = self.persist_labels();
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
        let mut props_map = HashMap::new();
        for (k, v) in props.iter() {
            props_map.insert(k.clone(), v.clone());
        }
        let entry = GraphWalEntry::UpdateNode {
            node_id: id,
            props: props_map,
        };
        let _ = self.log_wal_entry(&entry);

        self.inner.update_node(id, props)?;
        self.persist_nodes()?;
        Ok(())
    }

    fn delete_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        let entry = GraphWalEntry::DeleteNode { node_id: id };
        let _ = self.log_wal_entry(&entry);

        self.inner.delete_node(id)?;
        self.persist_nodes()?;
        Ok(())
    }

    fn create_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: &str,
        props: PropertyMap,
    ) -> Result<EdgeId, GraphError> {
        if !self.inner.nodes.contains(from) {
            return Err(GraphError::NodeNotFound(from));
        }
        if !self.inner.nodes.contains(to) {
            return Err(GraphError::NodeNotFound(to));
        }

        let edge_id = self.next_edge_id();
        let label_id = self.inner.labels.get_or_register(label);
        let edge = Edge::new(edge_id, from, to, label_id, props.clone());

        let mut props_map = HashMap::new();
        for (k, v) in props.iter() {
            props_map.insert(k.clone(), v.clone());
        }
        let entry = GraphWalEntry::CreateEdge {
            edge_id,
            from,
            to,
            label_id,
            props: props_map,
        };
        let _ = self.log_wal_entry(&entry);

        self.inner.edges.insert(edge.clone());
        self.inner.adjacency.add_edge(from, to, label_id, edge_id);
        self.inner.label_index.add_edge(edge_id, label_id);

        let _ = self.persist_edges();
        let _ = self.persist_labels();
        let _ = self.persist_meta();

        Ok(edge_id)
    }

    fn get_edge(&self, id: EdgeId) -> Option<Edge> {
        self.inner.get_edge(id)
    }

    fn edges_by_label(&self, label: &str) -> Vec<EdgeId> {
        self.inner.edges_by_label(label)
    }

    fn delete_edge(&mut self, id: EdgeId) -> Result<(), GraphError> {
        let entry = GraphWalEntry::DeleteEdge { edge_id: id };
        let _ = self.log_wal_entry(&entry);

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_disk_graph_store_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        let store = DiskGraphStore::new(path.clone());
        assert!(store.is_ok());
        assert!(path.join(META_FILE).exists());
    }

    #[test]
    fn test_create_and_get_node() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        let mut props = PropertyMap::new();
        props.insert("name", "batch-001");

        let node_id = store.create_node("Batch", props);
        assert!(store.get_node(node_id).is_some());
        assert_eq!(store.node_count(), 1);
    }

    #[test]
    fn test_persist_and_reload() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        {
            let mut store = DiskGraphStore::new(path.clone()).unwrap();
            let mut props = PropertyMap::new();
            props.insert("name", "batch-001");
            store.create_node("Batch", props);

            let mut props2 = PropertyMap::new();
            props2.insert("name", "device-001");
            store.create_node("Device", props2);
        }

        let store2 = DiskGraphStore::load(path).unwrap();
        assert_eq!(store2.node_count(), 2);
        assert!(store2.label_registry().get("Batch").is_some());
        assert!(store2.label_registry().get("Device").is_some());
    }

    #[test]
    fn test_edge_crud() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        let batch = store.create_node("Batch", PropertyMap::new());
        let device = store.create_node("Device", PropertyMap::new());

        let edge_id = store
            .create_edge(batch, device, "produced_by", PropertyMap::new())
            .unwrap();

        assert!(store.get_edge(edge_id).is_some());
        assert_eq!(store.edge_count(), 1);
    }

    #[test]
    fn test_delete_node_removes_edges() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        let batch = store.create_node("Batch", PropertyMap::new());
        let device = store.create_node("Device", PropertyMap::new());
        store
            .create_edge(batch, device, "produced_by", PropertyMap::new())
            .unwrap();

        assert_eq!(store.edge_count(), 1);

        store.delete_node(batch).unwrap();
        assert_eq!(store.edge_count(), 0);
    }

    #[test]
    fn test_update_node() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        let mut props = PropertyMap::new();
        props.insert("name", "original");
        let node_id = store.create_node("Batch", props);

        let mut new_props = PropertyMap::new();
        new_props.insert("name", "updated");
        store.update_node(node_id, new_props).unwrap();

        let node = store.get_node(node_id).unwrap();
        assert_eq!(
            node.get_property("name").unwrap().as_string(),
            Some(&"updated".to_string())
        );
    }

    #[test]
    fn test_nodes_by_label() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        store.create_node("Batch", PropertyMap::new());
        store.create_node("Batch", PropertyMap::new());
        store.create_node("Device", PropertyMap::new());

        let batch_nodes = store.nodes_by_label("Batch");
        assert_eq!(batch_nodes.len(), 2);

        let device_nodes = store.nodes_by_label("Device");
        assert_eq!(device_nodes.len(), 1);
    }

    #[test]
    fn test_neighbors() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        let batch = store.create_node("Batch", PropertyMap::new());
        let device1 = store.create_node("Device", PropertyMap::new());
        let device2 = store.create_node("Device", PropertyMap::new());

        store
            .create_edge(batch, device1, "produced_by", PropertyMap::new())
            .unwrap();
        store
            .create_edge(batch, device2, "produced_by", PropertyMap::new())
            .unwrap();

        let neighbors = store.outgoing_neighbors(batch);
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_bfs_traversal() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        let a = store.create_node("A", PropertyMap::new());
        let b = store.create_node("B", PropertyMap::new());
        let c = store.create_node("C", PropertyMap::new());

        store.create_edge(a, b, "link", PropertyMap::new()).unwrap();
        store.create_edge(b, c, "link", PropertyMap::new()).unwrap();

        let mut visited = Vec::new();
        store.bfs(a, |node_id| {
            visited.push(node_id);
            true
        });

        assert!(visited.contains(&a));
        assert!(visited.contains(&b));
        assert!(visited.contains(&c));
    }

    #[test]
    fn test_dfs_traversal() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        let a = store.create_node("A", PropertyMap::new());
        let b = store.create_node("B", PropertyMap::new());
        let c = store.create_node("C", PropertyMap::new());

        store.create_edge(a, b, "link", PropertyMap::new()).unwrap();
        store.create_edge(b, c, "link", PropertyMap::new()).unwrap();

        let mut visited = Vec::new();
        store.dfs(a, |node_id| {
            visited.push(node_id);
            true
        });

        assert!(visited.contains(&a));
        assert!(visited.contains(&b));
        assert!(visited.contains(&c));
    }

    #[test]
    fn test_without_wal_mode() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new_without_wal(dir.path().to_path_buf()).unwrap();

        store.create_node("Batch", PropertyMap::new());
        assert_eq!(store.node_count(), 1);
    }

    #[test]
    fn test_wal_disabled() {
        let dir = tempdir().unwrap();
        let mut store = DiskGraphStore::new(dir.path().to_path_buf()).unwrap();

        store.set_wal_enabled(false);
        store.create_node("Batch", PropertyMap::new());

        assert!(!store.wal_path.exists() || store.wal_path.metadata().unwrap().len() == 0);
    }
}
