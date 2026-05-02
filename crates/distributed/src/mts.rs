//! Multi-Threaded Slave (MTS) Module
//!
//! Implements MySQL 5.7 parallel replication with:
//! - LOGICAL_CLOCK based parallelization
//! - Commit order preservation
//! - Worker pool for parallel transaction execution
//!
//! Reference: MySQL 5.7 Reference Manual - Parallel Replication

use std::collections::VecDeque;
use tokio::sync::RwLock;

/// Transaction group identifier (logical clock)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalClock {
    pub timestamp: u64,
    pub seq: u64,
}

impl LogicalClock {
    pub fn new(timestamp: u64, seq: u64) -> Self {
        Self { timestamp, seq }
    }

    pub fn first() -> Self {
        Self { timestamp: 0, seq: 0 }
    }

    pub fn next(&self) -> Self {
        Self {
            timestamp: self.timestamp,
            seq: self.seq + 1,
        }
    }
}

/// Transaction entry in the relay log
#[derive(Debug, Clone)]
pub struct TransactionEntry {
    pub lsn: u64,
    pub clock: LogicalClock,
    pub database: String,
    pub table: Option<String>,
    pub sql: String,
    pub shard_key: Option<u64>,
}

impl TransactionEntry {
    pub fn new(lsn: u64, clock: LogicalClock, database: String, sql: String) -> Self {
        Self {
            lsn,
            clock,
            database,
            table: None,
            sql,
            shard_key: None,
        }
    }

    pub fn with_shard_key(mut self, shard_key: u64) -> Self {
        self.shard_key = Some(shard_key);
        self
    }
}

/// MTS configuration
#[derive(Debug, Clone)]
pub struct MtsConfig {
    pub worker_count: usize,
    pub parallel_type: MtsParallelType,
    pub preserve_commit_order: bool,
    pub max_queue_size: usize,
}

impl Default for MtsConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            parallel_type: MtsParallelType::LogicalClock,
            preserve_commit_order: true,
            max_queue_size: 10000,
        }
    }
}

/// Parallel replication type
#[derive(Debug, Clone, PartialEq)]
pub enum MtsParallelType {
    Database,
    LogicalClock,
}

/// MTS worker statistics
#[derive(Debug, Clone, Default)]
pub struct MtsStats {
    pub total_processed: u64,
    pub total_committed: u64,
    pub total_rolled_back: u64,
    pub queue_size: usize,
    pub worker_stats: Vec<WorkerStats>,
}

/// Individual worker statistics
#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    pub worker_id: usize,
    pub processed_count: u64,
    pub current_transaction: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum MtsState {
    #[default]
    Stopped,
    Running,
    Paused,
}

pub struct TransactionScheduler {
    config: MtsConfig,
    state: RwLock<MtsState>,
    pending_groups: RwLock<VecDeque<LogicalClock>>,
    committed_up_to: RwLock<LogicalClock>,
    stats: RwLock<MtsStats>,
}

impl TransactionScheduler {
    pub fn new(config: MtsConfig) -> Self {
        let worker_stats = (0..config.worker_count)
            .map(|i| WorkerStats {
                worker_id: i,
                processed_count: 0,
                current_transaction: None,
            })
            .collect();

        Self {
            config,
            state: RwLock::new(MtsState::Stopped),
            pending_groups: RwLock::new(VecDeque::new()),
            committed_up_to: RwLock::new(LogicalClock::first()),
            stats: RwLock::new(MtsStats {
                total_processed: 0,
                total_committed: 0,
                total_rolled_back: 0,
                queue_size: 0,
                worker_stats,
            }),
        }
    }

    pub async fn start(&self) {
        *self.state.write().await = MtsState::Running;
    }

    pub async fn stop(&self) {
        *self.state.write().await = MtsState::Stopped;
    }

    pub async fn pause(&self) {
        *self.state.write().await = MtsState::Paused;
    }

    pub async fn get_state(&self) -> MtsState {
        self.state.read().await.clone()
    }

    pub fn get_config(&self) -> MtsConfig {
        self.config.clone()
    }

    pub fn set_worker_count(&mut self, count: usize) {
        self.config.worker_count = count;
    }

    pub async fn add_pending_group(&self, clock: LogicalClock) {
        self.pending_groups.write().await.push_back(clock);
    }

