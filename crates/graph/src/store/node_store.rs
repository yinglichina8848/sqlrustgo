//! NodeStore - in-memory storage for nodes

use crate::model::{LabelId, Node, NodeId};
use dashmap::DashMap;

/// Node storage
#[derive(Clone, Default)]
pub struct NodeStore {
    /// Node data indexed by NodeId
    nodes: DashMap<NodeId, Node>,
    /// Index of node labels
    label_index: DashMap<LabelId, Vec<NodeId>>,
}

impl NodeStore {
    pub fn new() -> Self {
        NodeStore {
            nodes: DashMap::new(),
            label_index: DashMap::new(),
        }
    }

    /// Insert a node
    pub fn insert(&self, node: Node) {
        let node_id = node.id;
        let label = node.label;
        self.nodes.insert(node_id, node);
        // Update label index
        let mut entry = self.label_index.entry(label).or_default();
        if !entry.contains(&node_id) {
            entry.push(node_id);
        }
    }

    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(&id).map(|entry| entry.clone())
    }

    /// Get all nodes with a specific label
    pub fn get_by_label(&self, label: LabelId) -> Vec<NodeId> {
        self.label_index
            .get(&label)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get all node IDs
    pub fn ids(&self) -> Vec<NodeId> {
        self.nodes.iter().map(|r| *r.key()).collect()
    }

    /// Get total node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Check if node exists
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Remove a node by ID
    pub fn remove(&self, id: NodeId) -> Option<Node> {
        if let Some((_, node)) = self.nodes.remove(&id) {
            // Remove from label index
            if let Some(mut list) = self.label_index.get_mut(&node.label) {
                list.retain(|&nid| nid != id);
            }
            Some(node)
        } else {
            None
        }
    }

    /// Clear all nodes
    pub fn clear(&self) {
        self.nodes.clear();
        self.label_index.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: NodeId, label: LabelId, name: &str) -> Node {
        let mut props = crate::model::PropertyMap::new();
        props.insert("name", name);
        Node::new(id, label, props)
    }

    #[test]
    fn test_node_store_insert_get() {
        let store = NodeStore::new();
        let node = create_test_node(NodeId(1), LabelId(1), "batch-001");

        store.insert(node.clone());

        let retrieved = store.get(NodeId(1));
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved
                .unwrap()
                .properties
                .get("name")
                .unwrap()
                .as_string(),
            Some(&"batch-001".to_string())
        );
    }

    #[test]
    fn test_node_store_get_by_label() {
        let store = NodeStore::new();
        store.insert(create_test_node(NodeId(1), LabelId(1), "batch-001"));
        store.insert(create_test_node(NodeId(2), LabelId(1), "batch-002"));
        store.insert(create_test_node(NodeId(3), LabelId(2), "device-001"));

        let batch_nodes = store.get_by_label(LabelId(1));
        assert_eq!(batch_nodes.len(), 2);

        let device_nodes = store.get_by_label(LabelId(2));
        assert_eq!(device_nodes.len(), 1);
    }

    #[test]
    fn test_node_store_remove() {
        let store = NodeStore::new();
        store.insert(create_test_node(NodeId(1), LabelId(1), "batch-001"));

        assert!(store.contains(NodeId(1)));
        let removed = store.remove(NodeId(1));
        assert!(removed.is_some());
        assert!(!store.contains(NodeId(1)));
    }
}
