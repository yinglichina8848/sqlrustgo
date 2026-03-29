//! Slow Query Log Module
//!
//! Provides slow query logging with MySQL-compatible format.
//! Logs queries that exceed a configurable threshold.

use chrono::{DateTime, Utc};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::RwLock;

/// Configuration for slow query logging
#[derive(Debug, Clone)]
pub struct SlowQueryConfig {
    /// Whether slow query logging is enabled
    pub enabled: bool,
    /// Threshold in milliseconds (queries exceeding this are logged)
    pub threshold_ms: u64,
    /// Path to the log file
    pub log_path: PathBuf,
}

impl Default for SlowQueryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_ms: 1000,
            log_path: PathBuf::from("slow_query.log"),
        }
    }
}

/// A single slow query log record
#[derive(Debug, Clone)]
pub struct SlowQueryRecord {
    /// When the query was executed (UTC)
    pub timestamp: DateTime<Utc>,
    /// The SQL query text
    pub query: String,
    /// Query execution time in milliseconds
    pub duration_ms: u64,
    /// Number of rows returned
    pub rows: u64,
    /// Lock time in milliseconds (time spent waiting for locks)
    pub lock_time_ms: f64,
    /// Number of rows examined
    pub rows_examined: u64,
}

impl SlowQueryRecord {
    /// Format this record in MySQL slow query log format
    fn to_mysql_format(&self) -> String {
        let timestamp_rfc3339 = self.timestamp.to_rfc3339();
        let timestamp_unix = self.timestamp.timestamp();
        let duration_sec = self.duration_ms as f64 / 1000.0;
        let lock_time_sec = self.lock_time_ms / 1000.0;

        format!(
            "# Time: {ts}\n\
             # User@Host: sqlrustgo[sqlrustgo] @ localhost []\n\
             # Query_time: {qt:.6}  Lock_time: {lt:.6}  Rows_sent: {rs}  Rows_examined: {re}\n\
             SET timestamp={unix};\n\
             {query};",
            ts = timestamp_rfc3339,
            qt = duration_sec,
            lt = lock_time_sec,
            rs = self.rows,
            re = self.rows_examined,
            unix = timestamp_unix,
            query = self.query
        )
    }
}

/// Slow query logger
pub struct SlowQueryLog {
    threshold_ms: u64,
    log_path: PathBuf,
    /// In-memory buffer of recent slow queries (optional, for reading)
    recent_records: RwLock<Vec<SlowQueryRecord>>,
}

impl SlowQueryLog {
    /// Create a new SlowQueryLog
    pub fn new(threshold_ms: u64, log_path: PathBuf) -> Self {
        Self {
            threshold_ms,
            log_path,
            recent_records: RwLock::new(Vec::new()),
        }
    }

    /// Create from config
    pub fn from_config(config: &SlowQueryConfig) -> Self {
        Self::new(config.threshold_ms, config.log_path.clone())
    }

    /// Record a query if it exceeds the threshold
    pub fn maybe_log(&self, query: &str, duration_ms: u64, rows: u64) {
        if duration_ms < self.threshold_ms {
            return;
        }

        let record = SlowQueryRecord {
            timestamp: Utc::now(),
            query: query.to_string(),
            duration_ms,
            rows,
            lock_time_ms: 0.0,
            rows_examined: rows,
        };

        // Write to file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(file, "{}", record.to_mysql_format());
            let _ = writeln!(file);
        }

        // Keep in memory
        let mut recent = self.recent_records.write().unwrap();
        recent.push(record);
    }

    /// Read all slow query logs from the file
    pub fn read_logs(&self) -> Vec<SlowQueryRecord> {
        let mut records = Vec::new();

        let file = match File::open(&self.log_path) {
            Ok(f) => f,
            Err(_) => return records,
        };

        let reader = BufReader::new(file);
        let mut lines: Vec<String> = Vec::new();

        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("# Time:")
                || line.starts_with("# User@Host:")
                || line.starts_with("# Query_time:")
                || line.starts_with("SET timestamp=")
            {
                // Skip header lines for now
                continue;
            }

            if line.starts_with("#") {
                // Other comment lines
                continue;
            }

            if line.trim().is_empty() {
                // Empty line separates records
                if !lines.is_empty() {
                    if let Some(record) = self.parse_record(&lines) {
                        records.push(record);
                    }
                    lines.clear();
                }
                continue;
            }

            lines.push(line);
        }

        // Handle last record
        if !lines.is_empty() {
            if let Some(record) = self.parse_record(&lines) {
                records.push(record);
            }
        }

        records
    }

    /// Parse a record from lines
    fn parse_record(&self, lines: &[String]) -> Option<SlowQueryRecord> {
        if lines.is_empty() {
            return None;
        }

        let query = lines.last()?.trim().to_string();

        Some(SlowQueryRecord {
            timestamp: Utc::now(),
            query,
            duration_ms: self.threshold_ms,
            rows: 0,
            lock_time_ms: 0.0,
            rows_examined: 0,
        })
    }

    /// Get recent in-memory records
    pub fn get_recent(&self) -> Vec<SlowQueryRecord> {
        self.recent_records.read().unwrap().clone()
    }

    /// Clear recent records
    pub fn clear_recent(&self) {
        self.recent_records.write().unwrap().clear();
    }

    /// Get the threshold in milliseconds
    pub fn threshold_ms(&self) -> u64 {
        self.threshold_ms
    }

    /// Get the log path
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}

