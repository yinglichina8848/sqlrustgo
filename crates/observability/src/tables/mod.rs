pub mod events_statements;
pub mod lock_wait_graph;
pub mod recovery_history;
pub mod transaction_history;
pub mod wal_stats;

use events_statements::EventsStatementsCollector;
use lock_wait_graph::LockWaitGraph;
use recovery_history::RecoveryHistory;
use std::sync::RwLock;
use transaction_history::TransactionHistory;
use wal_stats::WalStatsCollector;

pub struct ObservabilityState {
    pub transaction_history: RwLock<TransactionHistory>,
    pub lock_wait_graph: RwLock<LockWaitGraph>,
    pub recovery_history: RwLock<RecoveryHistory>,
    pub wal_stats: RwLock<WalStatsCollector>,
    pub events_statements: RwLock<EventsStatementsCollector>,
}

impl ObservabilityState {
    pub fn new() -> Self {
        Self {
            transaction_history: RwLock::new(TransactionHistory::new(10000)),
            lock_wait_graph: RwLock::new(LockWaitGraph::new()),
            recovery_history: RwLock::new(RecoveryHistory::new(std::env::temp_dir(), 10000)),
            wal_stats: RwLock::new(WalStatsCollector::new()),
            events_statements: RwLock::new(EventsStatementsCollector::new()),
        }
    }
}

impl Default for ObservabilityState {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    pub static ref OBSERVABILITY: ObservabilityState = ObservabilityState::new();
}
