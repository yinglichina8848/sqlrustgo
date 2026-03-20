//! Storage Engine Integration Tests
//!
//! These tests verify end-to-end functionality of storage engine components:
//! - Buffer pool management
//! - B+Tree index operations
//! - Page operations

use sqlrustgo_storage::{BPlusTree, BufferPool, Page};
use std::sync::Arc;
use tempfile::TempDir;

fn create_temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

// ============================================================================
// Buffer Pool Integration Tests
// ============================================================================

#[test]
fn test_buffer_pool_basic_operations() {
    let pool = BufferPool::new(10);

    // Insert pages
    for i in 0..5 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    assert_eq!(pool.len(), 5);

    // Get existing page
    let page = pool.get(2);
    assert!(page.is_some());

    // Get non-existing page
    let missing = pool.get(99);
    assert!(missing.is_none());

    println!("✓ Buffer pool basic operations");
}

#[test]
fn test_buffer_pool_hit_rate_repeated_access() {
    let pool = Arc::new(BufferPool::new(50));

    // Insert test pages
    for i in 0..10 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // Repeatedly access same page - should have high hit rate
    for _ in 0..100 {
        let _ = pool.get(5);
    }

    let stats = pool.stats();
    let hit_rate = stats.hit_rate();

    println!("Repeated access hit rate: {:.2}%", hit_rate * 100.0);
    assert!(
        hit_rate >= 0.95,
        "Expected ≥95% hit rate, got {:.2}%",
        hit_rate * 100.0
    );
    println!("✓ Buffer pool hit rate with repeated access");
}

#[test]
fn test_buffer_pool_lru_eviction() {
    let pool = Arc::new(BufferPool::new(3));

    // Insert 3 pages to fill capacity
    for i in 0..3 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    // Access page 0 to make it recently used
    let _ = pool.get(0);

    // Insert new page - should evict LRU page (page 1)
    let page = Arc::new(Page::new(100));
    pool.insert(page);

    // Page 0 should still be accessible (was accessed recently)
    assert!(pool.get(0).is_some());

    println!("✓ Buffer pool LRU eviction");
}

#[test]
fn test_buffer_pool_stats() {
    let pool = Arc::new(BufferPool::new(10));

    // Insert some pages
    for i in 0..5 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    // Access pattern: hits and misses
    pool.get(0); // hit
    pool.get(1); // hit
    pool.get(0); // hit
    pool.get(99); // miss

    let stats = pool.stats();

    assert!(stats.hits >= 3);
    assert!(stats.misses >= 1);

    let hit_rate = stats.hit_rate();
    println!(
        "Stats: hits={}, misses={}, hit_rate={:.2}%",
        stats.hits,
        stats.misses,
        hit_rate * 100.0
    );
    assert!(hit_rate > 0.5, "Expected >50% hit rate");
    println!("✓ Buffer pool stats");
}

// ============================================================================
// B+Tree Index Integration Tests
// ============================================================================

#[test]
fn test_bplus_tree_insert_and_search() {
    let mut tree = BPlusTree::new();

    // Insert key-value pairs
    for i in 0..100 {
        tree.insert(i as i64, i as u32);
    }

    assert_eq!(tree.len(), 100);

    // Point queries
    for i in 0..100 {
        let result = tree.search(i as i64);
        assert!(result.is_some(), "Key {} should be found", i);
        assert_eq!(result.unwrap(), i as u32);
    }

    println!("✓ B+Tree insert and search");
}

#[test]
fn test_bplus_tree_range_query() {
    let mut tree = BPlusTree::new();

    // Insert sequential keys with values
    for i in 0..1000 {
        tree.insert(i as i64, i as u32);
    }

    // Range query
    let results = tree.range_query(100, 200);

    assert!(!results.is_empty(), "Range query should return results");

    // Verify all results are in range
    for value in &results {
        assert!(*value >= 100 && *value <= 200);
    }

    println!(
        "✓ B+Tree range query: {} results for range [100, 200]",
        results.len()
    );
}

