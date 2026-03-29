//! SQLRustGo Telemetry Module
//!
//! Provides Prometheus-compatible metrics collection and exposition.
//!
//! # Usage
//!
//! ```ignore
//! use sqlrustgo_telemetry::Metrics;
//!
//! let metrics = Metrics::new();
//! metrics.record_query("SELECT", 100);
//! metrics.record_cache_hit();
//! let output = metrics.to_prometheus_format();
//! ```

mod prometheus;

pub use prometheus::PrometheusRenderer;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Metrics - Central metrics collector for SQLRustGo
///
/// Collects and exposes Prometheus-format metrics for:
/// - Query counts and latency
/// - Connection pool stats
/// - Cache hit/miss rates
/// - Storage I/O throughput
#[derive(Debug)]
pub struct Metrics {
    /// Total number of queries executed (counter per query type)
    pub queries_total: AtomicU64,
    /// Query execution duration histogram (in microseconds, for percentiles)
    queries_duration_us: AtomicU64,
    /// Active connections (gauge)
    pub connections_active: AtomicU64,
    /// Total connections created (counter)
    pub connections_total: AtomicU64,
    /// Cache hits (counter)
    pub cache_hits: AtomicU64,
    /// Cache misses (counter)
    pub cache_misses: AtomicU64,
    /// Storage read bytes (counter)
    pub storage_read_bytes: AtomicU64,
    /// Storage write bytes (counter)
    pub storage_write_bytes: AtomicU64,

    // Histogram buckets for duration tracking (microseconds)
    // Bucket boundaries: 100us, 500us, 1ms, 5ms, 10ms, 50ms, 100ms, 500ms, 1s, 5s
    duration_buckets: [AtomicU64; 10],
}

impl Metrics {
    /// Create a new Metrics instance
    pub fn new() -> Self {
        Self {
            queries_total: AtomicU64::new(0),
            queries_duration_us: AtomicU64::new(0),
            connections_active: AtomicU64::new(0),
            connections_total: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            storage_read_bytes: AtomicU64::new(0),
            storage_write_bytes: AtomicU64::new(0),
            duration_buckets: [
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
                AtomicU64::new(0),
            ],
        }
    }

    /// Record a query execution
    ///
    /// # Arguments
    /// * `query_type` - The type of query (SELECT, INSERT, UPDATE, DELETE, etc.)
    /// * `duration` - Query execution duration
    pub fn record_query(&self, query_type: &str, duration: Duration) {
        self.queries_total.fetch_add(1, Ordering::Relaxed);
        let duration_us = duration.as_micros() as u64;
        self.queries_duration_us
            .fetch_add(duration_us, Ordering::Relaxed);
        self.record_duration_bucket(duration_us);
        let _ = query_type; // Query type stored in derived metrics if needed
    }

