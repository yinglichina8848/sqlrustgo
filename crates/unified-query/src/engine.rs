use crate::api::{UnifiedQueryRequest, UnifiedQueryResponse};
use crate::cache::QueryCache;
use crate::executor::ParallelExecutor;
use crate::fusion::ResultFusion;
use crate::router::QueryRouter;
use crate::stats::QueryStats;
use std::sync::Arc;

pub struct UnifiedQueryEngine {
    router: QueryRouter,
    executor: ParallelExecutor,
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
            executor: ParallelExecutor::new(),
            fusion: ResultFusion::new(),
            cache: Arc::new(QueryCache::new(1000)),
            stats: Arc::new(QueryStats::default()),
        }
    }

    pub async fn execute(&self, request: UnifiedQueryRequest) -> UnifiedQueryResponse {
        let plan = self.router.route(&request).expect("Invalid request");
        
        let results = self.executor.execute(&request, &plan).await;
        
        let fusion_result = self.fusion.fuse(results, &plan.weights, plan.top_k);
        
        UnifiedQueryResponse {
            sql_results: None,
            vector_results: None,
            graph_results: None,
            fusion_scores: fusion_result.scores,
            total: fusion_result.total as u64,
            execution_time_ms: 0,
            query_plan: crate::api::QueryPlanDetail {
                mode: format!("{:?}", request.mode),
                weights: plan.weights,
                steps: vec![],
                fusion_time_ms: 0,
                total_time_ms: 0,
            },
        }
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
    async fn test_engine_execute() {
        let engine = UnifiedQueryEngine::new();
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
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
    }
}
