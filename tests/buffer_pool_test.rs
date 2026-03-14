// Buffer Pool Tests
use sqlrustgo_storage::buffer_pool::{BufferPool, BufferPoolStats, MemoryPool};
use sqlrustgo_storage::page::Page;
use std::sync::Arc;

#[test]
fn test_buffer_pool_new() {
    let pool = BufferPool::new(100);
    assert_eq!(pool.capacity(), 100);
    assert!(pool.is_empty());
}

#[test]
fn test_buffer_pool_with_prefetch() {
    let pool = BufferPool::with_prefetch(100, 5);
    assert_eq!(pool.capacity(), 100);
    assert!(pool.is_empty());
}

#[test]
fn test_buffer_pool_insert_and_get() {
    let pool = BufferPool::new(10);
    let page = Arc::new(Page::new(1));
    pool.insert(page.clone());

    let retrieved = pool.get(1);
    assert!(retrieved.is_some());
}

#[test]
fn test_buffer_pool_get_not_found() {
    let pool = BufferPool::new(10);
    let retrieved = pool.get(999);
    assert!(retrieved.is_none());
}

#[test]
fn test_buffer_pool_lru_eviction() {
    let pool = BufferPool::new(2);

    // Insert 3 pages, should evict the first one
    for i in 1..=3 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    // First page should be evicted
    assert!(pool.get(1).is_none());
    // Third page should still be there
    assert!(pool.get(3).is_some());
}

#[test]
fn test_buffer_pool_remove() {
    let pool = BufferPool::new(10);
    let page = Arc::new(Page::new(1));
    pool.insert(page);

    assert!(pool.remove(1));
    assert!(!pool.remove(1)); // Second remove should fail
}

#[test]
fn test_buffer_pool_clear() {
    let pool = BufferPool::new(10);
    for i in 1..=5 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    pool.clear();
    assert!(pool.is_empty());
}

#[test]
fn test_buffer_pool_len() {
    let pool = BufferPool::new(10);
    assert_eq!(pool.len(), 0);

    for i in 1..=3 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }

    assert_eq!(pool.len(), 3);
}

#[test]
fn test_buffer_pool_stats() {
    let pool = BufferPool::new(10);

    // Insert a page and get it
    let page = Arc::new(Page::new(1));
    pool.insert(page);
    pool.get(1);

    let stats = pool.stats();
    assert!(stats.hits >= 1);
}

#[test]
fn test_buffer_pool_stats_hit_rate() {
    let pool = BufferPool::new(10);

    let stats = pool.stats();
    assert_eq!(stats.hit_rate(), 0.0);

    // Insert and get a page
    let page = Arc::new(Page::new(1));
    pool.insert(page);
    pool.get(1);

    let stats = pool.stats();
    assert!(stats.hit_rate() > 0.0);
}

#[test]
fn test_buffer_pool_reset_stats() {
    let pool = BufferPool::new(10);

    let page = Arc::new(Page::new(1));
    pool.insert(page);
    pool.get(1);

    pool.reset_stats();

    let stats = pool.stats();
    assert_eq!(stats.hits, 0);
}

#[test]
fn test_buffer_pool_allocate() {
    let pool = BufferPool::new(10);
    let _page_id = pool.allocate(1);
}

#[test]
fn test_buffer_pool_stats_hits_and_misses() {
    let pool = BufferPool::new(10);

    // Insert page
    let page = Arc::new(Page::new(1));
    pool.insert(page);

    // Hit
    pool.get(1);

    // Miss
    pool.get(999);

    let stats = pool.stats();
    assert!(stats.hits >= 1);
    assert!(stats.misses >= 1);
}

// MemoryPool tests
#[test]
fn test_memory_pool_new() {
    let pool = MemoryPool::new(4096);
    assert_eq!(pool.block_size(), 4096);
}

#[test]
fn test_memory_pool_allocate() {
    let pool = MemoryPool::new(4096);
    let buffer = pool.allocate();
    assert_eq!(buffer.len(), 4096);
}

#[test]
fn test_memory_pool_free() {
    let pool = MemoryPool::new(4096);
    let buffer = pool.allocate();
    pool.free(buffer);
    assert!(pool.free_count() > 0);
}
