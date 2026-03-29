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
/// use sqlrustgo_executor::ReusableVec;
///
/// let mut reusable = ReusableVec::<i32>::new();
///
/// // First use
/// reusable.push(1);
/// reusable.push(2);
/// assert_eq!(reusable.len(), 2);
/// reusable.clear();
///
/// // Second use - no reallocation
/// reusable.push(3);
/// reusable.push(4);
/// assert_eq!(reusable.len(), 2);
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
/// use sqlrustgo_executor::{with_thread_local_pool, ThreadLocalExecutorVecPool};
/// use sqlrustgo_types::Value;
///
/// with_thread_local_pool(|pool: &mut ThreadLocalExecutorVecPool| {
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

    #[test]
    fn test_reusable_vec_with_capacity() {
        let vec = ReusableVec::<i32>::with_capacity(100);
        assert!(vec.capacity() >= 100);
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_reusable_vec_get() {
        let mut vec = ReusableVec::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);
        assert_eq!(vec.get(0), Some(&10));
        assert_eq!(vec.get(1), Some(&20));
        assert_eq!(vec.get(2), Some(&30));
        assert_eq!(vec.get(3), None);
        assert_eq!(vec.get(usize::MAX), None);
    }

    #[test]
    fn test_reusable_vec_get_mut() {
        let mut vec = ReusableVec::new();
        vec.push(10);
        vec.push(20);
        if let Some(elem) = vec.get_mut(1) {
            *elem = 25;
        }
        assert_eq!(vec.get(1), Some(&25));
    }

    #[test]
    fn test_reusable_vec_iter() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_reusable_vec_iter_mut() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        for elem in vec.iter_mut() {
            *elem *= 2;
        }
        assert_eq!(vec.get(0), Some(&2));
        assert_eq!(vec.get(1), Some(&4));
        assert_eq!(vec.get(2), Some(&6));
    }

    #[test]
    fn test_reusable_vec_extend() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.extend(vec![2, 3, 4]);
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.get(3), Some(&4));
    }

    #[test]
    fn test_reusable_vec_drain() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        let drained: Vec<i32> = vec.drain().collect();
        assert_eq!(drained, vec![1, 2, 3]);
        assert!(vec.is_empty());
        assert!(vec.capacity() > 0);
    }

    #[test]
    fn test_reusable_vec_reset() {
        let mut vec = ReusableVec::with_capacity(10);
        vec.push(1);
        vec.push(2);
        let cap = vec.capacity();
        vec.reset();
        assert!(vec.is_empty());
        assert_eq!(vec.capacity(), cap);
    }

    #[test]
    fn test_reusable_vec_as_slice() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        let slice: &[i32] = vec.as_slice();
        assert_eq!(slice, &[1, 2]);
    }

    #[test]
    fn test_reusable_vec_as_mut_slice() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        let slice: &mut [i32] = vec.as_mut_slice();
        slice[0] = 10;
        assert_eq!(vec.get(0), Some(&10));
    }

    #[test]
    fn test_reusable_vec_append() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        let mut other = vec![10, 20];
        vec.append(&mut other);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(2), Some(&20));
        assert!(other.is_empty());
    }

    #[test]
    fn test_reusable_vec_reserve() {
        let mut vec: ReusableVec<i32> = ReusableVec::with_capacity(5);
        let initial_cap = vec.capacity();
        vec.reserve(100);
        assert!(vec.capacity() > initial_cap);
        assert!(!vec.is_empty() || initial_cap >= 5);
    }

    #[test]
    fn test_reusable_vec_resize() {
        let mut vec = ReusableVec::new();
        vec.resize(3, 5);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&5));
        assert_eq!(vec.get(1), Some(&5));
        assert_eq!(vec.get(2), Some(&5));
    }

    #[test]
    fn test_reusable_vec_truncate() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.truncate(2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(2), None);
    }

    #[test]
    fn test_reusable_vec_truncate_larger() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.truncate(100);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn test_reusable_vec_pop() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        let popped = vec.pop();
        assert_eq!(popped, Some(2));
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn test_reusable_vec_pop_empty() {
        let mut vec = ReusableVec::<i32>::new();
        let popped = vec.pop();
        assert_eq!(popped, None);
    }

    #[test]
    fn test_reusable_vec_last() {
        let mut vec = ReusableVec::new();
        assert_eq!(vec.last(), None);
        vec.push(1);
        vec.push(2);
        assert_eq!(vec.last(), Some(&2));
    }

    #[test]
    fn test_reusable_vec_first() {
        let mut vec = ReusableVec::new();
        assert_eq!(vec.first(), None);
        vec.push(1);
        vec.push(2);
        assert_eq!(vec.first(), Some(&1));
    }

    #[test]
    fn test_reusable_vec_contains() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert!(vec.contains(&2));
        assert!(!vec.contains(&4));
    }

    #[test]
    fn test_reusable_vec_binary_search() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(3);
        vec.push(5);
        vec.push(7);
        assert_eq!(vec.binary_search(&5), Ok(2));
        assert_eq!(vec.binary_search(&4), Err(2));
    }

    #[test]
    fn test_reusable_vec_reverse() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.reverse();
        assert_eq!(vec.get(0), Some(&3));
        assert_eq!(vec.get(2), Some(&1));
    }

    #[test]
    fn test_reusable_vec_sort() {
        let mut vec = ReusableVec::new();
        vec.push(3);
        vec.push(1);
        vec.push(2);
        vec.sort();
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&3));
    }

    #[test]
    fn test_reusable_vec_dedup() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(1);
        vec.push(2);
        vec.push(1);
        vec.dedup();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&1));
    }

    #[test]
    fn test_reusable_vec_swap_remove() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        let removed = vec.swap_remove(1);
        assert_eq!(removed, 2);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(1), Some(&4));
    }

    #[test]
    fn test_reusable_vec_insert() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(3);
        vec.insert(1, 2);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(1), Some(&2));
    }

    #[test]
    fn test_reusable_vec_remove() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        let removed = vec.remove(1);
        assert_eq!(removed, 2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(1), Some(&3));
    }

    #[test]
    fn test_reusable_vec_retain() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.retain(|x| x % 2 == 0);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), Some(&2));
        assert_eq!(vec.get(1), Some(&4));
    }

    #[test]
    fn test_reusable_vec_value_type() {
        let mut vec = ReusableVec::new();
        vec.push(Value::Integer(42));
        vec.push(Value::Text("hello".to_string()));
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_thread_local_pool_multiple_operations() {
        clear_thread_local_pool();
        with_thread_local_pool(|pool| {
            pool.rows.push(vec![Value::Integer(1)]);
            pool.indices.push(0);
            pool.rows.push(vec![Value::Integer(2)]);
            pool.indices.push(1);
            assert_eq!(pool.rows.len(), 2);
            assert_eq!(pool.indices.len(), 2);
        });
    }

    #[test]
    fn test_thread_local_pool_preserves_capacity() {
        clear_thread_local_pool();
        with_thread_local_pool(|pool| {
            pool.rows.push(vec![Value::Integer(1); 100]);
            let cap = pool.rows.capacity();
            pool.clear_all();
            assert_eq!(pool.rows.capacity(), cap);
        });
    }
}
