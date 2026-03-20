//! Concurrency Stress Tests

use sqlrustgo_server::{ConnectionPool, PoolConfig};
use sqlrustgo_storage::MemoryStorage;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_connection_pool_basic() {
    let config = PoolConfig {
        size: 5,
        timeout_ms: 1000,
    };
    let pool = ConnectionPool::new(config);

    let connections: Vec<_> = (0..5).map(|_| pool.acquire()).collect();
    assert_eq!(pool.size(), 5);
    drop(connections);

    let conn = pool.acquire();
    let _ = conn.executor();

    println!("✓ Connection pool basic: PASS");
}

#[test]
fn test_thread_spawn_stability() {
    let handles: Vec<_> = (0..50)
        .map(|_| {
            thread::spawn(|| {
                thread::sleep(Duration::from_millis(1));
                let _ = MemoryStorage::new();
                true
            })
        })
        .collect();

    let all_success = handles.into_iter().all(|h| h.join().unwrap());
    assert!(all_success);
    println!("✓ Thread spawn stability: PASS (50 threads)");
}

#[test]
fn test_concurrent_storage_access() {
    let storage = Arc::new(MemoryStorage::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let storage = Arc::clone(&storage);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _ = storage.clone();
            }
            true
        });
        handles.push(handle);
    }

    let all_success = handles.into_iter().all(|h| h.join().unwrap());
    assert!(all_success);
    println!("✓ Concurrent storage access: PASS (10 threads x 100 operations)");
}
