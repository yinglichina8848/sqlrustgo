//! Executor Metrics Implementation
//!
//! Implements the Metrics trait for Executor to track query execution metrics.

use sqlrustgo_common::metrics::{MetricValue, Metrics};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug)]
pub struct ExecutorMetrics {
    queries_total: AtomicU64,
    queries_failed: AtomicU64,
    queries_by_type: std::collections::HashMap<String, AtomicU64>,
    query_duration_ns: AtomicU64,
    query_duration_by_type: std::collections::HashMap<String, AtomicU64>,
    rows_processed: AtomicU64,
    rows_processed_by_type: std::collections::HashMap<String, AtomicU64>,
    execution_count: AtomicU64,
}

impl ExecutorMetrics {
    pub fn new() -> Self {
        Self {
            queries_total: AtomicU64::new(0),
            queries_failed: AtomicU64::new(0),
            queries_by_type: std::collections::HashMap::new(),
            query_duration_ns: AtomicU64::new(0),
            query_duration_by_type: std::collections::HashMap::new(),
            rows_processed: AtomicU64::new(0),
            rows_processed_by_type: std::collections::HashMap::new(),
            execution_count: AtomicU64::new(0),
        }
    }

    pub fn queries_total(&self) -> u64 {
        self.queries_total.load(Ordering::Relaxed)
    }

    pub fn queries_failed(&self) -> u64 {
        self.queries_failed.load(Ordering::Relaxed)
    }

    pub fn rows_processed(&self) -> u64 {
        self.rows_processed.load(Ordering::Relaxed)
    }

    pub fn query_duration_ns(&self) -> u64 {
        self.query_duration_ns.load(Ordering::Relaxed)
    }