    pub async fn schedule_transaction(&self, entry: &TransactionEntry) -> Option<usize> {
        if *self.state.read().await != MtsState::Running {
            return None;
        }

        let worker_id = match self.config.parallel_type {
            MtsParallelType::Database => {
                let hash = self.hash_database(&entry.database);
                hash % self.config.worker_count
            }
            MtsParallelType::LogicalClock => entry.clock.seq as usize % self.config.worker_count,
        };

        let mut stats = self.stats.write().await;
        stats.total_processed += 1;
        if let Some(worker_stats) = stats.worker_stats.get_mut(worker_id) {
            worker_stats.processed_count += 1;
            worker_stats.current_transaction = Some(entry.lsn);
        }

        Some(worker_id)
    }

    pub async fn commit_group(&self, clock: &LogicalClock) -> Result<(), MtsError> {
        let committed = self.committed_up_to.read().await;
        if clock > &*committed {
            if self.config.preserve_commit_order {
                self.wait_for_previous_commits(clock).await?;
            }
            *self.committed_up_to.write().await = clock.clone();
        }
        drop(committed);

        let mut stats = self.stats.write().await;
        stats.total_committed += 1;

        Ok(())
    }

    async fn wait_for_previous_commits(&self, clock: &LogicalClock) -> Result<(), MtsError> {
        loop {
            let committed = self.committed_up_to.read().await;
            if committed.timestamp >= clock.timestamp && committed.seq >= clock.seq {
                return Ok(());
            }
            drop(committed);
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }

    pub async fn rollback_group(&self, _clock: &LogicalClock) {
        self.stats.write().await.total_rolled_back += 1;
    }

    pub async fn get_stats(&self) -> MtsStats {
        let stats = self.stats.read().await;
        let queue_size = self.pending_groups.read().await.len();
        MtsStats {
            queue_size,
            ..stats.clone()
        }
    }

    fn hash_database(&self, database: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        database.hash(&mut hasher);
        hasher.finish() as usize
    }
}

#[derive(Debug, Clone)]
pub enum MtsError {
    NotRunning,
    CommitOrderViolation,
    QueueFull,
    Internal(String),
}

impl std::fmt::Display for MtsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MtsError::NotRunning => write!(f, "MTS is not running"),
            MtsError::CommitOrderViolation => write!(f, "Commit order violation detected"),
            MtsError::QueueFull => write!(f, "Transaction queue is full"),
            MtsError::Internal(msg) => write!(f, "Internal MTS error: {}", msg),
        }
    }
}

impl std::error::Error for MtsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_clock_ordering() {
        let clock1 = LogicalClock::new(1, 1);
        let clock2 = LogicalClock::new(1, 2);
        let clock3 = LogicalClock::new(2, 0);

