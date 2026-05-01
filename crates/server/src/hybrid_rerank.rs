// Hybrid Reranking Module
// Combines results from SQL, vector, and graph search with intelligent reranking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the reranker
#[derive(Debug, Clone, Deserialize)]
pub struct RerankConfig {
    /// Enable reranking
    #[serde(default = "default_rerank_enabled")]
    pub enabled: bool,
    /// Reranking algorithm: "rrf", "linear", "composite"
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    /// Weights for each search mode
    #[serde(default)]
    pub mode_weights: ModeWeights,
    /// Number of results to return after reranking
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Minimum score threshold
    #[serde(default)]
    pub min_score: Option<f32>,
}

fn default_rerank_enabled() -> bool {
    true
}

fn default_algorithm() -> String {
    "rrf".to_string() // Reciprocal Rank Fusion
}

fn default_top_k() -> usize {
    10
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "rrf".to_string(),
            mode_weights: ModeWeights::default(),
            top_k: 10,
            min_score: None,
        }
    }
}

/// Weights for combining scores from different search modes
#[derive(Debug, Clone, Deserialize)]
pub struct ModeWeights {
    /// Weight for SQL search results
    #[serde(default = "default_sql_weight")]
    pub sql: f32,
    /// Weight for vector search results
    #[serde(default = "default_vector_weight")]
    pub vector: f32,
    /// Weight for graph search results
    #[serde(default = "default_graph_weight")]
    pub graph: f32,
    /// Bonus for appearing in multiple modes
    #[serde(default = "default_multi_match_bonus")]
    pub multi_match_bonus: f32,
}

fn default_sql_weight() -> f32 {
    1.0
}

fn default_vector_weight() -> f32 {
    1.0
}

fn default_graph_weight() -> f32 {
    1.0
}

fn default_multi_match_bonus() -> f32 {
    0.1
}

impl Default for ModeWeights {
    fn default() -> Self {
        Self {
            sql: 1.0,
            vector: 1.0,
            graph: 1.0,
            multi_match_bonus: 0.1,
        }
    }
}

/// Source of a search result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    Sql,
    Vector,
    Graph,
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::Sql => write!(f, "sql"),
            SearchMode::Vector => write!(f, "vector"),
            SearchMode::Graph => write!(f, "graph"),
        }
    }
}

