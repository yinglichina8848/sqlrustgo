//! Buffer Pool Manager with LRU cache, prefetch, and memory pool optimization

use crate::page::Page;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};

/// Buffer Pool with LRU eviction, prefetch, pin/unpin and memory pool
pub struct BufferPool {
    /// Page storage with LRU tracking
    pages: Mutex<HashMap<u32, Arc<Page>>>,
    /// LRU queue - most recently used at front
    lru: Mutex<VecDeque<u32>>,
    /// Pin count for each page - page can't be evicted if pin > 0
    pin_count: Mutex<HashMap<u32, u32>>,
    /// Capacity
    capacity: usize,
    /// Prefetch window size
    prefetch_window: usize,
    /// Statistics
    stats: RwLock<BufferPoolStats>,
}

/// Buffer pool statistics
#[derive(Debug, Default, Clone)]
pub struct BufferPoolStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub prefetch_hits: u64,
}

impl BufferPoolStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f64 / total as f64
    }
}

impl BufferPool {
    /// Create a new buffer pool with LRU eviction
    pub fn new(capacity: usize) -> Self {
        Self {
            pages: Mutex::new(HashMap::new()),
            lru: Mutex::new(VecDeque::new()),
            pin_count: Mutex::new(HashMap::new()),
            capacity,
            prefetch_window: 2,
            stats: RwLock::new(BufferPoolStats::default()),
        }
    }

    /// Create buffer pool with custom prefetch window
    pub fn with_prefetch(capacity: usize, prefetch_window: usize) -> Self {
        Self {
            pages: Mutex::new(HashMap::new()),
            lru: Mutex::new(VecDeque::new()),
            pin_count: Mutex::new(HashMap::new()),
            capacity,
            prefetch_window,
            stats: RwLock::new(BufferPoolStats::default()),
        }
    }

    /// Pin a page - prevents it from being evicted
    ///
    /// When a transaction accesses a page, it should be pinned to prevent
    /// eviction while in use. Each pin must have a corresponding unpin.
    pub fn pin(&self, page_id: u32) {
        let mut pin_count = self.pin_count.lock().unwrap();
        *pin_count.entry(page_id).or_insert(0) += 1;
    }

