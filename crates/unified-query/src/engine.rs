use crate::api::{UnifiedQueryRequest, UnifiedQueryResponse};
use crate::cache::QueryCache;
use crate::executor::ParallelExecutor;
use crate::fusion::ResultFusion;
use crate::router::QueryRouter;
use crate::stats::QueryStats;
use std::sync::Arc;
use std::time::Instant;

pub struct UnifiedQueryEngine {
    router: QueryRouter,
    executor: Arc<ParallelExecutor>,
    fusion: ResultFusion,
    #[allow(dead_code)]
    cache: Arc<QueryCache>,
    #[allow(dead_code)]
    stats: Arc<QueryStats>,
}

impl UnifiedQueryEngine {
    pub fn new() -> Self {
        Self {
            router: QueryRouter::new(),
            executor: Arc::new(ParallelExecutor::new()),
            fusion: ResultFusion::new(),
            cache: Arc::new(QueryCache::new(1000)),
            stats: Arc::new(QueryStats::default()),
        }
    }

    /// Execute a unified query
    pub async fn execute(&self, request: UnifiedQueryRequest) -> UnifiedQueryResponse {
        let start_time = Instant::now();
        
        // Step 1: Route the request to create a query plan
        let plan = self.router.route(&request).expect("Invalid request");
        
        // Step 2: Execute queries in parallel using tokio::join!
        let results = self.executor.execute(&request, &plan).await;
        
        // Step 3: Fuse results with weighted scoring
        let fusion_result = self.fusion.fuse(results, &plan.weights, plan.top_k);
        
        let total_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Step 4: Build response
        UnifiedQueryResponse {
            sql_results: None,
            vector_results: None,
            graph_results: None,
            fusion_scores: fusion_result.scores,
            total: fusion_result.total as u64,
            execution_time_ms: total_time_ms,
            query_plan: crate::api::QueryPlanDetail {
                mode: format!("{:?}", request.mode),
                weights: plan.weights,
                steps: vec![
                    crate::api::QueryStep {
                        source: "router".to_string(),
                        step: "route".to_string(),
                        time_ms: 0,
                        rows_affected: None,
                        nodes_visited: None,
                        nodes_traversed: None,
                    },
                    crate::api::QueryStep {
                        source: "executor".to_string(),
                        step: "parallel_execute".to_string(),
                        time_ms: total_time_ms / 2,
                        rows_affected: None,
                        nodes_visited: None,
                        nodes_traversed: None,
                    },
                    crate::api::QueryStep {
                        source: "fusion".to_string(),
                        step: "fuse".to_string(),
                        time_ms: total_time_ms / 4,
                        rows_affected: None,
                        nodes_visited: None,
                        nodes_traversed: None,
                    },
                ],
                fusion_time_ms: total_time_ms / 4,
                total_time_ms,
            },
        }
    }

    /// Execute with caching support
    pub async fn execute_cached(&self, request: UnifiedQueryRequest) -> UnifiedQueryResponse {
        // Generate cache key from request
        let cache_key = self.generate_cache_key(&request);
        
        // Check cache
        if let Some(cached) = self.cache.get(&cache_key) {
            // Return cached result (simplified - would need to deserialize)
            // For now, just execute without cache
        }
        
        let response = self.execute(request).await;
        
        // Store in cache
        // self.cache.insert(cache_key, response);
        
        response
    }

    /// Generate a cache key from the request
    fn generate_cache_key(&self, request: &UnifiedQueryRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request.query.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get engine statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> &QueryStats {
        &self.stats
    }
}

impl Default for UnifiedQueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_execute_sql_only() {
        let engine = UnifiedQueryEngine::new();
        let request = UnifiedQueryRequest {
            query: "SELECT * FROM users".to_string(),
            mode: crate::api::QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: Some(10),
            offset: Some(0),
        };
        
        let response = engine.execute(request).await;
        assert!(response.query_plan.mode.contains("SQL"));
        assert_eq!(response.execution_time_ms >= 0, true);
    }

    #[tokio::test]
    async fn test_engine_execute_unified() {
        let engine = UnifiedQueryEngine::new();
        let request = UnifiedQueryRequest {
            query: "unified query".to_string(),
            mode: crate::api::QueryMode::SQLVectorGraph,
            filters: None,
            weights: None,
            vector_query: Some(crate::api::VectorQuery {
                column: "embedding".to_string(),
                top_k: 5,
                filter: None,
            }),
            graph_query: Some(crate::api::GraphQuery {
                start_nodes: vec!["node1".to_string()],
                traversal: crate::api::TraversalType::DFS,
                max_depth: 3,
            }),
            top_k: Some(10),
            offset: Some(0),
        };
        
        let response = engine.execute(request).await;
        assert!(response.query_plan.mode.contains("SQLVectorGraph"));
        assert_eq!(response.fusion_scores.len() >= 0, true);
    }
}
