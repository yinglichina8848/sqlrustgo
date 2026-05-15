use sqlrustgo_observability::observability_state::OBSERVABILITY;
use std::sync::RwLock;

lazy_static::lazy_static! {
    pub static ref PERF_SCHEMA: PerformanceSchema = PerformanceSchema::new();
}

pub struct RingBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    head: usize,
    count: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            head: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(item);
        } else {
            self.buffer[self.head] = item;
            self.head = (self.head + 1) % self.capacity;
        }
        self.count = self.count.saturating_add(1);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

pub struct PerformanceSchema {
    setup_actors: RwLock<Vec<SetupActorsRow>>,
    setup_instruments: RwLock<Vec<SetupInstrumentsRow>>,
    events_statements_history: RwLock<RingBuffer<EventsStatementsHistoryRow>>,
    events_waits_history: RwLock<RingBuffer<EventsWaitsHistoryRow>>,
    events_statements_summary: RwLock<Vec<EventsStatementsSummaryByDigestRow>>,
}

impl Default for PerformanceSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct StatementStats {
    pub digest: String,
    pub digest_text: String,
    pub timer_wait: u64,
    pub lock_time: u64,
    pub rows_affected: u64,
    pub rows_sent: u64,
    pub rows_examined: u64,
    pub errors: u64,
}

impl PerformanceSchema {
    pub fn new() -> Self {
        let default_instruments = vec![
            SetupInstrumentsRow {
                name: "statement/sql/select".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "STATEMENT".to_string(),
                volatility: 0,
            },
            SetupInstrumentsRow {
                name: "statement/sql/insert".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "STATEMENT".to_string(),
                volatility: 0,
            },
            SetupInstrumentsRow {
                name: "statement/sql/update".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "STATEMENT".to_string(),
                volatility: 0,
            },
            SetupInstrumentsRow {
                name: "statement/sql/delete".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "STATEMENT".to_string(),
                volatility: 0,
            },
            SetupInstrumentsRow {
                name: "statement/sql/create_table".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "STATEMENT".to_string(),
                volatility: 0,
            },
            SetupInstrumentsRow {
                name: "statement/sql/drop_table".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "STATEMENT".to_string(),
                volatility: 0,
            },
            SetupInstrumentsRow {
                name: "wait/io/socket".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "WAIT".to_string(),
                volatility: 1,
            },
            SetupInstrumentsRow {
                name: "wait/lock/table".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "WAIT".to_string(),
                volatility: 1,
            },
            SetupInstrumentsRow {
                name: "wait/sync/cond".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "WAIT".to_string(),
                volatility: 2,
            },
            SetupInstrumentsRow {
                name: "stage/sql/executing".to_string(),
                enabled: "YES".to_string(),
                timed: "YES".to_string(),
                properties: "STAGE".to_string(),
                volatility: 1,
            },
        ];

        Self {
            setup_actors: RwLock::new(vec![SetupActorsRow {
                trigger_id: "%".to_string(),
                flags: "YES".to_string(),
                enabled: "YES".to_string(),
                history: "YES".to_string(),
                properties: "DEFAULT".to_string(),
            }]),
            setup_instruments: RwLock::new(default_instruments),
            events_statements_history: RwLock::new(RingBuffer::new(1024)),
            events_waits_history: RwLock::new(RingBuffer::new(1024)),
            events_statements_summary: RwLock::new(Vec::new()),
        }
    }

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

    pub fn get_setup_actors() -> Vec<SetupActorsRow> {
        PERF_SCHEMA.setup_actors.read().unwrap().clone()
    }

    pub fn get_setup_instruments() -> Vec<SetupInstrumentsRow> {
        PERF_SCHEMA.setup_instruments.read().unwrap().clone()
    }

    pub fn update_setup_actors(&self, row: SetupActorsRow) {
        let mut actors = self.setup_actors.write().unwrap();
        if let Some(existing) = actors.iter_mut().find(|a| a.trigger_id == row.trigger_id) {
            *existing = row;
        } else {
            actors.push(row);
        }
    }

    pub fn update_setup_instruments(&self, name: &str, enabled: &str, timed: &str) {
        let mut instruments = self.setup_instruments.write().unwrap();
        if let Some(existing) = instruments.iter_mut().find(|i| i.name == name) {
            existing.enabled = enabled.to_string();
            existing.timed = timed.to_string();
        }
    }

    pub fn get_events_statements_current() -> Vec<EventsStatementsCurrentRow> {
        vec![]
    }

