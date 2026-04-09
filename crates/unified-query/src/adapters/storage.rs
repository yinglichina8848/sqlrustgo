use crate::api::UnifiedQueryRequest;
use crate::error::QueryResult;
use crate::QueryPlan;
use serde_json::Value;

pub struct StorageAdapter;

impl StorageAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(
        &self,
        _request: &UnifiedQueryRequest,
        _plan: &QueryPlan,
    ) -> QueryResult<Vec<Vec<Value>>> {
        // TODO: Integrate with sqlrustgo-storage
        QueryResult::Err("Not implemented".to_string())
    }
}

impl Default for StorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}
