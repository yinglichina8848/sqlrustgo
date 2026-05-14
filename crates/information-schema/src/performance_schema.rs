use sqlrustgo_observability::observability_state::OBSERVABILITY;

pub struct PerformanceSchema;

impl PerformanceSchema {
    pub fn get_transaction_history_rows(limit: Option<usize>) -> Vec<TransactionHistoryRow> {
        let history = OBSERVABILITY.transaction_history.read().unwrap();
        history
            .query(limit)
            .iter()
            .map(|e| TransactionHistoryRow {
                transaction_id: e.tx_id,
                transaction_uuid: e.tx_uuid.clone(),
                start_time: e.start_time,
                commit_time: e.commit_time,
                abort_time: e.abort_time,
                isolation_level: e.isolation_level.clone(),
                status: format!("{:?}", e.status),
            })
            .collect()
    }

    pub fn get_lock_wait_rows() -> Vec<LockWaitRow> {
        let graph = OBSERVABILITY.lock_wait_graph.read().unwrap();
        graph
            .get_active_waits()
            .iter()
            .map(|e| LockWaitRow {
                waiter_tx_id: e.waiter_tx_id,
                holder_tx_id: e.holder_tx_id,
                lock_key: e.lock_key.clone(),
                lock_mode: e.lock_mode.clone(),
                wait_start_time: e.wait_start_time,
            })
            .collect()
    }

    pub fn get_recovery_history_rows(limit: Option<usize>) -> Vec<RecoveryHistoryRow> {
        let history = OBSERVABILITY.recovery_history.read().unwrap();
        history
            .query(limit)
            .iter()
            .map(|e| RecoveryHistoryRow {
                recovery_id: e.recovery_id,
                crash_timestamp: e.crash_timestamp,
                recovery_timestamp: e.recovery_timestamp,
                lsn_recovered: e.lsn_recovered,
                transactions_replayed: e.transactions_replayed,
                status: format!("{:?}", e.status),
                error_message: e.error_message.clone(),
            })
            .collect()
    }

    pub fn get_wal_stats() -> WalStatsRow {
        let collector = OBSERVABILITY.wal_stats.read().unwrap();
        let stats = collector.get_stats();
        WalStatsRow {
            total_writes: stats.total_writes,
            total_bytes: stats.total_bytes,
            flush_count: stats.flush_count,
            replay_count: stats.replay_count,
            replay_time_ms: stats.replay_time_ms,
            last_flush_lsn: stats.last_flush_lsn,
            current_lsn: stats.current_lsn,
        }
    }

    pub fn detect_deadlocks() -> Vec<Vec<u64>> {
        let graph = OBSERVABILITY.lock_wait_graph.read().unwrap();
        graph.detect_deadlock()
    }
}

pub struct TransactionHistoryRow {
    pub transaction_id: u64,
    pub transaction_uuid: String,
    pub start_time: u64,
    pub commit_time: Option<u64>,
    pub abort_time: Option<u64>,
    pub isolation_level: String,
    pub status: String,
}

pub struct LockWaitRow {
    pub waiter_tx_id: u64,
    pub holder_tx_id: u64,
    pub lock_key: String,
    pub lock_mode: String,
    pub wait_start_time: u64,
}

pub struct RecoveryHistoryRow {
    pub recovery_id: u64,
    pub crash_timestamp: u64,
    pub recovery_timestamp: u64,
    pub lsn_recovered: u64,
    pub transactions_replayed: u64,
    pub status: String,
    pub error_message: Option<String>,
}

pub struct WalStatsRow {
    pub total_writes: u64,
    pub total_bytes: u64,
    pub flush_count: u64,
    pub replay_count: u64,
    pub replay_time_ms: u64,
    pub last_flush_lsn: u64,
    pub current_lsn: u64,
}