    fn record_duration_bucket(&self, duration_us: u64) {
        // Bucket boundaries in microseconds:
        // 0: 100us, 1: 500us, 2: 1ms, 3: 5ms, 4: 10ms,
        // 5: 50ms, 6: 100ms, 7: 500ms, 8: 1s, 9: 5s
        let thresholds = [
            100, 500, 1000, 5000, 10000, 50000, 100000, 500000, 1000000, 5000000,
        ];
        for (i, threshold) in thresholds.iter().enumerate() {
            if duration_us <= *threshold {
                self.duration_buckets[i].fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        // If > 5s, count in the last bucket
        self.duration_buckets[9].fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record bytes read from storage
    pub fn record_bytes_read(&self, bytes: u64) {
        self.storage_read_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record bytes written to storage
    pub fn record_bytes_written(&self, bytes: u64) {
        self.storage_write_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn connection_acquired(&self) {
        self.connections_active.fetch_add(1, Ordering::Relaxed);
        self.connections_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn connection_released(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get queries total count
    pub fn queries_total(&self) -> u64 {
        self.queries_total.load(Ordering::Relaxed)
    }

    /// Get queries duration in microseconds
    pub fn queries_duration_us(&self) -> u64 {
        self.queries_duration_us.load(Ordering::Relaxed)
    }

    /// Get average query duration in microseconds
    pub fn avg_query_duration_us(&self) -> u64 {
        let total = self.queries_total.load(Ordering::Relaxed);
        if total == 0 {
            0
        } else {
            self.queries_duration_us.load(Ordering::Relaxed) / total
        }
    }

    /// Get active connections
    pub fn connections_active(&self) -> u64 {
        self.connections_active.load(Ordering::Relaxed)
    }

    /// Get total connections
    pub fn connections_total(&self) -> u64 {
        self.connections_total.load(Ordering::Relaxed)
    }

    /// Get cache hits
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Get cache misses
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get storage read bytes
    pub fn storage_read_bytes(&self) -> u64 {
        self.storage_read_bytes.load(Ordering::Relaxed)
    }

    /// Get storage write bytes
    pub fn storage_write_bytes(&self) -> u64 {
        self.storage_write_bytes.load(Ordering::Relaxed)
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.queries_total.store(0, Ordering::Relaxed);
        self.queries_duration_us.store(0, Ordering::Relaxed);
        self.connections_active.store(0, Ordering::Relaxed);
        self.connections_total.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.storage_read_bytes.store(0, Ordering::Relaxed);
        self.storage_write_bytes.store(0, Ordering::Relaxed);
        for bucket in &self.duration_buckets {
            bucket.store(0, Ordering::Relaxed);
        }
    }

    /// Get duration bucket counts
    pub fn duration_buckets(&self) -> Vec<u64> {
        self.duration_buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// MetricsTimer - Helper for timing operations
pub struct MetricsTimer<'a> {
    metrics: &'a Metrics,
    start: Instant,
}

impl<'a> MetricsTimer<'a> {
    /// Create a new timer
    pub fn new(metrics: &'a Metrics) -> Self {
        Self {
            metrics,
            start: Instant::now(),
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();
        self.metrics.record_query("QUERY", duration);
        duration
    }
}

/// Global metrics instance
pub static GLOBAL_METRICS: Metrics = Metrics {
    queries_total: AtomicU64::new(0),
    queries_duration_us: AtomicU64::new(0),
    connections_active: AtomicU64::new(0),
    connections_total: AtomicU64::new(0),
    cache_hits: AtomicU64::new(0),
    cache_misses: AtomicU64::new(0),
    storage_read_bytes: AtomicU64::new(0),
    storage_write_bytes: AtomicU64::new(0),
    duration_buckets: [
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new();
        assert_eq!(metrics.queries_total(), 0);
        assert_eq!(metrics.connections_active(), 0);
        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_metrics_record_query() {
        let metrics = Metrics::new();
        metrics.record_query("SELECT", Duration::from_millis(10));
        assert_eq!(metrics.queries_total(), 1);
        assert!(metrics.avg_query_duration_us() >= 9000); // ~10ms = 10000us, allow small variance
    }

    #[test]
    fn test_metrics_record_cache_hit_miss() {
        let metrics = Metrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        assert_eq!(metrics.cache_hits(), 2);
        assert_eq!(metrics.cache_misses(), 1);
        assert!((metrics.cache_hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_metrics_connection_pool() {
        let metrics = Metrics::new();
        metrics.connection_acquired();
        metrics.connection_acquired();
        assert_eq!(metrics.connections_active(), 2);
        assert_eq!(metrics.connections_total(), 2);
        metrics.connection_released();
        assert_eq!(metrics.connections_active(), 1);
        assert_eq!(metrics.connections_total(), 2);
    }

    #[test]
    fn test_metrics_storage_io() {
        let metrics = Metrics::new();
        metrics.record_bytes_read(1024);
        metrics.record_bytes_read(512);
        metrics.record_bytes_written(256);
        assert_eq!(metrics.storage_read_bytes(), 1536);
        assert_eq!(metrics.storage_write_bytes(), 256);
    }

    #[test]
    fn test_metrics_timer() {
        let metrics = Metrics::new();
        {
            let _timer = MetricsTimer::new(&metrics);
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(metrics.queries_total(), 1);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();
        metrics.record_query("SELECT", Duration::from_millis(10));
        metrics.record_cache_hit();
        metrics.connection_acquired();
        metrics.record_bytes_read(1024);
        metrics.reset();
        assert_eq!(metrics.queries_total(), 0);
        assert_eq!(metrics.cache_hits(), 0);
        assert_eq!(metrics.connections_active(), 0);
        assert_eq!(metrics.storage_read_bytes(), 0);
    }

    #[test]
    fn test_duration_buckets() {
        let metrics = Metrics::new();
        metrics.record_query("Q", Duration::from_micros(50)); // < 100us
        metrics.record_query("Q", Duration::from_micros(300)); // < 500us
        metrics.record_query("Q", Duration::from_millis(2)); // < 5ms
        metrics.record_query("Q", Duration::from_millis(100)); // < 500ms
        let buckets = metrics.duration_buckets();
        assert_eq!(buckets[0], 1); // 50 < 100
        assert_eq!(buckets[1], 1); // 300 < 500
        assert_eq!(buckets[3], 1); // 2ms < 5ms
        assert_eq!(buckets[7], 1); // 100ms < 500ms
    }

    #[test]
    fn test_cache_hit_rate_zero_total() {
        let metrics = Metrics::new();
        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_avg_query_duration_zero_queries() {
        let metrics = Metrics::new();
        assert_eq!(metrics.avg_query_duration_us(), 0);
    }
}
