//! LabelIndex - index nodes/edges by label for fast lookup

use crate::model::{EdgeId, LabelId, NodeId};
use dashmap::DashMap;

/// Index for looking up nodes by label
#[derive(Clone, Default)]
pub struct LabelIndex {
    /// Node label index: LabelId -> Vec<NodeId>
    node_index: DashMap<LabelId, Vec<NodeId>>,
    /// Edge label index: LabelId -> Vec<EdgeId>
    edge_index: DashMap<LabelId, Vec<EdgeId>>,
}

impl LabelIndex {
    pub fn new() -> Self {
        LabelIndex {
            node_index: DashMap::new(),
            edge_index: DashMap::new(),
        }
    }

    // Node label operations

    /// Add a node to label index
    pub fn add_node(&self, node_id: NodeId, label: LabelId) {
        let mut entry = self.node_index.entry(label).or_insert_with(Vec::new);
        if !entry.contains(&node_id) {
            entry.push(node_id);
        }
    }

    /// Remove a node from label index
    pub fn remove_node(&self, node_id: NodeId, label: LabelId) {
        if let Some(mut entry) = self.node_index.get_mut(&label) {
            entry.retain(|&id| id != node_id);
        }
    }

    /// Get all node IDs with a label
    pub fn get_nodes_by_label(&self, label: LabelId) -> Vec<NodeId> {
        self.node_index
            .get(&label)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get count of nodes with a label
    pub fn node_count_by_label(&self, label: LabelId) -> usize {
        self.node_index.get(&label).map(|e| e.len()).unwrap_or(0)
    }

    // Edge label operations

    /// Add an edge to label index
    pub fn add_edge(&self, edge_id: EdgeId, label: LabelId) {
        let mut entry = self.edge_index.entry(label).or_insert_with(Vec::new);
        if !entry.contains(&edge_id) {
            entry.push(edge_id);
        }
    }

    /// Remove an edge from label index
    pub fn remove_edge(&self, edge_id: EdgeId, label: LabelId) {
        if let Some(mut entry) = self.edge_index.get_mut(&label) {
            entry.retain(|&id| id != edge_id);
        }
    }

    /// Get all edge IDs with a label
    pub fn get_edges_by_label(&self, label: LabelId) -> Vec<EdgeId> {
        self.edge_index
            .get(&label)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get count of edges with a label
    pub fn edge_count_by_label(&self, label: LabelId) -> usize {
        self.edge_index.get(&label).map(|e| e.len()).unwrap_or(0)
    }

    // Utility operations

    /// Clear all indices
    pub fn clear(&self) {
        self.node_index.clear();
        self.edge_index.clear();
    }

    /// Get all labels that have nodes
    pub fn node_labels(&self) -> Vec<LabelId> {
        self.node_index.iter().map(|r| *r.key()).collect()
    }

    /// Get all labels that have edges
    pub fn edge_labels(&self) -> Vec<LabelId> {
        self.edge_index.iter().map(|r| *r.key()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_index_nodes() {
        let index = LabelIndex::new();

        index.add_node(NodeId(1), LabelId(10));
        index.add_node(NodeId(2), LabelId(10));
        index.add_node(NodeId(3), LabelId(20));

        let nodes_10 = index.get_nodes_by_label(LabelId(10));
        assert_eq!(nodes_10.len(), 2);

        let nodes_20 = index.get_nodes_by_label(LabelId(20));
        assert_eq!(nodes_20.len(), 1);

        let count = index.node_count_by_label(LabelId(10));
        assert_eq!(count, 2);
    }

    #[test]
    fn test_label_index_remove_node() {
        let index = LabelIndex::new();

        index.add_node(NodeId(1), LabelId(10));
        index.add_node(NodeId(2), LabelId(10));
        index.remove_node(NodeId(1), LabelId(10));

        let nodes = index.get_nodes_by_label(LabelId(10));
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains(&NodeId(2)));
    }

    #[test]
    fn test_label_index_edges() {
        let index = LabelIndex::new();

        index.add_edge(EdgeId(1), LabelId(10));
        index.add_edge(EdgeId(2), LabelId(10));
        index.add_edge(EdgeId(3), LabelId(20));

        let edges_10 = index.get_edges_by_label(LabelId(10));
        assert_eq!(edges_10.len(), 2);

        let edges_20 = index.get_edges_by_label(LabelId(20));
        assert_eq!(edges_20.len(), 1);
    }

    #[test]
    fn test_label_index_labels() {
        let index = LabelIndex::new();

        index.add_node(NodeId(1), LabelId(10));
        index.add_edge(EdgeId(1), LabelId(20));

        let node_labels = index.node_labels();
        assert!(node_labels.contains(&LabelId(10)));

        let edge_labels = index.edge_labels();
        assert!(edge_labels.contains(&LabelId(20)));
    }
}
