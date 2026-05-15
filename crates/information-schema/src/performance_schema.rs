use sqlrustgo_observability::observability_state::OBSERVABILITY;
use std::sync::RwLock;

pub struct PerformanceSchema;

lazy_static::lazy_static! {
    static ref PS_STATE: RwLock<PsState> = RwLock::new(PsState::new());
}

struct PsState {
    setup_actors: Vec<SetupActorsRow>,
    setup_instruments: Vec<SetupInstrumentsRow>,
    events_statements_current: Vec<EventsStatementsRow>,
    events_statements_history: Vec<EventsStatementsRow>,
    events_waits_current: Vec<EventsWaitsRow>,
    events_waits_history: Vec<EventsWaitsRow>,
}

impl PsState {
    fn new() -> Self {
        let setup_actors = vec![SetupActorsRow {
            mid: 0,
            name: "%".to_string(),
            enabled: "YES".to_string(),
            history: "YES".to_string(),
            properties: "PROPERTIES".to_string(),
        }];

        let setup_instruments = vec![
            SetupInstrumentsRow {
                name: "statement/sql/select".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "PROPERTIES".to_string(),
            },
            SetupInstrumentsRow {
                name: "statement/sql/insert".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "PROPERTIES".to_string(),
            },
            SetupInstrumentsRow {
                name: "statement/sql/update".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "PROPERTIES".to_string(),
            },
            SetupInstrumentsRow {
                name: "statement/sql/delete".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "PROPERTIES".to_string(),
            },
            SetupInstrumentsRow {
                name: "wait/io/socket".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "PROPERTIES".to_string(),
            },
        ];

        Self {
            setup_actors,
            setup_instruments,
            events_statements_current: Vec::new(),
            events_statements_history: Vec::new(),
            events_waits_current: Vec::new(),
            events_waits_history: Vec::new(),
        }
    }
}

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

    pub fn get_setup_actors_rows() -> Vec<SetupActorsRow> {
        PS_STATE.read().unwrap().setup_actors.clone()
    }

    pub fn get_setup_instruments_rows() -> Vec<SetupInstrumentsRow> {
        PS_STATE.read().unwrap().setup_instruments.clone()
    }

    pub fn get_events_statements_current_rows() -> Vec<EventsStatementsRow> {
        PS_STATE.read().unwrap().events_statements_current.clone()
    }

    pub fn get_events_statements_history_rows() -> Vec<EventsStatementsRow> {
        PS_STATE.read().unwrap().events_statements_history.clone()
    }

    pub fn get_events_waits_current_rows() -> Vec<EventsWaitsRow> {
        PS_STATE.read().unwrap().events_waits_current.clone()
    }

    pub fn get_events_waits_history_rows() -> Vec<EventsWaitsRow> {
        PS_STATE.read().unwrap().events_waits_history.clone()
    }

    pub fn get_global_events_rows() -> Vec<GlobalEventsRow> {
        Vec::new()
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

#[derive(Clone)]
pub struct SetupActorsRow {
    pub mid: u32,
    pub name: String,
    pub enabled: String,
    pub history: String,
    pub properties: String,
}

#[derive(Clone)]
pub struct SetupInstrumentsRow {
    pub name: String,
    pub enabled: String,
    pub timed: String,
    pub properties: String,
}

#[derive(Clone)]
pub struct EventsStatementsRow {
    pub thread_id: u64,
    pub event_id: u64,
    pub event_name: String,
    pub sql_text: String,
    pub digest: Option<String>,
    pub digest_text: Option<String>,
    pub timer_start: u64,
    pub timer_end: u64,
    pub timer_wait: u64,
    pub lock_time: u64,
    pub rows_examined: u64,
    pub rows_sent: u64,
    pub rows_affected: u64,
    pub created_tmp_disk_tables: u64,
    pub created_tmp_tables: u64,
}

#[derive(Clone)]
pub struct EventsWaitsRow {
    pub thread_id: u64,
    pub event_id: u64,
    pub event_name: String,
    pub source: String,
    pub timer_start: u64,
    pub timer_end: u64,
    pub timer_wait: u64,
    pub operation: String,
    pub object_schema: Option<String>,
    pub object_name: Option<String>,
    pub object_instance_address: u64,
}

#[derive(Clone)]
pub struct EventsStatementsSummaryByDigestRow {
    pub schema_name: String,
    pub digest: String,
    pub digest_text: String,
    pub count_star: u64,
    pub sum_timer_wait: u64,
    pub min_timer_wait: u64,
    pub max_timer_wait: u64,
    pub avg_timer_wait: u64,
    pub sum_lock_time: u64,
    pub sum_rows_examined: u64,
    pub sum_rows_sent: u64,
    pub sum_rows_affected: u64,
    pub sum_created_tmp_disk_tables: u64,
    pub sum_created_tmp_tables: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub quantile_95: u64,
    pub quantile_99: u64,
    pub quantile_999: u64,
}

#[derive(Clone)]
pub struct GlobalEventsRow {
    pub event_name: String,
    pub count_star: u64,
    pub sum_timer_wait: u64,
    pub min_timer_wait: u64,
    pub max_timer_wait: u64,
    pub avg_timer_wait: u64,
    pub sum_lock_time: u64,
    pub count_read: u64,
    pub sum_bytes_received: u64,
    pub count_write: u64,
    pub sum_bytes_sent: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps_setup_actors() {
        let rows = PerformanceSchema::get_setup_actors_rows();
        assert!(!rows.is_empty());
        let row = &rows[0];
        assert_eq!(row.name, "%");
        assert_eq!(row.enabled, "YES");
        assert_eq!(row.history, "YES");
    }

    #[test]
    fn test_ps_setup_instruments() {
        let rows = PerformanceSchema::get_setup_instruments_rows();
        assert!(!rows.is_empty());
        assert!(rows.len() >= 5);
        let row = &rows[0];
        assert!(row.name.contains("statement/sql/select") || row.name.contains("statement/sql/insert") || row.name.contains("wait/io"));
        assert_eq!(row.enabled, "YES");
    }

    #[test]
    fn test_ps_events_statements() {
        let current = PerformanceSchema::get_events_statements_current_rows();
        let history = PerformanceSchema::get_events_statements_history_rows();
        assert!(current.is_empty());
        assert!(history.is_empty());
    }

    #[test]
    fn test_ps_events_waits() {
        let current = PerformanceSchema::get_events_waits_current_rows();
        let history = PerformanceSchema::get_events_waits_history_rows();
        assert!(current.is_empty());
        assert!(history.is_empty());
    }

    #[test]
    fn test_ps_global_events() {
        let rows = PerformanceSchema::get_global_events_rows();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_setup_actors_row_clone() {
        let row = SetupActorsRow {
            mid: 1,
            name: "test".to_string(),
            enabled: "YES".to_string(),
            history: "YES".to_string(),
            properties: "PROPERTIES".to_string(),
        };
        let cloned = row.clone();
        assert_eq!(cloned.mid, row.mid);
        assert_eq!(cloned.name, row.name);
    }

    #[test]
    fn test_setup_instruments_row_clone() {
        let row = SetupInstrumentsRow {
            name: "test".to_string(),
            enabled: "YES".to_string(),
            timed: "YES".to_string(),
            properties: "PROPERTIES".to_string(),
        };
        let cloned = row.clone();
        assert_eq!(cloned.name, row.name);
    }

    #[test]
    fn test_events_statements_row_clone() {
        let row = EventsStatementsRow {
            thread_id: 1,
            event_id: 1,
            event_name: "test".to_string(),
            sql_text: "SELECT 1".to_string(),
            digest: None,
            digest_text: None,
            timer_start: 0,
            timer_end: 100,
            timer_wait: 100,
            lock_time: 10,
            rows_examined: 0,
            rows_sent: 1,
            rows_affected: 0,
            created_tmp_disk_tables: 0,
            created_tmp_tables: 0,
        };
        let cloned = row.clone();
        assert_eq!(cloned.thread_id, row.thread_id);
        assert_eq!(cloned.sql_text, row.sql_text);
    }

    #[test]
    fn test_events_waits_row_clone() {
        let row = EventsWaitsRow {
            thread_id: 1,
            event_id: 1,
            event_name: "wait/io".to_string(),
            source: "test.rs".to_string(),
            timer_start: 0,
            timer_end: 100,
            timer_wait: 100,
            operation: "read".to_string(),
            object_schema: None,
            object_name: None,
            object_instance_address: 0,
        };
        let cloned = row.clone();
        assert_eq!(cloned.thread_id, row.thread_id);
        assert_eq!(cloned.operation, row.operation);
    }

    #[test]
    fn test_events_statements_summary_by_digest_row_clone() {
        let row = EventsStatementsSummaryByDigestRow {
            schema_name: "test".to_string(),
            digest: "abc123".to_string(),
            digest_text: "SELECT 1".to_string(),
            count_star: 10,
            sum_timer_wait: 1000,
            min_timer_wait: 50,
            max_timer_wait: 200,
            avg_timer_wait: 100,
            sum_lock_time: 100,
            sum_rows_examined: 100,
            sum_rows_sent: 10,
            sum_rows_affected: 0,
            sum_created_tmp_disk_tables: 0,
            sum_created_tmp_tables: 0,
            first_seen: 0,
            last_seen: 1000,
            quantile_95: 180,
            quantile_99: 195,
            quantile_999: 199,
        };
        let cloned = row.clone();
        assert_eq!(cloned.schema_name, row.schema_name);
        assert_eq!(cloned.count_star, row.count_star);
    }

    #[test]
    fn test_global_events_row_clone() {
        let row = GlobalEventsRow {
            event_name: "test".to_string(),
            count_star: 10,
            sum_timer_wait: 1000,
            min_timer_wait: 50,
            max_timer_wait: 200,
            avg_timer_wait: 100,
            sum_lock_time: 100,
            count_read: 5,
            sum_bytes_received: 1000,
            count_write: 5,
            sum_bytes_sent: 500,
        };
        let cloned = row.clone();
        assert_eq!(cloned.event_name, row.event_name);
        assert_eq!(cloned.count_star, row.count_star);
    }
}