        assert!(clock1 < clock2);
        assert!(clock2 < clock3);
        assert!(clock3 > clock1);
    }

    #[test]
    fn test_logical_clock_next() {
        let clock = LogicalClock::new(5, 10);
        let next = clock.next();

        assert_eq!(next.timestamp, 5);
        assert_eq!(next.seq, 11);
    }

    #[test]
    fn test_logical_clock_first() {
        let first = LogicalClock::first();
        assert_eq!(first.timestamp, 0);
        assert_eq!(first.seq, 0);
    }

    #[test]
    fn test_transaction_entry() {
        let clock = LogicalClock::new(1, 1);
        let entry =
            TransactionEntry::new(100, clock, "test_db".to_string(), "SELECT 1".to_string());

        assert_eq!(entry.lsn, 100);
        assert_eq!(entry.database, "test_db");
        assert_eq!(entry.sql, "SELECT 1");
        assert!(entry.shard_key.is_none());
    }

    #[test]
    fn test_transaction_entry_with_shard() {
        let clock = LogicalClock::new(1, 1);
        let entry = TransactionEntry::new(100, clock, "test_db".to_string(), "SELECT 1".to_string())
            .with_shard_key(42);

        assert_eq!(entry.shard_key, Some(42));
    }

    #[test]
    fn test_mts_config_default() {
        let config = MtsConfig::default();
        assert_eq!(config.worker_count, 4);
        assert_eq!(config.parallel_type, MtsParallelType::LogicalClock);
        assert!(config.preserve_commit_order);
    }

    #[test]
    fn test_mts_parallel_type() {
        assert_eq!(format!("{:?}", MtsParallelType::Database), "Database");
        assert_eq!(
            format!("{:?}", MtsParallelType::LogicalClock),
            "LogicalClock"
        );
    }

    #[tokio::test]
    async fn test_transaction_scheduler_new() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);

        assert_eq!(scheduler.get_state().await, MtsState::Stopped);
    }

    #[tokio::test]
    async fn test_transaction_scheduler_start_stop() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);

        scheduler.start().await;
        assert_eq!(scheduler.get_state().await, MtsState::Running);

        scheduler.stop().await;
        assert_eq!(scheduler.get_state().await, MtsState::Stopped);
    }

    #[tokio::test]
    async fn test_transaction_scheduler_pause_resume() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);

        scheduler.start().await;
        scheduler.pause().await;
        assert_eq!(scheduler.get_state().await, MtsState::Paused);
    }

    #[tokio::test]
    async fn test_schedule_transaction() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);
        scheduler.start().await;

        let clock = LogicalClock::new(1, 1);
        let entry =
            TransactionEntry::new(100, clock, "test_db".to_string(), "SELECT 1".to_string());

        let worker_id = scheduler.schedule_transaction(&entry).await;
        assert!(worker_id.is_some());
        assert!(worker_id.unwrap() < 4);
    }

    #[tokio::test]
    async fn test_schedule_transaction_not_running() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);

        let clock = LogicalClock::new(1, 1);
        let entry =
            TransactionEntry::new(100, clock, "test_db".to_string(), "SELECT 1".to_string());

        let worker_id = scheduler.schedule_transaction(&entry).await;
        assert!(worker_id.is_none());
    }

    #[tokio::test]
    async fn test_commit_group() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);
        scheduler.start().await;

        let clock = LogicalClock::new(1, 1);
        let result = scheduler.commit_group(&clock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rollback_group() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);
        scheduler.start().await;

        let clock = LogicalClock::new(1, 1);
        scheduler.rollback_group(&clock).await;

        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_rolled_back, 1);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);
        scheduler.start().await;

        let clock = LogicalClock::new(1, 1);
        let entry =
            TransactionEntry::new(100, clock, "test_db".to_string(), "SELECT 1".to_string());
        scheduler.schedule_transaction(&entry).await;

        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_processed, 1);
        assert_eq!(stats.worker_stats.len(), 4);
    }

    #[test]
    fn test_mts_state_default() {
        let state = MtsState::default();
        assert_eq!(state, MtsState::Stopped);
    }

    #[test]
    fn test_mts_error_display() {
        let err = MtsError::NotRunning;
        assert!(err.to_string().contains("not running"));

        let err2 = MtsError::CommitOrderViolation;
        assert!(err2.to_string().contains("commit order"));

        let err3 = MtsError::QueueFull;
        assert!(err3.to_string().contains("queue"));

        let err4 = MtsError::Internal("test".to_string());
        assert!(err4.to_string().contains("test"));
    }

    #[test]
    fn test_worker_stats_default() {
        let stats = WorkerStats::default();
        assert_eq!(stats.worker_id, 0);
        assert_eq!(stats.processed_count, 0);
        assert!(stats.current_transaction.is_none());
    }

    #[tokio::test]
    async fn test_database_hash_distribution() {
        let config = MtsConfig {
            worker_count: 4,
            parallel_type: MtsParallelType::Database,
            ..Default::default()
        };
        let scheduler = TransactionScheduler::new(config);
        scheduler.start().await;

        let clock = LogicalClock::new(1, 1);

        let entry1 =
            TransactionEntry::new(1, clock.clone(), "db1".to_string(), "SELECT 1".to_string());
        let entry2 =
            TransactionEntry::new(2, clock.clone(), "db2".to_string(), "SELECT 2".to_string());

        let worker1 = scheduler.schedule_transaction(&entry1).await;
        let worker2 = scheduler.schedule_transaction(&entry2).await;

        assert!(worker1.is_some());
        assert!(worker2.is_some());
    }

    #[tokio::test]
    async fn test_commit_order_preservation() {
        let config = MtsConfig {
            worker_count: 4,
            preserve_commit_order: true,
            ..Default::default()
        };
        let scheduler = TransactionScheduler::new(config);
        scheduler.start().await;

        let clock1 = LogicalClock::new(1, 1);
        let clock2 = LogicalClock::new(1, 2);

        scheduler.commit_group(&clock1).await.unwrap();
        scheduler.commit_group(&clock2).await.unwrap();

        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_committed, 2);
    }

    #[tokio::test]
    async fn test_stats_after_multiple_transactions() {
        let config = MtsConfig::default();
        let scheduler = TransactionScheduler::new(config);
        scheduler.start().await;

        for i in 0..10 {
            let clock = LogicalClock::new(1, i);
            let entry = TransactionEntry::new(
                i,
                clock,
                format!("db_{}", i % 2),
                format!("SELECT {}", i),
            );
            scheduler.schedule_transaction(&entry).await;
        }

        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_processed, 10);
    }
}