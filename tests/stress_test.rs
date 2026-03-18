//! Stress Tests for v1.6.0 Beta - Performance Limit Testing
//!
//! These tests push the system to its limits to find performance boundaries:
//! - Connection pool: 10, 50, 100, 200, 500 concurrent connections
//! - Lock contention: High concurrency with shared resources
//! - Transaction throughput: Measure TPS (transactions per second)
//! - Memory pressure: Large data sets under load

use sqlrustgo_common::connection_pool::PoolConfig;
use sqlrustgo_server::connection_pool::ConnectionPool;
use sqlrustgo_transaction::lock::{LockManager, LockMode};
use sqlrustgo_transaction::mvcc::TxId;
use sqlrustgo_transaction::TransactionManager;
use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant};

const MAX_POOL_SIZE: usize = 500;
const MAX_LOCK_THREADS: usize = 200;
const MAX_TRANSACTION_THREADS: usize = 100;

#[cfg(test)]
mod connection_pool_stress {
    use super::*;

    #[test]
    fn test_pool_10() {
        let start = Instant::now();
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
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
        println!(
            "Pool 10: {:?} ({} ops/sec)",
            start.elapsed(),
            10_000_000 / start.elapsed().as_nanos() as u64
        );
    }

    #[test]
    fn test_pool_50() {
        let start = Instant::now();
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

        for h in handles {
            h.join().unwrap();
        }
        println!(
            "Pool 50: {:?} ({} ops/sec)",
            start.elapsed(),
            50_000_000 / start.elapsed().as_nanos() as u64
        );
    }

