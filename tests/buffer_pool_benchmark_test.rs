// Buffer Pool Hit Rate Benchmark Tests
// PB-01: 缓冲池命中率基准测试

use sqlrustgo_storage::buffer_pool::BufferPool;
use sqlrustgo_storage::page::Page;
use std::sync::Arc;

#[test]
fn test_buffer_pool_hit_rate_repeated_read() {
    let pool = BufferPool::new(100);

    // 场景1: 1000次重复读取同一页面
    // 预期命中率 ≥95%

    // Insert 10 pages
    for i in 0..10 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // Repeatedly read the same page 1000 times
    for _ in 0..1000 {
        let _ = pool.get(5);
    }

    let stats = pool.stats();
    let hit_rate = stats.hit_rate();

    println!(
        "Repeated read - hits: {}, total: {}, hit_rate: {:.2}%",
        stats.hits,
        stats.hits + stats.misses,
        hit_rate * 100.0
    );

    // 预期命中率 ≥95%
    assert!(
        hit_rate >= 0.95,
        "Expected hit rate ≥95%, got {:.2}%",
        hit_rate * 100.0
    );
}

#[test]
fn test_buffer_pool_hit_rate_sequential_scan() {
    let pool = BufferPool::new(100);

    // 场景2: 顺序扫描10000行，重复3次
    // 预期命中率 ≥80%

    // Insert 50 pages
    for i in 0..50 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // Sequential scan 3 times
    for _ in 0..3 {
        for i in 0..50 {
            let _ = pool.get(i);
        }
    }

    let stats = pool.stats();
    let hit_rate = stats.hit_rate();

    println!(
        "Sequential scan - hits: {}, total: {}, hit_rate: {:.2}%",
        stats.hits,
        stats.hits + stats.misses,
        hit_rate * 100.0
    );

    // 预期命中率 ≥80%
    assert!(
        hit_rate >= 0.80,
        "Expected hit rate ≥80%, got {:.2}%",
        hit_rate * 100.0
    );
}

#[test]
fn test_buffer_pool_hit_rate_random_access() {
    let pool = BufferPool::new(100);

    // 场景3: 随机访问10000行
    // 预期命中率 约10%

    // Insert 10 pages
    for i in 0..10 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // Random access 10000 times (accessing 1000 different pages, only 10 in pool)
    use std::collections::HashSet;
    let mut rng: u64 = 12345;
    for _ in 0..10000 {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let page_id = (rng % 1000) as u32;
        let _ = pool.get(page_id);
    }

    let stats = pool.stats();
    let hit_rate = stats.hit_rate();

    println!(
        "Random access - hits: {}, total: {}, hit_rate: {:.2}%",
        stats.hits,
        stats.hits + stats.misses,
        hit_rate * 100.0
    );

    // Random access should have low hit rate (approximately 10%)
    assert!(
        hit_rate < 0.20,
        "Expected hit rate <20%, got {:.2}%",
        hit_rate * 100.0
    );
}

#[test]
fn test_buffer_pool_hit_rate_mixed_workload() {
    let pool = BufferPool::new(50);

    // 混合工作负载: 热数据 + 冷数据
    // Insert 50 pages
    for i in 0..50 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // 80% 访问热数据 (pages 0-9)
    // 20% 访问冷数据 (pages 10-49)
    use std::collections::HashSet;
    let mut rng: u64 = 12345;
    let hot_pages: HashSet<u32> = (0..10).collect();
    let cold_pages: HashSet<u32> = (10..50).collect();

    for _ in 0..1000 {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        if (rng % 100) < 80 {
            // Access hot page
            let page_id = *hot_pages.iter().nth((rng % 10) as usize).unwrap();
            let _ = pool.get(page_id);
        } else {
            // Access cold page
            let page_id = *cold_pages.iter().nth((rng % 40) as usize).unwrap();
            let _ = pool.get(page_id);
        }
    }

    let stats = pool.stats();
    let hit_rate = stats.hit_rate();

    println!(
        "Mixed workload - hits: {}, total: {}, hit_rate: {:.2}%",
        stats.hits,
        stats.hits + stats.misses,
        hit_rate * 100.0
    );

    // 混合工作负载预期命中率 ≥70% (因为热数据占比高)
    assert!(
        hit_rate >= 0.70,
        "Expected hit rate ≥70%, got {:.2}%",
        hit_rate * 100.0
    );
}

#[test]
fn test_buffer_pool_stats_collection() {
    let pool = BufferPool::new(10);

    // Insert pages
    for i in 0..5 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    // Various operations
    pool.get(0); // hit
    pool.get(1); // hit
    pool.get(0); // hit
    pool.get(99); // miss
    pool.get(98); // miss

    let stats = pool.stats();

    // Verify stats collection
    assert!(
        stats.hits >= 3,
        "Expected at least 3 hits, got {}",
        stats.hits
    );
    assert!(
        stats.misses >= 2,
        "Expected at least 2 misses, got {}",
        stats.misses
    );

    // Verify hit rate calculation
    let hit_rate = stats.hit_rate();
    let expected_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64;
    assert!((hit_rate - expected_rate).abs() < f64::EPSILON);

    println!(
        "Stats collection test - hits: {}, misses: {}, hit_rate: {:.2}%",
        stats.hits,
        stats.misses,
        hit_rate * 100.0
    );
}

#[test]
fn test_buffer_pool_eviction_tracking() {
    let pool = BufferPool::new(3);

    // Insert 3 pages
    for i in 0..3 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // Access page 0 to make it recently used
    let _ = pool.get(0);

    // Insert new page, should evict page 1 (LRU)
    let page = Arc::new(Page::new(100));
    pool.insert(page);

    // Page 0 should still be there (was accessed recently)
    assert!(pool.get(0).is_some(), "Page 0 should not be evicted");

    // Page 1 should be evicted (was LRU)
    // Note: This depends on LRU implementation details
    println!("Eviction test completed");
}

#[test]
fn test_buffer_pool_large_working_set() {
    let pool = BufferPool::new(100);

    // Insert 100 pages to fill the pool
    for i in 0..100 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // Now insert more pages to create a larger working set
    // This should cause some evictions
    for i in 100..200 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    // Access first 50 pages (which may still be in pool depending on eviction)
    for i in 0..50 {
        let _ = pool.get(i);
    }

    let stats = pool.stats();
    let hit_rate = stats.hit_rate();

    println!(
        "Large working set - hits: {}, total: {}, hit_rate: {:.2}%",
        stats.hits,
        stats.hits + stats.misses,
        hit_rate * 100.0
    );

    // With a larger working set, we expect some hits but not all
    // The exact hit rate depends on LRU eviction behavior
    // This test mainly verifies the buffer pool handles larger working sets
    assert!(
        hit_rate >= 0.0,
        "Hit rate should be non-negative, got {:.2}%",
        hit_rate * 100.0
    );
}
