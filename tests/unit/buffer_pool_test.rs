// Buffer Pool Tests
use sqlrustgo_storage::buffer_pool::{BufferPool, BufferPoolStats, BufferPoolWithClock, ClockProCache};
use sqlrustgo_storage::page::Page;
use std::sync::Arc;

fn create_test_page(page_id: u32, data_len: usize) -> Arc<Page> {
    let mut data = vec![0u8; data_len];
    data[0] = page_id as u8;
    Arc::new(Page::new(page_id, data))
}

#[test]
fn test_buffer_pool_stats_new() {
    let stats = BufferPoolStats::new();
    assert_eq!(stats.hits(), 0);
    assert_eq!(stats.misses(), 0);
    assert_eq!(stats.evictions(), 0);
}

#[test]
fn test_buffer_pool_stats_hit_rate() {
    let mut stats = BufferPoolStats::new();
    stats.record_hit();
    stats.record_hit();
    stats.record_miss();
    
    assert_eq!(stats.hits(), 2);
    assert_eq!(stats.misses(), 1);
    assert!((stats.hit_rate() - 0.666).abs() < 0.01);
}

#[test]
fn test_buffer_pool_new() {
    let pool = BufferPool::new(10);
    assert_eq!(pool.capacity(), 10);
    assert!(pool.is_empty());
    assert_eq!(pool.len(), 0);
}

#[test]
fn test_buffer_pool_insert_and_get() {
    let pool = BufferPool::new(5);
    let page = create_test_page(1, 100);
    
    pool.insert(page.clone());
    assert_eq!(pool.len(), 1);
    
    let retrieved = pool.get(1);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id(), 1);
}

#[test]
fn test_buffer_pool_get_missing() {
    let pool = BufferPool::new(5);
    let result = pool.get(999);
    assert!(result.is_none());
}

#[test]
fn test_buffer_pool_remove() {
    let pool = BufferPool::new(5);
    let page = create_test_page(1, 100);
    
    pool.insert(page);
    assert_eq!(pool.len(), 1);
    
    let removed = pool.remove(1);
    assert!(removed);
    assert_eq!(pool.len(), 0);
}

#[test]
fn test_buffer_pool_clear() {
    let pool = BufferPool::new(5);
    pool.insert(create_test_page(1, 100));
    pool.insert(create_test_page(2, 100));
    
    assert_eq!(pool.len(), 2);
    
    pool.clear();
    assert!(pool.is_empty());
}

#[test]
fn test_buffer_pool_stats_after_operations() {
    let pool = BufferPool::new(5);
    
    pool.insert(create_test_page(1, 100));
    let _ = pool.get(1);
    let _ = pool.get(2);
    
    let stats = pool.stats();
    assert_eq!(stats.hits(), 1);
    assert_eq!(stats.misses(), 1);
}

#[test]
fn test_buffer_pool_with_prefetch() {
    let pool = BufferPool::with_prefetch(5, 2);
    assert_eq!(pool.capacity(), 5);
}

#[test]
fn test_buffer_pool_allocate() {
    let pool = BufferPool::new(5);
    let page = pool.allocate(10);
    assert_eq!(page.id(), 10);
    assert_eq!(pool.len(), 1);
}

#[test]
fn test_clock_pro_cache_new() {
    let cache = ClockProCache::new(10, 3);
    assert_eq!(cache.capacity(), 10);
    assert!(cache.is_empty());
}

#[test]
fn test_clock_pro_cache_insert_and_get() {
    let cache = ClockProCache::new(5, 3);
    let page = create_test_page(1, 100);
    
    cache.insert(page.clone());
    assert_eq!(cache.len(), 1);
    
    let retrieved = cache.get(1);
    assert!(retrieved.is_some());
}

#[test]
fn test_clock_pro_cache_eviction() {
    let cache = ClockProCache::new(2, 3);
    
    cache.insert(create_test_page(1, 100));
    cache.insert(create_test_page(2, 100));
    cache.insert(create_test_page(3, 100));
    
    // After 3 inserts with capacity 2, one should be evicted
    assert!(cache.len() <= 2);
}
