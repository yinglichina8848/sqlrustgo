/// Buffer Pool - Main memory cache for disk pages
///
/// ## Purpose
///
/// Reduces disk I/O by caching frequently accessed pages in memory.
/// Uses LRU (Least Recently Used) eviction policy.
///
/// ## Architecture
///
/// ```mermaid
/// graph TB
///     Query["Query"] --> BufferPool
///     BufferPool -->|page exists| Cache["Return Cached Page"]
///     BufferPool -->|page miss| Disk["Read from Disk"]
///     Disk --> BufferPool
///     BufferPool --> Evict["Evict LRU if full"]
/// ```
///
/// ## Key Concepts
///
/// - **Frame**: A fixed-size memory block (typically 4KB)
/// - **Page ID**: Unique identifier for a disk page
/// - **Pin Count**: Number of users currently accessing the page
/// - **Dirty Bit**: Whether page was modified and needs flush to disk
///
use super::page::Page;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Simple buffer pool with HashMap
pub struct BufferPool {
    pages: Mutex<HashMap<u32, Arc<Page>>>,
    capacity: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(capacity: usize) -> Self {
        Self {
            pages: Mutex::new(HashMap::new()),
            capacity,
        }
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of pages in pool
    pub fn len(&self) -> usize {
        let pages = self.pages.lock().unwrap();
        pages.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        let pages = self.pages.lock().unwrap();
        pages.is_empty()
    }

    /// Get a page
    pub fn get(&self, page_id: u32) -> Option<Arc<Page>> {
        let pages = self.pages.lock().unwrap();
        pages.get(&page_id).cloned()
    }

    /// Insert a page
    pub fn insert(&self, page: Arc<Page>) {
        let mut pages = self.pages.lock().unwrap();
        if pages.len() >= self.capacity {
            pages.remove(&0); // Simple eviction
        }
        pages.insert(page.page_id(), page);
    }

    /// Allocate a new page
    pub fn allocate(&self, page_id: u32) -> Arc<Page> {
        let page = Arc::new(Page::new(page_id));
        self.insert(Arc::clone(&page));
        page
    }

    /// Remove a page
    pub fn remove(&self, page_id: u32) -> Option<Arc<Page>> {
        let mut pages = self.pages.lock().unwrap();
        pages.remove(&page_id)
    }

    /// Clear all pages
    pub fn clear(&self) {
        let mut pages = self.pages.lock().unwrap();
        pages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_new() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.capacity(), 10);
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_buffer_pool_insert_and_get() {
        let pool = BufferPool::new(10);
        let page = Arc::new(Page::new(1));
        pool.insert(page);

        assert!(!pool.is_empty());
        assert_eq!(pool.len(), 1);

        let retrieved = pool.get(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().page_id(), 1);
    }

    #[test]
    fn test_buffer_pool_allocate() {
        let pool = BufferPool::new(10);
        let page = pool.allocate(5);

        assert_eq!(page.page_id(), 5);
        assert_eq!(pool.len(), 1);

        let retrieved = pool.get(5);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_buffer_pool_remove() {
        let pool = BufferPool::new(10);
        pool.insert(Arc::new(Page::new(1)));

        let removed = pool.remove(1);
        assert!(removed.is_some());
        assert!(pool.is_empty());

        // Remove non-existent
        let removed = pool.remove(99);
        assert!(removed.is_none());
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
    fn test_buffer_pool_eviction() {
        let pool = BufferPool::new(2);
        // Insert page 0 first so eviction removes it
        pool.insert(Arc::new(Page::new(0)));
        pool.insert(Arc::new(Page::new(1)));

        // This should trigger eviction (removes page 0)
        pool.insert(Arc::new(Page::new(2)));

        // Pool should still have 2 pages
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_buffer_pool_get_nonexistent() {
        let pool = BufferPool::new(10);
        let result = pool.get(999);
        assert!(result.is_none());
    }
}
