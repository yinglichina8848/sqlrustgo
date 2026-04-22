//! QMD Bridge data types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of data in QMD
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QmdDataType {
    /// Vector data
    Vector,
    /// Graph data
    Graph,
    /// Document/text data
    Document,
    /// Mixed type
    Mixed,
}

/// Type of query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    /// K-nearest neighbors search
    Knn,
    /// Breadth-first search
    Bfs,
    /// Depth-first search
    Dfs,
    /// Range search
    Range,
    /// Hybrid search
    Hybrid,
}

/// Data format for QMD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QmdData {
    /// Unique identifier
    pub id: String,
    /// Type of data
    pub data_type: QmdDataType,
    /// Vector data (for vector type)
    pub vector: Option<Vec<f32>>,
    /// Graph structure (for graph type)
    pub graph: Option<GraphData>,
    /// Text content (for document type)
    pub text: Option<String>,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Timestamp
    pub timestamp: i64,
}

impl QmdData {
    /// Create a new vector data entry
    pub fn new_vector(id: &str, vector: Vec<f32>) -> Self {
        Self {
            id: id.to_string(),
            data_type: QmdDataType::Vector,
            vector: Some(vector),
            graph: None,
            text: None,
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    /// Create a new document data entry
    pub fn new_document(id: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            data_type: QmdDataType::Document,
            vector: None,
            graph: None,
            text: Some(text.to_string()),
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
}

/// Graph data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    /// Nodes with their properties
    pub nodes: Vec<GraphNode>,
    /// Edges between nodes
    pub edges: Vec<GraphEdge>,
}

/// A node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, String>,
}

/// An edge in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub properties: HashMap<String, String>,
}

/// Query for QMD search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QmdQuery {
    /// Type of query
    pub query_type: QueryType,
    /// Vector for vector search
    pub vector: Option<Vec<f32>>,
    /// Graph pattern for graph search
    pub graph_pattern: Option<String>,
    /// Text for text search
    pub text: Option<String>,
    /// Filters to apply
    pub filters: Vec<Filter>,
    /// Maximum results to return
    pub limit: usize,
    /// Distance threshold
    pub threshold: Option<f32>,
}

/// Filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

/// Filter operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    Contains,
}

/// Search result from QMD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Result data
    pub data: QmdData,
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Last sync timestamp
    pub last_sync: i64,
    /// Number of items synced
    pub items_synced: u64,
    /// Sync state
    pub state: SyncState,
    /// Error message if failed
    pub error: Option<String>,
}

/// State of sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    /// Idle, no sync in progress
    Idle,
    /// Sync in progress
    Syncing,
    /// Sync completed successfully
    Completed,
    /// Sync failed
    Failed,
}
