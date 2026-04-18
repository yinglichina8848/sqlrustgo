//! SQL + Vector Hybrid Search
//!
//! Implements SQL WHERE pre-filtering + Vector Top-K ranking
//! with configurable weighted scoring mechanism.

use crate::error::{VectorError, VectorResult};
use crate::metrics::DistanceMetric;
use crate::parallel_knn::simd::compute_similarity_simd;
use crate::parallel_knn::ParallelKnnIndex;
use crate::traits::{IndexEntry, VectorIndex};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;

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
    Equal { column: String, value: SqlValue },
    /// Range comparison (less than)
    LessThan { column: String, value: SqlValue },
    /// Range comparison (greater than)
    GreaterThan { column: String, value: SqlValue },
    /// Less than or equal
    LessThanEq { column: String, value: SqlValue },
    /// Greater than or equal
    GreaterThanEq { column: String, value: SqlValue },
    /// IN list
    In {
        column: String,
        values: Vec<SqlValue>,
    },
    /// AND combination
    And(Box<SqlPredicate>, Box<SqlPredicate>),
    /// OR combination
    Or(Box<SqlPredicate>, Box<SqlPredicate>),
    /// NOT negation
    Not(Box<SqlPredicate>),
}

impl SqlPredicate {
    pub fn and(left: SqlPredicate, right: SqlPredicate) -> Self {
        SqlPredicate::And(Box::new(left), Box::new(right))
    }

    pub fn or(left: SqlPredicate, right: SqlPredicate) -> Self {
        SqlPredicate::Or(Box::new(left), Box::new(right))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(predicate: SqlPredicate) -> Self {
        SqlPredicate::Not(Box::new(predicate))
    }
}

/// SQL value types for predicate evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Text(String),
}

impl SqlValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            SqlValue::Integer(i) => Some(*i as f64),
            SqlValue::Float(f) => Some(*f),
            SqlValue::Text(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            SqlValue::Integer(i) => Some(*i),
            SqlValue::Float(f) => Some(*f as i64),
            SqlValue::Text(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            SqlValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn compare(&self, op: &CompareOp, other: &SqlValue) -> bool {
        match op {
            CompareOp::Equal => self == other,
            CompareOp::NotEqual => self != other,
            CompareOp::LessThan => {
                let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) else {
                    return false;
                };
                a < b
            }
            CompareOp::LessThanOrEqual => {
                let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) else {
                    return false;
                };
                a <= b
            }
            CompareOp::GreaterThan => {
                let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) else {
                    return false;
                };
                a > b
            }
            CompareOp::GreaterThanOrEqual => {
                let (Some(a), Some(b)) = (self.as_f64(), other.as_f64()) else {
                    return false;
                };
                a >= b
            }
            CompareOp::In => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    In,
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
    vectors: RwLock<HashMap<u64, Vec<f32>>>,
    row_data: RwLock<HashMap<u64, HashMap<String, SqlValue>>>,
    metric: DistanceMetric,
    config: HybridSearchConfig,
}

