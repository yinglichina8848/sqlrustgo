use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QueryMode {
    #[default]
    SQL,
    Vector,
    Graph,
    SQLVector,
    SQLGraph,
    VectorGraph,
    SQLVectorGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights {
    pub sql: f32,
    pub vector: f32,
    pub graph: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            sql: 0.4,
            vector: 0.3,
            graph: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQuery {
    pub column: String,
    pub top_k: u32,
    #[serde(default)]
    pub filter: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQuery {
    pub start_nodes: Vec<String>,
    pub traversal: TraversalType,
    pub max_depth: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalType {
    BFS,
    DFS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedQueryRequest {
    pub query: String,
    pub mode: QueryMode,
    #[serde(default)]
    pub filters: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub weights: Option<Weights>,
    #[serde(default)]
    pub vector_query: Option<VectorQuery>,
    #[serde(default)]
    pub graph_query: Option<GraphQuery>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedQueryResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_results: Option<Vec<Vec<serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_results: Option<Vec<VectorResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_results: Option<Vec<GraphResult>>,
    pub fusion_scores: Vec<FusionScore>,
    pub total: u64,
    pub execution_time_ms: u64,
    pub query_plan: QueryPlanDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorResult {
    pub id: String,
    pub score: f32,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResult {
    pub path: Vec<String>,
    pub score: f32,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionScore {
    pub id: String,
    pub score: f32,
    pub source: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlanDetail {
    pub mode: String,
    pub weights: Weights,
    pub steps: Vec<QueryStep>,
    pub fusion_time_ms: u64,
    pub total_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStep {
    pub source: String,
    pub step: String,
    pub time_ms: u64,
    #[serde(default)]
    pub rows_affected: Option<u64>,
    #[serde(default)]
    pub nodes_visited: Option<u64>,
    #[serde(default)]
    pub nodes_traversed: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_query_request_default() {
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: Some(10),
            offset: Some(0),
        };
        assert_eq!(request.top_k, Some(10));
        assert_eq!(request.mode, QueryMode::SQL);
    }

    #[test]
    fn test_weights_default() {
        let weights = Weights::default();
        assert_eq!(weights.sql, 0.4);
        assert_eq!(weights.vector, 0.3);
        assert_eq!(weights.graph, 0.3);
    }

    #[test]
    fn test_query_mode_variants() {
        let modes = vec![
            QueryMode::SQL,
            QueryMode::Vector,
            QueryMode::Graph,
            QueryMode::SQLVector,
            QueryMode::SQLGraph,
            QueryMode::VectorGraph,
            QueryMode::SQLVectorGraph,
        ];
        assert_eq!(modes.len(), 7);
    }
}
