// adapters/mod.rs
pub mod storage;
pub mod vector;
pub mod graph;

pub use storage::StorageAdapter;
pub use vector::VectorAdapter;
pub use graph::GraphAdapter;
