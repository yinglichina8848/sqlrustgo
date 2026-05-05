//! Node definition for Code Intelligence Graph

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Node type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Module,
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Test,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Module => write!(f, "module"),
            NodeType::Function => write!(f, "function"),
            NodeType::Struct => write!(f, "struct"),
            NodeType::Enum => write!(f, "enum"),
            NodeType::Trait => write!(f, "trait"),
            NodeType::Impl => write!(f, "impl"),
            NodeType::Test => write!(f, "test"),
        }
    }
}

/// Core node representation (P0 minimal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Stable node ID (SHA256 hash of fully-qualified name, first 16 hex chars)
    pub id: String,
    /// Display name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Source file path (relative to repo root)
    pub file_path: String,
    /// Line where node starts (1-indexed)
    pub line_start: usize,
    /// Line where node ends (1-indexed)
    pub line_end: usize,
    /// Function/trait signature (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl Node {
    /// Compute stable node ID from name + file_path
    pub fn compute_id(name: &str, file_path: &str) -> String {
        let fqn = format!("{}:{}", file_path, name);
        let mut hasher = Sha256::new();
        hasher.update(fqn.as_bytes());
        hex::encode(&hasher.finalize()[..8])
    }

    /// Create a new node from a symbol extracted by the AST parser
    pub fn new(
        name: String,
        node_type: NodeType,
        file_path: String,
        line_start: usize,
        line_end: usize,
        signature: Option<String>,
    ) -> Self {
        let id = Self::compute_id(&name, &file_path);
        Self {
            id,
            name,
            node_type,
            file_path,
            line_start,
            line_end,
            signature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_stable() {
        let id1 = Node::compute_id("DeadlockDetector", "crates/transaction/src/deadlock.rs");
        let id2 = Node::compute_id("DeadlockDetector", "crates/transaction/src/deadlock.rs");
        assert_eq!(id1, id2, "Same name+path must produce same ID");
    }

    #[test]
    fn test_node_id_unique() {
        let id1 = Node::compute_id("DeadlockDetector", "crates/transaction/src/deadlock.rs");
        let id2 = Node::compute_id("try_wait_edge", "crates/transaction/src/deadlock.rs");
        assert_ne!(id1, id2, "Different names must produce different IDs");
    }

    #[test]
    fn test_node_creation() {
        let node = Node::new(
            "add".to_string(),
            NodeType::Function,
            "src/lib.rs".to_string(),
            10,
            15,
            Some("fn add(a: i32, b: i32) -> i32".to_string()),
        );
        assert_eq!(node.name, "add");
        assert!(matches!(node.node_type, NodeType::Function));
        assert_eq!(node.id.len(), 16);
    }
}
