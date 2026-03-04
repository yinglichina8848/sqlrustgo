//! Buffer Pool Manager

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

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_basic() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.capacity(), 10);
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
    fn test_buffer_pool_allocate() {
        let pool = BufferPool::new(10);
        let page = pool.allocate(5);
        assert_eq!(page.page_id(), 5);
        let retrieved = pool.get(5);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_buffer_pool_empty() {
        let pool = BufferPool::new(5);
        let retrieved = pool.get(999);
        assert!(retrieved.is_none());
    }
}
