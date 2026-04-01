//! ReusableVec - Reusable vector for execution engine
//!
//! Avoids repeated Vec::new(), grow(), drop(), free() cycles
//! in hot paths of the execution engine.
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

use std::mem;

/// A vector that can be reused across multiple operations
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
        Self {
            inner: Vec::new(),
        }
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

    /// Pop an element
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
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

    /// Swap-remove an element
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T
    where
        T: Clone,
    {
        self.inner.swap_remove(index)
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
// ExecutorContext integration
// ============================================================

/// Thread-local pool of reusable vectors for executor operations
#[derive(Default)]
pub struct ReusableVecPool {
    /// Pool of reusable row vectors
    pub rows: ReusableVec<Vec<crate::Value>>,
    /// Pool of reusable expression result vectors
    pub expr_results: ReusableVec<crate::ExecutorResult>,
    /// Pool of reusable index vectors
    pub indices: ReusableVec<u32>,
    /// Pool of reusable filter masks
    pub filter_masks: ReusableVec<bool>,
}

impl ReusableVecPool {
    /// Create a new pool with default capacities
    pub fn new() -> Self {
        Self {
            rows: ReusableVec::with_capacity(1024),
            expr_results: ReusableVec::with_capacity(256),
            indices: ReusableVec::with_capacity(1024),
            filter_masks: ReusableVec::with_capacity(1024),
        }
    }

    /// Clear all pools
    #[inline]
    pub fn clear_all(&mut self) {
        self.rows.clear();
        self.expr_results.clear();
        self.indices.clear();
        self.filter_masks.clear();
    }

    /// Reset all pools (keep capacity)
    #[inline]
    pub fn reset(&mut self) {
        self.rows.reset();
        self.expr_results.reset();
        self.indices.reset();
        self.filter_masks.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;

    #[test]
    fn test_reusable_vec_basic() {
        let mut vec = ReusableVec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len(), 3);
        // Vec allocates on first push, so capacity will be > 0
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
        assert_eq!(vec.capacity(), cap); // Capacity preserved
    }

    #[test]
    fn test_reusable_vec_reuse() {
        let mut vec = ReusableVec::with_capacity(1024);
        vec.push(1);
        let cap = vec.capacity();
        vec.clear();
        vec.push(2);
        // Should not reallocate
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
    fn test_reusable_vec_pool() {
        let mut pool = ReusableVecPool::new();
        pool.rows.push(vec![Value::Integer(1), Value::Integer(2)]);
        pool.indices.push(0);
        pool.indices.push(1);

        assert_eq!(pool.rows.len(), 1);
        assert_eq!(pool.indices.len(), 2);

        pool.clear_all();

        assert!(pool.rows.is_empty());
        assert!(pool.indices.is_empty());
    }
}
