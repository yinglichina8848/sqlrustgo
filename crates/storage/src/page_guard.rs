//! PageGuard - RAII wrapper for thread-safe page access
//!
//! PageGuard provides automatic lifecycle management for pages fetched from
//! the buffer pool. When a page is fetched, it's automatically pinned to
//! prevent eviction. When the guard is dropped, the page is automatically
//! unpinned, allowing it to be evicted if needed.
//!
//! ## Usage
//!
//! ```ignore
//! // Page is automatically pinned on fetch
//! let page_guard = buffer_pool.fetch_page(page_id)?;
//!
//! // Use the page...
//! let page = page_guard.page();
//!
//! // Page is automatically unpinned when guard goes out of scope
//! drop(page_guard);
//! // Or implicitly dropped at end of scope
//! ```
//!
//! ## Benefits
//!
//! - **Memory safety**: Never forget to unpin a page
//! - **Exception safety**: Page is unpinned even if code panics
//! - **Clear semantics**: Explicit ownership of pinned page

use std::sync::Arc;

use crate::page::{Page, PageType};

/// Trait for pool-like types that support pin/unpin
pub trait PoolLike {
    fn pin(&self, page_id: u32);
    fn unpin(&self, page_id: u32) -> bool;
}

/// RAII guard for page access with automatic pin/unpin lifecycle management
///
/// PageGuard wraps a page fetched from the buffer pool and automatically
/// manages its pin count. When created, the page is pinned. When dropped,
/// the page is unpinned, allowing it to be evicted from the buffer pool
/// if necessary.
///
/// # Example
///
/// ```ignore
/// {
///     let guard = pool.fetch_page(42)?;
///     // Page 42 is now pinned and cannot be evicted
///     let page = guard.page();
///     // ... use page ...
/// } // guard dropped, page 42 is now unpinned
/// ```
pub struct PageGuard<'a, P: PoolLike> {
    /// The page being guarded (Arc allows sharing)
    page: Arc<Page>,
    /// Reference to buffer pool for unpin on drop
    pool: &'a P,
    /// Whether this guard owns the pin (false for read-only guards)
    exclusive: bool,
}

impl<'a, P: PoolLike> PageGuard<'a, P> {
    /// Create a new page guard (typically called by BufferPool::fetch_page)
    pub(crate) fn new(page: Arc<Page>, pool: &'a P, exclusive: bool) -> Self {
        // Pin the page when guard is created
        pool.pin(page.page_id());

        Self {
            page,
            pool,
            exclusive,
        }
    }

    /// Get a reference to the underlying page
    #[inline]
    pub fn page(&self) -> &Arc<Page> {
        &self.page
    }

    /// Get the page ID
    #[inline]
    pub fn page_id(&self) -> u32 {
        self.page.page_id()
    }

    /// Get the page data directly
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.page.data
    }

    /// Check if this is an exclusive (write) guard
    #[inline]
    pub fn is_exclusive(&self) -> bool {
        self.exclusive
    }

    /// Take ownership of the underlying page, bypassing the guard's Drop
    ///
    /// This consumes the guard and returns the page without unpinning it.
    /// The caller takes over responsibility for the page's pin lifecycle.
    ///
    /// Note: The caller MUST ensure the page is eventually unpinned,
    /// otherwise the page will remain pinned indefinitely.
    #[inline]
    pub fn into_page(self) -> Arc<Page> {
        use std::mem::ManuallyDrop;
        // Prevent the guard's Drop from running
        let guard = ManuallyDrop::new(self);
        // Clone the page Arc - this transfers ownership without unpinning
        // ManuallyDrop ensures Drop won't run on the guard
        Arc::clone(&guard.page)
    }
}

impl<'a, P: PoolLike> Drop for PageGuard<'a, P> {
    fn drop(&mut self) {
        // Unpin the page when guard is dropped
        self.pool.unpin(self.page.page_id());
    }
}

impl<'a, P: PoolLike> std::fmt::Debug for PageGuard<'a, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageGuard")
            .field("page_id", &self.page.page_id())
            .field("exclusive", &self.exclusive)
            .finish()
    }
}

