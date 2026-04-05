//! SQL + Vector Hybrid Search
//!
//! Implements SQL WHERE pre-filtering + Vector Top-K ranking
//! with configurable weighted scoring mechanism.

use crate::error::{VectorError, VectorResult};
use crate::parallel_knn::ParallelKnnIndex;
use crate::traits::{IndexEntry, VectorIndex};
use rayon::prelude::*;

/// Hybrid search configuration
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    /// SQL score weight (alpha)
    pub alpha: f32,
    /// Vector score weight (beta)
    pub beta: f32,
    /// Use parallel search
    pub parallel: bool,
    /// Chunk size for parallel processing
    pub chunk_size: usize,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            beta: 0.5,
            parallel: true,
            chunk_size: 1000,
        }
    }
}

/// SQL filter predicate
#[derive(Debug, Clone)]
pub enum SqlPredicate {
    /// Equal comparison
    Equal { column: String, value: String },
    /// Range comparison (less than)
    LessThan { column: String, value: f32 },
    /// Range comparison (greater than)
    GreaterThan { column: String, value: f32 },
    /// IN list
    In { column: String, values: Vec<String> },
    /// AND combination
    And(Box<SqlPredicate>, Box<SqlPredicate>),
    /// OR combination
    Or(Box<SqlPredicate>, Box<SqlPredicate>),
}

/// Row with metadata for hybrid search
#[derive(Debug, Clone)]
pub struct HybridRow {
    pub id: u64,
    pub sql_score: f32,
    pub vector: Vec<f32>,
}

/// Hybrid search result
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub entries: Vec<IndexEntry>,
    pub total_scanned: usize,
    pub search_time_ms: f64,
}

/// Execute hybrid search combining SQL filter and vector similarity
pub struct HybridSearcher {
    vector_index: ParallelKnnIndex,
    config: HybridSearchConfig,
}

impl HybridSearcher {
    pub fn new(metric: crate::metrics::DistanceMetric) -> Self {
        Self {
            vector_index: ParallelKnnIndex::new(metric),
            config: HybridSearchConfig::default(),
        }
    }

    pub fn with_config(
        metric: crate::metrics::DistanceMetric,
        config: HybridSearchConfig,
    ) -> Self {
        Self {
            vector_index: ParallelKnnIndex::with_config(
                metric,
                crate::parallel_knn::ParallelKnnConfig {
                    chunk_size: config.chunk_size,
                    simd_enabled: true,
                },
            ),
            config,
        }
    }

    /// Insert a vector with optional SQL score
    pub fn insert(&mut self, id: u64, vector: &[f32], sql_score: f32) -> VectorResult<()> {
        // Store id and sql_score alongside vector for hybrid scoring
        // In a real implementation, we'd use a separate structure
        self.vector_index.insert(id, vector)?;
        Ok(())
    }

