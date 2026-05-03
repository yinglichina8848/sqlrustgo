//! Edge definition for Code Intelligence Graph

use serde::{Deserialize, Serialize};

/// Edge type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// Function calls another function
    Calls,
    /// Module imports another module
    Imports,
    /// Module/struct contains a member
    Contains,
    /// Struct implements a trait
    Implements,
    /// Test exercises a function
    TestedBy,
    /// Function returns a type
    Returns,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::Calls => write!(f, "calls"),
            EdgeType::Imports => write!(f, "imports"),
            EdgeType::Contains => write!(f, "contains"),
            EdgeType::Implements => write!(f, "implements"),
            EdgeType::TestedBy => write!(f, "tested_by"),
            EdgeType::Returns => write!(f, "returns"),
        }
    }
}

/// Directed edge between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Edge type
    pub edge_type: EdgeType,
}

impl Edge {
    pub fn new(from: String, to: String, edge_type: EdgeType) -> Self {
        Self { from, to, edge_type }
    }
}
