//! Events Statements Collector for Performance Schema
//!
//! Collects statement events for events_statements_current and events_statements_history.
//! This is the foundation for Performance Schema statement instrumentation.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

/// Statement event - represents a single SQL statement execution
/// Similar to MySQL's events_statements_current table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementEvent {
    /// Unique event ID
    pub event_id: u64,
    /// Thread ID (session ID)
    pub thread_id: u64,
    /// SQL text
    pub sql_text: String,
    /// SQL digest (normalized form)
    pub digest: Option<String>,
    /// Timer start (nanoseconds since epoch)
    pub timer_start: u64,
    /// Timer end (nanoseconds since epoch)
    pub timer_end: u64,
    /// Timer wait (timer_end - timer_start)
    pub timer_wait: u64,
    /// Lock time (nanoseconds)
    pub lock_time: u64,
    /// Rows examined during execution
    pub rows_examined: u64,
    /// Rows sent to client
    pub rows_sent: u64,
    /// Rows affected (INSERT/UPDATE/DELETE)
    pub rows_affected: u64,
    /// Created temporary disk tables
    pub created_tmp_disk_tables: u64,
    /// Created temporary tables
    pub created_tmp_tables: u64,
    /// Error count
    pub error_count: u64,
    /// SQL command type (SELECT, INSERT, UPDATE, DELETE, etc.)
    pub command_type: String,
}

impl StatementEvent {
    /// Create a new statement event
    pub fn new(thread_id: u64, sql_text: String, command_type: String) -> Self {
        let now = current_timestamp_nanos();
        Self {
            event_id: NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed),
            thread_id,
            sql_text,
            digest: None,
            timer_start: now,
            timer_end: 0,
            timer_wait: 0,
            lock_time: 0,
            rows_examined: 0,
            rows_sent: 0,
            rows_affected: 0,
            created_tmp_disk_tables: 0,
            created_tmp_tables: 0,
            error_count: 0,
            command_type,
        }
    }

    /// End the statement and calculate timer_wait
    pub fn end(&mut self, timer_end: u64) {
        self.timer_end = timer_end;
        self.timer_wait = timer_end.saturating_sub(self.timer_start);
    }

    /// Record rows examined
    pub fn set_rows_examined(&mut self, rows: u64) {
        self.rows_examined = rows;
    }

    /// Record rows sent
    pub fn set_rows_sent(&mut self, rows: u64) {
        self.rows_sent = rows;
    }

    /// Record rows affected
    pub fn set_rows_affected(&mut self, rows: u64) {
        self.rows_affected = rows;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
}

/// Global event ID counter
static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

/// Get current timestamp in nanoseconds
fn current_timestamp_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

/// Events Statements Collector
/// Collects current and history statement events
pub struct EventsStatementsCollector {
    /// Current executing statement (None when not executing)
    current: std::sync::RwLock<Option<StatementEvent>>,
    /// Statement history (RingBuffer)
    history: std::sync::RwLock<VecDeque<StatementEvent>>,
    /// Maximum history size
    max_history_size: usize,
    /// Thread-local thread ID counter
    thread_id_counter: AtomicU64,
}

impl EventsStatementsCollector {
    /// Create a new collector with default configuration
    pub fn new() -> Self {
        Self::with_max_history(1000)
    }