    pub fn get_events_statements_history(_limit: Option<usize>) -> Vec<EventsStatementsHistoryRow> {
        PERF_SCHEMA
            .events_statements_history
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    pub fn get_events_statements_summary_by_digest(
        _limit: Option<usize>,
    ) -> Vec<EventsStatementsSummaryByDigestRow> {
        PERF_SCHEMA
            .events_statements_summary
            .read()
            .unwrap()
            .clone()
    }

    pub fn record_statement(row: EventsStatementsHistoryRow) {
        let mut history = PERF_SCHEMA.events_statements_history.write().unwrap();
        history.push(row);
    }

    pub fn update_statement_summary(stats: StatementStats) {
        let mut summary = PERF_SCHEMA.events_statements_summary.write().unwrap();
        if let Some(existing) = summary.iter_mut().find(|s| s.digest == stats.digest) {
            existing.count_star += 1;
            existing.sum_timer_wait += stats.timer_wait;
            existing.sum_lock_time += stats.lock_time;
            existing.sum_rows_affected += stats.rows_affected;
            existing.sum_rows_sent += stats.rows_sent;
            existing.sum_rows_examined += stats.rows_examined;
            existing.sum_errors += stats.errors;
            if stats.timer_wait < existing.min_timer_wait {
                existing.min_timer_wait = stats.timer_wait;
            }
            if stats.timer_wait > existing.max_timer_wait {
                existing.max_timer_wait = stats.timer_wait;
            }
            existing.avg_timer_wait = existing.sum_timer_wait / existing.count_star;
        } else {
            summary.push(EventsStatementsSummaryByDigestRow {
                schema_name: "def".to_string(),
                digest: stats.digest,
                digest_text: stats.digest_text,
                count_star: 1,
                sum_timer_wait: stats.timer_wait,
                min_timer_wait: stats.timer_wait,
                avg_timer_wait: stats.timer_wait,
                max_timer_wait: stats.timer_wait,
                sum_lock_time: stats.lock_time,
                sum_errors: stats.errors,
                sum_rows_affected: stats.rows_affected,
                sum_rows_sent: stats.rows_sent,
                sum_rows_examined: stats.rows_examined,
            });
        }
    }

    pub fn get_global_events() -> Vec<GlobalEventsRow> {
        vec![]
    }

    pub fn get_events_waits_current() -> Vec<EventsWaitsCurrentRow> {
        vec![]
    }

    pub fn get_events_waits_history(_limit: Option<usize>) -> Vec<EventsWaitsHistoryRow> {
        PERF_SCHEMA
            .events_waits_history
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    pub fn record_wait(row: EventsWaitsHistoryRow) {
        let mut history = PERF_SCHEMA.events_waits_history.write().unwrap();
        history.push(row);
    }
}

#[derive(Clone, Debug)]
pub struct TransactionHistoryRow {
    pub transaction_id: u64,
    pub transaction_uuid: String,
    pub start_time: u64,
    pub commit_time: Option<u64>,
    pub abort_time: Option<u64>,
    pub isolation_level: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct LockWaitRow {
    pub waiter_tx_id: u64,
    pub holder_tx_id: u64,
    pub lock_key: String,
    pub lock_mode: String,
    pub wait_start_time: u64,
}

#[derive(Clone, Debug)]
pub struct RecoveryHistoryRow {
    pub recovery_id: u64,
    pub crash_timestamp: u64,
    pub recovery_timestamp: u64,
    pub lsn_recovered: u64,
    pub transactions_replayed: u64,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct WalStatsRow {
    pub total_writes: u64,
    pub total_bytes: u64,
    pub flush_count: u64,
    pub replay_count: u64,
    pub replay_time_ms: u64,
    pub last_flush_lsn: u64,
    pub current_lsn: u64,
}

#[derive(Clone, Debug)]
pub struct SetupActorsRow {
    pub trigger_id: String,
    pub flags: String,
    pub enabled: String,
    pub history: String,
    pub properties: String,
}

#[derive(Clone, Debug)]
pub struct SetupInstrumentsRow {
    pub name: String,
    pub enabled: String,
    pub timed: String,
    pub properties: String,
    pub volatility: i32,
}

#[derive(Clone, Debug)]
pub struct EventsStatementsSummaryByDigestRow {
    pub schema_name: String,
    pub digest: String,
    pub digest_text: String,
    pub count_star: u64,
    pub sum_timer_wait: u64,
    pub min_timer_wait: u64,
    pub avg_timer_wait: u64,
    pub max_timer_wait: u64,
    pub sum_lock_time: u64,
    pub sum_errors: u64,
    pub sum_rows_affected: u64,
    pub sum_rows_sent: u64,
    pub sum_rows_examined: u64,
}

#[derive(Clone, Debug)]
pub struct EventsStatementsCurrentRow {
    pub thread_id: u64,
    pub event_id: u64,
    pub event_name: String,
    pub source: String,
    pub timer_start: u64,
    pub timer_end: u64,
    pub timer_wait: u64,
    pub lock_time: u64,
    pub sql_text: String,
    pub digest: String,
    pub digest_text: String,
}

#[derive(Clone, Debug)]
pub struct EventsStatementsHistoryRow {
    pub thread_id: u64,
    pub event_id: u64,
    pub event_name: String,
    pub source: String,
    pub timer_start: u64,
    pub timer_end: u64,
    pub timer_wait: u64,
    pub lock_time: u64,
    pub sql_text: String,
    pub digest: String,
    pub digest_text: String,
}

#[derive(Clone, Debug)]
pub struct GlobalEventsRow {
    pub event_name: String,
    pub count_star: u64,
    pub sum_timer_wait: u64,
    pub min_timer_wait: u64,
    pub avg_timer_wait: u64,
    pub max_timer_wait: u64,
}

#[derive(Clone, Debug)]
pub struct EventsWaitsCurrentRow {
    pub thread_id: u64,
    pub event_id: u64,
    pub event_name: String,
    pub source: String,
    pub timer_start: u64,
    pub timer_end: u64,
    pub timer_wait: u64,
    pub object_schema: String,
    pub object_name: String,
    pub index_name: String,
    pub operation: String,
    pub number_of_bytes: i64,
}

#[derive(Clone, Debug)]
pub struct EventsWaitsHistoryRow {
    pub thread_id: u64,
    pub event_id: u64,
    pub event_name: String,
    pub source: String,
    pub timer_start: u64,
    pub timer_end: u64,
    pub timer_wait: u64,
    pub object_schema: String,
    pub object_name: String,
    pub index_name: String,
    pub operation: String,
    pub number_of_bytes: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut buffer: RingBuffer<i32> = RingBuffer::new(3);
        assert!(buffer.is_empty());

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());

        let items: Vec<i32> = buffer.iter().cloned().collect();
        assert!(items.contains(&1));
        assert!(items.contains(&2));
        assert!(items.contains(&3));
    }

    #[test]
    fn test_ring_buffer_overwrite() {
        let mut buffer: RingBuffer<i32> = RingBuffer::new(3);
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4);

        assert_eq!(buffer.len(), 3);
        let items: Vec<i32> = buffer.iter().cloned().collect();
        assert!(!items.contains(&1));
        assert!(items.contains(&2));
        assert!(items.contains(&3));
        assert!(items.contains(&4));
    }

    #[test]
    fn test_setup_actors_default() {
        let actors = PerformanceSchema::get_setup_actors();
        assert!(!actors.is_empty());
        let actor = &actors[0];
        assert_eq!(actor.trigger_id, "%");
        assert_eq!(actor.enabled, "YES");
        assert_eq!(actor.history, "YES");
    }

    #[test]
    fn test_setup_instruments_default() {
        let instruments = PerformanceSchema::get_setup_instruments();
        assert!(!instruments.is_empty());

        let select_instrument = instruments
            .iter()
            .find(|i| i.name == "statement/sql/select");
        assert!(select_instrument.is_some());
        let instrument = select_instrument.unwrap();
        assert_eq!(instrument.enabled, "YES");
        assert_eq!(instrument.timed, "YES");
        assert_eq!(instrument.properties, "STATEMENT");
    }

    #[test]
    fn test_statement_stats_record() {
        let row = EventsStatementsHistoryRow {
            thread_id: 1,
            event_id: 1,
            event_name: "statement/sql/select".to_string(),
            source: "test.rs:100".to_string(),
            timer_start: 1000,
            timer_end: 2000,
            timer_wait: 1000,
            lock_time: 100,
            sql_text: "SELECT 1".to_string(),
            digest: "abc123".to_string(),
            digest_text: "SELECT 1".to_string(),
        };

        PerformanceSchema::record_statement(row);

        let history = PerformanceSchema::get_events_statements_history(None);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_statement_summary_update() {
        let stats = StatementStats {
            digest: "digest1".to_string(),
            digest_text: "SELECT 1".to_string(),
            timer_wait: 1000,
            lock_time: 100,
            rows_affected: 0,
            rows_sent: 1,
            rows_examined: 1,
            errors: 0,
        };

        PerformanceSchema::update_statement_summary(stats.clone());
        PerformanceSchema::update_statement_summary(stats);

        let summary = PerformanceSchema::get_events_statements_summary_by_digest(None);
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].count_star, 2);
        assert_eq!(summary[0].sum_timer_wait, 2000);
    }

    #[test]
    fn test_wait_history_record() {
        let row = EventsWaitsHistoryRow {
            thread_id: 1,
            event_id: 1,
            event_name: "wait/io/socket".to_string(),
            source: "test.rs:100".to_string(),
            timer_start: 1000,
            timer_end: 1500,
            timer_wait: 500,
            object_schema: "test".to_string(),
            object_name: "socket".to_string(),
            index_name: "".to_string(),
            operation: "read".to_string(),
            number_of_bytes: 1024,
        };

        PerformanceSchema::record_wait(row);

        let history = PerformanceSchema::get_events_waits_history(None);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].operation, "read");
    }
}
