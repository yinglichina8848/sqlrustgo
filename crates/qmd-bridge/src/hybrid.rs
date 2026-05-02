//! Hybrid search implementation

use crate::types::{Filter, QmdData, SearchResult};

/// Configuration for hybrid search
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    /// Weight for vector search results
    pub vector_weight: f32,
    /// Weight for graph search results
    pub graph_weight: f32,
    /// Weight for text search results
    pub text_weight: f32,
    /// Maximum results to return
    pub limit: usize,
    /// Enable reranking
    pub enable_rerank: bool,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.4,
            graph_weight: 0.3,
            text_weight: 0.3,
            limit: 10,
            enable_rerank: true,
        }
    }
}

/// Hybrid query combining multiple search modalities
#[derive(Debug, Clone)]
pub struct HybridQuery {
    /// Vector for vector search
    pub vector: Option<Vec<f32>>,
    /// Graph pattern for graph search
    pub graph_pattern: Option<String>,
    /// Text query for text search
    pub text_query: Option<String>,
    /// Search limit
    pub limit: usize,
    /// Optional filters
    pub filters: Vec<Filter>,
}

impl HybridQuery {
    /// Create a new hybrid query with vector
    pub fn with_vector(vector: Vec<f32>, limit: usize) -> Self {
        Self {
            vector: Some(vector),
            graph_pattern: None,
            text_query: None,
            limit,
            filters: Vec::new(),
        }
    }

    /// Add graph pattern to query
    pub fn with_graph_pattern(mut self, pattern: String) -> Self {
        self.graph_pattern = Some(pattern);
        self
    }

    /// Add text query to query
    pub fn with_text_query(mut self, text: String) -> Self {
        self.text_query = Some(text);
        self
    }
}

/// Result of hybrid search
#[derive(Debug, Clone)]
pub struct HybridResult {
    /// Combined and reranked results
    pub results: Vec<HybridSearchResultItem>,
    /// Vector search results (unranked)
    pub vector_results: Vec<SearchResult>,
    /// Graph search results (unranked)
    pub graph_results: Vec<SearchResult>,
    /// Text search results (unranked)
    pub text_results: Vec<SearchResult>,
}

/// A single item in hybrid search results
#[derive(Debug, Clone)]
pub struct HybridSearchResultItem {
    /// Result ID
    pub id: String,
    /// Combined score
    pub score: f32,
    /// Vector score component
    pub vector_score: Option<f32>,
    /// Graph score component
    pub graph_score: Option<f32>,
    /// Text score component
    pub text_score: Option<f32>,
    /// Result data
    pub data: QmdData,
}

/// Hybrid searcher combining vector, graph, and text search
pub struct HybridSearcher {
    config: HybridSearchConfig,
}

impl HybridSearcher {
    /// Create a new hybrid searcher with default config
    pub fn new() -> Self {
        Self {
            config: HybridSearchConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: HybridSearchConfig) -> Self {
        Self { config }
    }

    /// Combine and rerank results from multiple search modalities
    pub fn rerank(
        &self,
        vector_results: Vec<SearchResult>,
        graph_results: Vec<SearchResult>,
        text_results: Vec<SearchResult>,
    ) -> HybridResult {
        let mut combined: Vec<HybridSearchResultItem> = Vec::new();

        // Process vector results
        for r in &vector_results {
            combined.push(HybridSearchResultItem {
                id: r.id.clone(),
                score: r.score * self.config.vector_weight,
                vector_score: Some(r.score),
                graph_score: None,
                text_score: None,
                data: r.data.clone(),
            });
        }

        // Process graph results
        for r in &graph_results {
            // Check if already in combined (from vector)
            if let Some(existing) = combined.iter_mut().find(|x| x.id == r.id) {
                existing.score += r.score * self.config.graph_weight;
                existing.graph_score = Some(r.score);
            } else {
                combined.push(HybridSearchResultItem {
                    id: r.id.clone(),
                    score: r.score * self.config.graph_weight,
                    vector_score: None,
                    graph_score: Some(r.score),
                    text_score: None,
                    data: r.data.clone(),
                });
            }
        }

        // Process text results
        for r in &text_results {
            // Check if already in combined
            if let Some(existing) = combined.iter_mut().find(|x| x.id == r.id) {
                existing.score += r.score * self.config.text_weight;
                existing.text_score = Some(r.score);
            } else {
                combined.push(HybridSearchResultItem {
                    id: r.id.clone(),
                    score: r.score * self.config.text_weight,
                    vector_score: None,
                    graph_score: None,
                    text_score: Some(r.score),
                    data: r.data.clone(),
                });
            }
        }

        // Sort by combined score
        combined.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        combined.truncate(self.config.limit);

        HybridResult {
            results: combined,
            vector_results,
            graph_results,
            text_results,
        }
    }
}

impl Default for HybridSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_qmd_data(id: &str, vector: Vec<f32>) -> QmdData {
        QmdData {
            id: id.to_string(),
            data_type: QmdDataType::Vector,
            vector: Some(vector),
            graph: None,
            text: None,
            metadata: std::collections::HashMap::new(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_hybrid_rerank() {
        let searcher = HybridSearcher::new();

        let vector_results = vec![SearchResult {
            id: "1".to_string(),
            score: 0.9,
            data: create_test_qmd_data("1", vec![0.1, 0.2]),
        }];

        let graph_results = vec![SearchResult {
            id: "2".to_string(),
            score: 0.8,
            data: create_test_qmd_data("2", vec![0.3, 0.4]),
        }];

        let text_results = vec![];

        let result = searcher.rerank(vector_results, graph_results, text_results);

        assert_eq!(result.results.len(), 2);
        assert_eq!(result.results[0].id, "1"); // 0.9 * 0.4 = 0.36
        assert_eq!(result.results[1].id, "2"); // 0.8 * 0.3 = 0.24
    }

    #[test]
    fn test_hybrid_combined_id() {
        let searcher = HybridSearcher::new();

        // Same ID appearing in both vector and graph
        let vector_results = vec![SearchResult {
            id: "1".to_string(),
            score: 0.9,
            data: create_test_qmd_data("1", vec![0.1, 0.2]),
        }];

        let graph_results = vec![SearchResult {
            id: "1".to_string(),
            score: 0.8,
            data: create_test_qmd_data("1", vec![0.1, 0.2]),
        }];

        let text_results = vec![];

        let result = searcher.rerank(vector_results, graph_results, text_results);

        assert_eq!(result.results.len(), 1);
        // Combined score: 0.9 * 0.4 + 0.8 * 0.3 = 0.36 + 0.24 = 0.60
        assert!((result.results[0].score - 0.60).abs() < 0.001);
    }
}
