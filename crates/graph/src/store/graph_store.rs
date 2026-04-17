//! GraphStore trait - the main API for graph operations

use crate::error::GraphError;
use crate::model::*;
use crate::store::{AdjacencyIndex, EdgeStore, LabelIndex, LabelRegistry, NodeStore};

/// Main graph store trait - provides the complete graph API
pub trait GraphStore {
    // Node operations

    /// Create a new node
    fn create_node(&mut self, label: &str, props: PropertyMap) -> NodeId;

    /// Get a node by ID
    fn get_node(&self, id: NodeId) -> Option<Node>;

    /// Get all nodes with a label
    fn nodes_by_label(&self, label: &str) -> Vec<NodeId>;

    /// Update node properties
    fn update_node(&mut self, id: NodeId, props: PropertyMap) -> Result<(), GraphError>;

    /// Delete a node
    fn delete_node(&mut self, id: NodeId) -> Result<(), GraphError>;

    // Edge operations

    /// Create a new edge
    fn create_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: &str,
        props: PropertyMap,
    ) -> Result<EdgeId, GraphError>;

    /// Get an edge by ID
    fn get_edge(&self, id: EdgeId) -> Option<Edge>;

    /// Get edges by label
    fn edges_by_label(&self, label: &str) -> Vec<EdgeId>;

    /// Delete an edge
    fn delete_edge(&mut self, id: EdgeId) -> Result<(), GraphError>;

    // Traversal operations

    /// Get outgoing neighbors of a node
    fn outgoing_neighbors(&self, node: NodeId) -> Vec<NodeId>;

    /// Get incoming neighbors of a node
    fn incoming_neighbors(&self, node: NodeId) -> Vec<NodeId>;

    /// Get neighbors by edge label
    fn neighbors_by_edge_label(&self, node: NodeId, edge_label: &str) -> Vec<NodeId>;

    /// Get incoming neighbors of a node filtered by edge label
    fn incoming_neighbors_by_edge_label(&self, node: NodeId, edge_label: &str) -> Vec<NodeId>;

    /// BFS traversal from a node
    fn bfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId) -> bool;

    /// DFS traversal from a node
    fn dfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId) -> bool;

    // Metadata

    /// Get total node count
    fn node_count(&self) -> usize;

    /// Get total edge count
    fn edge_count(&self) -> usize;

    /// Get label registry
    fn label_registry(&self) -> &LabelRegistry;
}

/// In-memory implementation of GraphStore
#[derive(Clone, Default)]
pub struct InMemoryGraphStore {
    pub(crate) nodes: NodeStore,
    pub(crate) edges: EdgeStore,
    pub(crate) adjacency: AdjacencyIndex,
    pub(crate) labels: LabelRegistry,
    pub(crate) label_index: LabelIndex,
    pub(crate) next_node_id: NodeId,
    pub(crate) next_edge_id: EdgeId,
}

impl InMemoryGraphStore {
    pub fn new() -> Self {
        InMemoryGraphStore {
            nodes: NodeStore::new(),
            edges: EdgeStore::new(),
            adjacency: AdjacencyIndex::new(),
            labels: LabelRegistry::new(),
            label_index: LabelIndex::new(),
            next_node_id: NodeId::MIN,
            next_edge_id: EdgeId::MIN,
        }
    }

    pub fn create_node_with_id(&mut self, label: &str, props: PropertyMap, node_id: NodeId) {
        let label_id = self.labels.get_or_register(label);
        let node = Node::new(node_id, label_id, props);
        self.nodes.insert(node);
        self.label_index.add_node(node_id, label_id);
    }

    fn next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id = self.next_node_id.next();
        id
    }

    fn next_edge_id(&mut self) -> EdgeId {
        let id = self.next_edge_id;
        self.next_edge_id = self.next_edge_id.next();
        id
    }
}

impl GraphStore for InMemoryGraphStore {
    fn create_node(&mut self, label: &str, props: PropertyMap) -> NodeId {
        let label_id = self.labels.get_or_register(label);
        let node_id = self.next_node_id();
        let node = Node::new(node_id, label_id, props);
        self.nodes.insert(node);
        self.label_index.add_node(node_id, label_id);
        node_id
    }

