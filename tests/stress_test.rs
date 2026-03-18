//! Stress Tests for v1.6.0 Beta

use sqlrustgo_common::connection_pool::PoolConfig;
use sqlrustgo_server::connection_pool::ConnectionPool;
use sqlrustgo_transaction::lock::{LockError, LockManager, LockMode};
use sqlrustgo_transaction::mvcc::TxId;
use sqlrustgo_transaction::TransactionManager;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod connection_pool_stress {
    use super::*;

    #[test]
    fn test_pool_10_connections() {
        let config = PoolConfig {
            size: 10,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let pool = pool.clone();
                thread::spawn(move || {
                    let _conn = pool.acquire();
                    thread::sleep(Duration::from_millis(10));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_pool_20_connections() {
        let config = PoolConfig {
            size: 20,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);

        let handles: Vec<_> = (0..20)
            .map(|_| {
                let pool = pool.clone();
                thread::spawn(move || {
                    let _conn = pool.acquire();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_pool_50_connections() {
        let config = PoolConfig {
            size: 50,
            timeout_ms: 10000,
        };
        let pool = ConnectionPool::new(config);

        let handles: Vec<_> = (0..50)
            .map(|_| {
                let pool = pool.clone();
                thread::spawn(move || {
                    let _conn = pool.acquire();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_pool_rapid_acquire_release() {
        let config = PoolConfig {
            size: 10,
            timeout_ms: 1000,
        };
        let pool = ConnectionPool::new(config);

        for _ in 0..100 {
            let _conn = pool.acquire();
        }
    }
}

#[cfg(test)]
mod lock_stress {
    use super::*;

    #[test]
    fn test_concurrent_shared_locks() {
        let mut manager = LockManager::new();
        let key = vec![1, 2, 3];

        for i in 0..10 {
            let result = manager.acquire_lock(TxId::new(i), key.clone(), LockMode::Shared);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_concurrent_exclusive_locks() {
        let mut manager = LockManager::new();

        for i in 0..20 {
            let key = vec![i as u8];
            let result = manager.acquire_lock(TxId::new(i), key, LockMode::Exclusive);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_lock_release_and_reacquire() {
        let mut manager = LockManager::new();
        let key = vec![1];

        for _ in 0..10 {
            manager
                .acquire_lock(TxId::new(1), key.clone(), LockMode::Exclusive)
                .unwrap();
            manager.release_lock(TxId::new(1), &key).unwrap();
        }
    }
}

#[cfg(test)]
mod transaction_stress {
    use super::*;

    #[test]
    fn test_transaction_begin_commit() {
        let mgr = RwLock::new(TransactionManager::new());

        let tx_id = mgr.write().unwrap().begin().unwrap();
        assert!(tx_id.is_valid());

        let result = mgr.write().unwrap().commit();
        assert!(result.is_ok());
    }

    #[test]
    fn test_transaction_rollback() {
        let mgr = RwLock::new(TransactionManager::new());

        let tx_id = mgr.write().unwrap().begin().unwrap();
        assert!(tx_id.is_valid());

        let result = mgr.write().unwrap().rollback();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_transactions() {
        // Each transaction needs its own manager (or we need to commit before starting next)
        let mgr = RwLock::new(TransactionManager::new());

        for i in 0..5 {
            let tx_id = mgr.write().unwrap().begin().unwrap();
            assert!(tx_id.is_valid());
            mgr.write().unwrap().commit().unwrap();
        }
    }
}
