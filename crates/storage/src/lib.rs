// SQLRustGo storage module

pub mod backup;
pub mod binary_format;
pub mod binary_storage;
pub mod bplus_tree;
pub mod buffer_pool;
// pub mod columnar; // TODO: re-enable once StorageEngine trait is synced
// NOTE: ColumnarStorage has trait sync issues; tpch-import uses MemoryStorage + Parquet export

pub mod engine;
pub mod file_storage;
pub mod fts;
pub mod page;
pub mod predicate;
pub mod read_write_split;
pub mod row_format;
pub mod wal;

pub use binary_format::BinaryFormat;
pub use binary_storage::BinaryTableStorage;
pub use bplus_tree::BPlusTree;
pub use buffer_pool::BufferPool;
pub use engine::{
    evaluate_check_constraint, ColumnDefinition, ForeignKeyAction, ForeignKeyConstraint,
    MemoryStorage, Record, StorageEngine, TableData, TableInfo, TriggerEvent, TriggerInfo,
    TriggerTiming, UniqueConstraint,
};
pub use file_storage::FileStorage;
pub use fts::{
    ChineseTokenizer, InvertedIndex, MultiLanguageTokenizer, SimpleTokenizer, Tokenizer,
};
pub use page::Page;

// Re-export row_format types
pub use row_format::{
    ClusteredLeafRecord, ClusterKey, DefaultRowIdGenerator, OverflowPage, RowHeader,
    RowIdGenerator, VarLenSlot, encode_row, decode_row,
};
