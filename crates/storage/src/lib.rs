// SQLRustGo Storage Module

pub mod bplus_tree;
pub mod buffer_pool;
pub mod engine;
pub mod page;
pub mod file_storage;
pub mod binary_format;

pub use bplus_tree::BPlusTree;
pub use buffer_pool::BufferPool;
pub use engine::{MemoryStorage, StorageEngine};
pub use page::Page;
pub use file_storage::FileStorage;
pub use binary_format::BinaryFormat;

// Re-export types for convenience
pub use engine::{Record, TableInfo, ColumnDefinition, TableData};
