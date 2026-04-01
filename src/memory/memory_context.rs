//! MemoryContext - Unified Memory Management Entry Point
//!
//! This module provides the unified memory architecture that supports both:
//! - Row-based execution pipeline
//! - Columnar execution pipeline
//!
//! ## Architecture
//!
//! ```text
//! MemoryContext
//! ├── GlobalArena (engine lifecycle)
//! ├── QueryArena (per-query lifecycle)
//! └── BatchArena (per-batch lifecycle)
//! ```
//!
//! ## Usage
//!
//! ```rust
//! let context = MemoryContext::new();
//!
//! // Row-based execution uses QueryArena
//! let row = context.query.alloc(Row::new());
//!
//! // Columnar execution uses BatchArena
//! let batch = context.batch.alloc(VectorBatch::new());
//! ```
//!
//! ## Design Rationale
//!
//! By unifying memory management from the start, we avoid the need to
//! refactor the execution engine when Columnar Storage is implemented.
//! This follows the DuckDB/TiDB approach of shared allocator infrastructure.

use std::sync::Arc;
use super::{GlobalArena, QueryArena, BatchArena};

/// Unified memory management entry point for SQLRustGo
///
/// MemoryContext provides access to three levels of memory arenas:
/// - `global`: Engine-level, lives for database lifetime
/// - `query`: Per-query, reset after each query
/// - `batch`: Per-batch, optimized for columnar operations
pub struct MemoryContext {
    /// Global arena for engine-level allocations
    pub global: Arc<GlobalArena>,
    /// Query arena for per-query allocations
    pub query: QueryArena,
    /// Batch arena for columnar batch allocations
    pub batch: BatchArena,
}

impl MemoryContext {
    /// Create a new MemoryContext with default capacities
    pub fn new() -> Self {
        Self::with_capacities(4096, 1024 * 1024, 64 * 1024)
    }

    /// Create a MemoryContext with explicit capacities
    ///
    /// # Arguments
    /// * `batch_capacity` - Initial capacity for batch arena (bytes)
    /// * `query_capacity` - Initial capacity for query arena (bytes)
    /// * `global_capacity` - Initial capacity for global arena (bytes)
    pub fn with_capacities(
        batch_capacity: usize,
        query_capacity: usize,
        global_capacity: usize,
    ) -> Self {
        Self {
            global: Arc::new(GlobalArena::with_capacity(global_capacity)),
            query: QueryArena::with_capacity(query_capacity),
            batch: BatchArena::with_capacity(batch_capacity),
        }
    }

    /// Create with batch size hint (for columnar workloads)
    ///
    /// # Arguments
    /// * `batch_size` - Expected rows per batch (e.g., 1024 or 2048)
    pub fn with_batch_size(batch_size: usize) -> Self {
        Self::with_capacities(
            batch_size * 64,      // batch: ~64 bytes per row estimate
            1024 * 1024,            // query: 1MB
            4 * 1024 * 1024,        // global: 4MB
        )
    }

    /// Reset query-level memory (call after each query)
    #[inline]
    pub fn reset_query(&mut self) {
        self.query.reset();
    }

    /// Reset batch-level memory (call after each batch)
    #[inline]
    pub fn reset_batch(&mut self) {
        self.batch.reset();
    }

    /// Reset all arenas
    #[inline]
    pub fn reset_all(&mut self) {
        self.query.reset();
        self.batch.reset();
    }

    /// Get global arena (cloned Arc)
    pub fn global_arena(&self) -> Arc<GlobalArena> {
        Arc::clone(&self.global)
    }
}

impl Default for MemoryContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for MemoryContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryContext")
            .field("global", &self.global)
            .field("query", &self.query)
            .field("batch", &self.batch)
            .finish()
    }
}

/// Trait for unified arena allocation
///
/// Both row-based and columnar pipelines use this trait
/// to allocate memory, enabling shared allocator infrastructure.
///
/// # Example
///
/// ```rust
/// // Row-based pipeline
/// impl ArenaAlloc for QueryArena {
///     fn alloc<T>(&self, value: T) -> &mut T {
///         self.bump.alloc(value)
///     }
/// }
///
/// // Columnar pipeline
/// impl ArenaAlloc for BatchArena {
///     fn alloc<T>(&self, value: T) -> &mut T {
///         self.bump.alloc(value)
///     }
/// }
/// ```
pub trait ArenaAlloc {
    /// Allocate memory for a value
    fn alloc<T>(&self, value: T) -> &mut T;

    /// Allocate a vector with the given capacity
    fn alloc_vec<T>(&self, capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }

    /// Allocate a string
    fn alloc_string(&self, s: &str) -> String {
        String::from(s)
    }
}

impl ArenaAlloc for QueryArena {
    #[inline]
    fn alloc<T>(&self, value: T) -> &mut T {
        QueryArena::alloc(self, value)
    }

    #[inline]
    fn alloc_vec<T>(&self, capacity: usize) -> Vec<T> {
        QueryArena::alloc_vec(self, capacity)
    }

    #[inline]
    fn alloc_string(&self, s: &str) -> String {
        QueryArena::alloc_string(self, s)
    }
}

impl ArenaAlloc for BatchArena {
    #[inline]
    fn alloc<T>(&self, value: T) -> &mut T {
        BatchArena::alloc(self, value)
    }

    #[inline]
    fn alloc_vec<T>(&self, capacity: usize) -> Vec<T> {
        BatchArena::alloc_vector_with_capacity(self, capacity)
    }
}

impl ArenaAlloc for MemoryContext {
    #[inline]
    fn alloc<T>(&self, value: T) -> &mut T {
        self.query.alloc(value)
    }

    #[inline]
    fn alloc_vec<T>(&self, capacity: usize) -> Vec<T> {
        self.query.alloc_vec(capacity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_context_new() {
        let ctx = MemoryContext::new();
        assert!(ctx.global.total_allocated() == 0);
    }

    #[test]
    fn test_memory_context_query_alloc() {
        let ctx = MemoryContext::new();
        let value = ctx.query.alloc(42i32);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_memory_context_batch_alloc() {
        let ctx = MemoryContext::with_batch_size(1024);
        let value = ctx.batch.alloc(42i32);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_arena_alloc_trait() {
        let ctx = MemoryContext::new();
        // Use trait object style
        let value: &mut i32 = ctx.alloc(42);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_reset_query() {
        let mut ctx = MemoryContext::new();
        ctx.query.alloc(1i32);
        ctx.query.alloc(2i32);
        ctx.reset_query();
        // Can allocate after reset
        let value = ctx.query.alloc(100i32);
        assert_eq!(*value, 100);
    }

    #[test]
    fn test_reset_batch() {
        let mut ctx = MemoryContext::with_batch_size(1024);
        ctx.batch.alloc(1i32);
        ctx.batch.alloc(2i32);
        ctx.reset_batch();
        // Can allocate after reset
        let value = ctx.batch.alloc(100i32);
        assert_eq!(*value, 100);
    }
}
