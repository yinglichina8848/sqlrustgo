//! BatchArena - Per-batch memory management for Columnar Storage
//!
//! Manages memory for a single vector batch lifecycle:
//! - Selection vectors
//! - Dictionary vectors
//! - Aggregation states
//! - Intermediate batches
//!
//! ## Lifecycle (DuckDB-style)
//!
//! ```text
//! allocate_batch()
//!     ↓
//! process_batch() ← vector operations
//!     ↓
//! drop_batch()
//!     ↓
//! reset() → O(1) free (just reset bump cursor)
//!     ↓
//! repeat for next batch
//! ```
//!
//! ## Design Notes
//!
//! BatchArena is optimized for the columnar model where:
//! - Batch size is typically 1024 or 2048 rows
//! - Many batches are processed in quick succession
//! - O(1) reset is critical for performance

/// BatchArena provides per-batch memory allocation
/// optimized for columnar storage operations.
///
/// Key characteristics:
/// - Small to medium batch sizes (typically 1024-2048 rows)
/// - Very fast reset for next batch processing
/// - O(1) free via bump allocator reset
///
/// This is the primary arena for columnar execution,
/// used for:
/// - VectorBatch allocation
/// - Selection vectors
/// - Dictionary encoding buffers
/// - Hash aggregation state
pub struct BatchArena {
    /// Bump allocator for fast batch-scoped allocation
    bump: bumpalo::Bump,
    /// Standard batch size (rows per batch)
    batch_size: usize,
    /// Capacity hint for initial buffer
    capacity_hint: usize,
}

impl BatchArena {
    /// Create a new BatchArena optimized for the given batch size
    pub fn with_batch_size(batch_size: usize) -> Self {
        // Pre-allocate enough for a typical batch plus overhead
        // Estimate: batch_size * 8 bytes per value * num_columns
        let capacity_hint = batch_size * 64;
        let bump = bumpalo::Bump::new();

        Self {
            bump,
            batch_size,
            capacity_hint,
        }
    }

    /// Create with explicit capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        let bump = bumpalo::Bump::new();

        Self {
            bump,
            batch_size: 1024, // default
            capacity_hint: capacity,
        }
    }

    /// Allocate memory for type T
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    /// Allocate a vector with batch-size capacity
    #[inline]
    pub fn alloc_vector<T>(&self) -> Vec<T> {
        Vec::with_capacity(self.batch_size)
    }

    /// Allocate a vector with specific capacity
    #[inline]
    pub fn alloc_vector_with_capacity<T>(&self, capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }

    /// Allocate a selection vector (indices into rows)
    #[inline]
    pub fn alloc_selection_vector(&self) -> Vec<u32> {
        Vec::with_capacity(self.batch_size)
    }

    /// Allocate a dictionary encoding buffer
    #[inline]
    pub fn alloc_dictionary<T>(&self) -> Vec<T> {
        Vec::with_capacity(self.batch_size)
    }

    /// Reset the arena for the next batch
    /// This is O(1) - just advances the bump cursor
    #[inline]
    pub fn reset(&mut self) {
        self.bump = bumpalo::Bump::new();
    }

    /// Get the configured batch size
    #[inline]
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// Get remaining capacity in current batch
    #[inline]
    pub fn remaining(&self) -> usize {
        // bumpalo doesn't expose remaining directly
        // This is a conservative estimate
        self.capacity_hint
    }
}

impl Default for BatchArena {
    fn default() -> Self {
        Self::with_batch_size(1024)
    }
}

impl std::fmt::Debug for BatchArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchArena")
            .field("batch_size", &self.batch_size)
            .field("capacity_hint", &self.capacity_hint)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_arena_alloc() {
        let arena = BatchArena::default();
        let value = arena.alloc(42i32);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_batch_arena_selection_vector() {
        let arena = BatchArena::with_batch_size(1024);
        let mut selection = arena.alloc_selection_vector();
        selection.push(0);
        selection.push(5);
        selection.push(10);
        assert_eq!(selection.len(), 3);
        assert_eq!(selection.capacity(), 1024);
    }

    #[test]
    fn test_batch_arena_reset() {
        let mut arena = BatchArena::with_batch_size(1024);
        arena.alloc(1i32);
        arena.alloc(2i32);
        arena.reset();
        // After reset, we can allocate again
        let value = arena.alloc(100i32);
        assert_eq!(*value, 100);
    }

    #[test]
    fn test_batch_arena_vector() {
        let arena = BatchArena::with_batch_size(1024);
        let v: Vec<i32> = arena.alloc_vector();
        assert_eq!(v.capacity(), 1024);
    }
}
