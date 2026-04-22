// SQLRustGo Hybrid Search Endpoints
// Provides unified retrieval API combining SQL, vector, and graph search

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hybrid search request combining multiple retrieval modes
#[derive(Debug, Deserialize)]
pub struct HybridSearchRequest {
    /// SQL WHERE clause or full query
    pub query: String,
    /// Query mode: "sql_vector_graph", "sql_vector", "sql_graph", "vector", "graph"
    #[serde(default = "default_hybrid_mode")]
    pub mode: String,
    /// Number of results to return
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Vector search configuration
    #[serde(default)]
    pub vector: Option<VectorConfig>,
    /// Graph traversal configuration
    #[serde(default)]
    pub graph: Option<GraphConfig>,
    /// SQL result filters
    #[serde(default)]
    pub filters: Option<SearchFilters>,
    /// Enable reranking of combined results
    #[serde(default)]
    pub enable_rerank: bool,
    /// Minimum score threshold
    #[serde(default)]
    pub min_score: Option<f32>,
}

fn default_hybrid_mode() -> String {
    "sql_vector_graph".to_string()
}

fn default_top_k() -> usize {
    10
}

/// Vector search configuration
#[derive(Debug, Deserialize)]
pub struct VectorConfig {
    /// Column name for vector search
    pub column: String,
    /// Query embedding vector
    pub query_vector: Vec<f32>,
    /// Number of vector results
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Minimum similarity score
    #[serde(default)]
    pub min_score: Option<f32>,
}

/// Graph traversal configuration
#[derive(Debug, Deserialize)]
pub struct GraphConfig {
    /// Starting node ID
    pub start_node: String,
    /// Traversal algorithm: "bfs" or "dfs"
    #[serde(default = "default_traversal")]
    pub traversal: String,
    /// Maximum traversal depth
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Edge type filter
    #[serde(default)]
    pub edge_type: Option<String>,
}

fn default_traversal() -> String {
    "bfs".to_string()
}

/// Search result filters
#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    /// Date range filter
    #[serde(default)]
    pub date_range: Option<DateRange>,
    /// Department filter
    #[serde(default)]
    pub department: Option<String>,
    /// Custom key-value filters
    #[serde(default)]
    pub custom: Option<HashMap<String, String>>,
}

/// Date range for filtering
#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

/// Hybrid search response
#[derive(Debug, Serialize)]
pub struct HybridSearchResponse {
    /// Whether the search was successful
    pub success: bool,
    /// SQL query results (if applicable)
    pub sql_results: Option<SqlSearchResults>,
    /// Vector search results (if applicable)
    pub vector_results: Option<VectorSearchResults>,
    /// Graph search results (if applicable)
    pub graph_results: Option<GraphSearchResults>,
    /// Reranked combined results (if enable_rerank=true)
    pub reranked_results: Option<Vec<RerankedResult>>,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// SQL search results
#[derive(Debug, Serialize)]
pub struct SqlSearchResults {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

/// Vector search results
#[derive(Debug, Serialize)]
pub struct VectorSearchResults {
    pub results: Vec<VectorSearchResult>,
    pub total_scanned: usize,
}

/// Single vector search result
#[derive(Debug, Serialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Graph search results
#[derive(Debug, Serialize)]
pub struct GraphSearchResults {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub total_scanned: usize,
}

/// Graph node
#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub properties: Option<HashMap<String, serde_json::Value>>,
}

/// Graph edge
#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: Option<f32>,
}

/// Reranked search result combining multiple retrieval modes
#[derive(Debug, Serialize)]
pub struct RerankedResult {
    pub id: String,
    pub source_modes: Vec<String>, // "sql", "vector", "graph"
    pub combined_score: f32,
    pub result_type: String, // "document", "entity", "relationship"
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub hybrid_api_available: bool,
    pub qmd_bridge_available: bool,
}

/// Configuration for the hybrid search API
#[derive(Debug, Deserialize)]
pub struct HybridApiConfig {
    /// Enable hybrid search endpoints
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Default top_k for searches
    #[serde(default = "default_top_k")]
    pub default_top_k: usize,
    /// Default reranking enabled
    #[serde(default)]
    pub rerank_default: bool,
    /// QMD bridge connection string
    #[serde(default)]
    pub qmd_bridge_url: Option<String>,
}

fn default_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_search_request_default_mode() {
        let json = r#"{"query": "test", "mode": "sql_vector_graph"}"#;
        let req: HybridSearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.mode, "sql_vector_graph");
        assert_eq!(req.top_k, 10);
    }

    #[test]
    fn test_hybrid_search_request_vector_only() {
        let json = r#"{
            "query": "test",
            "mode": "vector",
            "vector": {
                "column": "embedding",
                "query_vector": [0.1, 0.2, 0.3]
            }
        }"#;
        let req: HybridSearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.mode, "vector");
        assert!(req.vector.is_some());
    }

    #[test]
    fn test_reranked_result_serialization() {
        let result = RerankedResult {
            id: "doc1".to_string(),
            source_modes: vec!["sql".to_string(), "vector".to_string()],
            combined_score: 0.95,
            result_type: "document".to_string(),
            metadata: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("combined_score"));
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse {
            status: "ok".to_string(),
            hybrid_api_available: true,
            qmd_bridge_available: true,
        };
        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("hybrid_api_available"));
    }
}
