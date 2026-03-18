use crossbeam_channel::{bounded, Receiver, Sender};
pub use sqlrustgo_common::connection_pool::PoolConfig;
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
    pub fn new(storage: Arc<MemoryStorage>) -> Self {
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
        let storage = Arc::new(MemoryStorage::new());
        Self::new(storage)
    }
}

impl Clone for PooledSession {
    fn clone(&self) -> Self {
        let storage_ptr = Arc::as_ptr(&self.storage);
        let executor = LocalExecutor::new(unsafe { &*storage_ptr });
        Self {
            executor,
            storage: Arc::clone(&self.storage),
            transaction_id: None,
            in_use: false,
        }
    }
}

#[derive(Clone)]
pub struct ConnectionPool {
    #[allow(dead_code)]
    shared_storage: Arc<MemoryStorage>,
    #[allow(dead_code)]
    sessions: Arc<Vec<PooledSession>>,
    available: Sender<PooledSession>,
    received: Receiver<PooledSession>,
    config: PoolConfig,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        let shared_storage = Arc::new(MemoryStorage::new());
        let (available, received) = bounded(config.size);
        let mut sessions = Vec::with_capacity(config.size);

        for _ in 0..config.size {
            let session = PooledSession::new(Arc::clone(&shared_storage));
            let _ = available.send(session.clone());
            sessions.push(session);
        }

        Self {
            shared_storage,
            sessions: Arc::new(sessions),
            available,
            received,
            config,
        }
    }

    pub fn acquire(&self) -> PooledConnection {
        let session = self.received.recv().unwrap();
        PooledConnection {
            session,
            pool: self.clone(),
        }
    }

    pub fn size(&self) -> usize {
        self.config.size
    }

    fn release(&self, session: PooledSession) {
        let _ = self.available.send(session);
    }
}

pub struct PooledConnection {
    session: PooledSession,
    pool: ConnectionPool,
}

impl PooledConnection {
    pub fn executor(&self) -> &LocalExecutor<'_> {
        &self.session.executor
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        self.pool.release(self.session.clone());
    }
}
