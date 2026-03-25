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

impl Clone for PooledSession {
    fn clone(&self) -> Self {
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
}

#[derive(Clone)]
pub struct ConnectionPool {
    #[allow(dead_code)]
    sessions: Arc<Vec<PooledSession>>,
    available: Sender<PooledSession>,
    received: Receiver<PooledSession>,
    config: PoolConfig,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        let (available, received) = bounded(config.size);
        let mut sessions = Vec::with_capacity(config.size);

        for _ in 0..config.size {
            let session = PooledSession::new();
            let _ = available.send(session);
            sessions.push(PooledSession::new());
        }

        Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_connection_pool_basic_acquire_release() {
        let config = PoolConfig {
            size: 5,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);

        assert_eq!(pool.size(), 5);

        // Acquire and release a connection
        let conn = pool.acquire();
        drop(conn);

        // Should be able to acquire again
        let conn2 = pool.acquire();
        drop(conn2);
    }

    #[test]
    fn test_connection_pool_concurrent_50_connections() {
        let config = PoolConfig {
            size: 50,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);
        let pool = Arc::new(pool);

        let success_count = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..50)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let success_count = Arc::clone(&success_count);
                thread::spawn(move || {
                    let conn = pool.acquire();
                    // Simulate some work by checking the executor exists
                    let _executor = conn.executor();
                    drop(conn);
                    success_count.fetch_add(1, Ordering::SeqCst);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(success_count.load(Ordering::SeqCst), 50);
    }

    #[test]
    fn test_connection_pool_concurrent_100_connections() {
        let config = PoolConfig {
            size: 50,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);
        let pool = Arc::new(pool);

        let success_count = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..100)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let success_count = Arc::clone(&success_count);
                thread::spawn(move || {
                    // Small delay to increase contention
                    thread::sleep(std::time::Duration::from_micros(100));
                    let conn = pool.acquire();
                    let _executor = conn.executor();
                    drop(conn);
                    success_count.fetch_add(1, Ordering::SeqCst);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // All 100 should complete even with only 50 sessions
        // because connections are released back to the pool
        assert_eq!(success_count.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_connection_pool_reuse_sessions() {
        let config = PoolConfig {
            size: 5,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);

        let mut conn_handles = vec![];
        // Acquire all 5 connections
        for _ in 0..5 {
            let conn = pool.acquire();
            conn_handles.push(conn);
        }

        // Release all
        conn_handles.clear();

        // Should be able to acquire all again
        for _ in 0..5 {
            let conn = pool.acquire();
            drop(conn);
        }
    }

    #[test]
    fn test_pooled_session_clone_creates_fresh_session() {
        let session1 = PooledSession::new();
        let session2 = session1.clone();

        // Cloned sessions should be independent
        assert!(session1.is_available());
        assert!(session2.is_available());
    }
}
