//! Metrics Module
//!
//! This module defines the Metrics trait and related structures for
//! collecting and reporting database performance and usage metrics.
//!
//! # Usage
//!
//! Implement the `Metrics` trait for your metrics collector:
//!
//! ```ignore
//! use sqlrustgo_common::metrics::{Metrics, MetricValue};
//!
//! struct MyMetricsCollector {
//!     query_count: u64,
//!     error_count: u64,
//! }
//!
//! impl Metrics for MyMetricsCollector {
//!     fn record_query(&mut self, _query_type: &str, _duration_ms: u64) {
//!         self.query_count += 1;
//!     }
//!
//!     fn record_error(&mut self) {
//!         self.error_count += 1;
//!     }
//!
//!     fn get_metric(&self, name: &str) -> Option<MetricValue> {
//!         match name {
//!             "queries" => Some(MetricValue::Counter(self.query_count)),
//!             "errors" => Some(MetricValue::Counter(self.error_count)),
//!             _ => None,
//!         }
//!     }
//! }
//! ```

use std::time::{Duration, Instant};

/// MetricValue - represents different types of metric values
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    /// Counter metric - monotonically increasing value
    Counter(u64),
    /// Gauge metric - can increase or decrease
    Gauge(f64),
    /// Histogram metric - distribution of values
    Histogram(Vec<u64>),
    /// Timing metric - duration in milliseconds
    Timing(u64),
}

impl MetricValue {
    /// Get the value as u64 if possible
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            MetricValue::Counter(v) => Some(*v),
            MetricValue::Gauge(v) => Some(*v as u64),
            MetricValue::Timing(v) => Some(*v),
            MetricValue::Histogram(_) => None,
        }
    }

    /// Get the value as f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Counter(v) => Some(*v as f64),
            MetricValue::Gauge(v) => Some(*v),
            MetricValue::Timing(v) => Some(*v as f64),
            MetricValue::Histogram(_) => None,
        }
    }
}

impl Default for MetricValue {
    fn default() -> Self {
        MetricValue::Counter(0)
    }
}

/// QueryType - classifies different types of SQL queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryType {
    /// SELECT query
    Select,
    /// INSERT query
    Insert,
    /// UPDATE query
    Update,
    /// DELETE query
    Delete,
    /// CREATE TABLE/VIEW/INDEX etc.
    Create,
    /// DROP TABLE/VIEW/INDEX etc.
    Drop,
    /// ALTER TABLE
    Alter,
    /// Transaction control (BEGIN/COMMIT/ROLLBACK)
    Transaction,
    /// Other/unknown query type
    #[default]
    Unknown,
}

impl QueryType {
    /// Convert query type to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            QueryType::Select => "SELECT",
            QueryType::Insert => "INSERT",
            QueryType::Update => "UPDATE",
            QueryType::Delete => "DELETE",
            QueryType::Create => "CREATE",
            QueryType::Drop => "DROP",
            QueryType::Alter => "ALTER",
            QueryType::Transaction => "TRANSACTION",
            QueryType::Unknown => "UNKNOWN",
        }
    }

    /// Parse query type from SQL string
    pub fn from_sql(sql: &str) -> Self {
        let sql_upper = sql.trim().to_uppercase();
        if sql_upper.starts_with("SELECT") {
            QueryType::Select
        } else if sql_upper.starts_with("INSERT") {
            QueryType::Insert
        } else if sql_upper.starts_with("UPDATE") {
            QueryType::Update
        } else if sql_upper.starts_with("DELETE") {
            QueryType::Delete
        } else if sql_upper.starts_with("CREATE") {
            QueryType::Create
        } else if sql_upper.starts_with("DROP") {
            QueryType::Drop
        } else if sql_upper.starts_with("ALTER") {
            QueryType::Alter
        } else if sql_upper.starts_with("BEGIN")
            || sql_upper.starts_with("COMMIT")
            || sql_upper.starts_with("ROLLBACK")
        {
            QueryType::Transaction
        } else {
            QueryType::Unknown
        }
    }
}

/// Metrics trait - defines interface for collecting database metrics
///
/// Implement this trait to create custom metrics collectors.
/// The database engine will call these methods to record various
/// operations and events.
pub trait Metrics: Send + Sync {
    /// Record a query execution
    ///
    /// # Arguments
    /// * `query_type` - The type of query (SELECT, INSERT, etc.)
    /// * `duration_ms` - Query execution time in milliseconds
    fn record_query(&mut self, query_type: &str, duration_ms: u64);

