use crate::adapters::{GraphAdapter, StorageAdapter, VectorAdapter};
use crate::api::{GraphResult, UnifiedQueryRequest, VectorResult};
use crate::error::QueryResult;
use crate::router::QueryPlan;
use serde_json::Value;

pub struct ParallelExecutor {
    storage: StorageAdapter,
    vector: VectorAdapter,
    graph: GraphAdapter,
}

pub struct ParallelQueryResults {
    pub sql_results: Option<QueryResult<Vec<Vec<Value>>>>,
    pub vector_results: Option<QueryResult<Vec<VectorResult>>>,
    pub graph_results: Option<QueryResult<Vec<GraphResult>>>,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            storage: StorageAdapter::new(),
            vector: VectorAdapter::new(),
            graph: GraphAdapter::new(),
        }
    }

    pub async fn execute(
        &self,
        request: &UnifiedQueryRequest,
        plan: &QueryPlan,
    ) -> ParallelQueryResults {
        let sql_future = if plan.execute_sql {
            Some(self.storage.execute(request, plan))
        } else {
            None
        };

        let vector_future = if plan.execute_vector {
            Some(self.vector.execute(request, plan))
        } else {
            None
        };

        let graph_future = if plan.execute_graph {
            Some(self.graph.execute(request, plan))
        } else {
            None
        };

        let (sql_results, vector_results, graph_results) = tokio::join!(
            async {
                if let Some(f) = sql_future {
                    f.await
                } else {
                    QueryResult::Err("Not executed".to_string())
                }
            },
            async {
                if let Some(f) = vector_future {
                    f.await
                } else {
                    QueryResult::Err("Not executed".to_string())
                }
            },
            async {
                if let Some(f) = graph_future {
                    f.await
                } else {
                    QueryResult::Err("Not executed".to_string())
                }
            },
        );

        ParallelQueryResults {
            sql_results: if plan.execute_sql { Some(sql_results) } else { None },
            vector_results: if plan.execute_vector { Some(vector_results) } else { None },
            graph_results: if plan.execute_graph { Some(graph_results) } else { None },
        }
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_executor_sql_only() {
        let executor = ParallelExecutor::new();
        let plan = QueryPlan {
            execute_sql: true,
            execute_vector: false,
            execute_graph: false,
            weights: Default::default(),
            top_k: 10,
            offset: 0,
        };
        
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: crate::api::QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: None,
            offset: None,
        };
        
        let results = executor.execute(&request, &plan).await;
        assert!(results.sql_results.is_some());
        assert!(results.vector_results.is_none());
        assert!(results.graph_results.is_none());
    }
}