    /// Unpin a page - allows it to be evicted after all pins are released
    ///
    /// Called when a transaction finishes using the page. When pin_count
    /// reaches 0, the page becomes a candidate for eviction.
    pub fn unpin(&self, page_id: u32) -> bool {
        let mut pin_count = self.pin_count.lock().unwrap();
        if let Some(count) = pin_count.get_mut(&page_id) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    pin_count.remove(&page_id);
                    return true; // Page is now unpinned
                }
            }
        }
        false
    }

    /// Check if a page is pinned
    pub fn is_pinned(&self, page_id: u32) -> bool {
        let pin_count = self.pin_count.lock().unwrap();
        pin_count.get(&page_id).copied().unwrap_or(0) > 0
    }

    /// Get pin count for a page
    pub fn pin_count(&self, page_id: u32) -> u32 {
        let pin_count = self.pin_count.lock().unwrap();
        pin_count.get(&page_id).copied().unwrap_or(0)
    }

    /// Get a page - returns None if not in pool
    pub fn get(&self, page_id: u32) -> Option<Arc<Page>> {
        let pages = self.pages.lock().unwrap();

        if let Some(page) = pages.get(&page_id).cloned() {
            // Update LRU - move to front
            let mut lru = self.lru.lock().unwrap();
            lru.retain(|&id| id != page_id);
            lru.push_front(page_id);

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.hits += 1;

            Some(page)
        } else {
            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.misses += 1;

            None
        }
    }

    /// Get or load a page
    pub fn get_or_load<F>(&self, page_id: u32, loader: F) -> Arc<Page>
    where
        F: FnOnce(u32) -> Arc<Page>,
    {
        // Try to get from pool
        if let Some(page) = self.get(page_id) {
            return page;
        }

        // Load page
        let page = loader(page_id);
        self.insert(Arc::clone(&page));
        page
    }

    /// Insert a page with LRU eviction
    pub fn insert(&self, page: Arc<Page>) {
        let page_id = page.page_id();

        let mut pages = self.pages.lock().unwrap();
        let mut lru = self.lru.lock().unwrap();

        // If page already exists, just update LRU
        if pages.contains_key(&page_id) {
            lru.retain(|&id| id != page_id);
            lru.push_front(page_id);
            return;
        }

        // Evict if at capacity - skip pinned pages
        while pages.len() >= self.capacity && !lru.is_empty() {
            if let Some(evicted_id) = lru.pop_back() {
                // Skip pinned pages - they cannot be evicted
                if self.is_pinned(evicted_id) {
                    // Move to front, will try next candidate
                    lru.push_front(evicted_id);
                    continue;
                }
                pages.remove(&evicted_id);
                let mut stats = self.stats.write().unwrap();
                stats.evictions += 1;
            }
        }

        // Insert new page
        pages.insert(page_id, page);
        lru.push_front(page_id);
    }

    /// Allocate a new page
    pub fn allocate(&self, page_id: u32) -> Arc<Page> {
        let page = Arc::new(Page::new(page_id));
        self.insert(Arc::clone(&page));
        page
    }

    /// Prefetch pages - loads pages in advance
    pub fn prefetch(&self, page_ids: &[u32], loader: impl Fn(u32) -> Arc<Page>) {
        for page_id in page_ids.iter().take(self.prefetch_window) {
            if self.get(*page_id).is_none() {
                let page = loader(*page_id);
                self.insert(page);

                let mut stats = self.stats.write().unwrap();
                stats.prefetch_hits += 1;
            }
        }
    }

    /// Prefetch sequential pages
    pub fn prefetch_range(&self, start_page: u32, count: usize, loader: impl Fn(u32) -> Arc<Page>) {
        let page_ids: Vec<u32> = (start_page..start_page + count as u32).collect();
        self.prefetch(&page_ids, loader);
    }

    /// Remove a page from pool
    pub fn remove(&self, page_id: u32) -> bool {
        let mut pages = self.pages.lock().unwrap();
        let mut lru = self.lru.lock().unwrap();

        lru.retain(|&id| id != page_id);
        pages.remove(&page_id).is_some()
    }

    /// Clear all pages
    pub fn clear(&self) {
        let mut pages = self.pages.lock().unwrap();
        let mut lru = self.lru.lock().unwrap();
        pages.clear();
        lru.clear();
    }

    /// Get current number of pages
    pub fn len(&self) -> usize {
        let pages = self.pages.lock().unwrap();
        pages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get statistics
    pub fn stats(&self) -> BufferPoolStats {
        let stats = self.stats.read().unwrap();
        BufferPoolStats {
            hits: stats.hits,
            misses: stats.misses,
            evictions: stats.evictions,
            prefetch_hits: stats.prefetch_hits,
        }
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().unwrap();
        stats.hit_rate()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = BufferPoolStats::default();
    }
}

/// Simple memory pool for reducing allocations
pub struct MemoryPool {
    /// Free list for reusable buffers
    free_list: Mutex<Vec<Vec<u8>>>,
    /// Default block size
    block_size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(block_size: usize) -> Self {
        Self {
            free_list: Mutex::new(Vec::new()),
            block_size,
        }
    }

    /// Allocate a buffer - either from pool or fresh
    pub fn allocate(&self) -> Vec<u8> {
        let mut free_list = self.free_list.lock().unwrap();

        if let Some(mut buffer) = free_list.pop() {
            buffer.resize(self.block_size, 0);
            buffer
        } else {
            vec![0u8; self.block_size]
        }
    }