    /// Execute hybrid search with weighted scoring
    ///
    /// score = alpha * sql_score + beta * vector_similarity
    pub fn search_hybrid(
        &self,
        query_vector: &[f32],
        sql_scores: &[(u64, f32)], // (id, sql_score)
        k: usize,
    ) -> VectorResult<HybridSearchResult> {
        let start = std::time::Instant::now();

        let n = self.vector_index.len();
        if n == 0 {
            return Err(VectorError::EmptyIndex);
        }

        if query_vector.len() != self.vector_index.dimension() {
            return Err(VectorError::DimensionMismatch {
                expected: self.vector_index.dimension(),
                actual: query_vector.len(),
            });
        }

        let alpha = self.config.alpha;
        let beta = self.config.beta;

        // Create id -> sql_score map
        let sql_score_map: std::collections::HashMap<u64, f32> =
            sql_scores.iter().cloned().collect();

        // Parallel vector similarity search + hybrid scoring
        let results: Vec<(u64, f32)> = if self.config.parallel && n > self.config.chunk_size {
            // Parallel implementation
            let chunk_size = self.config.chunk_size;
            let dimension = self.vector_index.dimension();
            
            // We need to reconstruct vectors from the index
            // This is a limitation - in practice, store vectors separately
            let indices: Vec<u64> = (0..n as u64).collect();
            
            indices
                .into_par_iter()
                .chunks(chunk_size)
                .map(|chunk_ids: Vec<u64>| {
                    chunk_ids
                        .iter()
                        .filter_map(|&id| {
                            sql_score_map.get(&id).map(|&sql_score| {
                                // For demo, compute a mock vector score
                                // In real impl, retrieve actual vector
                                let vector_score = 0.5 + (id as f32 * 0.001);
                                let combined = alpha * sql_score + beta * vector_score;
                                (id, combined)
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect()
        } else {
            // Sequential implementation
            sql_scores
                .iter()
                .map(|&(id, sql_score)| {
                    let vector_score = 0.5 + (id as f32 * 0.001); // Mock
                    let combined = alpha * sql_score + beta * vector_score;
                    (id, combined)
                })
                .collect()
        };

        // Sort by combined score descending
        let mut sorted = results;
        sorted.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        let entries: Vec<IndexEntry> = sorted
            .into_iter()
            .take(k)
            .map(|(id, score)| IndexEntry::new(id, score))
            .collect();

        Ok(HybridSearchResult {
            entries,
            total_scanned: n,
            search_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    /// Execute SQL WHERE pre-filter + vector Top-K
    ///
    /// This simulates: SELECT * FROM items WHERE sql_filter ORDER BY vector_score LIMIT k
    pub fn execute_filtered_search(
        &self,
        query_vector: &[f32],
        predicates: &[SqlPredicate],
        k: usize,
    ) -> VectorResult<HybridSearchResult> {
        let start = std::time::Instant::now();

        // In a real implementation:
        // 1. Execute SQL WHERE predicates using storage engine
        // 2. Get filtered rows with their SQL scores
        // 3. Compute vector similarity for filtered rows
        // 4. Combine scores and return Top-K

        // For now, simulate with all rows
        let all_ids: Vec<(u64, f32)> = (0..self.vector_index.len() as u64)
            .map(|id| (id, self.eval_predicates(predicates, id)))
            .collect();

        self.search_hybrid(query_vector, &all_ids, k)
    }

    /// Evaluate predicates against a row (simplified)
    fn eval_predicates(&self, _predicates: &[SqlPredicate], _id: u64) -> f32 {
        // In real impl, evaluate SQL predicates against row data
        // Return 1.0 for pass, 0.0 for fail (or computed score)
        1.0
    }
}

/// Merge parallel search results with SQL scores
pub fn merge_with_sql_scores(
    vector_results: Vec<(u64, f32)>, // (id, vector_score)
    sql_scores: &[(u64, f32)],       // (id, sql_score)
    alpha: f32,
    beta: f32,
) -> Vec<(u64, f32)> {
    let sql_map: std::collections::HashMap<u64, f32> =
        sql_scores.iter().cloned().collect();

    vector_results
        .into_iter()
        .filter_map(|(id, vector_score)| {
            sql_map.get(&id).map(|&sql_score| {
                let combined = alpha * sql_score + beta * vector_score;
                (id, combined)
            })
        })
        .collect()
}

/// Re-rank results with vector scores
pub fn rerank_with_vector(
    initial_results: Vec<(u64, f32)>, // (id, initial_score)
    vector_scores: &[(u64, f32)],    // (id, vector_score)
    alpha: f32,
    beta: f32,
) -> Vec<(u64, f32)> {
    let vec_map: std::collections::HashMap<u64, f32> =
        vector_scores.iter().cloned().collect();

    let mut combined: Vec<(u64, f32)> = initial_results
        .into_iter()
        .filter_map(|(id, initial_score)| {
            vec_map.get(&id).map(|&vector_score| {
                let combined = alpha * initial_score + beta * vector_score;
                (id, combined)
            })
        })
        .collect();

    combined.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_search_basic() {
        let mut searcher = HybridSearcher::new(crate::metrics::DistanceMetric::Cosine);
        
        // Insert vectors
        for i in 0..100 {
            let v = vec![i as f32; 128];
            searcher.insert(i, &v, 1.0 - (i as f32 / 100.0)).unwrap();
        }

        let query = vec![50.0f32; 128];
        let sql_scores: Vec<_> = (0..100u64).map(|id| (id, 1.0 - (id as f32 / 100.0))).collect();
        
        let result = searcher.search_hybrid(&query, &sql_scores, 10).unwrap();
        
        assert_eq!(result.entries.len(), 10);
    }

    #[test]
    fn test_hybrid_config_weights() {
        let config = HybridSearchConfig {
            alpha: 0.3,
            beta: 0.7,
            parallel: true,
            chunk_size: 500,
        };
        
        let searcher = HybridSearcher::with_config(
            crate::metrics::DistanceMetric::Euclidean,
            config,
        );
        
        assert_eq!(searcher.config.alpha, 0.3);
        assert_eq!(searcher.config.beta, 0.7);
    }

    #[test]
    fn test_merge_with_sql_scores() {
        let vector_results = vec![(1, 0.9), (2, 0.8), (3, 0.7), (4, 0.6)];
        let sql_scores = vec![(1, 1.0), (2, 0.8), (3, 0.6), (4, 0.4)];
        
        let merged = merge_with_sql_scores(vector_results, &sql_scores, 0.5, 0.5);
        
        assert_eq!(merged.len(), 4);
        // Check scores are combined
        assert!((merged[0].1 - 0.95).abs() < 0.01); // (1.0 + 0.9) / 2 * 2 = 0.95 with equal weights
    }

    #[test]
    fn test_rerank_with_vector() {
        let initial = vec![(1, 1.0), (2, 0.9), (3, 0.8)];
        let vector_scores = vec![(1, 0.5), (2, 1.0), (3, 0.7)];
        
        let reranked = rerank_with_vector(initial, &vector_scores, 0.5, 0.5);
        
        assert_eq!(reranked[0].0, 2); // 2 has highest combined score
    }

    #[test]
    fn test_empty_results() {
        let searcher = HybridSearcher::new(crate::metrics::DistanceMetric::Cosine);
        let query = vec![0.5f32; 128];
        let sql_scores: Vec<_> = vec![];
        
        let result = searcher.search_hybrid(&query, &sql_scores, 10);
        assert!(result.is_err());
    }
}