impl HybridSearcher {
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            vector_index: ParallelKnnIndex::new(metric),
            vectors: RwLock::new(HashMap::new()),
            row_data: RwLock::new(HashMap::new()),
            metric,
            config: HybridSearchConfig::default(),
        }
    }

    pub fn with_config(metric: DistanceMetric, config: HybridSearchConfig) -> Self {
        Self {
            vector_index: ParallelKnnIndex::with_config(
                metric,
                crate::parallel_knn::ParallelKnnConfig {
                    chunk_size: config.chunk_size,
                    simd_enabled: true,
                },
            ),
            vectors: RwLock::new(HashMap::new()),
            row_data: RwLock::new(HashMap::new()),
            metric,
            config,
        }
    }

    /// Insert a vector with optional SQL score
    pub fn insert(&mut self, id: u64, vector: &[f32], _sql_score: f32) -> VectorResult<()> {
        self.vectors.write().insert(id, vector.to_vec());
        self.vector_index.insert(id, vector)?;
        Ok(())
    }

    /// Insert a vector with row data for predicate filtering
    pub fn insert_with_row(
        &mut self,
        id: u64,
        vector: &[f32],
        row: HashMap<String, SqlValue>,
    ) -> VectorResult<()> {
        self.vectors.write().insert(id, vector.to_vec());
        self.row_data.write().insert(id, row);
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
        let metric = self.metric;
        let vectors = self.vectors.read();

        // Create id -> sql_score map
        let sql_score_map: HashMap<u64, f32> = sql_scores.iter().cloned().collect();

        // Parallel vector similarity search + hybrid scoring with REAL vector computation
        let results: Vec<(u64, f32)> = if self.config.parallel && n > self.config.chunk_size {
            // Parallel implementation - compute real vector similarity
            let chunk_size = self.config.chunk_size;
            // Get all IDs that have SQL scores and vectors
            let candidate_ids: Vec<u64> = sql_scores
                .iter()
                .filter(|&&(id, _)| vectors.contains_key(&id))
                .map(|&(id, _)| id)
                .collect();

            candidate_ids
                .into_par_iter()
                .chunks(chunk_size)
                .map(|chunk_ids: Vec<u64>| {
                    chunk_ids
                        .iter()
                        .filter_map(|&id| {
                            let sql_score = *sql_score_map.get(&id)?;
                            let vector = vectors.get(&id)?;
                            // Compute REAL vector similarity using SIMD-optimized computation
                            let vector_score =
                                compute_similarity_simd(query_vector, vector, metric);
                            let combined = alpha * sql_score + beta * vector_score;
                            Some((id, combined))
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect()
        } else {
            // Sequential implementation - compute real vector similarity
            sql_scores
                .iter()
                .filter_map(|&(id, sql_score)| {
                    let vector = vectors.get(&id)?;
                    // Compute REAL vector similarity using SIMD-optimized computation
                    let vector_score = compute_similarity_simd(query_vector, vector, metric);
                    let combined = alpha * sql_score + beta * vector_score;
                    Some((id, combined))
                })
                .collect()
        };

        // Sort by combined score descending
        let mut sorted = results;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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
    /// This implements: SELECT * FROM items WHERE sql_filter ORDER BY vector_score LIMIT k
    pub fn execute_filtered_search(
        &self,
        query_vector: &[f32],
        predicates: &[SqlPredicate],
        k: usize,
    ) -> VectorResult<HybridSearchResult> {
        let row_data = self.row_data.read();
        let vectors = self.vectors.read();

        let all_ids: Vec<(u64, f32)> = (0..self.vector_index.len() as u64)
            .filter_map(|id| {
                let row = row_data.get(&id)?;
                if self.eval_predicates(predicates, row) {
                    let sql_score = self.compute_sql_score_from_row(row);
                    Some((id, sql_score))
                } else {
                    None
                }
            })
            .collect();

        drop(row_data);
        drop(vectors);

        self.search_hybrid(query_vector, &all_ids, k)
    }

    /// Evaluate predicates against a row
    fn eval_predicates(
        &self,
        predicates: &[SqlPredicate],
        row: &HashMap<String, SqlValue>,
    ) -> bool {
        if predicates.is_empty() {
            return true;
        }
        predicates.iter().all(|p| self.eval_predicate(p, row))
    }

    /// Evaluate a single predicate against a row
    fn eval_predicate(&self, predicate: &SqlPredicate, row: &HashMap<String, SqlValue>) -> bool {
        match predicate {
            SqlPredicate::Equal { column, value } => {
                row.get(column).map(|v| v == value).unwrap_or(false)
            }
            SqlPredicate::LessThan { column, value } => row
                .get(column)
                .map(|v| v.compare(&CompareOp::LessThan, value))
                .unwrap_or(false),
            SqlPredicate::GreaterThan { column, value } => row
                .get(column)
                .map(|v| v.compare(&CompareOp::GreaterThan, value))
                .unwrap_or(false),
            SqlPredicate::LessThanEq { column, value } => row
                .get(column)
                .map(|v| v.compare(&CompareOp::LessThanOrEqual, value))
                .unwrap_or(false),
            SqlPredicate::GreaterThanEq { column, value } => row
                .get(column)
                .map(|v| v.compare(&CompareOp::GreaterThanOrEqual, value))
                .unwrap_or(false),
            SqlPredicate::In { column, values } => {
                row.get(column).map(|v| values.contains(v)).unwrap_or(false)
            }
            SqlPredicate::And(left, right) => {
                self.eval_predicate(left, row) && self.eval_predicate(right, row)
            }
            SqlPredicate::Or(left, right) => {
                self.eval_predicate(left, row) || self.eval_predicate(right, row)
            }
            SqlPredicate::Not(p) => !self.eval_predicate(p, row),
        }
    }

    /// Compute SQL score from row data (placeholder - can be extended)
    fn compute_sql_score_from_row(&self, row: &HashMap<String, SqlValue>) -> f32 {
        row.get("_score").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32
    }

    /// Get the number of vectors in the index
    pub fn len(&self) -> usize {
        self.vector_index.len()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the distance metric used by this searcher
    pub fn metric(&self) -> DistanceMetric {
        self.metric
    }
}

/// Merge parallel search results with SQL scores
pub fn merge_with_sql_scores(
    vector_results: Vec<(u64, f32)>, // (id, vector_score)
    sql_scores: &[(u64, f32)],       // (id, sql_score)
    alpha: f32,
    beta: f32,
) -> Vec<(u64, f32)> {
    let sql_map: std::collections::HashMap<u64, f32> = sql_scores.iter().cloned().collect();

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
    vector_scores: &[(u64, f32)],     // (id, vector_score)
    alpha: f32,
    beta: f32,
) -> Vec<(u64, f32)> {
    let vec_map: std::collections::HashMap<u64, f32> = vector_scores.iter().cloned().collect();

    let mut combined: Vec<(u64, f32)> = initial_results
        .into_iter()
        .filter_map(|(id, initial_score)| {
            vec_map.get(&id).map(|&vector_score| {
                let combined = alpha * initial_score + beta * vector_score;
                (id, combined)
            })
        })
        .collect();

    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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
        let sql_scores: Vec<_> = (0..100u64)
            .map(|id| (id, 1.0 - (id as f32 / 100.0)))
            .collect();

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

        let searcher =
            HybridSearcher::with_config(crate::metrics::DistanceMetric::Euclidean, config);

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