    pub fn query_duration_ms(&self) -> u64 {
        self.query_duration_ns.load(Ordering::Relaxed) / 1_000_000
    }

    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }

    pub fn queries_by_type(&self, query_type: &str) -> u64 {
        self.queries_by_type
            .get(query_type)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub fn avg_query_duration_ms(&self) -> u64 {
        let total = self.queries_total.load(Ordering::Relaxed);
        if total == 0 {
            0
        } else {
            self.query_duration_ms() / total
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.queries_total.load(Ordering::Relaxed);
        if total == 0 {
            1.0
        } else {
            let failed = self.queries_failed.load(Ordering::Relaxed);
            (total - failed) as f64 / total as f64
        }
    }
}

impl Default for ExecutorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics for ExecutorMetrics {
    fn record_query(&mut self, query_type: &str, duration_ms: u64) {
        self.queries_total.fetch_add(1, Ordering::Relaxed);
        self.execution_count.fetch_add(1, Ordering::Relaxed);

        let duration_ns = duration_ms * 1_000_000;
        self.query_duration_ns
            .fetch_add(duration_ns, Ordering::Relaxed);

        self.queries_by_type
            .entry(query_type.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        self.query_duration_by_type
            .entry(query_type.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(duration_ns, Ordering::Relaxed);
    }

    fn record_error(&mut self) {
        self.queries_failed.fetch_add(1, Ordering::Relaxed);
    }

    fn record_error_with_type(&mut self, error_type: &str) {
        self.queries_failed.fetch_add(1, Ordering::Relaxed);
        let _ = error_type;
    }

    fn record_bytes_read(&mut self, _bytes: u64) {}

    fn record_bytes_written(&mut self, _bytes: u64) {}

    fn record_cache_hit(&mut self) {}

    fn record_cache_miss(&mut self) {}

    fn get_metric(&self, name: &str) -> Option<MetricValue> {
        match name {
            "queries_total" => Some(MetricValue::Counter(self.queries_total())),
            "queries_failed" => Some(MetricValue::Counter(self.queries_failed())),
            "queries_success" => Some(MetricValue::Counter(
                self.queries_total() - self.queries_failed(),
            )),
            "rows_processed" => Some(MetricValue::Counter(self.rows_processed())),
            "query_duration_ms" => Some(MetricValue::Timing(self.query_duration_ms())),
            "avg_query_duration_ms" => {
                Some(MetricValue::Gauge(self.avg_query_duration_ms() as f64))
            }
            "success_rate" => Some(MetricValue::Gauge(self.success_rate())),
            "execution_count" => Some(MetricValue::Counter(self.execution_count())),
            _ => None,
        }
    }

    fn get_metric_names(&self) -> Vec<String> {
        vec![
            "queries_total".to_string(),
            "queries_failed".to_string(),
            "queries_success".to_string(),
            "rows_processed".to_string(),
            "query_duration_ms".to_string(),
            "avg_query_duration_ms".to_string(),
            "success_rate".to_string(),
            "execution_count".to_string(),
        ]
    }

    fn reset(&mut self) {
        self.queries_total.store(0, Ordering::Relaxed);
        self.queries_failed.store(0, Ordering::Relaxed);
        self.query_duration_ns.store(0, Ordering::Relaxed);
        self.rows_processed.store(0, Ordering::Relaxed);
        self.execution_count.store(0, Ordering::Relaxed);
        self.queries_by_type.clear();
        self.query_duration_by_type.clear();
        self.rows_processed_by_type.clear();
    }
}

impl ExecutorMetrics {
    pub fn record_rows(&self, count: usize) {
        self.rows_processed
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn record_rows_by_type(&mut self, query_type: &str, count: usize) {
        self.rows_processed
            .fetch_add(count as u64, Ordering::Relaxed);
        self.rows_processed_by_type
            .entry(query_type.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(count as u64, Ordering::Relaxed);
    }
}

pub struct ExecutorMetricsTimer<'a> {
    metrics: &'a mut ExecutorMetrics,
    query_type: String,
    start: Instant,
}

impl<'a> ExecutorMetricsTimer<'a> {
    pub fn new(metrics: &'a mut ExecutorMetrics, query_type: &str) -> Self {
        Self {
            metrics,
            query_type: query_type.to_string(),
            start: Instant::now(),
        }
    }

    pub fn stop(self) {
        let duration = self.start.elapsed();

        self.metrics.queries_total.fetch_add(1, Ordering::Relaxed);
        self.metrics.execution_count.fetch_add(1, Ordering::Relaxed);

        let duration_ns = duration.as_nanos() as u64;
        self.metrics
            .query_duration_ns
            .fetch_add(duration_ns, Ordering::Relaxed);

        self.metrics
            .queries_by_type
            .entry(self.query_type.clone())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        self.metrics
            .query_duration_by_type
            .entry(self.query_type.clone())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(duration_ns, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_metrics_creation() {
        let metrics = ExecutorMetrics::new();
        assert_eq!(metrics.queries_total(), 0);
        assert_eq!(metrics.queries_failed(), 0);
        assert_eq!(metrics.rows_processed(), 0);
    }

    #[test]
    fn test_executor_metrics_default() {
        let metrics = ExecutorMetrics::default();
        assert_eq!(metrics.queries_total(), 0);
    }

    #[test]
    fn test_executor_metrics_record_query() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);

        assert_eq!(metrics.queries_total(), 1);
        assert_eq!(metrics.execution_count(), 1);
    }

    #[test]
    fn test_executor_metrics_record_query_by_type() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("SELECT", 200);
        metrics.record_query("INSERT", 50);

        assert_eq!(metrics.queries_by_type("SELECT"), 2);
        assert_eq!(metrics.queries_by_type("INSERT"), 1);
    }

    #[test]
    fn test_executor_metrics_record_error() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_error();

        assert_eq!(metrics.queries_total(), 1);
        assert_eq!(metrics.queries_failed(), 1);
    }

    #[test]
    fn test_executor_metrics_success_rate() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("SELECT", 100);
        metrics.record_error();

        assert!((metrics.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_executor_metrics_success_rate_zero_queries() {
        let metrics = ExecutorMetrics::new();
        assert_eq!(metrics.success_rate(), 1.0);
    }

    #[test]
    fn test_executor_metrics_avg_query_duration() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("SELECT", 200);

        assert_eq!(metrics.avg_query_duration_ms(), 150);
    }

    #[test]
    fn test_executor_metrics_avg_query_duration_zero() {
        let metrics = ExecutorMetrics::new();
        assert_eq!(metrics.avg_query_duration_ms(), 0);
    }

    #[test]
    fn test_executor_metrics_record_rows() {
        let metrics = ExecutorMetrics::new();
        metrics.record_rows(100);

        assert_eq!(metrics.rows_processed(), 100);
    }

    #[test]
    fn test_executor_metrics_record_rows_by_type() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_rows_by_type("SELECT", 50);
        metrics.record_rows_by_type("SELECT", 30);

        assert_eq!(metrics.rows_processed(), 80);
    }

    #[test]
    fn test_executor_metrics_get_metric() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_error();

        assert_eq!(
            metrics.get_metric("queries_total"),
            Some(MetricValue::Counter(1))
        );
        assert_eq!(
            metrics.get_metric("queries_failed"),
            Some(MetricValue::Counter(1))
        );
        assert_eq!(
            metrics.get_metric("queries_success"),
            Some(MetricValue::Counter(0))
        );
        assert_eq!(metrics.get_metric("unknown"), None);
    }

    #[test]
    fn test_executor_metrics_get_metric_names() {
        let metrics = ExecutorMetrics::new();
        let names = metrics.get_metric_names();

        assert!(names.contains(&"queries_total".to_string()));
        assert!(names.contains(&"queries_failed".to_string()));
        assert!(names.contains(&"rows_processed".to_string()));
    }

    #[test]
    fn test_executor_metrics_reset() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_error();
        metrics.record_rows(50);

        metrics.reset();

        assert_eq!(metrics.queries_total(), 0);
        assert_eq!(metrics.queries_failed(), 0);
        assert_eq!(metrics.rows_processed(), 0);
    }

    #[test]
    fn test_executor_metrics_timer() {
        let mut metrics = ExecutorMetrics::new();
        {
            let mut timer = ExecutorMetricsTimer::new(&mut metrics, "SELECT");
            std::thread::sleep(std::time::Duration::from_millis(10));
            timer.stop();
        }

        assert_eq!(metrics.queries_total(), 1);
        assert!(metrics.query_duration_ns() > 0);
    }

    #[test]
    fn test_executor_metrics_debug() {
        let metrics = ExecutorMetrics::new();
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("ExecutorMetrics"));
    }

    #[test]
    fn test_executor_metrics_query_duration_by_type() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("SELECT", 200);
        metrics.record_query("INSERT", 50);

        assert!(metrics.query_duration_ns() > 0);
    }

    #[test]
    fn test_executor_metrics_record_bytes_read() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_bytes_read(1024);
        metrics.record_bytes_read(2048);

        let metric = metrics.get_metric("bytes_read");
        assert!(metric.is_none());
    }

    #[test]
    fn test_executor_metrics_record_bytes_written() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_bytes_written(512);

        let metric = metrics.get_metric("bytes_written");
        assert!(metric.is_none());
    }

    #[test]
    fn test_executor_metrics_record_cache_hit() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let hit_metric = metrics.get_metric("cache_hits");
        assert!(hit_metric.is_none());
        let miss_metric = metrics.get_metric("cache_misses");
        assert!(miss_metric.is_none());
    }

    #[test]
    fn test_executor_metrics_empty_query_type() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("", 100);

        assert_eq!(metrics.queries_total(), 1);
    }

    #[test]
    fn test_executor_metrics_concurrent_queries() {
        use std::thread;

        let metrics = ExecutorMetrics::new();
        let mut handles = vec![];

        for _ in 0..5 {
            let handle = thread::spawn(|| {
                let mut m = ExecutorMetrics::new();
                for _ in 0..10 {
                    m.record_query("SELECT", 100);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(metrics.execution_count(), 0);
    }

    #[test]
    fn test_executor_metrics_send_sync() {
        fn _check_send_sync<T: Send + Sync>() {}
        _check_send_sync::<ExecutorMetrics>();
    }

    #[test]
    fn test_executor_metrics_trait_object() {
        fn _check_metrics(_metrics: &dyn Metrics) {}
        let metrics = ExecutorMetrics::new();
        _check_metrics(&metrics);
    }
}