impl Default for SlowQueryLog {
    fn default() -> Self {
        Self::new(1000, PathBuf::from("slow_query.log"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_slow_query_config_default() {
        let config = SlowQueryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.threshold_ms, 1000);
        assert_eq!(config.log_path, PathBuf::from("slow_query.log"));
    }

    #[test]
    fn test_slow_query_log_creation() {
        let log = SlowQueryLog::new(500, PathBuf::from("/tmp/slow.log"));
        assert_eq!(log.threshold_ms(), 500);
        assert_eq!(log.log_path(), &PathBuf::from("/tmp/slow.log"));
    }

    #[test]
    fn test_slow_query_log_from_config() {
        let config = SlowQueryConfig {
            enabled: true,
            threshold_ms: 2000,
            log_path: PathBuf::from("/tmp/test.log"),
        };
        let log = SlowQueryLog::from_config(&config);
        assert_eq!(log.threshold_ms(), 2000);
    }

    #[test]
    fn test_maybe_log_below_threshold() {
        let log_path = temp_dir().join("test_below_threshold.log");
        let log = SlowQueryLog::new(1000, log_path.clone());

        // Should not log anything
        log.maybe_log("SELECT 1", 500, 10);

        let records = log.read_logs();
        // May or may not have records depending on previous test runs
        let recent = log.get_recent();
        assert!(recent.is_empty() || recent.iter().all(|r| r.duration_ms >= 1000));

        std::fs::remove_file(log_path).ok();
    }

    #[test]
    fn test_maybe_log_above_threshold() {
        let log_path = temp_dir().join("test_above_threshold.log");
        let log = SlowQueryLog::new(100, log_path.clone());

        log.maybe_log("SELECT * FROM orders WHERE status = 'pending'", 500, 100);

        let recent = log.get_recent();
        assert_eq!(recent.len(), 1);
        assert_eq!(
            recent[0].query,
            "SELECT * FROM orders WHERE status = 'pending'"
        );
        assert_eq!(recent[0].duration_ms, 500);
        assert_eq!(recent[0].rows, 100);

        std::fs::remove_file(log_path).ok();
    }

    #[test]
    fn test_slow_query_record_mysql_format() {
        let record = SlowQueryRecord {
            timestamp: DateTime::parse_from_rfc3339("2026-03-28T10:00:00.000Z")
                .unwrap()
                .with_timezone(&Utc),
            query: "SELECT * FROM orders WHERE status = 'pending'".to_string(),
            duration_ms: 5234,
            rows: 100,
            lock_time_ms: 0.123,
            rows_examined: 1000000,
        };

        let formatted = record.to_mysql_format();
        assert!(formatted.contains("# Time: 2026-03-28T10:00:00+00:00"));
        assert!(formatted.contains("Query_time: 5.234000"));
        assert!(formatted.contains("Lock_time: 0.000123"));
        assert!(formatted.contains("Rows_sent: 100"));
        assert!(formatted.contains("Rows_examined: 1000000"));
        assert!(formatted.contains("SELECT * FROM orders WHERE status = 'pending'"));
    }

    #[test]
    fn test_clear_recent() {
        let log_path = temp_dir().join("test_clear.log");
        let log = SlowQueryLog::new(1, log_path);

        log.maybe_log("SELECT 1", 100, 1);
        assert_eq!(log.get_recent().len(), 1);

        log.clear_recent();
        assert!(log.get_recent().is_empty());
    }

    #[test]
    fn test_read_logs_empty_file() {
        let log_path = temp_dir().join("test_empty.log");
        let log = SlowQueryLog::new(1000, log_path);

        let records = log.read_logs();
        assert!(records.is_empty());
    }
}