    /// Create a new collector with custom max history size
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            current: std::sync::RwLock::new(None),
            history: std::sync::RwLock::new(VecDeque::with_capacity(max_history)),
            max_history_size: max_history,
            thread_id_counter: AtomicU64::new(1),
        }
    }

    /// Get next thread ID
    pub fn next_thread_id(&self) -> u64 {
        self.thread_id_counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Begin a new statement - returns the event ID
    /// Call this when a SQL statement starts executing
    pub fn begin_statement(&self, sql_text: String, command_type: String) -> u64 {
        let thread_id = self.next_thread_id();
        let event = StatementEvent::new(thread_id, sql_text, command_type);
        let event_id = event.event_id;

        *self.current.write().unwrap() = Some(event);
        event_id
    }

    /// End the current statement
    /// Call this when a SQL statement finishes executing
    pub fn end_statement(
        &self,
        event_id: u64,
        rows_examined: u64,
        rows_sent: u64,
        rows_affected: u64,
        error_count: u64,
    ) {
        let mut current_guard = self.current.write().unwrap();
        if let Some(ref mut event) = *current_guard {
            if event.event_id == event_id {
                event.timer_end = current_timestamp_nanos();
                event.timer_wait = event.timer_end.saturating_sub(event.timer_start);
                event.rows_examined = rows_examined;
                event.rows_sent = rows_sent;
                event.rows_affected = rows_affected;
                event.error_count = error_count;

                // Move current to history
                let completed_event = current_guard.take().unwrap();
                drop(current_guard);

                let mut history = self.history.write().unwrap();
                if history.len() >= self.max_history_size {
                    history.pop_front();
                }
                history.push_back(completed_event);
            }
        }
    }

    /// Get the current executing statement (if any)
    pub fn get_current(&self) -> Option<StatementEvent> {
        self.current.read().unwrap().clone()
    }

    /// Get statement history
    pub fn get_history(&self, limit: Option<usize>) -> Vec<StatementEvent> {
        let history = self.history.read().unwrap();
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get the number of events in history
    pub fn history_len(&self) -> usize {
        self.history.read().unwrap().len()
    }

    /// Check if currently executing a statement
    pub fn is_executing(&self) -> bool {
        self.current.read().unwrap().is_some()
    }

    /// Clear all history
    pub fn clear_history(&self) {
        self.history.write().unwrap().clear();
    }

    /// Reset the collector (clear all state)
    pub fn reset(&self) {
        *self.current.write().unwrap() = None;
        self.history.write().unwrap().clear();
    }
}

impl Default for EventsStatementsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for statement scope
/// Automatically ends the statement when dropped
pub struct StatementScope<'a> {
    collector: &'a EventsStatementsCollector,
    event_id: u64,
    rows_examined: u64,
    rows_sent: u64,
    rows_affected: u64,
    error_count: u64,
}

impl<'a> StatementScope<'a> {
    /// Create a new statement scope
    pub fn new(collector: &'a EventsStatementsCollector, sql: String, command: String) -> Self {
        let event_id = collector.begin_statement(sql, command);
        Self {
            collector,
            event_id,
            rows_examined: 0,
            rows_sent: 0,
            rows_affected: 0,
            error_count: 0,
        }
    }

    /// Set rows examined (can be called multiple times)
    pub fn set_rows_examined(&mut self, rows: u64) {
        self.rows_examined = rows;
        if let Some(ref mut event) = *self.collector.current.write().unwrap() {
            if event.event_id == self.event_id {
                event.rows_examined = rows;
            }
        }
    }

    /// Set rows sent
    pub fn set_rows_sent(&mut self, rows: u64) {
        self.rows_sent = rows;
        if let Some(ref mut event) = *self.collector.current.write().unwrap() {
            if event.event_id == self.event_id {
                event.rows_sent = rows;
            }
        }
    }

    /// Set rows affected
    pub fn set_rows_affected(&mut self, rows: u64) {
        self.rows_affected = rows;
        if let Some(ref mut event) = *self.collector.current.write().unwrap() {
            if event.event_id == self.event_id {
                event.rows_affected = rows;
            }
        }
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
        if let Some(ref mut event) = *self.collector.current.write().unwrap() {
            if event.event_id == self.event_id {
                event.error_count += 1;
            }
        }
    }
}

