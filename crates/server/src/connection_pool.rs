use sqlrustgo_executor::LocalExecutor;
use sqlrustgo_storage::MemoryStorage;
use std::sync::Arc;

pub struct PooledSession {
    pub executor: LocalExecutor<'static>,
    pub storage: Arc<MemoryStorage>,
    pub transaction_id: Option<u64>,
    in_use: bool,
}

impl std::fmt::Debug for PooledSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledSession")
            .field("transaction_id", &self.transaction_id)
            .field("in_use", &self.in_use)
            .finish()
    }
}

impl PooledSession {
    pub fn new() -> Self {
        let storage = Arc::new(MemoryStorage::new());
        let storage_ptr = Arc::as_ptr(&storage);
        let executor = LocalExecutor::new(unsafe { &*storage_ptr });
        Self {
            executor,
            storage,
            transaction_id: None,
            in_use: false,
        }
    }

    pub fn is_available(&self) -> bool {
        !self.in_use
    }
}

impl Default for PooledSession {
    fn default() -> Self {
        Self::new()
    }
}
