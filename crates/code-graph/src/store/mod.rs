//! JSON store for Code Intelligence Graph
//!
//! P0: Simple JSON serialization/deserialization with version tracking

use crate::graph::CodeGraph;
use std::fs;

/// Save graph to JSON file
pub fn save_graph(graph: &CodeGraph, path: &str) -> Result<(), StoreError> {
    let json = serde_json::to_string_pretty(graph)
        .map_err(|e| StoreError::Serialize(e.to_string()))?;
    fs::write(path, json)
        .map_err(|e| StoreError::Io(e.to_string()))?;
    Ok(())
}

/// Load graph from JSON file
pub fn load_graph(path: &str) -> Result<CodeGraph, StoreError> {
    let content = fs::read_to_string(path)
        .map_err(|e| StoreError::Io(e.to_string()))?;
    serde_json::from_str(&content)
        .map_err(|e| StoreError::Deserialize(e.to_string()))
}

/// Compute hash of a graph for change detection
pub fn graph_hash(graph: &CodeGraph) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", graph.version, graph.nodes.len()).as_bytes());
    for node in &graph.nodes {
        hasher.update(node.id.as_bytes());
    }
    hex::encode(&hasher.finalize()[..8])
}

#[derive(Debug)]
pub enum StoreError {
    Io(String),
    Serialize(String),
    Deserialize(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(s) => write!(f, "IO error: {}", s),
            StoreError::Serialize(s) => write!(f, "Serialize error: {}", s),
            StoreError::Deserialize(s) => write!(f, "Deserialize error: {}", s),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, Edge};

    #[test]
    fn test_save_load_roundtrip() {
        let graph = CodeGraph::new();
        let nodes = vec![
            Node::new(
                "add".to_string(),
                crate::graph::NodeType::Function,
                "src/lib.rs".to_string(),
                10,
                15,
                None,
            ),
        ];
        let mut g = CodeGraph::new();
        g.nodes = nodes;

        let temp_path = "/tmp/test_graph.json";
        save_graph(&g, temp_path).unwrap();
        let loaded = load_graph(temp_path).unwrap();
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "add");

        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_graph_hash() {
        let mut g1 = CodeGraph::new();
        g1.nodes.push(Node::new("test".to_string(), crate::graph::NodeType::Function, "a.rs".to_string(), 1, 1, None));

        let mut g2 = CodeGraph::new();
        g2.nodes.push(Node::new("test".to_string(), crate::graph::NodeType::Function, "a.rs".to_string(), 1, 1, None));

        assert_eq!(graph_hash(&g1), graph_hash(&g2), "Same content should produce same hash");
    }
}
