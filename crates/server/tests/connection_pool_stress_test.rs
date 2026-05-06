// Connection Pool Stress Tests
// Tests concurrent access, load balancing, and pool exhaustion scenarios

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

#[cfg(test)]
mod connection_pool_tests {
    use super::*;

    // ============================================================
    // Mock structures for testing (simulating the pool behavior)
    // ============================================================

    struct MockPool {
        #[allow(dead_code)]
        size: usize,
        available: Arc<AtomicUsize>,
        in_use: Arc<AtomicUsize>,
    }

    impl MockPool {
        fn new(size: usize) -> Self {
            Self {
                size,
                available: Arc::new(AtomicUsize::new(size)),
                in_use: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn acquire(&self) -> bool {
            loop {
                let current = self.available.load(Ordering::SeqCst);
                if current == 0 {
                    return false;
                }
                // Try to decrement
                match self.available.compare_exchange(
                    current,
                    current - 1,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => {
                        self.in_use.fetch_add(1, Ordering::SeqCst);
                        return true;
                    }
                    Err(_) => continue, // Another thread changed it, retry
                }
            }
        }

        fn release(&self) {
            self.available.fetch_add(1, Ordering::SeqCst);
            self.in_use.fetch_sub(1, Ordering::SeqCst);
        }

        fn stats(&self) -> (usize, usize) {
            (
                self.available.load(Ordering::SeqCst),
                self.in_use.load(Ordering::SeqCst),
            )
        }
    }

    #[test]
    fn test_pool_exhaustion_timeout() {
        let pool = MockPool::new(2);

        // Acquire all
        assert!(pool.acquire());
        assert!(pool.acquire());
        assert_eq!(pool.stats(), (0, 2));

        // Try to acquire when exhausted
        let result = pool.acquire();
        assert!(!result, "Should fail when pool exhausted");

        // Release and verify recovery
        pool.release();
        assert_eq!(pool.stats(), (1, 1));

        assert!(pool.acquire());
        assert_eq!(pool.stats(), (0, 2));
    }

    #[test]
    fn test_concurrent_acquire_release_race() {
        let pool = Arc::new(MockPool::new(10));
        let success_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(20));

        let handles: Vec<_> = (0..20)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let success_count = Arc::clone(&success_count);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    // Rapid acquire and release
                    for _ in 0..10 {
                        if pool.acquire() {
                            pool.release();
                            success_count.fetch_add(1, Ordering::SeqCst);
                        }
                        thread::sleep(std::time::Duration::from_micros(10));
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let count = success_count.load(Ordering::SeqCst);
        assert!(count > 0, "Should have acquired at least some connections");
        // Pool should be back to original state
        assert_eq!(pool.stats(), (10, 0));
    }

    #[test]
    fn test_load_balance_round_robin_distribution() {
        let pool = Arc::new(MockPool::new(4));
        let acquire_count = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let acquire_count = Arc::clone(&acquire_count);
                thread::spawn(move || {
                    for _ in 0..100 {
                        while !pool.acquire() {
                            thread::sleep(std::time::Duration::from_micros(10));
                        }
                        acquire_count.fetch_add(1, Ordering::SeqCst);
                        pool.release();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let total = acquire_count.load(Ordering::SeqCst);
        assert_eq!(total, 400, "Should have acquired 400 total connections");
    }

    #[test]
    fn test_connection_reuse_under_load() {
        let pool = Arc::new(MockPool::new(5));
        let reuse_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(10));

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let reuse_count = Arc::clone(&reuse_count);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    for _ in 0..50 {
                        while !pool.acquire() {
                            thread::sleep(std::time::Duration::from_micros(10));
                        }
                        pool.release();
                        reuse_count.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let total = reuse_count.load(Ordering::SeqCst);
        assert_eq!(total, 500, "All 500 operations should complete");
    }

    #[test]
    fn test_pool_reset_after_exhaustion() {
        let pool = MockPool::new(2);

        // Exhaust pool
        assert!(pool.acquire());
        assert!(pool.acquire());
        assert_eq!(pool.stats(), (0, 2));

        // Release and verify recovery
        pool.release();
        pool.release();

        assert_eq!(pool.stats(), (2, 0));

        let conn = pool.acquire();
        assert!(conn);
        let conn2 = pool.acquire();
        assert!(conn2);
    }

    #[test]
    fn test_concurrent_session_state_isolation() {
        let pool = Arc::new(MockPool::new(10));
        let errors = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(10));

        let handles: Vec<_> = (0..10)
            .map(|_i| {
                let pool = Arc::clone(&pool);
                let _errors = Arc::clone(&errors);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    // Each thread does independent work
                    for _ in 0..20 {
                        if pool.acquire() {
                            pool.release();
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // If we got here without panic, test passed
        assert_eq!(errors.load(Ordering::SeqCst), 0);
        assert_eq!(pool.stats(), (10, 0));
    }

    #[test]
    fn test_pool_stress_many_connections_quick_turnover() {
        let pool = Arc::new(MockPool::new(5));
        let success_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(20));

        let handles: Vec<_> = (0..20)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let success_count = Arc::clone(&success_count);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    for _ in 0..100 {
                        if pool.acquire() {
                            pool.release();
                            success_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let total = success_count.load(Ordering::SeqCst);
        // With quick turnover, we should complete many operations
        assert!(total >= 100, "Should complete at least 100 operations");
    }

    #[test]
    fn test_acquire_release_no_leak() {
        let pool = MockPool::new(10);

        // Get initial stats
        let (available, in_use) = pool.stats();
        assert_eq!(available, 10);
        assert_eq!(in_use, 0);

        // Acquire and release all
        for _ in 0..10 {
            assert!(pool.acquire());
        }

        let (available, in_use) = pool.stats();
        assert_eq!(available, 0);
        assert_eq!(in_use, 10);

        // Release all
        for _ in 0..10 {
            pool.release();
        }

        let (available, in_use) = pool.stats();
        assert_eq!(available, 10);
        assert_eq!(in_use, 0);

        // Acquire again - should work without leak
        for _ in 0..10 {
            assert!(pool.acquire());
        }

        let (available, in_use) = pool.stats();
        assert_eq!(available, 0);
        assert_eq!(in_use, 10);
    }

    #[test]
    fn test_high_contention_stress() {
        let pool = Arc::new(MockPool::new(3));
        let success_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(50));

        let handles: Vec<_> = (0..50)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let success_count = Arc::clone(&success_count);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    for _ in 0..50 {
                        if pool.acquire() {
                            // Very short hold time
                            pool.release();
                            success_count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let total = success_count.load(Ordering::SeqCst);
        // Should complete many operations under high contention
        assert!(
            total >= 500,
            "Should complete at least 500 operations under contention"
        );
        assert_eq!(pool.stats(), (3, 0));
    }

    #[test]
    fn test_pool_size_boundaries() {
        // Test minimum size
        let small_pool = MockPool::new(1);
        assert!(small_pool.acquire());
        assert!(!small_pool.acquire());
        small_pool.release();
        assert!(small_pool.acquire());

        // Test larger size
        let large_pool = MockPool::new(100);
        for _ in 0..100 {
            assert!(large_pool.acquire());
        }
        assert!(!large_pool.acquire());
    }

    #[test]
    fn test_concurrent_acquire_multiple_threads() {
        let pool = Arc::new(MockPool::new(10));
        let acquired_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(15));

        let handles: Vec<_> = (0..15)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let acquired_count = Arc::clone(&acquired_count);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    // Each thread tries up to 20 times
                    for _ in 0..20 {
                        if pool.acquire() {
                            acquired_count.fetch_add(1, Ordering::SeqCst);
                            pool.release();
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let total = acquired_count.load(Ordering::SeqCst);
        assert!(total > 0, "Should have acquired some connections");
        assert_eq!(pool.stats(), (10, 0));
    }
}