    /// Record a query with detailed timing
    ///
    /// # Arguments
    /// * `query_type` - The type of query
    /// * `duration` - Query execution duration
    fn record_query_timed(&mut self, query_type: QueryType, duration: Duration) {
        self.record_query(query_type.as_str(), duration.as_millis() as u64);
    }

    /// Record an error occurrence
    fn record_error(&mut self);

    /// Record an error with type
    ///
    /// # Arguments
    /// * `error_type` - The type/category of error
    fn record_error_with_type(&mut self, error_type: &str);

    /// Record bytes read from storage
    ///
    /// # Arguments
    /// * `bytes` - Number of bytes read
    fn record_bytes_read(&mut self, bytes: u64);

    /// Record bytes written to storage
    ///
    /// # Arguments
    /// * `bytes` - Number of bytes written
    fn record_bytes_written(&mut self, bytes: u64);

    /// Record a cache hit
    fn record_cache_hit(&mut self);

    /// Record a cache miss
    fn record_cache_miss(&mut self);

    /// Get a metric value by name
    ///
    /// # Arguments
    /// * `name` - The name of the metric
    ///
    /// # Returns
    /// The metric value if found, None otherwise
    fn get_metric(&self, name: &str) -> Option<MetricValue>;

    /// Get all metric names
    fn get_metric_names(&self) -> Vec<String>;

    /// Reset all metrics (useful for testing)
    fn reset(&mut self);
}

/// Timer helper for measuring execution time
pub struct MetricsTimer<'a> {
    metrics: &'a mut dyn Metrics,
    query_type: QueryType,
    start: Instant,
}

impl<'a> MetricsTimer<'a> {
    /// Create a new timer
    pub fn new(metrics: &'a mut dyn Metrics, query_type: QueryType) -> Self {
        Self {
            metrics,
            query_type,
            start: Instant::now(),
        }
    }

    /// Stop the timer and record the metric
    pub fn stop(self) {
        let duration = self.start.elapsed();
        self.metrics.record_query_timed(self.query_type, duration);
    }
}

/// DefaultMetrics - simple in-memory metrics implementation
#[derive(Debug, Default)]
pub struct DefaultMetrics {
    /// Total query count
    query_count: u64,
    /// Query counts by type
    query_counts_by_type: std::collections::HashMap<String, u64>,
    /// Total query time in milliseconds
    total_query_time_ms: u64,
    /// Query times by type
    query_times_by_type: std::collections::HashMap<String, u64>,
    /// Error count
    error_count: u64,
    /// Errors by type
    errors_by_type: std::collections::HashMap<String, u64>,
    /// Bytes read
    bytes_read: u64,
    /// Bytes written
    bytes_written: u64,
    /// Cache hits
    cache_hits: u64,
    /// Cache misses
    cache_misses: u64,
}

impl DefaultMetrics {
    /// Create a new DefaultMetrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Get query count
    pub fn query_count(&self) -> u64 {
        self.query_count
    }

    /// Get total query time in milliseconds
    pub fn total_query_time_ms(&self) -> u64 {
        self.total_query_time_ms
    }

    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.error_count
    }

    /// Get bytes read
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    /// Get bytes written
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Get cache hits
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits
    }

    /// Get cache misses
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses
    }

    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Get query count by type
    pub fn query_count_by_type(&self, query_type: &str) -> u64 {
        self.query_counts_by_type
            .get(query_type)
            .copied()
            .unwrap_or(0)
    }

    /// Get average query time in milliseconds
    pub fn avg_query_time_ms(&self) -> u64 {
        if self.query_count == 0 {
            0
        } else {
            self.total_query_time_ms / self.query_count
        }
    }
}

impl Metrics for DefaultMetrics {
    fn record_query(&mut self, query_type: &str, duration_ms: u64) {
        self.query_count += 1;
        self.total_query_time_ms += duration_ms;

        *self
            .query_counts_by_type
            .entry(query_type.to_string())
            .or_insert(0) += 1;
        *self
            .query_times_by_type
            .entry(query_type.to_string())
            .or_insert(0) += duration_ms;
    }

    fn record_error(&mut self) {
        self.error_count += 1;
    }

