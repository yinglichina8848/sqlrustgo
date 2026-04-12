//! VectorScan Executor - Volcano Model implementation for vector similarity search
//!
//! This module provides VectorScanVolcanoExecutor which implements the VolcanoExecutor trait
//! for performing vector similarity searches with optional SQL predicate pre-filtering.

use crate::VolcanoExecutor;
use parking_lot::RwLock;
use sqlrustgo_planner::{Expr, Schema};
use sqlrustgo_types::{RowRef, SqlResult, Value};
use sqlrustgo_vector::{HybridSearchConfig, HybridSearcher};
use std::any::Any;
use std::sync::Arc;

/// VectorScanVolcanoExecutor - executes vector similarity search operations
///
/// This executor takes a vector index and performs similarity search.
/// It can optionally use SQL predicates for pre-filtering before vector search.
#[allow(dead_code)]
pub struct VectorScanVolcanoExecutor {
    /// Vector index reference (Arc for shared access)
    vector_index: Arc<RwLock<HybridSearcher>>,
    /// Query vector for similarity search
    query_vector: Vec<f32>,
    /// Number of top results to return
    top_k: usize,
    /// Optional SQL predicate for pre-filtering
    predicate: Option<Expr>,
    /// Schema of the output (matches table schema)
    schema: Schema,
    /// Input schema for predicate evaluation
    input_schema: Schema,
    /// SQL scores for hybrid search (pre-filtered IDs with scores)
    sql_scores: Vec<(u64, f32)>,
    /// Current result buffer
    results: Vec<(u64, f32)>, // (row_id, combined_score)
    result_idx: usize,
    /// Whether executor is initialized
    initialized: bool,
}

impl VectorScanVolcanoExecutor {
    /// Create a new VectorScanVolcanoExecutor with default config
    pub fn new(
        vector_index: Arc<RwLock<HybridSearcher>>,
        query_vector: Vec<f32>,
        top_k: usize,
        schema: Schema,
    ) -> Self {
        Self {
            vector_index,
            query_vector,
            top_k,
            predicate: None,
            schema: schema.clone(),
            input_schema: schema,
            sql_scores: Vec::new(),
            results: Vec::new(),
            result_idx: 0,
            initialized: false,
        }
    }

    /// Create with custom config
    /// Note: This creates a fresh searcher with the given config.
    /// For simplicity, this implementation doesn't preserve existing vectors.
    pub fn with_config(
        vector_index: Arc<RwLock<HybridSearcher>>,
        query_vector: Vec<f32>,
        top_k: usize,
        config: HybridSearchConfig,
        schema: Schema,
    ) -> Self {
        // Get metric from existing searcher and create new one with config
        let metric = {
            let searcher = vector_index.read();
            searcher.metric()
        };
        let _new_searcher = HybridSearcher::with_config(metric, config);
        // Note: In a full implementation, we would transfer vectors to new searcher
        // For now, we just note that with_config is primarily for new indexes

        Self {
            vector_index,
            query_vector,
            top_k,
            predicate: None,
            schema: schema.clone(),
            input_schema: schema,
            sql_scores: Vec::new(),
            results: Vec::new(),
            result_idx: 0,
            initialized: false,
        }
    }

    /// Create with SQL predicate pre-filter
    pub fn with_predicate(
        vector_index: Arc<RwLock<HybridSearcher>>,
        query_vector: Vec<f32>,
        top_k: usize,
        predicate: Expr,
        schema: Schema,
        _input_schema: Schema,
    ) -> Self {
        Self {
            vector_index,
            query_vector,
            top_k,
            predicate: Some(predicate),
            schema: schema.clone(),
            input_schema: schema,
            sql_scores: Vec::new(),
            results: Vec::new(),
            result_idx: 0,
            initialized: false,
        }
    }

    /// Set SQL scores for hybrid search (from pre-filter step)
    pub fn with_sql_scores(mut self, sql_scores: Vec<(u64, f32)>) -> Self {
        self.sql_scores = sql_scores;
        self
    }

    /// Get the query vector
    pub fn query_vector(&self) -> &[f32] {
        &self.query_vector
    }

    /// Get top_k setting
    pub fn top_k(&self) -> usize {
        self.top_k
    }
}