#[test]
fn test_bplus_tree_keys() {
    let mut tree = BPlusTree::new();

    // Insert with various keys
    for i in vec![5, 2, 8, 1, 9, 3, 7, 4, 6, 0] {
        tree.insert(i, i as u32);
    }

    let keys = tree.keys();
    assert_eq!(keys.len(), 10);

    // Keys should be sorted
    for i in 0..keys.len() - 1 {
        assert!(keys[i] < keys[i + 1]);
    }

    println!("✓ B+Tree keys sorted: {:?}", keys);
}

// ============================================================================
// Page Tests
// ============================================================================

#[test]
fn test_page_creation_and_data() {
    let page = Page::new(42);

    assert_eq!(page.page_id, 42);
    assert!(!page.data.is_empty());
    assert_eq!(page.data.len(), 4096); // PAGE_SIZE

    println!("✓ Page creation");
}

#[test]
fn test_page_checksum() {
    let page = Page::new(1);

    // Checksum should be valid for new page
    assert!(page.verify_checksum());

    // Page data can be read (even if we can't modify it easily)
    assert!(!page.data.is_empty());

    println!("✓ Page checksum verification");
}

// ============================================================================
// End-to-End Integration Test
// ============================================================================

#[test]
fn test_storage_workflow() {
    let _temp_dir = create_temp_dir();

    // 1. Create B+Tree index
    let mut tree = BPlusTree::new();

    // 2. Insert indexed data
    for i in 0..50 {
        tree.insert(i as i64, (i * 100) as u32);
    }

    // 3. Verify index
    assert_eq!(tree.len(), 50);

    for i in 0..50 {
        let result = tree.search(i as i64);
        assert!(result.is_some());
    }

    // 4. Range query
    let range_results = tree.range_query(10, 30);
    assert!(!range_results.is_empty());

    // 5. Create buffer pool
    let pool = Arc::new(BufferPool::new(100));

    // 6. Use buffer pool with pages
    for i in 0..10 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // 7. Repeated operations
    for _ in 0..100 {
        let _ = tree.search(25);
    }

    let _stats = pool.stats();
    println!(
        "✓ Storage workflow: index size={}, range results={}",
        tree.len(),
        range_results.len()
    );
}

#[test]
fn test_buffer_pool_sequential_scan_simulation() {
    let pool = Arc::new(BufferPool::new(100));

    // Insert 50 pages
    for i in 0..50 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.reset_stats();

    // Simulate sequential scan: access each page once
    for i in 0..50 {
        let _ = pool.get(i);
    }

    let stats = pool.stats();
    let hit_rate = stats.hit_rate();

    // First sequential scan should have 0% hit rate (cold start)
    println!(
        "First scan: hits={}, total={}, hit_rate={:.2}%",
        stats.hits,
        stats.hits + stats.misses,
        hit_rate * 100.0
    );

    // Second sequential scan should have high hit rate
    pool.reset_stats();
    for _ in 0..2 {
        for i in 0..50 {
            let _ = pool.get(i);
        }
    }

    let stats2 = pool.stats();
    let hit_rate2 = stats2.hit_rate();

    println!(
        "Multiple scans: hits={}, total={}, hit_rate={:.2}%",
        stats2.hits,
        stats2.hits + stats2.misses,
        hit_rate2 * 100.0
    );

    // After multiple scans, should have high hit rate
    assert!(
        hit_rate2 >= 0.80,
        "Expected ≥80% hit rate after multiple scans, got {:.2}%",
        hit_rate2 * 100.0
    );
    println!("✓ Buffer pool sequential scan simulation");
}

#[test]
fn test_bplus_tree_mixed_operations() {
    let mut tree = BPlusTree::new();

    // Insert even numbers
    for i in (0..200).step_by(2) {
        tree.insert(i as i64, i as u32);
    }

    // Insert odd numbers
    for i in (1..200).step_by(2) {
        tree.insert(i as i64, i as u32);
    }

    assert_eq!(tree.len(), 200);

    // Search specific keys
    assert!(tree.search(0).is_some());
    assert!(tree.search(199).is_some());
    assert!(tree.search(100).is_some());

    // Range queries
    let small_range = tree.range_query(0, 10);
    println!("Range [0,10]: {} results", small_range.len());

    let large_range = tree.range_query(50, 150);
    println!("Range [50,150]: {} results", large_range.len());

    assert!(!small_range.is_empty());
    assert!(!large_range.is_empty());

    println!("✓ B+Tree mixed operations");
}
