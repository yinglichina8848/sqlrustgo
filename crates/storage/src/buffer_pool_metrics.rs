//! Buffer Pool Metrics Implementation
//!
//! Implements the Metrics trait for BufferPool to track performance metrics.

use crate::buffer_pool::{BufferPool, BufferPoolStats};
use sqlrustgo_common::metrics::Metrics;
use std::sync::Arc;

pub struct BufferPoolMetrics {
    buffer_pool: Arc<BufferPool>,
    queries_total: std::sync::atomic::AtomicU64,
    queries_failed: std::sync::atomic::AtomicU64,
    query_duration_ns: std::sync::atomic::AtomicU64,
    rows_processed: std::sync::atomic::AtomicU64,
}

impl BufferPoolMetrics {
    pub fn new(buffer_pool: Arc<BufferPool>) -> Self {
        Self {
            buffer_pool,
            queries_total: std::sync::atomic::AtomicU64::new(0),
            queries_failed: std::sync::atomic::AtomicU64::new(0),
            query_duration_ns: std::sync::atomic::AtomicU64::new(0),
            rows_processed: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn buffer_pool_stats(&self) -> BufferPoolStats {
        self.buffer_pool.stats()
    }

    pub fn queries_total(&self) -> u64 {
        self.queries_total
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn queries_failed(&self) -> u64 {
        self.queries_failed
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn rows_processed(&self) -> u64 {
        self.rows_processed
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Metrics for BufferPoolMetrics {
    fn record_query(&mut self, _query_type: &str, duration_ms: u64) {
        self.queries_total
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let duration_ns = duration_ms * 1_000_000;
        self.query_duration_ns
            .fetch_add(duration_ns, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_error(&mut self) {
        self.queries_failed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_error_with_type(&mut self, _error_type: &str) {
        self.queries_failed
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_bytes_read(&mut self, _bytes: u64) {}

    fn record_bytes_written(&mut self, _bytes: u64) {}

    fn record_cache_hit(&mut self) {}

    fn record_cache_miss(&mut self) {}

    fn get_metric(&self, name: &str) -> Option<sqlrustgo_common::metrics::MetricValue> {
        match name {
            "queries_total" => Some(sqlrustgo_common::metrics::MetricValue::Counter(
                self.queries_total(),
            )),
            "queries_failed" => Some(sqlrustgo_common::metrics::MetricValue::Counter(
                self.queries_failed(),
            )),
            "rows_processed" => Some(sqlrustgo_common::metrics::MetricValue::Counter(
                self.rows_processed(),
            )),
            _ => None,
        }
    }

    fn get_metric_names(&self) -> Vec<String> {
        vec![
            "queries_total".to_string(),
            "queries_failed".to_string(),
            "rows_processed".to_string(),
            "buffer_pool_hits".to_string(),
            "buffer_pool_misses".to_string(),
            "buffer_pool_evictions".to_string(),
        ]
    }

    fn reset(&mut self) {
        self.queries_total
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.queries_failed
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.query_duration_ns
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.rows_processed
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl BufferPoolMetrics {
    pub fn record_rows(&self, count: usize) {
        self.rows_processed
            .fetch_add(count as u64, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_common::metrics::Metrics;

    #[test]
    fn test_buffer_pool_metrics_creation() {
        let pool = Arc::new(BufferPool::new(100));
        let metrics = BufferPoolMetrics::new(pool);
        assert_eq!(metrics.queries_total(), 0);
        assert_eq!(metrics.queries_failed(), 0);
    }

    #[test]
    fn test_buffer_pool_metrics_record_query() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_query("SELECT", 10);
        assert_eq!(metrics.queries_total(), 1);
    }

    #[test]
    fn test_buffer_pool_metrics_record_error() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_error();
        assert_eq!(metrics.queries_failed(), 1);
    }

    #[test]
    fn test_buffer_pool_metrics_get_metric() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_query("SELECT", 10);

        let queries = metrics.get_metric("queries_total");
        assert!(queries.is_some());
    }

    #[test]
    fn test_buffer_pool_metrics_reset() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_query("SELECT", 10);
        metrics.reset();

        assert_eq!(metrics.queries_total(), 0);
    }

    #[test]
    fn test_buffer_pool_metrics_record_rows() {
        let pool = Arc::new(BufferPool::new(100));
        let metrics = BufferPoolMetrics::new(pool);

        metrics.record_rows(100);
        assert_eq!(metrics.rows_processed(), 100);
    }

    #[test]
    fn test_buffer_pool_metrics_buffer_pool_stats() {
        let pool = Arc::new(BufferPool::new(100));
        let metrics = BufferPoolMetrics::new(pool);

        let stats = metrics.buffer_pool_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_buffer_pool_metrics_record_error_with_type() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_error_with_type("timeout");
        assert_eq!(metrics.queries_failed(), 1);
    }

    #[test]
    fn test_buffer_pool_metrics_record_bytes_read() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_bytes_read(1024);
    }

    #[test]
    fn test_buffer_pool_metrics_record_bytes_written() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_bytes_written(2048);
    }

    #[test]
    fn test_buffer_pool_metrics_record_cache_hit() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_cache_hit();
    }

    #[test]
    fn test_buffer_pool_metrics_record_cache_miss() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_cache_miss();
    }

    #[test]
    fn test_buffer_pool_metrics_get_metric_unknown() {
        let pool = Arc::new(BufferPool::new(100));
        let metrics = BufferPoolMetrics::new(pool);

        let result = metrics.get_metric("unknown_metric");
        assert!(result.is_none());
    }

    #[test]
    fn test_buffer_pool_metrics_get_metric_names() {
        let pool = Arc::new(BufferPool::new(100));
        let metrics = BufferPoolMetrics::new(pool);

        let names = metrics.get_metric_names();
        assert!(names.contains(&"queries_total".to_string()));
        assert!(names.contains(&"buffer_pool_hits".to_string()));
    }

    #[test]
    fn test_buffer_pool_metrics_multiple_queries() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_query("SELECT", 10);
        metrics.record_query("INSERT", 5);
        metrics.record_query("UPDATE", 3);

        assert_eq!(metrics.queries_total(), 3);
    }

    #[test]
    fn test_buffer_pool_metrics_reset_clears_all() {
        let pool = Arc::new(BufferPool::new(100));
        let mut metrics = BufferPoolMetrics::new(pool);

        metrics.record_query("SELECT", 10);
        metrics.record_error();
        metrics.record_rows(100);

        metrics.reset();

        assert_eq!(metrics.queries_total(), 0);
        assert_eq!(metrics.queries_failed(), 0);
        assert_eq!(metrics.rows_processed(), 0);
    }
}
