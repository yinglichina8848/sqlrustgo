use crate::api::{UnifiedQueryRequest, VectorResult};
use crate::error::QueryResult;
use crate::QueryPlan;

pub struct VectorAdapter;

impl VectorAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &self,
        _request: &UnifiedQueryRequest,
        _plan: &QueryPlan,
    ) -> QueryResult<Vec<VectorResult>> {
        // TODO: Integrate with sqlrustgo-vector
        QueryResult::Err("Not implemented".to_string())
    }
}

impl Default for VectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}