impl<'a, P: PoolLike> std::ops::Deref for PageGuard<'a, P> {
    type Target = Page;

    fn deref(&self) -> &Self::Target {
        &self.page
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock BufferPool for testing
    struct MockBufferPool {
        pin_count: std::sync::Mutex<std::collections::HashMap<u32, u32>>,
    }

    impl PoolLike for MockBufferPool {
        fn pin(&self, page_id: u32) {
            let mut count = self.pin_count.lock().unwrap();
            *count.entry(page_id).or_insert(0) += 1;
        }

        fn unpin(&self, page_id: u32) -> bool {
            let mut count = self.pin_count.lock().unwrap();
            if let Some(c) = count.get_mut(&page_id) {
                if *c > 0 {
                    *c -= 1;
                    if *c == 0 {
                        count.remove(&page_id);
                    }
                    return true;
                }
            }
            false
        }
    }

    impl MockBufferPool {
        fn new() -> Self {
            Self {
                pin_count: std::sync::Mutex::new(std::collections::HashMap::new()),
            }
        }

        fn pin_count(&self, page_id: u32) -> u32 {
            let count = self.pin_count.lock().unwrap();
            *count.get(&page_id).unwrap_or(&0)
        }
    }

    #[test]
    fn test_page_guard_creation_pins_page() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(1));

        assert_eq!(pool.pin_count(1), 0);

        let _guard = PageGuard::new(Arc::clone(&page), &pool, true);

        assert_eq!(pool.pin_count(1), 1);
    }

    #[test]
    fn test_page_guard_drop_unpins_page() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(1));

        {
            let guard = PageGuard::new(Arc::clone(&page), &pool, true);
            assert_eq!(pool.pin_count(1), 1);
        } // guard dropped

        assert_eq!(pool.pin_count(1), 0);
    }

    #[test]
    fn test_page_guard_multiple_pins() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(1));

        let _guard1 = PageGuard::new(Arc::clone(&page), &pool, true);
        assert_eq!(pool.pin_count(1), 1);

        let _guard2 = PageGuard::new(Arc::clone(&page), &pool, true);
        assert_eq!(pool.pin_count(1), 2);

        // Drop first guard
        drop(_guard1);
        assert_eq!(pool.pin_count(1), 1);

        // Drop second guard
        drop(_guard2);
        assert_eq!(pool.pin_count(1), 0);
    }

    #[test]
    fn test_page_guard_page_access() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(42));

        let guard = PageGuard::new(Arc::clone(&page), &pool, true);

        assert_eq!(guard.page_id(), 42);
        assert_eq!(guard.page().page_id(), 42);
    }

    #[test]
    fn test_page_guard_debug() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(1));

        let guard = PageGuard::new(Arc::clone(&page), &pool, true);
        let debug = format!("{:?}", guard);

        assert!(debug.contains("PageGuard"));
        assert!(debug.contains("1"));
    }

    #[test]
    fn test_page_guard_exclusive_flag() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(1));

        let exclusive_guard = PageGuard::new(Arc::clone(&page), &pool, true);
        assert!(exclusive_guard.is_exclusive());

        let shared_guard = PageGuard::new(Arc::clone(&page), &pool, false);
        assert!(!shared_guard.is_exclusive());
    }

    #[test]
    fn test_page_guard_deref() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(99));

        let guard = PageGuard::new(Arc::clone(&page), &pool, true);

        // Should be able to access page fields through deref
        assert_eq!(guard.page_id(), 99);
        assert_eq!(guard.page_type(), PageType::Free);
    }

    #[test]
    fn test_page_guard_into_page() {
        let pool = MockBufferPool::new();
        let page = Arc::new(Page::new(1));

        let guard = PageGuard::new(Arc::clone(&page), &pool, true);
        let taken_page = guard.into_page();

        // Page should still be pinned (pool reference was forgotten)
        assert_eq!(pool.pin_count(1), 1);
        assert_eq!(taken_page.page_id(), 1);

        // Manually unpin to clean up
        pool.unpin(1);
    }
}
