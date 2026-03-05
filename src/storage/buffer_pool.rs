//! Buffer Pool Manager with LRU cache, prefetch, and memory pool optimization

use super::page::Page;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};

/// Buffer Pool with LRU eviction, prefetch, and memory pool
pub struct BufferPool {
    /// Page storage with LRU tracking
    pages: Mutex<HashMap<u32, Arc<Page>>>,
    /// LRU queue - most recently used at front
    lru: Mutex<VecDeque<u32>>,
    /// Capacity
    capacity: usize,
    /// Prefetch window size
    prefetch_window: usize,
    /// Statistics
    stats: RwLock<BufferPoolStats>,
}

/// Buffer pool statistics
#[derive(Debug, Default)]
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
            capacity,
            prefetch_window,
            stats: RwLock::new(BufferPoolStats::default()),
        }
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
        self.insert(page.clone());
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

        // Evict if at capacity
        while pages.len() >= self.capacity && !lru.is_empty() {
            if let Some(evicted_id) = lru.pop_back() {
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
}
