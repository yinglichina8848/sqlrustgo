//! ReusableVec - Thread-local reusable vectors for executor hot paths
//!
//! Reduces Vec allocation cycles in the execution engine by reusing
//! pre-allocated buffers across query execution.
//!
//! ## Problem
//!
//! Current execution pattern:
//! ```text
//! Vec::new() → grow() → process() → drop() → free()
//! Vec::new() → grow() → process() → drop() → free()
//! Vec::new() → grow() → process() → drop() → free()
//! ```
//!
//! ## Solution
//!
//! ReusableVec pattern:
//! ```text
//! reusable_vec.clear() → grow() → process()
//! reusable_vec.clear() → grow() → process()
//! reusable_vec.clear() → grow() → process()
//! ```
//!
//! Benefits:
//! - Reduces malloc/free calls
//! - Keeps allocated capacity
//! - Improves cache locality
//!
//! ## Thread Safety
//!
//! ThreadLocalExecutorVecPool uses thread-local storage, providing:
//! - Zero contention between threads
//! - O(1) access time
//! - Automatic cleanup on thread exit

use std::cell::RefCell;

/// A vector that can be reused across multiple operations without reallocation
///
/// # Example
///
/// ```rust
/// let mut reusable = ReusableVec::<i32>::new();
///
/// // First use
/// reusable.push(1);
/// reusable.push(2);
/// process(&reusable);
/// reusable.clear();
///
/// // Second use - no reallocation
/// reusable.push(3);
/// reusable.push(4);
/// process(&reusable);
/// ```
pub struct ReusableVec<T> {
    /// The inner buffer
    inner: Vec<T>,
}

impl<T> ReusableVec<T> {
    /// Create a new ReusableVec with default capacity
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Create with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Clear all elements without deallocating
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get the number of elements
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the current capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Push an element
    #[inline]
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Extend with an iterator
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }

    /// Get a reference to an element
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Get a mutable reference
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }

    /// Iterate over elements
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.inner.iter()
    }

    /// Iterate mutably
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.inner.iter_mut()
    }

    /// Drain all elements
    #[inline]
    pub fn drain(&mut self) -> std::vec::Drain<'_, T> {
        self.inner.drain(..)
    }

    /// Reset to empty state but keep capacity
    #[inline]
    pub fn reset(&mut self) {
        self.inner.clear();
    }

    /// Take all elements, leaving an empty vec
    #[inline]
    pub fn take(&mut self) -> Vec<T> {
        std::mem::take(&mut self.inner)
    }

    /// As a slice
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    /// As a mutable slice
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.inner
    }

    /// Append another vec
    #[inline]
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.inner.append(other);
    }

    /// Reserve additional capacity
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Resize to have `len()` elements with the given value
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.inner.resize(new_len, value);
    }

    /// Truncate to `len()` elements
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.inner.truncate(len);
    }
}

impl<T> Default for ReusableVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::ops::Deref for ReusableVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for ReusableVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> std::fmt::Debug for ReusableVec<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReusableVec")
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .finish()
    }
}

// ============================================================
// Thread-Local Executor Vec Pool
// ============================================================

/// Thread-local pool of reusable vectors for executor operations
///
/// Each thread maintains its own pool of reusable vectors, providing:
/// - Zero contention (thread-local storage)
/// - O(1) access time
/// - Automatic memory release when thread exits
pub struct ThreadLocalExecutorVecPool {
    /// Pool of reusable row vectors (Vec<Vec<Value>>)
    pub rows: ReusableVec<Vec<sqlrustgo_types::Value>>,
    /// Pool of reusable expression result vectors
    pub expr_results: ReusableVec<sqlrustgo_types::Value>,
    /// Pool of reusable index vectors (for joins, aggregations)
    pub indices: ReusableVec<u32>,
    /// Pool of reusable filter masks
    pub filter_masks: ReusableVec<bool>,
    /// Pool of reusable key vectors (for hashing)
    pub keys: ReusableVec<sqlrustgo_types::Value>,
    /// Pool of reusable row buffers for projection
    pub projection_buffers: ReusableVec<Vec<sqlrustgo_types::Value>>,
}

impl ThreadLocalExecutorVecPool {
    /// Create a new pool with default capacities optimized for query execution
    pub fn new() -> Self {
        Self {
            rows: ReusableVec::with_capacity(1024),
            expr_results: ReusableVec::with_capacity(256),
            indices: ReusableVec::with_capacity(1024),
            filter_masks: ReusableVec::with_capacity(1024),
            keys: ReusableVec::with_capacity(256),
            projection_buffers: ReusableVec::with_capacity(64),
        }
    }