impl VolcanoExecutor for VectorScanVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        // Perform vector search
        let index = self.vector_index.read();

        let search_result = if self.sql_scores.is_empty() {
            // Pure vector search (no pre-filter)
            // Get all vectors with uniform SQL score of 1.0
            let n = index.len() as u64;
            let all_scores: Vec<(u64, f32)> = (0..n).map(|id| (id, 1.0)).collect();
            index.search_hybrid(&self.query_vector, &all_scores, self.top_k)
        } else {
            // Hybrid search with SQL pre-filter scores
            index.search_hybrid(&self.query_vector, &self.sql_scores, self.top_k)
        };

        match search_result {
            Ok(result) => {
                self.results = result
                    .entries
                    .into_iter()
                    .map(|e| (e.id, e.score))
                    .collect();
                self.result_idx = 0;
                self.initialized = true;
                Ok(())
            }
            Err(e) => Err(sqlrustgo_types::SqlError::ExecutionError(format!(
                "Vector search failed: {:?}",
                e
            ))),
        }
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(sqlrustgo_types::SqlError::ExecutionError(
                "Executor not initialized".to_string(),
            ));
        }

        if self.result_idx >= self.results.len() {
            return Ok(None);
        }

        let (id, score) = self.results[self.result_idx];
        self.result_idx += 1;

        // Return row as [id, score] - in real impl, would join with actual table data
        // The schema determines what columns are returned
        let row = vec![Value::Integer(id as i64), Value::Float(score as f64)];

        Ok(Some(row))
    }

    fn next_ref(&mut self) -> SqlResult<Option<RowRef<'_>>> {
        // Vector scan doesn't support zero-copy yet
        Ok(None)
    }

    fn close(&mut self) -> SqlResult<()> {
        self.results.clear();
        self.result_idx = 0;
        self.initialized = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn name(&self) -> &str {
        "VectorScan"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{DataType, Field};
    use sqlrustgo_vector::DistanceMetric;

    fn create_test_schema() -> Schema {
        Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("embedding".to_string(), DataType::Text),
        ])
    }

    #[test]
    fn test_vector_scan_name() {
        let vector_index = Arc::new(RwLock::new(HybridSearcher::new(DistanceMetric::Cosine)));
        let query = vec![0.1f32; 128];

        let executor =
            VectorScanVolcanoExecutor::new(vector_index, query, 10, create_test_schema());

        assert_eq!(executor.name(), "VectorScan");
    }

    #[test]
    fn test_vector_scan_schema() {
        let vector_index = Arc::new(RwLock::new(HybridSearcher::new(DistanceMetric::Cosine)));
        let query = vec![0.1f32; 128];

        let executor =
            VectorScanVolcanoExecutor::new(vector_index, query, 10, create_test_schema());

        assert_eq!(executor.schema().fields.len(), 2);
    }

    #[test]
    fn test_vector_scan_init_not_initialized() {
        let vector_index = Arc::new(RwLock::new(HybridSearcher::new(DistanceMetric::Cosine)));
        let query = vec![0.1f32; 128];

        let executor =
            VectorScanVolcanoExecutor::new(vector_index, query, 10, create_test_schema());

        assert!(!executor.is_initialized());
    }

    #[test]
    fn test_vector_scan_init_and_next() {
        let vector_index = Arc::new(RwLock::new(HybridSearcher::new(DistanceMetric::Cosine)));
        let query = vec![0.1f32; 128];

        let mut executor =
            VectorScanVolcanoExecutor::new(vector_index, query, 10, create_test_schema());

        // Insert some vectors
        {
            let mut index = executor.vector_index.write();
            for i in 0..5u64 {
                let vec = vec![i as f32 * 0.1; 128];
                let _ = index.insert(i, &vec, 1.0);
            }
        }

        // Init should succeed
        let init_result = executor.init();
        assert!(init_result.is_ok());
        assert!(executor.is_initialized());
    }

    #[test]
    fn test_vector_scan_with_sql_scores() {
        let vector_index = Arc::new(RwLock::new(HybridSearcher::new(DistanceMetric::Cosine)));
        let query = vec![0.1f32; 128];

        let sql_scores = vec![(0u64, 1.0), (1u64, 0.8), (2u64, 0.6)];

        let executor =
            VectorScanVolcanoExecutor::new(vector_index, query, 10, create_test_schema())
                .with_sql_scores(sql_scores);

        assert_eq!(executor.name(), "VectorScan");
    }

    #[test]
    fn test_vector_scan_query_vector_access() {
        let vector_index = Arc::new(RwLock::new(HybridSearcher::new(DistanceMetric::Cosine)));
        let query = vec![0.5f32; 128];

        let executor =
            VectorScanVolcanoExecutor::new(vector_index, query.clone(), 10, create_test_schema());

        assert_eq!(executor.query_vector(), &query);
        assert_eq!(executor.top_k(), 10);
    }

    #[test]
    fn test_vector_scan_close_resets_state() {
        let vector_index = Arc::new(RwLock::new(HybridSearcher::new(DistanceMetric::Cosine)));
        let query = vec![0.1f32; 128];

        let mut executor =
            VectorScanVolcanoExecutor::new(vector_index, query, 10, create_test_schema());

        // Insert vectors and init
        {
            let mut index = executor.vector_index.write();
            for i in 0..3u64 {
                let vec = vec![i as f32; 128];
                let _ = index.insert(i, &vec, 1.0);
            }
        }

        executor.init().unwrap();
        assert!(executor.is_initialized());

        // Close should reset
        let close_result = executor.close();
        assert!(close_result.is_ok());
        assert!(!executor.is_initialized());
    }
}
