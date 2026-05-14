use super::registry::{IdempotencyRecord, IdempotencyRegistry};
use std::sync::Arc;

pub struct IdempotencyWalAdapter {
    registry: Arc<IdempotencyRegistry>,
}

impl IdempotencyWalAdapter {
    pub fn new(registry: Arc<IdempotencyRegistry>) -> Self {
        Self { registry }
    }

    pub fn log_pending(
        &self,
        _record: &IdempotencyRecord,
    ) -> Result<(), super::registry::IdempotencyError> {
        Ok(())
    }

    pub fn log_commit(&self, key: &str) -> Result<(), super::registry::IdempotencyError> {
        self.registry.mark_committed(key)
    }
}
