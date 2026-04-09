use crate::api::{QueryMode, UnifiedQueryRequest, Weights};
use crate::error::UnifiedQueryError;

#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub execute_sql: bool,
    pub execute_vector: bool,
    pub execute_graph: bool,
    pub weights: Weights,
    pub top_k: u32,
    pub offset: u32,
}

pub struct QueryRouter;

impl QueryRouter {
    pub fn new() -> Self {
        Self
    }

    pub fn route(&self, request: &UnifiedQueryRequest) -> Result<QueryPlan, UnifiedQueryError> {
        let (execute_sql, execute_vector, execute_graph) = match request.mode {
            QueryMode::SQL => (true, false, false),
            QueryMode::Vector => (false, true, false),
            QueryMode::Graph => (false, false, true),
            QueryMode::SQLVector => (true, true, false),
            QueryMode::SQLGraph => (true, false, true),
            QueryMode::VectorGraph => (false, true, true),
            QueryMode::SQLVectorGraph => (true, true, true),
        };

        let weights = request.weights.clone().unwrap_or_default();
        let top_k = request.top_k.unwrap_or(10);
        let offset = request.offset.unwrap_or(0);

        Ok(QueryPlan {
            execute_sql,
            execute_vector,
            execute_graph,
            weights,
            top_k,
            offset,
        })
    }
}

impl Default for QueryRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_sql_mode() {
        let router = QueryRouter::new();
        let request = UnifiedQueryRequest {
            query: "SELECT * FROM users".to_string(),
            mode: QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: None,
            offset: None,
        };

        let plan = router.route(&request).unwrap();
        assert!(plan.execute_sql);
        assert!(!plan.execute_vector);
        assert!(!plan.execute_graph);
    }

    #[test]
    fn test_router_unified_mode() {
        let router = QueryRouter::new();
        let request = UnifiedQueryRequest {
            query: "test".to_string(),
            mode: QueryMode::SQLVectorGraph,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: None,
            offset: None,
        };

        let plan = router.route(&request).unwrap();
        assert!(plan.execute_sql);
        assert!(plan.execute_vector);
        assert!(plan.execute_graph);
    }
}