impl<'a> Drop for StatementScope<'a> {
    fn drop(&mut self) {
        self.collector
            .end_statement(self.event_id, self.rows_examined, self.rows_sent, self.rows_affected, self.error_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_event_creation() {
        let event = StatementEvent::new(1, "SELECT 1".to_string(), "SELECT".to_string());
        assert_eq!(event.thread_id, 1);
        assert_eq!(event.sql_text, "SELECT 1");
        assert_eq!(event.command_type, "SELECT");
        assert!(event.timer_start > 0);
        assert_eq!(event.timer_end, 0);
        assert_eq!(event.rows_examined, 0);
        assert_eq!(event.rows_sent, 0);
    }

    #[test]
    fn test_statement_event_end() {
        let mut event = StatementEvent::new(1, "SELECT 1".to_string(), "SELECT".to_string());
        event.end(event.timer_start + 1_000_000); // 1ms later
        assert!(event.timer_wait > 0);
        assert_eq!(event.timer_end, event.timer_start + 1_000_000);
    }

    #[test]
    fn test_events_statements_collector_new() {
        let collector = EventsStatementsCollector::new();
        assert!(!collector.is_executing());
        assert_eq!(collector.history_len(), 0);
    }

    #[test]
    fn test_begin_and_end_statement() {
        let collector = EventsStatementsCollector::new();

        let event_id = collector.begin_statement("SELECT 1".to_string(), "SELECT".to_string());
        assert!(collector.is_executing());

        collector.end_statement(event_id, 100, 10, 0, 0);

        assert!(!collector.is_executing());
        let history = collector.get_history(None);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].sql_text, "SELECT 1");
        assert_eq!(history[0].rows_examined, 100);
        assert_eq!(history[0].rows_sent, 10);
    }

    #[test]
    fn test_statement_scope() {
        let collector = EventsStatementsCollector::new();

        {
            let mut _scope = StatementScope::new(
                &collector,
                "SELECT * FROM t".to_string(),
                "SELECT".to_string(),
            );
            assert!(collector.is_executing());
            _scope.set_rows_examined(1000);
            _scope.set_rows_sent(100);
        }

        assert!(!collector.is_executing());
        let history = collector.get_history(None);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].rows_examined, 1000);
        assert_eq!(history[0].rows_sent, 100);
    }

    #[test]
    fn test_history_limit() {
        let collector = EventsStatementsCollector::with_max_history(3);

        for i in 0..5 {
            let event_id = collector.begin_statement(format!("SELECT {}", i), "SELECT".to_string());
            collector.end_statement(event_id, 0, 0, 0, 0);
        }

        let history = collector.get_history(None);
        assert_eq!(history.len(), 3);
        // Most recent should be first
        assert!(history[0].sql_text.contains("4"));
    }

    #[test]
    fn test_clear_history() {
        let collector = EventsStatementsCollector::new();

        let event_id = collector.begin_statement("SELECT 1".to_string(), "SELECT".to_string());
        collector.end_statement(event_id, 0, 0, 0, 0);

        assert_eq!(collector.history_len(), 1);
        collector.clear_history();
        assert_eq!(collector.history_len(), 0);
    }

    #[test]
    fn test_reset() {
        let collector = EventsStatementsCollector::new();

        let event_id = collector.begin_statement("SELECT 1".to_string(), "SELECT".to_string());
        collector.end_statement(event_id, 0, 0, 0, 0);

        collector.reset();
        assert!(!collector.is_executing());
        assert_eq!(collector.history_len(), 0);
    }

    #[test]
    fn test_error_count() {
        let collector = EventsStatementsCollector::new();

        let event_id = collector.begin_statement("SELECT 1".to_string(), "SELECT".to_string());
        collector.end_statement(event_id, 0, 0, 0, 1);

        let history = collector.get_history(None);
        assert_eq!(history[0].error_count, 1);
    }

    #[test]
    fn test_concurrent_statements() {
        use std::sync::Arc;
        use std::thread;

        let collector = Arc::new(EventsStatementsCollector::new());
        let mut handles = vec![];

        for i in 0..3 {
            let collector = Arc::clone(&collector);
            let handle = thread::spawn(move || {
                let event_id =
                    collector.begin_statement(format!("SELECT {} FROM t", i), "SELECT".to_string());
                collector.end_statement(event_id, i as u64 * 100, i as u64 * 10, 0, 0);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All 3 statements should be in history
        let history = collector.get_history(None);
        assert_eq!(history.len(), 3);
    }
}