    fn record_error_with_type(&mut self, error_type: &str) {
        self.error_count += 1;
        *self
            .errors_by_type
            .entry(error_type.to_string())
            .or_insert(0) += 1;
    }

    fn record_bytes_read(&mut self, bytes: u64) {
        self.bytes_read += bytes;
    }

    fn record_bytes_written(&mut self, bytes: u64) {
        self.bytes_written += bytes;
    }

    fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    fn get_metric(&self, name: &str) -> Option<MetricValue> {
        match name {
            "queries" => Some(MetricValue::Counter(self.query_count)),
            "query_time_ms" => Some(MetricValue::Timing(self.total_query_time_ms)),
            "errors" => Some(MetricValue::Counter(self.error_count)),
            "bytes_read" => Some(MetricValue::Counter(self.bytes_read)),
            "bytes_written" => Some(MetricValue::Counter(self.bytes_written)),
            "cache_hits" => Some(MetricValue::Counter(self.cache_hits)),
            "cache_misses" => Some(MetricValue::Counter(self.cache_misses)),
            "cache_hit_rate" => Some(MetricValue::Gauge(self.cache_hit_rate())),
            "avg_query_time_ms" => Some(MetricValue::Gauge(self.avg_query_time_ms() as f64)),
            _ => None,
        }
    }

    fn get_metric_names(&self) -> Vec<String> {
        vec![
            "queries".to_string(),
            "query_time_ms".to_string(),
            "errors".to_string(),
            "bytes_read".to_string(),
            "bytes_written".to_string(),
            "cache_hits".to_string(),
            "cache_misses".to_string(),
            "cache_hit_rate".to_string(),
            "avg_query_time_ms".to_string(),
        ]
    }

    fn reset(&mut self) {
        self.query_count = 0;
        self.query_counts_by_type.clear();
        self.total_query_time_ms = 0;
        self.query_times_by_type.clear();
        self.error_count = 0;
        self.errors_by_type.clear();
        self.bytes_read = 0;
        self.bytes_written = 0;
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test MetricValue
    #[test]
    fn test_metric_value_counter() {
        let value = MetricValue::Counter(42);
        assert_eq!(value.as_u64(), Some(42));
        assert_eq!(value.as_f64(), Some(42.0));
    }

    #[test]
    fn test_metric_value_gauge() {
        let value = MetricValue::Gauge(3.14);
        assert_eq!(value.as_u64(), Some(3));
        assert_eq!(value.as_f64(), Some(3.14));
    }

    #[test]
    fn test_metric_value_timing() {
        let value = MetricValue::Timing(100);
        assert_eq!(value.as_u64(), Some(100));
        assert_eq!(value.as_f64(), Some(100.0));
    }

    #[test]
    fn test_metric_value_histogram() {
        let value = MetricValue::Histogram(vec![1, 2, 3, 4, 5]);
        assert_eq!(value.as_u64(), None);
        assert_eq!(value.as_f64(), None);
    }

    #[test]
    fn test_metric_value_default() {
        let value = MetricValue::default();
        assert_eq!(value, MetricValue::Counter(0));
    }

    // Test QueryType
    #[test]
    fn test_query_type_as_str() {
        assert_eq!(QueryType::Select.as_str(), "SELECT");
        assert_eq!(QueryType::Insert.as_str(), "INSERT");
        assert_eq!(QueryType::Update.as_str(), "UPDATE");
        assert_eq!(QueryType::Delete.as_str(), "DELETE");
    }

    #[test]
    fn test_query_type_from_sql() {
        assert_eq!(
            QueryType::from_sql("SELECT * FROM users"),
            QueryType::Select
        );
        assert_eq!(
            QueryType::from_sql("INSERT INTO users VALUES (1)"),
            QueryType::Insert
        );
        assert_eq!(
            QueryType::from_sql("UPDATE users SET name = 'Bob'"),
            QueryType::Update
        );
        assert_eq!(
            QueryType::from_sql("DELETE FROM users WHERE id = 1"),
            QueryType::Delete
        );
        assert_eq!(
            QueryType::from_sql("CREATE TABLE users (id INT)"),
            QueryType::Create
        );
        assert_eq!(QueryType::from_sql("DROP TABLE users"), QueryType::Drop);
        assert_eq!(
            QueryType::from_sql("ALTER TABLE users ADD name TEXT"),
            QueryType::Alter
        );
        assert_eq!(QueryType::from_sql("BEGIN"), QueryType::Transaction);
        assert_eq!(QueryType::from_sql("COMMIT"), QueryType::Transaction);
    }

    #[test]
    fn test_query_type_from_sql_lowercase() {
        assert_eq!(
            QueryType::from_sql("select * from users"),
            QueryType::Select
        );
        assert_eq!(
            QueryType::from_sql("insert into users values (1)"),
            QueryType::Insert
        );
    }

    #[test]
    fn test_query_type_unknown() {
        assert_eq!(QueryType::from_sql("EXPLAIN SELECT 1"), QueryType::Unknown);
    }

    // Test DefaultMetrics
    #[test]
    fn test_default_metrics_new() {
        let metrics = DefaultMetrics::new();
        assert_eq!(metrics.query_count(), 0);
        assert_eq!(metrics.error_count(), 0);
    }

    #[test]
    fn test_default_metrics_record_query() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_query("SELECT", 100);
        assert_eq!(metrics.query_count(), 1);
        assert_eq!(metrics.total_query_time_ms(), 100);
    }

