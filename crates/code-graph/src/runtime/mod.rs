//! Graph runtime: core query API for Code Intelligence Graph
//!
//! Provides O(1)/O(log n) lookups via in-memory indices

use crate::graph::{CodeGraph, Node, NodeType};
use std::collections::HashMap;

/// Graph runtime with indexed lookups
pub struct GraphRuntime {
    graph: CodeGraph,
    /// O(1) node lookup by ID
    node_by_id: HashMap<String, usize>,
    /// O(1) nodes by file path
    nodes_by_file: HashMap<String, Vec<usize>>,
    /// O(1) nodes by type
    nodes_by_type: HashMap<NodeType, Vec<usize>>,
    /// O(1) outgoing edges by source node ID
    edges_from: HashMap<String, Vec<usize>>,
    /// O(1) incoming edges by target node ID
    edges_to: HashMap<String, Vec<usize>>,
}

impl GraphRuntime {
    /// Build a new runtime from a graph (builds all indices)
    pub fn new(graph: CodeGraph) -> Self {
        let mut runtime = Self {
            node_by_id: HashMap::new(),
            nodes_by_file: HashMap::new(),
            nodes_by_type: HashMap::new(),
            edges_from: HashMap::new(),
            edges_to: HashMap::new(),
            graph,
        };
        runtime.build_indices();
        runtime
    }

    /// Build all indices from the graph
    fn build_indices(&mut self) {
        // Index nodes
        for (idx, node) in self.graph.nodes.iter().enumerate() {
            self.node_by_id.insert(node.id.clone(), idx);
            self.nodes_by_file
                .entry(node.file_path.clone())
                .or_default()
                .push(idx);
            self.nodes_by_type
                .entry(node.node_type)
                .or_default()
                .push(idx);
        }

        // Index edges
        for (idx, edge) in self.graph.edges.iter().enumerate() {
            self.edges_from
                .entry(edge.from.clone())
                .or_default()
                .push(idx);
            self.edges_to.entry(edge.to.clone()).or_default().push(idx);
        }
    }

    /// API 1: Get a node by ID — O(1)
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.node_by_id.get(id).map(|&idx| &self.graph.nodes[idx])
    }

    /// API 2: Get all neighbors (both incoming + outgoing) — O(degree)
    pub fn get_neighbors(&self, id: &str) -> Vec<&Node> {
        let mut neighbor_indices: Vec<usize> = Vec::new();

        // Outgoing
        if let Some(edge_indices) = self.edges_from.get(id) {
            for &edge_idx in edge_indices {
                let edge = &self.graph.edges[edge_idx];
                if let Some(&target_idx) = self.node_by_id.get(&edge.to) {
                    neighbor_indices.push(target_idx);
                }
            }
        }

        // Incoming
        if let Some(edge_indices) = self.edges_to.get(id) {
            for &edge_idx in edge_indices {
                let edge = &self.graph.edges[edge_idx];
                if let Some(&source_idx) = self.node_by_id.get(&edge.from) {
                    neighbor_indices.push(source_idx);
                }
            }
        }

        neighbor_indices.sort();
        neighbor_indices.dedup();
        neighbor_indices
            .into_iter()
            .map(|idx| &self.graph.nodes[idx])
            .collect()
    }

    /// API 3: Locate all nodes in a file — O(k) where k = nodes in file
    pub fn locate_by_file(&self, file_path: &str) -> Vec<&Node> {
        self.nodes_by_file
            .get(file_path)
            .map(|indices| indices.iter().map(|&idx| &self.graph.nodes[idx]).collect())
            .unwrap_or_default()
    }

    /// API 4: Get all nodes of a specific type
    pub fn get_nodes_by_type(&self, node_type: NodeType) -> Vec<&Node> {
        self.nodes_by_type
            .get(&node_type)
            .map(|indices| indices.iter().map(|&idx| &self.graph.nodes[idx]).collect())
            .unwrap_or_default()
    }

    /// Get the underlying graph
    pub fn graph(&self) -> &CodeGraph {
        &self.graph
    }

    /// Get stats
    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            total_nodes: self.graph.nodes.len(),
            total_edges: self.graph.edges.len(),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeStats {
    pub total_nodes: usize,
    pub total_edges: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CodeGraph, Edge, EdgeType, Node, NodeType};

    fn make_test_graph() -> GraphRuntime {
        let nodes = vec![
            Node::new(
                "add".to_string(),
                NodeType::Function,
                "src/lib.rs".to_string(),
                10,
                15,
                None,
            ),
            Node::new(
                "User".to_string(),
                NodeType::Struct,
                "src/lib.rs".to_string(),
                1,
                9,
                None,
            ),
            Node::new(
                "main".to_string(),
                NodeType::Function,
                "src/main.rs".to_string(),
                1,
                20,
                None,
            ),
        ];
        let mut graph = CodeGraph::new();
        graph.nodes = nodes;
        graph.edges = vec![
            Edge::new(
                "mod:src/lib.rs".to_string(),
                Node::compute_id("add", "src/lib.rs"),
                EdgeType::Contains,
            ),
            Edge::new(
                "mod:src/lib.rs".to_string(),
                Node::compute_id("User", "src/lib.rs"),
                EdgeType::Contains,
            ),
        ];
        GraphRuntime::new(graph)
    }

    #[test]
    fn test_get_node() {
        let rt = make_test_graph();
        let id = Node::compute_id("add", "src/lib.rs");
        let node = rt.get_node(&id);
        assert!(node.is_some());
        assert_eq!(node.unwrap().name, "add");
    }

    #[test]
    fn test_get_neighbors() {
        let rt = make_test_graph();
        let neighbors = rt.get_neighbors("mod:src/lib.rs");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_locate_by_file() {
        let rt = make_test_graph();
        let nodes = rt.locate_by_file("src/lib.rs");
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_get_nodes_by_type() {
        let rt = make_test_graph();
        let functions = rt.get_nodes_by_type(NodeType::Function);
        assert_eq!(functions.len(), 2);
    }

    #[test]
    fn test_stats() {
        let rt = make_test_graph();
        let stats = rt.stats();
        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.total_edges, 2);
    }
}
