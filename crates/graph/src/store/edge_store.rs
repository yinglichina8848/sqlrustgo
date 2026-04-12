//! EdgeStore - in-memory storage for edges

use crate::model::{Edge, EdgeId, NodeId};
use dashmap::DashMap;

/// Edge storage
#[derive(Clone, Default)]
pub struct EdgeStore {
    /// Edge data indexed by EdgeId
    edges: DashMap<EdgeId, Edge>,
    /// Outgoing edges index: NodeId -> Vec<EdgeId>
    outgoing: DashMap<NodeId, Vec<EdgeId>>,
    /// Incoming edges index: NodeId -> Vec<EdgeId>
    incoming: DashMap<NodeId, Vec<EdgeId>>,
}

impl EdgeStore {
    pub fn new() -> Self {
        EdgeStore {
            edges: DashMap::new(),
            outgoing: DashMap::new(),
            incoming: DashMap::new(),
        }
    }

    /// Insert an edge
    pub fn insert(&self, edge: Edge) {
        let edge_id = edge.id;
        let from = edge.from;
        let to = edge.to;

        self.edges.insert(edge_id, edge);

        // Update outgoing index
        let mut outgoing_entry = self.outgoing.entry(from).or_default();
        if !outgoing_entry.contains(&edge_id) {
            outgoing_entry.push(edge_id);
        }

        // Update incoming index
        let mut incoming_entry = self.incoming.entry(to).or_default();
        if !incoming_entry.contains(&edge_id) {
            incoming_entry.push(edge_id);
        }
    }

    /// Get an edge by ID
    pub fn get(&self, id: EdgeId) -> Option<Edge> {
        self.edges.get(&id).map(|entry| entry.clone())
    }

    /// Get outgoing edge IDs from a node
    pub fn get_outgoing(&self, node: NodeId) -> Vec<EdgeId> {
        self.outgoing
            .get(&node)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get incoming edge IDs to a node
    pub fn get_incoming(&self, node: NodeId) -> Vec<EdgeId> {
        self.incoming
            .get(&node)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Get all edge IDs
    pub fn ids(&self) -> Vec<EdgeId> {
        self.edges.iter().map(|r| *r.key()).collect()
    }

    /// Get total edge count
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Check if edge exists
    pub fn contains(&self, id: EdgeId) -> bool {
        self.edges.contains_key(&id)
    }

    /// Remove an edge by ID
    pub fn remove(&self, id: EdgeId) -> Option<Edge> {
        if let Some((_, edge)) = self.edges.remove(&id) {
            // Remove from indices
            if let Some(mut out_list) = self.outgoing.get_mut(&edge.from) {
                out_list.retain(|&eid| eid != id);
            }
            if let Some(mut in_list) = self.incoming.get_mut(&edge.to) {
                in_list.retain(|&eid| eid != id);
            }
            Some(edge)
        } else {
            None
        }
    }

    /// Clear all edges
    pub fn clear(&self) {
        self.edges.clear();
        self.outgoing.clear();
        self.incoming.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Direction, LabelId, PropertyMap};

    fn create_test_edge(id: EdgeId, from: NodeId, to: NodeId, label: LabelId) -> Edge {
        Edge::with_direction(id, from, to, label, PropertyMap::new(), Direction::Outgoing)
    }

    #[test]
    fn test_edge_store_insert_get() {
        let store = EdgeStore::new();
        let edge = create_test_edge(EdgeId(1), NodeId(100), NodeId(200), LabelId(1));

        store.insert(edge.clone());

        let retrieved = store.get(EdgeId(1));
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.from, NodeId(100));
        assert_eq!(retrieved.to, NodeId(200));
    }

    #[test]
    fn test_edge_store_outgoing_incoming() {
        let store = EdgeStore::new();
        store.insert(create_test_edge(
            EdgeId(1),
            NodeId(100),
            NodeId(200),
            LabelId(1),
        ));
        store.insert(create_test_edge(
            EdgeId(2),
            NodeId(100),
            NodeId(300),
            LabelId(1),
        ));
        store.insert(create_test_edge(
            EdgeId(3),
            NodeId(200),
            NodeId(100),
            LabelId(2),
        ));

        let outgoing = store.get_outgoing(NodeId(100));
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.contains(&EdgeId(1)));
        assert!(outgoing.contains(&EdgeId(2)));

        let incoming = store.get_incoming(NodeId(100));
        assert_eq!(incoming.len(), 1);
        assert!(incoming.contains(&EdgeId(3)));
    }

    #[test]
    fn test_edge_store_remove() {
        let store = EdgeStore::new();
        store.insert(create_test_edge(
            EdgeId(1),
            NodeId(100),
            NodeId(200),
            LabelId(1),
        ));

        assert!(store.contains(EdgeId(1)));
        let removed = store.remove(EdgeId(1));
        assert!(removed.is_some());
        assert!(!store.contains(EdgeId(1)));
    }
}