    #[test]
    fn test_default_metrics_record_query_by_type() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("SELECT", 200);
        metrics.record_query("INSERT", 50);
        assert_eq!(metrics.query_count_by_type("SELECT"), 2);
        assert_eq!(metrics.query_count_by_type("INSERT"), 1);
    }

    #[test]
    fn test_default_metrics_record_error() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_error();
        metrics.record_error();
        assert_eq!(metrics.error_count(), 2);
    }

    #[test]
    fn test_default_metrics_record_error_with_type() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_error_with_type("parse_error");
        metrics.record_error_with_type("parse_error");
        metrics.record_error_with_type("execution_error");
        assert_eq!(metrics.error_count(), 3);
    }

    #[test]
    fn test_default_metrics_record_bytes() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_bytes_read(1024);
        metrics.record_bytes_written(512);
        assert_eq!(metrics.bytes_read(), 1024);
        assert_eq!(metrics.bytes_written(), 512);
    }

    #[test]
    fn test_default_metrics_cache() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        assert_eq!(metrics.cache_hits(), 2);
        assert_eq!(metrics.cache_misses(), 1);
        assert!((metrics.cache_hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_default_metrics_cache_hit_rate_zero() {
        let metrics = DefaultMetrics::new();
        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_default_metrics_avg_query_time() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("SELECT", 200);
        assert_eq!(metrics.avg_query_time_ms(), 150);
    }

    #[test]
    fn test_default_metrics_avg_query_time_zero() {
        let metrics = DefaultMetrics::new();
        assert_eq!(metrics.avg_query_time_ms(), 0);
    }

    #[test]
    fn test_default_metrics_get_metric() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_error();

        assert_eq!(metrics.get_metric("queries"), Some(MetricValue::Counter(1)));
        assert_eq!(metrics.get_metric("errors"), Some(MetricValue::Counter(1)));
        assert_eq!(metrics.get_metric("unknown"), None);
    }

    #[test]
    fn test_default_metrics_get_metric_names() {
        let metrics = DefaultMetrics::new();
        let names = metrics.get_metric_names();
        assert!(names.contains(&"queries".to_string()));
        assert!(names.contains(&"errors".to_string()));
    }

    #[test]
    fn test_default_metrics_reset() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_error();
        metrics.reset();

        assert_eq!(metrics.query_count(), 0);
        assert_eq!(metrics.error_count(), 0);
    }

    #[test]
    fn test_default_metrics_debug() {
        let metrics = DefaultMetrics::new();
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("DefaultMetrics"));
    }

    // Test MetricsTimer
    #[test]
    fn test_metrics_timer() {
        let mut metrics = DefaultMetrics::new();
        {
            let timer = MetricsTimer::new(&mut metrics, QueryType::Select);
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(10));
            timer.stop();
        }
        assert_eq!(metrics.query_count(), 1);
        assert!(metrics.total_query_time_ms() > 0);
    }

    // Test trait object
    #[test]
    fn test_metrics_trait_object() {
        fn _check_metrics(_metrics: &dyn Metrics) {}
        let metrics = DefaultMetrics::new();
        _check_metrics(&metrics);
    }

    #[test]
    fn test_metrics_send_sync() {
        fn _check_send_sync<T: Send + Sync>() {}
        _check_send_sync::<DefaultMetrics>();
    }
}
