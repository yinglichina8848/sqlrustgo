//! Global observability state
//!
//! This module provides the global OBSERVABILITY state that aggregates
//! all observability data (transaction history, lock waits, WAL stats, etc.)
//! and makes it accessible to both the execution engine and information_schema.

use crate::tables::{
    lock_wait_graph::LockWaitGraph, recovery_history::RecoveryHistory,
    transaction_history::TransactionHistory, wal_stats::WalStatsCollector,
};
use std::sync::RwLock;

lazy_static::lazy_static! {
    /// Global observability state
    pub static ref OBSERVABILITY: ObservabilityState = ObservabilityState::new();
}

/// Thread-safe observability state that holds all observability data
pub struct ObservabilityState {
    pub transaction_history: RwLock<TransactionHistory>,
    pub lock_wait_graph: RwLock<LockWaitGraph>,
    pub recovery_history: RwLock<RecoveryHistory>,
    pub wal_stats: RwLock<WalStatsCollector>,
}

impl ObservabilityState {
    /// Create a new ObservabilityState with default configurations
    fn new() -> Self {
        Self {
            transaction_history: RwLock::new(TransactionHistory::new(10000)),
            lock_wait_graph: RwLock::new(LockWaitGraph::new()),
            recovery_history: RwLock::new(RecoveryHistory::new(std::env::temp_dir(), 10000)),
            wal_stats: RwLock::new(WalStatsCollector::new()),
        }
    }
}

impl Default for ObservabilityState {
    fn default() -> Self {
        Self::new()
    }
}
