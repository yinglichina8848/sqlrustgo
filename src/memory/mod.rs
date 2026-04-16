//! Unified Memory Architecture for SQLRustGo
//!
//! This module provides a三层Arena design that supports both:
//! - Row-based execution pipeline
//! - Columnar execution pipeline
//!
//! ## Architecture
//!
//! ```text
//! GlobalArena (engine lifecycle)
//!   └── buffer metadata, page descriptors
//!
//! QueryArena (per-query lifecycle)
//!   └── expression temps, row temps, sort buffers
//!
//! BatchArena (per-batch lifecycle)
//!   └── selection vectors, dictionary vectors, aggregate states
//! ```

pub mod batch_arena;
pub mod global_arena;
pub mod memory_context;
pub mod query_arena;
pub mod reusable_vec;

pub use batch_arena::BatchArena;
pub use global_arena::GlobalArena;
pub use memory_context::{ArenaAlloc, MemoryContext};
pub use query_arena::QueryArena;
pub use reusable_vec::ReusableVec;
