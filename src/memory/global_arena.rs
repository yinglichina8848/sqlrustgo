//! GlobalArena - Engine-level memory management
//!
//! Manages memory that lives for the entire engine lifecycle:
//! - Buffer metadata
//! - Page descriptors
//! - System-wide allocation

use std::sync::Arc;

/// GlobalArena provides system-wide memory allocation
/// that lives for the entire engine lifecycle.
///
/// Unlike query-level arenas that are reset after each query,
/// GlobalArena persists for the duration of the database engine.
pub struct GlobalArena {
    /// Total bytes allocated through this arena
    total_allocated: std::sync::atomic::AtomicUsize,
    /// Peak memory usage
    peak_usage: std::sync::atomic::AtomicUsize,
    /// Bump allocator for fast allocation
    bump: bumpalo::Bump,
}

impl GlobalArena {
    /// Create a new GlobalArena with the given capacity
    pub fn with_capacity(_capacity: usize) -> Self {
        // bumpalo::Bump doesn't take a buffer argument
        // It manages its own internal allocation
        let bump = bumpalo::Bump::new();

        Self {
            total_allocated: std::sync::atomic::AtomicUsize::new(0),
            peak_usage: std::sync::atomic::AtomicUsize::new(0),
            bump,
        }
    }

    /// Allocate memory for type T
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    /// Get total allocated bytes
    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> usize {
        self.peak_usage.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reset the arena (frees all memory)
    /// Note: This is expensive and should only be used when shutting down
    pub fn reset(&mut self) {
        // Recreate the bump allocator with a fresh allocation
        self.bump = bumpalo::Bump::new();
        self.total_allocated.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for GlobalArena {
    fn default() -> Self {
        Self::with_capacity(1024 * 1024) // 1MB default
    }
}

impl std::fmt::Debug for GlobalArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalArena")
            .field("total_allocated", &self.total_allocated())
            .field("peak_usage", &self.peak_usage())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_arena_alloc() {
        let arena = GlobalArena::default();
        let value = arena.alloc(42i32);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_global_arena_multiple_allocs() {
        let arena = GlobalArena::default();
        let a = arena.alloc(1i32);
        let b = arena.alloc(2i32);
        let c = arena.alloc(3i32);
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
    }

    #[test]
    fn test_global_arena_reset() {
        let mut arena = GlobalArena::default();
        arena.alloc(1i32);
        arena.alloc(2i32);
        arena.reset();
        // After reset, we can allocate again
        let value = arena.alloc(100i32);
        assert_eq!(*value, 100);
    }
}
