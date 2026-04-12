//! SQL Execution Logging Module
//!
//! Provides SQL execution logging and monitoring capabilities:
//! - SQL execution log
//! - Error logging
//! - Execution plan display
//! - Configurable log levels
//! - Log persistence and recovery

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
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
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warning" | "warn" => Some(LogLevel::Warning),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
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

impl SqlLogEntry {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        format!(
            "{}|{}|{}ms|{}|{}|{}",
            self.timestamp.elapsed().as_millis(),
            self.sql,
            self.duration_ms,
            self.level.to_str(),
            if self.success { "OK" } else { "FAILED" },
            self.message.as_deref().unwrap_or("")
        )
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() < 5 {
            return None;
        }

        let level = match parts[3] {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARNING" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            _ => return None,
        };

        let success = parts[4] == "OK";

        Some(SqlLogEntry {
            sql: parts[1].to_string(),
            timestamp: Instant::now(),
            duration_ms: parts[2].trim_end_matches("ms").parse().unwrap_or(0),
            level,
            message: if parts.len() > 5 && !parts[5].is_empty() {
                Some(parts[5].to_string())
            } else {
                None
            },
            success,
        })
    }
}

pub struct ExecutionLog {
    entries: RwLock<Vec<SqlLogEntry>>,
    max_entries: usize,
    current_level: RwLock<LogLevel>,
    log_file: RwLock<Option<PathBuf>>,
}

