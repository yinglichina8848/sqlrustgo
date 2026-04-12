//! AdjacencyIndex - fast neighbor lookup by edge label

use crate::model::{EdgeId, LabelId, NodeId};
use dashmap::DashMap;
use std::sync::Arc;

/// Adjacency index: for each node, map edge label to neighboring nodes
#[derive(Clone, Default)]
pub struct AdjacencyIndex {
    /// Map: NodeId -> (EdgeLabel -> Vec of (neighbor_node_id, edge_id))
    #[allow(clippy::type_complexity)]
    index: Arc<DashMap<NodeId, DashMap<LabelId, Vec<(NodeId, EdgeId)>>>>,
}

impl AdjacencyIndex {
    pub fn new() -> Self {
        AdjacencyIndex {
            index: Arc::new(DashMap::new()),
        }
    }

    /// Add an adjacency entry
    pub fn add_edge(&self, from: NodeId, to: NodeId, label: LabelId, edge_id: EdgeId) {
        let entry = self.index.entry(from).or_default();
        let mut label_entry = entry.entry(label).or_default();
        if !label_entry.iter().any(|(n, e)| *n == to && *e == edge_id) {
            label_entry.push((to, edge_id));
        }
    }

    /// Remove an adjacency entry
    pub fn remove_edge(&self, from: NodeId, to: NodeId, label: LabelId, edge_id: EdgeId) {
        if let Some(entry) = self.index.get_mut(&from) {
            if let Some(mut label_entry) = entry.get_mut(&label) {
                label_entry.retain(|(n, e)| *n != to || *e != edge_id);
            }
        }
    }

    /// Get neighbors by edge label (outgoing)
    pub fn get_neighbors(&self, node: NodeId, label: LabelId) -> Vec<NodeId> {
        if let Some(entry) = self.index.get(&node) {
            if let Some(label_entry) = entry.get(&label) {
                return label_entry.iter().map(|(n, _)| *n).collect();
            }
        }
        Vec::new()
    }

    /// Get all outgoing neighbors
    pub fn get_all_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        if let Some(entry) = self.index.get(&node) {
            let mut result = Vec::new();
            for label_entry in entry.iter() {
                for (n, _) in label_entry.value().iter() {
                    result.push(*n);
                }
            }
            return result;
        }
        Vec::new()
    }

    /// Get (neighbor, edge_id) pairs by edge label
    pub fn get_edges(&self, node: NodeId, label: LabelId) -> Vec<(NodeId, EdgeId)> {
        if let Some(entry) = self.index.get(&node) {
            if let Some(label_entry) = entry.get(&label) {
                return label_entry.clone();
            }
        }
        Vec::new()
    }

    /// Clear all entries for a node
    pub fn clear_node(&self, node: NodeId) {
        self.index.remove(&node);
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.index.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacency_add_get() {
        let index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), LabelId(10), EdgeId(100));
        index.add_edge(NodeId(1), NodeId(3), LabelId(10), EdgeId(101));
        index.add_edge(NodeId(1), NodeId(4), LabelId(20), EdgeId(102));

        let neighbors = index.get_neighbors(NodeId(1), LabelId(10));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&NodeId(2)));
        assert!(neighbors.contains(&NodeId(3)));

        let neighbors_20 = index.get_neighbors(NodeId(1), LabelId(20));
        assert_eq!(neighbors_20.len(), 1);
        assert!(neighbors_20.contains(&NodeId(4)));
    }

    #[test]
    fn test_adjacency_get_edges() {
        let index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), LabelId(10), EdgeId(100));
        index.add_edge(NodeId(1), NodeId(3), LabelId(10), EdgeId(101));

        let edges = index.get_edges(NodeId(1), LabelId(10));
        assert_eq!(edges.len(), 2);
        assert!(edges.contains(&(NodeId(2), EdgeId(100))));
        assert!(edges.contains(&(NodeId(3), EdgeId(101))));
    }

    #[test]
    fn test_adjacency_get_all_neighbors() {
        let index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), LabelId(10), EdgeId(100));
        index.add_edge(NodeId(1), NodeId(3), LabelId(20), EdgeId(101));

        let all = index.get_all_neighbors(NodeId(1));
        assert_eq!(all.len(), 2);
        assert!(all.contains(&NodeId(2)));
        assert!(all.contains(&NodeId(3)));
    }

    #[test]
    fn test_adjacency_remove() {
        let index = AdjacencyIndex::new();
        index.add_edge(NodeId(1), NodeId(2), LabelId(10), EdgeId(100));
        index.remove_edge(NodeId(1), NodeId(2), LabelId(10), EdgeId(100));

        let neighbors = index.get_neighbors(NodeId(1), LabelId(10));
        assert!(neighbors.is_empty());
    }
}