/// A single search result from any search mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredResult {
    /// Unique identifier for the result
    pub id: String,
    /// Source search mode(s)
    pub sources: Vec<SearchMode>,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// Mode-specific scores
    pub mode_scores: HashMap<SearchMode, f32>,
    /// Result type: "document", "entity", "relationship"
    #[serde(default)]
    pub result_type: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ScoredResult {
    /// Create a new scored result
    pub fn new(id: String, mode: SearchMode, score: f32) -> Self {
        let mut mode_scores = HashMap::new();
        mode_scores.insert(mode, score);
        Self {
            id,
            sources: vec![mode],
            score,
            mode_scores,
            result_type: "document".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add a score from another mode
    pub fn add_mode_score(&mut self, mode: SearchMode, score: f32) {
        if !self.sources.contains(&mode) {
            self.sources.push(mode);
        }
        self.mode_scores.insert(mode, score);
    }

    /// Calculate the multi-match bonus
    pub fn multi_match_bonus(&self, bonus: f32) -> f32 {
        if self.sources.len() > 1 {
            bonus * (self.sources.len() - 1) as f32
        } else {
            0.0
        }
    }
}

/// Reranked result with combined score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankedItem {
    /// Unique identifier
    pub id: String,
    /// Combined/ reranked score
    pub combined_score: f32,
    /// Source modes that contributed to this result
    pub source_modes: Vec<SearchMode>,
    /// Result type
    pub result_type: String,
    /// Score breakdown
    pub score_breakdown: ScoreBreakdown,
    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Score breakdown for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub sql_score: Option<f32>,
    pub vector_score: Option<f32>,
    pub graph_score: Option<f32>,
    pub multi_match_bonus: f32,
    pub final_score: f32,
}

/// Hybrid Reranker combining multiple search results
#[derive(Debug, Clone)]
pub struct HybridReranker {
    config: RerankConfig,
}

impl HybridReranker {
    /// Create a new reranker with the given configuration
    pub fn new(config: RerankConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(RerankConfig::default())
    }

    /// Rerank results using the configured algorithm
    pub fn rerank(&self, results: Vec<ScoredResult>) -> Vec<RerankedItem> {
        match self.config.algorithm.as_str() {
            "rrf" => self.reciprocal_rank_fusion(results),
            "linear" => self.linear_weighted_fusion(results),
            "composite" => self.composite_fusion(results),
            _ => self.reciprocal_rank_fusion(results),
        }
    }

    /// Reciprocal Rank Fusion (RRF) algorithm
    /// Based on: "Reciprocal Rank Fusion outperforms Condorcet and individual Rank Fusion Methods"
    fn reciprocal_rank_fusion(&self, results: Vec<ScoredResult>) -> Vec<RerankedItem> {
        use std::collections::HashMap;

        // Build rank maps for each mode
        let mut sql_ranks: HashMap<&str, usize> = HashMap::new();
        let mut vector_ranks: HashMap<&str, usize> = HashMap::new();
        let mut graph_ranks: HashMap<&str, usize> = HashMap::new();

        // Sort by score descending and assign ranks
        let mut sql_sorted: Vec<_> = results
            .iter()
            .filter(|r| r.mode_scores.contains_key(&SearchMode::Sql))
            .collect();
        sql_sorted.sort_by(|a, b| {
            b.mode_scores
                .get(&SearchMode::Sql)
                .unwrap_or(&0.0)
                .partial_cmp(a.mode_scores.get(&SearchMode::Sql).unwrap_or(&0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, r) in sql_sorted.iter().enumerate() {
            sql_ranks.insert(r.id.as_str(), i + 1);
        }

        let mut vector_sorted: Vec<_> = results
            .iter()
            .filter(|r| r.mode_scores.contains_key(&SearchMode::Vector))
            .collect();
        vector_sorted.sort_by(|a, b| {
            b.mode_scores
                .get(&SearchMode::Vector)
                .unwrap_or(&0.0)
                .partial_cmp(a.mode_scores.get(&SearchMode::Vector).unwrap_or(&0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, r) in vector_sorted.iter().enumerate() {
            vector_ranks.insert(r.id.as_str(), i + 1);
        }

        let mut graph_sorted: Vec<_> = results
            .iter()
            .filter(|r| r.mode_scores.contains_key(&SearchMode::Graph))
            .collect();
        graph_sorted.sort_by(|a, b| {
            b.mode_scores
                .get(&SearchMode::Graph)
                .unwrap_or(&0.0)
                .partial_cmp(a.mode_scores.get(&SearchMode::Graph).unwrap_or(&0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, r) in graph_sorted.iter().enumerate() {
            graph_ranks.insert(r.id.as_str(), i + 1);
        }

        // Calculate RRF scores
        let k = 60; // standard RRF constant
        let w = &self.config.mode_weights;

        let mut rrf_scores: HashMap<String, (f32, Vec<SearchMode>, ScoreBreakdown)> =
            HashMap::new();

        for result in &results {
            let sql_rank = sql_ranks.get(result.id.as_str()).copied().unwrap_or(0);
            let vector_rank = vector_ranks.get(result.id.as_str()).copied().unwrap_or(0);
            let graph_rank = graph_ranks.get(result.id.as_str()).copied().unwrap_or(0);

            let sql_contrib = if sql_rank > 0 {
                w.sql / (k + sql_rank) as f32
            } else {
                0.0
            };
            let vector_contrib = if vector_rank > 0 {
                w.vector / (k + vector_rank) as f32
            } else {
                0.0
            };
            let graph_contrib = if graph_rank > 0 {
                w.graph / (k + graph_rank) as f32
            } else {
                0.0
            };

            let multi_match = if result.sources.len() > 1 {
                w.multi_match_bonus
            } else {
                0.0
            };
            let final_score = sql_contrib + vector_contrib + graph_contrib + multi_match;

            rrf_scores.insert(
                result.id.clone(),
                (
                    final_score,
                    result.sources.clone(),
                    ScoreBreakdown {
                        sql_score: result.mode_scores.get(&SearchMode::Sql).copied(),
                        vector_score: result.mode_scores.get(&SearchMode::Vector).copied(),
                        graph_score: result.mode_scores.get(&SearchMode::Graph).copied(),
                        multi_match_bonus: multi_match,
                        final_score,
                    },
                ),
            );
        }

        // Sort by RRF score and take top_k
        let mut items: Vec<_> = rrf_scores
            .into_iter()
            .map(
                |(id, (combined_score, source_modes, breakdown))| RerankedItem {
                    id,
                    combined_score,
                    source_modes,
                    result_type: "document".to_string(),
                    score_breakdown: breakdown,
                    metadata: None,
                },
            )
            .collect();

        items.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(min_score) = self.config.min_score {
            items.retain(|item| item.combined_score >= min_score);
        }

        items.truncate(self.config.top_k);
        items
    }

    /// Linear weighted fusion
    fn linear_weighted_fusion(&self, results: Vec<ScoredResult>) -> Vec<RerankedItem> {
        let w = &self.config.mode_weights;

        let mut combined: HashMap<String, (f32, Vec<SearchMode>, ScoreBreakdown)> = HashMap::new();

        for result in results {
            let entry = combined.entry(result.id.clone()).or_insert_with(|| {
                (
                    0.0,
                    result.sources.clone(),
                    ScoreBreakdown {
                        sql_score: None,
                        vector_score: None,
                        graph_score: None,
                        multi_match_bonus: 0.0,
                        final_score: 0.0,
                    },
                )
            });

            if let Some(&sql_score) = result.mode_scores.get(&SearchMode::Sql) {
                entry.0 += sql_score * w.sql;
                entry.2.sql_score = Some(sql_score);
            }
            if let Some(&vector_score) = result.mode_scores.get(&SearchMode::Vector) {
                entry.0 += vector_score * w.vector;
                entry.2.vector_score = Some(vector_score);
            }
            if let Some(&graph_score) = result.mode_scores.get(&SearchMode::Graph) {
                entry.0 += graph_score * w.graph;
                entry.2.graph_score = Some(graph_score);
            }

            if entry.1.len() > 1 {
                entry.2.multi_match_bonus = w.multi_match_bonus;
                entry.0 += w.multi_match_bonus;
            }
            entry.2.final_score = entry.0;
        }

        let mut items: Vec<_> = combined
            .into_iter()
            .map(
                |(id, (combined_score, source_modes, breakdown))| RerankedItem {
                    id,
                    combined_score,
                    source_modes,
                    result_type: "document".to_string(),
                    score_breakdown: breakdown,
                    metadata: None,
                },
            )
            .collect();

        items.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(min_score) = self.config.min_score {
            items.retain(|item| item.combined_score >= min_score);
        }

        items.truncate(self.config.top_k);
        items
    }

    /// Composite fusion combining RRF with score-based weighting
    fn composite_fusion(&self, results: Vec<ScoredResult>) -> Vec<RerankedItem> {
        // Use RRF as base, then boost by raw scores
        let rrf_items = self.reciprocal_rank_fusion(results);

        rrf_items
            .into_iter()
            .map(|mut item| {
                // Calculate boost from average original score
                let mut score_sum = 0.0;
                let mut count = 0;
                if let Some(s) = item.score_breakdown.sql_score {
                    score_sum += s;
                    count += 1;
                }
                if let Some(s) = item.score_breakdown.vector_score {
                    score_sum += s;
                    count += 1;
                }
                if let Some(s) = item.score_breakdown.graph_score {
                    score_sum += s;
                    count += 1;
                }

                if count > 0 {
                    let avg_score = score_sum / count as f32;
                    item.combined_score = item.combined_score * 0.7 + avg_score * 0.3;
                }

                item.score_breakdown.final_score = item.combined_score;
                item
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_mode_result() {
        let result = ScoredResult::new("doc1".to_string(), SearchMode::Sql, 0.9);
        assert_eq!(result.id, "doc1");
        assert_eq!(result.sources, vec![SearchMode::Sql]);
        assert_eq!(result.score, 0.9);
    }

    #[test]
    fn test_multi_mode_result() {
        let mut result = ScoredResult::new("doc1".to_string(), SearchMode::Sql, 0.9);
        result.add_mode_score(SearchMode::Vector, 0.8);
        assert_eq!(result.sources.len(), 2);
        assert!(result.sources.contains(&SearchMode::Vector));
    }

    #[test]
    fn test_rrf_reranking() {
        let config = RerankConfig::default();
        let reranker = HybridReranker::new(config);

        let results = vec![
            {
                let mut r = ScoredResult::new("doc1".to_string(), SearchMode::Sql, 0.9);
                r.result_type = "document".to_string();
                r
            },
            {
                let mut r = ScoredResult::new("doc2".to_string(), SearchMode::Vector, 0.85);
                r.result_type = "document".to_string();
                r
            },
        ];

        let reranked = reranker.rerank(results);
        assert!(!reranked.is_empty());
    }

    #[test]
    fn test_multi_match_bonus() {
        let mut result = ScoredResult::new("doc1".to_string(), SearchMode::Sql, 0.5);
        result.add_mode_score(SearchMode::Vector, 0.5);
        result.add_mode_score(SearchMode::Graph, 0.5);

        let bonus = result.multi_match_bonus(0.1);
        assert_eq!(bonus, 0.2); // 2 additional modes * 0.1
    }

    #[test]
    fn test_rerank_config_defaults() {
        let config = RerankConfig::default();
        assert!(config.enabled);
        assert_eq!(config.algorithm, "rrf");
        assert_eq!(config.top_k, 10);
    }

    #[test]
    fn test_mode_weights_defaults() {
        let weights = ModeWeights::default();
        assert_eq!(weights.sql, 1.0);
        assert_eq!(weights.vector, 1.0);
        assert_eq!(weights.graph, 1.0);
    }
}
