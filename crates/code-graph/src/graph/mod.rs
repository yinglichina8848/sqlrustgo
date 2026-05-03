//! Graph data structures for Code Intelligence Graph

pub mod node;
pub mod edge;
pub mod builder;

pub use node::{Node, NodeType};
pub use edge::{Edge, EdgeType};
pub use builder::build_graph;

use serde::{Deserialize, Serialize};

/// Minimal code graph for P0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub version: u64,
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            version: 1,
        }
    }

    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(nodes),
            edges: Vec::with_capacity(edges),
            version: 1,
        }
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }
}

impl Default for CodeGraph {
    fn default() -> Self { Self::new() }
}