    fn get_node(&self, id: NodeId) -> Option<Node> {
        self.nodes.get(id)
    }

    fn nodes_by_label(&self, label: &str) -> Vec<NodeId> {
        self.labels
            .get(label)
            .map(|label_id| self.label_index.get_nodes_by_label(label_id))
            .unwrap_or_default()
    }

    fn update_node(&mut self, id: NodeId, props: PropertyMap) -> Result<(), GraphError> {
        if let Some(node) = self.nodes.get(id) {
            let mut node_mut = node.clone();
            node_mut.properties.extend(props);
            self.nodes.insert(node_mut);
            Ok(())
        } else {
            Err(GraphError::NodeNotFound(id))
        }
    }

    fn delete_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        // Remove all edges connected to this node
        let outgoing = self.edges.get_outgoing(id);
        let incoming = self.edges.get_incoming(id);
        for edge_id in outgoing.into_iter().chain(incoming) {
            let _ = self.delete_edge(edge_id);
        }

        // Remove node from index
        if let Some(node) = self.nodes.remove(id) {
            self.label_index.remove_node(id, node.label);
            Ok(())
        } else {
            Err(GraphError::NodeNotFound(id))
        }
    }

    fn create_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: &str,
        props: PropertyMap,
    ) -> Result<EdgeId, GraphError> {
        // Verify nodes exist
        if !self.nodes.contains(from) {
            return Err(GraphError::NodeNotFound(from));
        }
        if !self.nodes.contains(to) {
            return Err(GraphError::NodeNotFound(to));
        }

        let label_id = self.labels.get_or_register(label);
        let edge_id = self.next_edge_id();
        let edge = Edge::new(edge_id, from, to, label_id, props);

        self.edges.insert(edge.clone());
        self.adjacency.add_edge(from, to, label_id, edge_id);
        self.label_index.add_edge(edge_id, label_id);

        Ok(edge_id)
    }

    fn get_edge(&self, id: EdgeId) -> Option<Edge> {
        self.edges.get(id)
    }

    fn edges_by_label(&self, label: &str) -> Vec<EdgeId> {
        self.labels
            .get(label)
            .map(|label_id| self.label_index.get_edges_by_label(label_id))
            .unwrap_or_default()
    }

    fn delete_edge(&mut self, id: EdgeId) -> Result<(), GraphError> {
        if let Some(edge) = self.edges.remove(id) {
            self.adjacency
                .remove_edge(edge.from, edge.to, edge.label, edge.id);
            self.label_index.remove_edge(id, edge.label);
            Ok(())
        } else {
            Err(GraphError::EdgeNotFound(id))
        }
    }

    fn outgoing_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.adjacency.get_all_neighbors(node)
    }

    fn incoming_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        let incoming_edges = self.edges.get_incoming(node);
        incoming_edges
            .iter()
            .filter_map(|&edge_id| self.edges.get(edge_id))
            .map(|edge| edge.from)
            .collect()
    }

    fn neighbors_by_edge_label(&self, node: NodeId, edge_label: &str) -> Vec<NodeId> {
        self.labels
            .get(edge_label)
            .map(|label_id| self.adjacency.get_neighbors(node, label_id))
            .unwrap_or_default()
    }

    fn incoming_neighbors_by_edge_label(&self, node: NodeId, edge_label: &str) -> Vec<NodeId> {
        let label_id = match self.labels.get(edge_label) {
            Some(id) => id,
            None => return vec![],
        };
        self.edges
            .get_incoming(node)
            .iter()
            .filter_map(|&edge_id| self.edges.get(edge_id))
            .filter(|edge| edge.label == label_id)
            .map(|edge| edge.from)
            .collect()
    }

    fn bfs<F>(&self, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId) -> bool,
    {
        use std::collections::VecDeque;
        let mut queue = VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            if !visitor(node) {
                return;
            }

            for neighbor in self.outgoing_neighbors(node) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    fn dfs<F>(&self, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId) -> bool,
    {
        use std::collections::HashSet;
        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(node) = stack.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);

            if !visitor(node) {
                return;
            }

            for neighbor in self.outgoing_neighbors(node) {
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn label_registry(&self) -> &LabelRegistry {
        &self.labels
    }
}