    /// Create with explicit row capacity
    pub fn with_row_capacity(capacity: usize) -> Self {
        Self {
            rows: ReusableVec::with_capacity(capacity),
            expr_results: ReusableVec::with_capacity(256),
            indices: ReusableVec::with_capacity(1024),
            filter_masks: ReusableVec::with_capacity(1024),
            keys: ReusableVec::with_capacity(256),
            projection_buffers: ReusableVec::with_capacity(64),
        }
    }

    /// Clear all pools (keep capacity)
    #[inline]
    pub fn clear_all(&mut self) {
        self.rows.clear();
        self.expr_results.clear();
        self.indices.clear();
        self.filter_masks.clear();
        self.keys.clear();
        self.projection_buffers.clear();
    }

    /// Reset all pools (same as clear for bumpalo-style arenas)
    #[inline]
    pub fn reset(&mut self) {
        self.clear_all();
    }

    /// Get a reusable row buffer, clearing it first if needed
    #[inline]
    pub fn take_rows(&mut self) -> Vec<Vec<sqlrustgo_types::Value>> {
        self.rows.take()
    }

    /// Return a row buffer to the pool
    #[inline]
    pub fn return_rows(&mut self, rows: Vec<Vec<sqlrustgo_types::Value>>) {
        self.rows.inner = rows;
    }

    /// Get a reusable index vector, clearing it first if needed
    #[inline]
    pub fn take_indices(&mut self) -> Vec<u32> {
        self.indices.take()
    }

    /// Return an index vector to the pool
    #[inline]
    pub fn return_indices(&mut self, indices: Vec<u32>) {
        self.indices.inner = indices;
    }
}

impl Default for ThreadLocalExecutorVecPool {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Global Thread-Local Registry
// ============================================================

thread_local! {
    static EXECUTOR_VEC_POOL: RefCell<Option<ThreadLocalExecutorVecPool>> = RefCell::new(None);
}

/// Execute a closure with the thread-local executor vec pool
///
/// This provides exclusive access to the pool without lifetime issues.
/// The pool is created on first access.
///
/// # Example
///
/// ```rust
/// with_thread_local_pool(|pool| {
///     pool.rows.push(vec![Value::Integer(1)]);
/// });
/// ```
pub fn with_thread_local_pool<F, R>(f: F) -> R
where
    F: FnOnce(&mut ThreadLocalExecutorVecPool) -> R,
{
    EXECUTOR_VEC_POOL.with(|cell| {
        let mut pool = cell.borrow_mut();
        if pool.is_none() {
            *pool = Some(ThreadLocalExecutorVecPool::new());
        }
        // Safety: We have exclusive access via &mut
        // The RefCell ensures only one thread accesses this at a time
        f(pool.as_mut().unwrap())
    })
}

/// Clear the thread-local pool (call at query end)
pub fn clear_thread_local_pool() {
    EXECUTOR_VEC_POOL.with(|cell| {
        let mut pool = cell.borrow_mut();
        if let Some(p) = pool.as_mut() {
            p.reset();
        }
    });
}

/// Reset the pool to initial state (same as clear)
pub fn reset_thread_local_pool() {
    clear_thread_local_pool();
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_reusable_vec_basic() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len(), 3);
        assert!(vec.capacity() >= 3);
    }

    #[test]
    fn test_reusable_vec_clear() {
        let mut vec = ReusableVec::with_capacity(10);
        vec.push(1);
        vec.push(2);
        assert_eq!(vec.len(), 2);
        let cap = vec.capacity();
        vec.clear();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), cap);
    }

    #[test]
    fn test_reusable_vec_reuse() {
        let mut vec = ReusableVec::with_capacity(1024);
        vec.push(1);
        let cap = vec.capacity();
        vec.clear();
        vec.push(2);
        assert_eq!(vec.capacity(), cap);
    }

    #[test]
    fn test_reusable_vec_take() {
        let mut vec = ReusableVec::with_capacity(10);
        vec.push(1);
        vec.push(2);
        let taken = vec.take();
        assert_eq!(taken, vec![1, 2]);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_thread_local_pool() {
        with_thread_local_pool(|pool| {
            pool.rows.push(vec![Value::Integer(1), Value::Integer(2)]);
            assert_eq!(pool.rows.len(), 1);
        });
        // After reset, pool should be empty
        reset_thread_local_pool();
        with_thread_local_pool(|pool| {
            assert!(pool.rows.is_empty());
        });
    }

    #[test]
    fn test_pool_clear_all() {
        with_thread_local_pool(|pool| {
            pool.rows.push(vec![Value::Integer(1)]);
            pool.indices.push(42);
            pool.clear_all();
            assert!(pool.rows.is_empty());
            assert!(pool.indices.is_empty());
        });
    }
}
