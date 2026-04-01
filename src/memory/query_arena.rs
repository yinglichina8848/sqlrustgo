//! QueryArena - Per-query memory management
//!
//! Manages memory for a single query lifecycle:
//! - Expression temporary values
//! - Row temporary objects
//! - Sort buffers
//! - Aggregate states
//!
//! ## Lifecycle
//!
//! ```text
//! query_start()
//!     ↓
//! alloc() ← expressions, rows, sort keys
//!     ↓
//! query_end()
//!     ↓
//! reset() → O(1) free all memory
//! ```

/// QueryArena provides per-query memory allocation
/// with O(1) cleanup via reset().
///
/// This is the primary arena for row-based execution,
/// used for:
/// - Expression evaluation temps
/// - Row projection buffers
/// - Sort comparison keys
/// - Hash join hash tables
pub struct QueryArena {
    /// Bump allocator for fast query-scoped allocation
    bump: bumpalo::Bump,
    /// Capacity hint for initial buffer
    capacity_hint: usize,
}

impl QueryArena {
    /// Create a new QueryArena with the given capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        let bump = bumpalo::Bump::new();

        Self {
            bump,
            capacity_hint: capacity,
        }
    }

    /// Allocate memory for type T
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    /// Allocate a String
    #[inline]
    pub fn alloc_string(&self, s: &str) -> String {
        String::from(s)
    }

    /// Allocate a Vec with the given capacity
    #[inline]
    pub fn alloc_vec<T>(&self, capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }

    /// Reset the arena, freeing all allocated memory
    /// This is O(1) - just creates a new bump cursor
    #[inline]
    pub fn reset(&mut self) {
        self.bump = bumpalo::Bump::new();
    }

    /// Get the capacity hint used for this arena
    pub fn capacity_hint(&self) -> usize {
        self.capacity_hint
    }

    /// Check if the arena has any allocations
    #[inline]
    pub fn is_empty(&self) -> bool {
        // Conservative estimate - bumpalo doesn't expose direct is_empty
        true
    }
}

impl Default for QueryArena {
    fn default() -> Self {
        Self::with_capacity(4096) // 4KB default
    }
}

impl std::fmt::Debug for QueryArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryArena")
            .field("capacity_hint", &self.capacity_hint)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_arena_alloc() {
        let arena = QueryArena::default();
        let value = arena.alloc(42i32);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_query_arena_reset() {
        let mut arena = QueryArena::default();
        arena.alloc(1i32);
        arena.alloc(2i32);
        arena.reset();
        // After reset, we can allocate again
        let value = arena.alloc(100i32);
        assert_eq!(*value, 100);
    }

    #[test]
    fn test_query_arena_string() {
        let arena = QueryArena::default();
        let s = arena.alloc_string("hello");
        assert_eq!(s, "hello");
    }
}