    /// Return a buffer to the pool
    pub fn free(&self, mut buffer: Vec<u8>) {
        // Only return buffers of correct size
        if buffer.len() == self.block_size {
            buffer.clear();
            let mut free_list = self.free_list.lock().unwrap();
            free_list.push(buffer);
        }
    }

    /// Get block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Get number of free blocks
    pub fn free_count(&self) -> usize {
        let free_list = self.free_list.lock().unwrap();
        free_list.len()
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new(4096) // Default 4KB blocks
    }
}

/// LRU-K Buffer Pool with enhanced eviction algorithm
/// LRU-K tracks the last K accesses to each page, better handling of access patterns
pub struct BufferPoolLruK {
    /// Page storage
    pages: Mutex<HashMap<u32, Arc<Page>>>,
    /// K value - number of recent accesses to track
    k_value: usize,
    /// Access history: page_id -> list of access timestamps
    access_history: Mutex<HashMap<u32, Vec<u64>>>,
    /// Global access counter
    access_counter: Mutex<u64>,
    /// Capacity
    capacity: usize,
    /// Statistics
    stats: RwLock<BufferPoolStats>,
}

impl BufferPoolLruK {
    /// Create a new LRU-K buffer pool
    pub fn new(capacity: usize, k: usize) -> Self {
        Self {
            pages: Mutex::new(HashMap::new()),
            k_value: k,
            access_history: Mutex::new(HashMap::new()),
            access_counter: Mutex::new(0),
            capacity,
            stats: RwLock::new(BufferPoolStats::default()),
        }
    }

    /// Get a page - returns None if not in pool
    pub fn get(&self, page_id: u32) -> Option<Arc<Page>> {
        let pages = self.pages.lock().unwrap();

        if let Some(page) = pages.get(&page_id).cloned() {
            // Record access
            self.record_access(page_id);

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.hits += 1;

            Some(page)
        } else {
            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.misses += 1;

            None
        }
    }

    /// Record a page access for LRU-K tracking
    fn record_access(&self, page_id: u32) {
        let mut counter = self.access_counter.lock().unwrap();
        *counter += 1;
        let timestamp = *counter;

        let mut history = self.access_history.lock().unwrap();
        let entry = history.entry(page_id).or_default();
        entry.push(timestamp);

        // Keep only last K accesses
        if entry.len() > self.k_value {
            entry.remove(0);
        }
    }

    /// Get or load a page
    pub fn get_or_load<F>(&self, page_id: u32, loader: F) -> Arc<Page>
    where
        F: FnOnce() -> Arc<Page>,
    {
        if let Some(page) = self.get(page_id) {
            return page;
        }

        // Load page
        let page = loader();
        self.insert(page.clone());
        page
    }

    /// Insert a page into the pool
    pub fn insert(&self, page: Arc<Page>) {
        let page_id = page.page_id();
        let mut pages = self.pages.lock().unwrap();

        // Check capacity
        if pages.len() >= self.capacity && !pages.contains_key(&page_id) {
            // Evict using LRU-K algorithm
            self.evict(&mut pages);
        }

        pages.insert(page_id, page);
        self.record_access(page_id);
    }

    /// Evict a page using LRU-K algorithm
    fn evict(&self, pages: &mut HashMap<u32, Arc<Page>>) {
        let history = self.access_history.lock().unwrap();

        // Find page with oldest K-th access (earliest access in their K-history)
        let victim = history
            .iter()
            .filter(|(id, _)| pages.contains_key(*id))
            .min_by_key(|(_, accesses)| {
                if accesses.is_empty() {
                    u64::MAX
                } else {
                    accesses[0]
                }
            })
            .map(|(id, _)| *id);

        if let Some(page_id) = victim {
            pages.remove(&page_id);
            let mut stats = self.stats.write().unwrap();
            stats.evictions += 1;
        }
    }

