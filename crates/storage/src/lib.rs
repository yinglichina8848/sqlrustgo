// SQLRustGo storage module

pub mod backup;
pub mod backup_storage;
pub mod binary_format;
pub mod binary_storage;
pub mod bplus_tree;
pub mod buffer_pool;
pub mod checkpoint;
pub mod page_access_tracker;
pub mod tier_manager;
// pub mod columnar; // TODO: re-enable once StorageEngine trait is synced
// NOTE: ColumnarStorage has trait sync issues; tpch-import uses MemoryStorage + Parquet export

pub mod clustered_index;
pub mod encrypted_file;
pub mod encryption;
pub mod engine;
pub mod file_storage;
pub mod fts;
pub mod key_manager;
pub mod page;
pub mod predicate;
pub mod read_write_split;
pub mod recovery;
pub mod row_format;
pub mod storage_tier;
pub mod wal;

pub use binary_format::BinaryFormat;
pub use binary_storage::BinaryTableStorage;
pub use bplus_tree::BPlusTree;
pub use buffer_pool::{BufferPool, EncryptedBufferPool};
pub use encrypted_file::EncryptedFileStorage;
pub use encryption::{AesEncryptionManager, Crypt, DecryptedPage, EncryptedPage, EncryptionError};
pub use engine::{
    evaluate_check_constraint, ColumnDefinition, ForeignKeyAction, ForeignKeyConstraint,
    MemoryStorage, Record, StorageEngine, TableData, TableInfo, TriggerEvent, TriggerInfo,
    TriggerTiming, UniqueConstraint,
};
pub use file_storage::FileStorage;
pub use fts::{
    ChineseTokenizer, InvertedIndex, MultiLanguageTokenizer, SimpleTokenizer, Tokenizer,
};
pub use key_manager::{BasicKeyManager, KeyManager};
pub use page::Page;
pub use page_access_tracker::PageAccessTracker;
pub use storage_tier::StorageTier;
pub use tier_manager::{TierManager, TierManagerConfig};

// Re-export row_format types
pub use row_format::{
    decode_row, encode_row, ClusterKey, ClusteredLeafRecord, DefaultRowIdGenerator, OverflowPage,
    RowHeader, RowIdGenerator, VarLenSlot,
};

// Re-export clustered_index types
pub use clustered_index::{
    ClusteredLeafIter, ClusteredLeafPage, ClusteredPageTransaction, ClusteredWalEntry,
    ClusteredWalManager, OverflowManager,
};

// Re-export recovery types
pub use recovery::{
    compute_crc32, PITRRegistry, PITRSnapshot, PageChecksum, PageChecksumStore, RecoveryManifest,
    RecoveryVerificationResult, RecoveryVerifier, WalChainEntry, WalChainState,
    WAL_GENESIS_PREV_HASH,
};