impl ExecutionLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::with_capacity(max_entries)),
            max_entries,
            current_level: RwLock::new(LogLevel::Info),
            log_file: RwLock::new(None),
        }
    }

    pub fn set_level(&self, level: LogLevel) {
        *self.current_level.write().unwrap() = level;
    }

    pub fn get_level(&self) -> LogLevel {
        *self.current_level.read().unwrap()
    }

    pub fn set_log_file(&self, path: PathBuf) {
        *self.log_file.write().unwrap() = Some(path);
    }

    pub fn get_log_file(&self) -> Option<PathBuf> {
        self.log_file.read().unwrap().clone()
    }

    pub fn log(&self, entry: SqlLogEntry) {
        let level = *self.current_level.read().unwrap();
        if entry.level as u8 >= level as u8 {
            if let Some(ref path) = *self.log_file.read().unwrap() {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = writeln!(file, "{}", entry.to_string());
                }
            }

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

    pub fn persist_to_file(&self, path: &PathBuf) -> io::Result<usize> {
        let mut file = File::create(path)?;
        let entries = self.entries.read().unwrap();
        for entry in entries.iter() {
            writeln!(file, "{}", entry.to_string())?;
        }
        file.flush()?;
        Ok(entries.len())
    }

    pub fn recover_from_file(&self, path: &PathBuf) -> io::Result<usize> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines().map_while(Result::ok) {
            if let Some(entry) = SqlLogEntry::from_str(&line) {
                let mut entries = self.entries.write().unwrap();
                if entries.len() >= self.max_entries {
                    entries.remove(0);
                }
                entries.push(entry);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn backup(&self, backup_path: &PathBuf) -> io::Result<usize> {
        self.persist_to_file(backup_path)
    }

    pub fn restore(&self, backup_path: &PathBuf) -> io::Result<usize> {
        self.clear();
        self.recover_from_file(backup_path)
    }

    pub fn get_entry_count(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn get_error_count(&self) -> usize {
        self.entries
            .read()
            .unwrap()
            .iter()
            .filter(|e| !e.success)
            .count()
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
    use std::env::temp_dir;
    use std::fs;

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

    #[test]
    fn test_log_persistence() {
        let log = ExecutionLog::new(10);
        log.log_sql("SELECT 1", 10, true, None);
        log.log_sql("SELECT 2", 20, true, None);

        let path = temp_dir().join("test_log_persist.txt");
        let count = log.persist_to_file(&path).unwrap();
        assert_eq!(count, 2);

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("SELECT 1"));
        assert!(content.contains("SELECT 2"));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_log_recovery() {
        let log = ExecutionLog::new(10);

        let path = temp_dir().join("test_log_recover.txt");
        {
            let mut file = File::create(&path).unwrap();
            writeln!(file, "1000|SELECT 1|10ms|INFO|OK|").unwrap();
            writeln!(file, "2000|SELECT 2|20ms|ERROR|FAILED|table not found").unwrap();
        }

        let count = log.recover_from_file(&path).unwrap();
        assert_eq!(count, 2);

        let entries = log.get_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sql, "SELECT 1");
        assert_eq!(entries[1].sql, "SELECT 2");
        assert!(!entries[1].success);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_log_backup_restore() {
        let log = ExecutionLog::new(10);
        log.log_sql("SELECT 1", 10, true, None);
        log.log_sql("SELECT 2", 20, false, Some("error".to_string()));

        let backup_path = temp_dir().join("test_log_backup.txt");
        let backup_count = log.backup(&backup_path).unwrap();
        assert_eq!(backup_count, 2);

        let log2 = ExecutionLog::new(10);
        let restore_count = log2.restore(&backup_path).unwrap();
        assert_eq!(restore_count, 2);

        let entries = log2.get_entries();
        assert_eq!(entries[0].sql, "SELECT 1");
        assert!(!entries[1].success);

        fs::remove_file(backup_path).ok();
    }

    #[test]
    fn test_log_corruption_recovery() {
        let log = ExecutionLog::new(10);

        let path = temp_dir().join("test_log_corrupt.txt");
        {
            let mut file = File::create(&path).unwrap();
            writeln!(file, "1000|SELECT 1|10ms|INFO|OK|").unwrap();
            writeln!(file, "INVALID LINE").unwrap();
            writeln!(file, "2000|SELECT 2|20ms|ERROR|FAILED|").unwrap();
        }

        let count = log.recover_from_file(&path).unwrap();
        assert_eq!(count, 2);

        let entries = log.get_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sql, "SELECT 1");
        assert_eq!(entries[1].sql, "SELECT 2");

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_power_failure_simulation() {
        let log = ExecutionLog::new(10);

        let path = temp_dir().join("test_power_failure.txt");

        log.set_log_file(path.clone());
        log.log_sql("SELECT 1", 10, true, None);
        log.log_sql("SELECT 2", 20, true, None);

        let incomplete_path = temp_dir().join("test_power_failure_incomplete.txt");
        {
            let mut file = File::create(&incomplete_path).unwrap();
            writeln!(file, "1000|SELECT 1|10ms|INFO|OK|").unwrap();
        }

        let log2 = ExecutionLog::new(10);
        let count = log2.recover_from_file(&incomplete_path).unwrap();
        assert_eq!(count, 1);

        let entries = log2.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].sql, "SELECT 1");

        fs::remove_file(path).ok();
        fs::remove_file(incomplete_path).ok();
    }

    #[test]
    fn test_log_rotation() {
        let log = ExecutionLog::new(3);

        log.log_sql("SELECT 1", 10, true, None);
        log.log_sql("SELECT 2", 20, true, None);
        log.log_sql("SELECT 3", 30, true, None);

        assert_eq!(log.get_entry_count(), 3);

        log.log_sql("SELECT 4", 40, true, None);

        assert_eq!(log.get_entry_count(), 3);

        let entries = log.get_entries();
        assert!(entries.iter().all(|e| e.sql != "SELECT 1"));
    }

    #[test]
    fn test_error_count() {
        let log = ExecutionLog::new(10);

        log.log_sql("SELECT 1", 10, true, None);
        log.log_sql("SELECT 2", 20, false, Some("error".to_string()));
        log.log_sql("SELECT 3", 30, true, None);
        log.log_sql("SELECT 4", 40, false, Some("error".to_string()));

        assert_eq!(log.get_error_count(), 2);
    }

    #[test]
    fn test_sql_log_entry_serialization() {
        let entry = SqlLogEntry {
            sql: "SELECT * FROM users".to_string(),
            timestamp: Instant::now(),
            duration_ms: 100,
            level: LogLevel::Info,
            message: None,
            success: true,
        };

        let serialized = entry.to_string();
        let deserialized = SqlLogEntry::from_str(&serialized).unwrap();

        assert_eq!(deserialized.sql, "SELECT * FROM users");
        assert_eq!(deserialized.duration_ms, 100);
        assert!(deserialized.success);
    }
}