    /// Get pool size
    pub fn len(&self) -> usize {
        self.pages.lock().unwrap().len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get statistics
    pub fn stats(&self) -> BufferPoolStats {
        self.stats.read().unwrap().clone()
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f64 {
        self.stats.read().unwrap().hit_rate()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = BufferPoolStats::default();
    }

    /// Clear the pool
    pub fn clear(&self) {
        let mut pages = self.pages.lock().unwrap();
        pages.clear();
        let mut history = self.access_history.lock().unwrap();
        history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_basic() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.capacity(), 10);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_buffer_pool_get_page() {
        let pool = BufferPool::new(10);
        let page = Arc::new(Page::new(1));
        pool.insert(page);
        let retrieved = pool.get(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().page_id(), 1);
    }

    #[test]
    fn test_buffer_pool_lru_eviction() {
        let pool = BufferPool::new(3);

        pool.insert(Arc::new(Page::new(1)));
        pool.insert(Arc::new(Page::new(2)));
        pool.insert(Arc::new(Page::new(3)));

        // Access page 1 to make it most recently used
        let _ = pool.get(1);

        // Insert new page - should evict page 2 (LRU)
        pool.insert(Arc::new(Page::new(4)));

        assert!(pool.get(1).is_some()); // Should exist (was accessed)
        assert!(pool.get(2).is_none()); // Should be evicted
        assert!(pool.get(3).is_some());
        assert!(pool.get(4).is_some());
    }

    #[test]
    fn test_buffer_pool_stats() {
        let pool = BufferPool::new(10);

        pool.insert(Arc::new(Page::new(1)));
        let _ = pool.get(1); // Hit
        let _ = pool.get(999); // Miss

        let stats = pool.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_buffer_pool_hit_rate() {
        let pool = BufferPool::new(10);

        pool.insert(Arc::new(Page::new(1)));
        for _ in 0..9 {
            let _ = pool.get(1); // 9 hits
        }
        let _ = pool.get(999); // 1 miss

        assert!((pool.hit_rate() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_buffer_pool_prefetch() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let pool = BufferPool::with_prefetch(10, 3);

        let load_count = Rc::new(RefCell::new(0));
        let loader = |page_id: u32| {
            *load_count.borrow_mut() += 1;
            Arc::new(Page::new(page_id))
        };

        // Prefetch pages 1, 2, 3
        pool.prefetch_range(1, 3, loader);

        // Should have loaded 3 pages
        assert!(*load_count.borrow() >= 3);

        // Getting prefetched pages should be hits
        let _ = pool.get(1);
        let _ = pool.get(2);

        let stats = pool.stats();
        assert!(stats.prefetch_hits >= 2);
    }

    #[test]
    fn test_buffer_pool_remove() {
        let pool = BufferPool::new(10);
        pool.insert(Arc::new(Page::new(1)));

        assert!(pool.remove(1));
        assert!(pool.get(1).is_none());

        // Remove non-existent should return false
        assert!(!pool.remove(999));
    }

    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::new(1024);

        let buf1 = pool.allocate();
        assert_eq!(buf1.len(), 1024);

        pool.free(buf1);

        // Next allocation should get the freed buffer
        let buf2 = pool.allocate();
        assert_eq!(buf2.len(), 1024);
        assert_eq!(pool.free_count(), 0); // Used and returned
    }

    #[test]
    fn test_memory_pool_wrong_size() {
        let pool = MemoryPool::new(1024);

        let mut buf = vec![0u8; 2048];
        pool.free(buf); // Wrong size, should not be pooled

        assert_eq!(pool.free_count(), 0);
    }

    #[test]
    fn test_buffer_pool_clear() {
        let pool = BufferPool::new(10);
        pool.insert(Arc::new(Page::new(1)));
        pool.insert(Arc::new(Page::new(2)));

        assert_eq!(pool.len(), 2);
        pool.clear();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_buffer_pool_allocate() {
        let pool = BufferPool::new(10);
        let page = pool.allocate(1);

        assert_eq!(page.page_id(), 1);
        assert!(pool.get(1).is_some());
    }

    #[test]
    fn test_buffer_pool_get_or_load() {
        let pool = BufferPool::new(10);
        let load_count = std::sync::atomic::AtomicUsize::new(0);

        let page = pool.get_or_load(1, |_| {
            load_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Arc::new(Page::new(1))
        });

        assert_eq!(page.page_id(), 1);
        assert_eq!(load_count.load(std::sync::atomic::Ordering::SeqCst), 1);

        let page2 = pool.get_or_load(1, |_| {
            load_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Arc::new(Page::new(1))
        });

        assert_eq!(page2.page_id(), 1);
        assert_eq!(load_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_buffer_pool_reset_stats() {
        let pool = BufferPool::new(10);
        pool.insert(Arc::new(Page::new(1)));
        let _ = pool.get(1);
        let _ = pool.get(999);

        pool.reset_stats();

        let stats = pool.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_buffer_pool_len() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.len(), 0);

        pool.insert(Arc::new(Page::new(1)));
        pool.insert(Arc::new(Page::new(2)));

        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_buffer_pool_insert_existing() {
        let pool = BufferPool::new(10);
        pool.insert(Arc::new(Page::new(1)));
        pool.insert(Arc::new(Page::new(1)));

        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_buffer_pool_prefetch_loader() {
        let pool = BufferPool::with_prefetch(10, 2);

        pool.prefetch(&[100, 101, 102], |page_id| Arc::new(Page::new(page_id)));

        assert!(pool.get(100).is_some());
        assert!(pool.get(101).is_some());
    }

    #[test]
    fn test_buffer_pool_stats_evictions() {
        let pool = BufferPool::new(2);

        pool.insert(Arc::new(Page::new(1)));
        pool.insert(Arc::new(Page::new(2)));
        pool.insert(Arc::new(Page::new(3)));

        let stats = pool.stats();
        assert!(stats.evictions >= 1);
    }

    #[test]
    fn test_buffer_pool_with_prefetch() {
        let pool = BufferPool::with_prefetch(10, 5);
        assert_eq!(pool.capacity(), 10);
    }

    #[test]
    fn test_memory_pool_multiple_allocations() {
        let pool = MemoryPool::new(1024);

        let buf1 = pool.allocate();
        let buf2 = pool.allocate();

        assert_eq!(buf1.len(), 1024);
        assert_eq!(buf2.len(), 1024);
    }

    #[test]
    fn test_memory_pool_block_size() {
        let pool = MemoryPool::new(2048);
        assert_eq!(pool.block_size(), 2048);
    }

    #[test]
    fn test_memory_pool_default() {
        let pool = MemoryPool::default();
        assert_eq!(pool.block_size(), 4096);
    }

    #[test]
    fn test_buffer_pool_stats_default() {
        let stats = BufferPoolStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.prefetch_hits, 0);
    }

    #[test]
    fn test_buffer_pool_stats_hit_rate_zero() {
        let stats = BufferPoolStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_buffer_pool_stats_hit_rate_full() {
        let stats = BufferPoolStats {
            hits: 10,
            misses: 0,
            evictions: 0,
            prefetch_hits: 0,
        };
        assert_eq!(stats.hit_rate(), 1.0);
    }

    #[test]
    fn test_lruk_buffer_pool_creation() {
        let pool = BufferPoolLruK::new(100, 2);
        assert_eq!(pool.capacity(), 100);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_lruk_buffer_pool_insert_and_get() {
        let pool = BufferPoolLruK::new(10, 2);

        let page = Arc::new(Page::new_data(1, 100));
        pool.insert(page);

        assert_eq!(pool.len(), 1);
        assert!(pool.get(1).is_some());
    }

    #[test]
    fn test_lruk_buffer_pool_eviction() {
        let pool = BufferPoolLruK::new(2, 2);

        // Insert 3 pages, should evict 1
        let page1 = Arc::new(Page::new_data(1, 100));
        let page2 = Arc::new(Page::new_data(2, 100));
        let page3 = Arc::new(Page::new_data(3, 100));

        pool.insert(page1);
        pool.insert(page2);
        pool.insert(page3);

        // Pool should have 2 pages
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_lruk_buffer_pool_hit_rate() {
        let pool = BufferPoolLruK::new(10, 2);

        let page = Arc::new(Page::new_data(1, 100));
        pool.insert(page);

        // Access multiple times
        pool.get(1);
        pool.get(1);
        pool.get(1);

        assert!(pool.hit_rate() > 0.0);
    }

    #[test]
    fn test_lruk_buffer_pool_clear() {
        let pool = BufferPoolLruK::new(10, 2);

        let page = Arc::new(Page::new_data(1, 100));
        pool.insert(page);

        pool.clear();

        assert!(pool.is_empty());
    }

    #[test]
    fn test_buffer_pool_pin_unpin() {
        let pool = BufferPool::new(10);
        pool.insert(Arc::new(Page::new(1)));

        assert!(!pool.is_pinned(1));
        assert_eq!(pool.pin_count(1), 0);

        pool.pin(1);
        assert!(pool.is_pinned(1));
        assert_eq!(pool.pin_count(1), 1);

        pool.pin(1);
        assert_eq!(pool.pin_count(1), 2);

        assert!(!pool.unpin(1)); // Still pinned
        assert_eq!(pool.pin_count(1), 1);

        assert!(pool.unpin(1)); // Now unpinned
        assert!(!pool.is_pinned(1));
        assert_eq!(pool.pin_count(1), 0);
    }

    #[test]
    fn test_buffer_pool_skip_pinned_on_eviction() {
        let pool = BufferPool::new(3); // Capacity of 3

        // Insert 3 pages
        pool.insert(Arc::new(Page::new(1)));
        pool.insert(Arc::new(Page::new(2)));
        pool.insert(Arc::new(Page::new(3)));

        // Access pages to set LRU order: 3 is MRU, 1 is LRU
        let _ = pool.get(1);
        let _ = pool.get(2);
        let _ = pool.get(3);
        // LRU order is now: [3, 2, 1] (3 is front/MRU, 1 is back/LRU)

        // Pin the LRU page (page 1)
        pool.pin(1);

        // Insert new page - should evict page 2 (LRU), NOT page 1 (which is pinned)
        pool.insert(Arc::new(Page::new(4)));

        // Page 1 should still be there (pinned), page 2 should be evicted
        assert!(pool.get(1).is_some()); // Pinned page should remain
        assert!(pool.get(2).is_none()); // Should be evicted
        assert!(pool.get(3).is_some());
        assert!(pool.get(4).is_some());

        // Unpin page 1
        pool.unpin(1);

        // Insert another page - now page 1 is the only unpinned LRU candidate
        pool.insert(Arc::new(Page::new(5)));

        // Now page 1 should be evicted
        assert!(pool.get(1).is_none()); // Now evicted
    }

    #[test]
    fn test_buffer_pool_unpin_allows_eviction() {
        let pool = BufferPool::new(2);

        pool.insert(Arc::new(Page::new(1)));
        pool.insert(Arc::new(Page::new(2)));

        // Pin both pages
        pool.pin(1);
        pool.pin(2);

        // Try to insert new page - cannot evict any (both pinned)
        pool.insert(Arc::new(Page::new(3)));

        // Pool is still at capacity (pinned pages can't be evicted)
        assert_eq!(pool.len(), 2);

        // Unpin one page
        pool.unpin(1);

        // Now insert should evict page 1
        pool.insert(Arc::new(Page::new(4)));

        assert!(pool.get(1).is_none()); // Evicted
        assert!(pool.get(2).is_some()); // Still pinned
        assert!(pool.get(3).is_some());
        assert!(pool.get(4).is_some());
    }
}
