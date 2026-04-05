// SQLRustGo storage module

pub mod backup;
pub mod backup_scheduler;
pub mod backup_storage;
pub mod binary_format;
pub mod binary_storage;
pub mod bplus_tree;
pub mod buffer_pool;
pub mod buffer_pool_metrics;
pub mod checkpoint;
pub mod clock_replacer;
pub mod columnar;
pub mod engine;
pub mod failover_manager;
pub mod file_storage;
pub mod heap;
pub mod page;
pub mod page_guard;
pub mod parquet;
pub mod pitr_recovery;
pub mod read_write_split;
pub mod replication;
pub mod stats;
pub mod wal;

pub use backup::{BackupExporter, BackupFormat, DataRestorer};
pub use backup_scheduler::{
    BackupCompressor, BackupSchedule, BackupScheduleType, BackupScheduler, IncrementalBackupPoint,
    WalBackupManager,
};
pub use backup_storage::{
    BackupStorageManager, BackupTransfer, LocalBackupStorage, RemoteBackupStorage, RemoteConfig,
    StorageBackend,
};

pub use binary_format::BinaryFormat;
pub use bplus_tree::BPlusTree;
pub use buffer_pool::BufferPool;
pub use buffer_pool_metrics::BufferPoolMetrics;
pub use clock_replacer::ClockReplacer;

pub use columnar::{
    Bitmap, ColumnChunk, ColumnSegment, ColumnStats, ColumnStatsDisk, ColumnarStorage,
    CompressionType, TableStore,
};

pub use engine::{
    ColumnDefinition, ForeignKeyAction, ForeignKeyConstraint, MemoryStorage, Record, StorageEngine,
    TableData, TableInfo, TriggerEvent, TriggerInfo, TriggerTiming, ViewInfo,
};
pub use failover_manager::{FailoverConfig, FailoverManager, FailoverState, NodeInfo, NodeType};
pub use file_storage::FileStorage;
pub use heap::{HeapStorage, RowId};
pub use page::Page;
pub use page_guard::PageGuard;
pub use parquet::{export_to_parquet, import_from_parquet};
pub use pitr_recovery::{
    PITRRecovery, PartialRecovery, PartialRecoveryResult, RecoveryHistory, RecoveryPlan,
    RecoveryPoint, RecoveryRecord, RecoveryResult, RecoveryStatus, RecoveryTarget,
    RecoveryValidator, ValidationResult,
};
pub use read_write_split::{
    Connection, ConnectionPool, ConsistencyMode, LoadBalanceStrategy, NodeRole, QueryType,
    ReadAfterWriteConsistency, ReadWriteRouter, ReadWriteSplitConfig, ReplicaNode, RouteResult,
};
pub use replication::{
    BinlogEvent, BinlogEventType, BinlogReader, BinlogWriter, MasterNode, ReplicationConfig,
    SlaveNode,
};
pub use stats::{ColumnStats as TableColumnStats, StatsManager, TableStats};
pub use checkpoint::{CheckpointConfig, CheckpointManager, CheckpointMetadata};
pub use wal::{WalEntry, WalEntryType, WalManager, WalReader, WalWriter};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        let _ = BPlusTree::new();
        let _ = BufferPool::new(1024);
        let _ = MemoryStorage::new();
        let _ = FileStorage::new(std::env::temp_dir());
    }

    #[test]
    fn test_bplus_tree_creation() {
        let tree = BPlusTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_buffer_pool_creation() {
        let pool = BufferPool::new(4096);
        assert_eq!(pool.capacity(), 4096);
    }

    #[test]
    fn test_memory_storage_creation() {
        let storage = MemoryStorage::new();
        let tables = storage.list_tables();
        assert!(tables.is_empty());
    }
}
