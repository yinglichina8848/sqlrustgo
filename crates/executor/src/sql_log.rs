//! SQL Execution Logging Module
//!
//! Provides SQL execution logging and monitoring capabilities:
//! - SQL execution log
//! - Error logging
//! - Execution plan display
//! - Configurable log levels

use std::sync::LazyLock;
use std::sync::RwLock;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warning" | "warn" => Some(LogLevel::Warning),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SqlLogEntry {
    pub sql: String,
    pub timestamp: Instant,
    pub duration_ms: u64,
    pub level: LogLevel,
    pub message: Option<String>,
    pub success: bool,
}

pub struct ExecutionLog {
    entries: RwLock<Vec<SqlLogEntry>>,
    max_entries: usize,
    current_level: RwLock<LogLevel>,
}

impl ExecutionLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::with_capacity(max_entries)),
            max_entries,
            current_level: RwLock::new(LogLevel::Info),
        }
    }

    pub fn set_level(&self, level: LogLevel) {
        *self.current_level.write().unwrap() = level;
    }

    pub fn get_level(&self) -> LogLevel {
        *self.current_level.read().unwrap()
    }

    pub fn log(&self, entry: SqlLogEntry) {
        let level = *self.current_level.read().unwrap();
        if entry.level as u8 >= level as u8 {
            let mut entries = self.entries.write().unwrap();
            if entries.len() >= self.max_entries {
                entries.remove(0);
            }
            entries.push(entry);
        }
    }

    pub fn get_entries(&self) -> Vec<SqlLogEntry> {
        self.entries.read().unwrap().clone()
    }

    pub fn clear(&self) {
        self.entries.write().unwrap().clear();
    }

    pub fn log_sql(&self, sql: &str, duration_ms: u64, success: bool, message: Option<String>) {
        let level = if success {
            LogLevel::Info
        } else {
            LogLevel::Error
        };
        self.log(SqlLogEntry {
            sql: sql.to_string(),
            timestamp: Instant::now(),
            duration_ms,
            level,
            message,
            success,
        });
    }

    pub fn log_error(&self, sql: &str, error: &str) {
        self.log(SqlLogEntry {
            sql: sql.to_string(),
            timestamp: Instant::now(),
            duration_ms: 0,
            level: LogLevel::Error,
            message: Some(error.to_string()),
            success: false,
        });
    }
}

impl Default for ExecutionLog {
    fn default() -> Self {
        Self::new(1000)
    }
}

static GLOBAL_LOG: LazyLock<ExecutionLog> = LazyLock::new(|| ExecutionLog::new(1000));

pub fn global_execution_log() -> &'static ExecutionLog {
    &GLOBAL_LOG
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("warning"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("warn"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
    }

    #[test]
    fn test_execution_log() {
        let log = ExecutionLog::new(10);

        log.log_sql("SELECT 1", 10, true, None);
        log.log_sql("SELECT 2", 20, false, Some("error".to_string()));

        let entries = log.get_entries();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].success);
        assert!(!entries[1].success);
    }

    #[test]
    fn test_log_level_filtering() {
        let log = ExecutionLog::new(10);
        log.set_level(LogLevel::Warning);

        log.log_sql("debug query", 1, true, None);
        log.log_sql("error query", 1, false, None);

        let entries = log.get_entries();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].success);
    }

    #[test]
    fn test_log_clear() {
        let log = ExecutionLog::new(10);
        log.log_sql("SELECT 1", 1, true, None);
        log.clear();
        assert!(log.get_entries().is_empty());
    }
}
