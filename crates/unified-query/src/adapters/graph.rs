use crate::api::{GraphResult, UnifiedQueryRequest};
use crate::error::QueryResult;
use crate::QueryPlan;

pub struct GraphAdapter;

impl GraphAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &self,
        _request: &UnifiedQueryRequest,
        _plan: &QueryPlan,
    ) -> QueryResult<Vec<GraphResult>> {
        // TODO: Integrate with sqlrustgo-graph
        QueryResult::Err("Not implemented".to_string())
    }
}

impl Default for GraphAdapter {
    fn default() -> Self {
        Self::new()
    }
}