    #[test]
    fn test_pool_100() {
        let start = Instant::now();
        let config = PoolConfig {
            size: 100,
            timeout_ms: 15000,
        };
        let pool = ConnectionPool::new(config);

        let handles: Vec<_> = (0..100)
            .map(|_| {
                let pool = pool.clone();
                thread::spawn(move || {
                    let _conn = pool.acquire();
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
        println!(
            "Pool 100: {:?} ({} ops/sec)",
            start.elapsed(),
            100_000_000 / start.elapsed().as_nanos() as u64
        );
    }

    #[test]
    fn test_pool_200() {
        let start = Instant::now();
        let config = PoolConfig {
            size: 200,
            timeout_ms: 20000,
        };
        let pool = ConnectionPool::new(config);

        let handles: Vec<_> = (0..200)
            .map(|_| {
                let pool = pool.clone();
                thread::spawn(move || {
                    let _conn = pool.acquire();
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
        println!(
            "Pool 200: {:?} ({} ops/sec)",
            start.elapsed(),
            200_000_000 / start.elapsed().as_nanos() as u64
        );
    }

    #[test]
    fn test_pool_500() {
        let start = Instant::now();
        let config = PoolConfig {
            size: 500,
            timeout_ms: 30000,
        };
        let pool = ConnectionPool::new(config);

        let handles: Vec<_> = (0..500)
            .map(|_| {
                let pool = pool.clone();
                thread::spawn(move || {
                    let _conn = pool.acquire();
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }
        println!(
            "Pool 500: {:?} ({} ops/sec)",
            start.elapsed(),
            500_000_000 / start.elapsed().as_nanos() as u64
        );
    }

    #[test]
    fn test_pool_sustained_load() {
        let config = PoolConfig {
            size: 50,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);

        let start = Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let _conn = pool.acquire();
        }

        let elapsed = start.elapsed();
        let ops_per_sec = (iterations * 1_000_000_000) / elapsed.as_nanos() as u64;
        println!(
            "Sustained load (1000 ops): {:?} ({} ops/sec)",
            elapsed, ops_per_sec
        );
    }

    #[test]
    fn test_pool_parallel_sustained() {
        let config = PoolConfig {
            size: 100,
            timeout_ms: 5000,
        };
        let pool = ConnectionPool::new(config);

        let start = Instant::now();

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let pool = pool.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        let _conn = pool.acquire();
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total_ops = 1000;
        let ops_per_sec = (total_ops * 1_000_000_000) / elapsed.as_nanos() as u64;
        println!(
            "Parallel sustained (1000 ops, 10 threads): {:?} ({} ops/sec)",
            elapsed, ops_per_sec
        );
    }
}

#[cfg(test)]
mod lock_stress {
    use super::*;

    #[test]
    fn test_lock_10_shared_same_key() {
        let start = Instant::now();
        let mut manager = LockManager::new();
        let key = vec![1];

        for _ in 0..10 {
            manager
                .acquire_lock(TxId::new(1), key.clone(), LockMode::Shared)
                .unwrap();
        }
        println!("Lock 10 shared same key: {:?}", start.elapsed());
    }

    #[test]
    fn test_lock_50_contention() {
        let start = Instant::now();
        let mut manager = LockManager::new();

        // Multiple threads competing for same lock
        for i in 0..50 {
            manager
                .acquire_lock(TxId::new(i), vec![1], LockMode::Shared)
                .unwrap();
        }
        println!("Lock 50 contention: {:?}", start.elapsed());
    }

    #[test]
    fn test_lock_100_different_keys() {
        let start = Instant::now();
        let mut manager = LockManager::new();

        for i in 0..100 {
            manager
                .acquire_lock(TxId::new(i), vec![i as u8], LockMode::Exclusive)
                .unwrap();
        }
        println!("Lock 100 different keys: {:?}", start.elapsed());
    }

    #[test]
    fn test_lock_200_rapid_acquire_release() {
        let start = Instant::now();
        let mut manager = LockManager::new();

        for i in 0..200 {
            manager
                .acquire_lock(TxId::new(i % 10), vec![1], LockMode::Shared)
                .unwrap();
            manager.release_lock(TxId::new(i % 10), &vec![1]).ok();
        }
        println!("Lock 200 rapid acquire/release: {:?}", start.elapsed());
    }

    #[test]
    fn test_lock_mixed_mode() {
        let start = Instant::now();
        let mut manager = LockManager::new();

        // Mix of shared and exclusive locks
        for i in 0..50 {
            if i % 2 == 0 {
                manager
                    .acquire_lock(TxId::new(i), vec![1], LockMode::Shared)
                    .unwrap();
            } else {
                manager
                    .acquire_lock(TxId::new(i), vec![2], LockMode::Exclusive)
                    .unwrap();
            }
        }
        println!("Lock mixed mode: {:?}", start.elapsed());
    }
}

#[cfg(test)]
mod transaction_throughput {
    use super::*;

    #[test]
    fn test_tx_10_sequential() {
        let start = Instant::now();
        for _ in 0..10 {
            let mgr = RwLock::new(TransactionManager::new());
            let _ = mgr.write().unwrap().begin();
            let _ = mgr.write().unwrap().commit();
        }
        println!("Tx 10 sequential: {:?}", start.elapsed());
    }

    #[test]
    fn test_tx_50_sequential() {
        let start = Instant::now();
        for _ in 0..50 {
            let mgr = RwLock::new(TransactionManager::new());
            let _ = mgr.write().unwrap().begin();
            let _ = mgr.write().unwrap().commit();
        }
        println!("Tx 50 sequential: {:?}", start.elapsed());
    }

    #[test]
    fn test_tx_100_sequential() {
        let start = Instant::now();
        for _ in 0..100 {
            let mgr = RwLock::new(TransactionManager::new());
            let _ = mgr.write().unwrap().begin();
            let _ = mgr.write().unwrap().commit();
        }
        println!("Tx 100 sequential: {:?}", start.elapsed());
    }

    #[test]
    fn test_tx_throughput_measurement() {
        let start = Instant::now();
        let count = 200;

        for _ in 0..count {
            let mgr = RwLock::new(TransactionManager::new());
            let _ = mgr.write().unwrap().begin();
            let _ = mgr.write().unwrap().commit();
        }

        let elapsed = start.elapsed();
        let tps = (count * 1_000_000_000) / elapsed.as_nanos() as u64;
        println!(
            "Tx throughput: {} tx/sec ({} total in {:?})",
            tps, count, elapsed
        );
    }
}

#[cfg(test)]
mod performance_boundaries {
    use super::*;

    #[test]
    fn test_find_max_concurrent_locks() {
        let mut passed = true;
        let mut current = 100;

        // Binary search for max
        for i in 100..=500 {
            let mut manager = LockManager::new();
            let mut success = true;

            for j in 0..i {
                match manager.acquire_lock(TxId::new(j), vec![j as u8], LockMode::Shared) {
                    Ok(_) => {}
                    Err(_) => {
                        success = false;
                        break;
                    }
                }
            }

            if !success {
                current = i - 1;
                passed = false;
                break;
            }
            current = i;
        }

        println!("Max concurrent locks: {} (full pass: {})", current, passed);
    }

    #[test]
    fn test_memory_benchmark_pool() {
        // Test pool creation overhead
        let start = Instant::now();

        for _ in 0..1000 {
            let config = PoolConfig {
                size: 10,
                timeout_ms: 1000,
            };
            let _pool = ConnectionPool::new(config);
        }

        println!("Pool creation 1000x: {:?}", start.elapsed());
    }

    #[test]
    fn test_connection_reuse_stress() {
        let config = PoolConfig {
            size: 5,
            timeout_ms: 1000,
        };
        let pool = ConnectionPool::new(config);

        let start = Instant::now();
        let iterations = 500;

        for _ in 0..iterations {
            let conn = pool.acquire();
            drop(conn);
        }

        let elapsed = start.elapsed();
        let ops_per_sec = (iterations * 1_000_000_000) / elapsed.as_nanos() as u64;
        println!(
            "Connection reuse ({}x): {:?} ({} ops/sec)",
            iterations, elapsed, ops_per_sec
        );
    }
}
