use sqlrustgo_common::metrics::{MetricValue, Metrics};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct QueryCacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    invalidations: AtomicU64,
}

impl QueryCacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_invalidation(&self, count: usize) {
        self.invalidations
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

impl Default for QueryCacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics for QueryCacheMetrics {
    fn record_cache_hit(&mut self) {
        self.record_hit();
    }

    fn record_cache_miss(&mut self) {
        self.record_miss();
    }

    fn record_query(&mut self, _: &str, _: u64) {}

    fn record_error(&mut self) {}

    fn record_error_with_type(&mut self, _: &str) {}

    fn record_bytes_read(&mut self, _: u64) {}

    fn record_bytes_written(&mut self, _: u64) {}

    fn get_metric(&self, name: &str) -> Option<MetricValue> {
        match name {
            "query_cache_hits" => Some(MetricValue::Counter(self.hits.load(Ordering::Relaxed))),
            "query_cache_misses" => Some(MetricValue::Counter(self.misses.load(Ordering::Relaxed))),
            "query_cache_evictions" => {
                Some(MetricValue::Counter(self.evictions.load(Ordering::Relaxed)))
            }
            "query_cache_invalidations" => Some(MetricValue::Counter(
                self.invalidations.load(Ordering::Relaxed),
            )),
            "query_cache_hit_rate" => Some(MetricValue::Gauge(self.hit_rate())),
            _ => None,
        }
    }

    fn get_metric_names(&self) -> Vec<String> {
        vec![
            "query_cache_hits".to_string(),
            "query_cache_misses".to_string(),
            "query_cache_evictions".to_string(),
            "query_cache_invalidations".to_string(),
            "query_cache_hit_rate".to_string(),
        ]
    }

    fn reset(&mut self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.invalidations.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_cache_metrics_new() {
        let metrics = QueryCacheMetrics::new();
        assert_eq!(metrics.hit_rate(), 0.0);
    }

    #[test]
    fn test_query_cache_metrics_record_hit() {
        let metrics = QueryCacheMetrics::new();
        metrics.record_hit();
        assert_eq!(metrics.hits.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_query_cache_metrics_record_miss() {
        let metrics = QueryCacheMetrics::new();
        metrics.record_miss();
        assert_eq!(metrics.misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_query_cache_metrics_record_eviction() {
        let metrics = QueryCacheMetrics::new();
        metrics.record_eviction();
        assert_eq!(metrics.evictions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_query_cache_metrics_record_invalidation() {
        let metrics = QueryCacheMetrics::new();
        metrics.record_invalidation(5);
        assert_eq!(metrics.invalidations.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_query_cache_metrics_hit_rate() {
        let metrics = QueryCacheMetrics::new();
        metrics.record_hit();
        metrics.record_hit();
        metrics.record_hit();
        metrics.record_miss();
        assert!((metrics.hit_rate() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_query_cache_metrics_hit_rate_zero_total() {
        let metrics = QueryCacheMetrics::new();
        assert_eq!(metrics.hit_rate(), 0.0);
    }

    #[test]
    fn test_query_cache_metrics_record_cache_hit() {
        let mut metrics = QueryCacheMetrics::new();
        metrics.record_cache_hit();
        assert_eq!(metrics.hits.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_query_cache_metrics_record_cache_miss() {
        let mut metrics = QueryCacheMetrics::new();
        metrics.record_cache_miss();
        assert_eq!(metrics.misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_query_cache_metrics_get_metric() {
        let metrics = QueryCacheMetrics::new();
        metrics.record_hit();
        metrics.record_miss();

        assert_eq!(
            metrics.get_metric("query_cache_hits"),
            Some(MetricValue::Counter(1))
        );
        assert_eq!(
            metrics.get_metric("query_cache_misses"),
            Some(MetricValue::Counter(1))
        );
        assert_eq!(
            metrics.get_metric("query_cache_hit_rate"),
            Some(MetricValue::Gauge(0.5))
        );
        assert_eq!(metrics.get_metric("unknown"), None);
    }

    #[test]
    fn test_query_cache_metrics_get_metric_names() {
        let metrics = QueryCacheMetrics::new();
        let names = metrics.get_metric_names();
        assert!(names.contains(&"query_cache_hits".to_string()));
        assert!(names.contains(&"query_cache_misses".to_string()));
        assert!(names.contains(&"query_cache_evictions".to_string()));
        assert!(names.contains(&"query_cache_invalidations".to_string()));
        assert!(names.contains(&"query_cache_hit_rate".to_string()));
    }

    #[test]
    fn test_query_cache_metrics_reset() {
        let mut metrics = QueryCacheMetrics::new();
        metrics.record_hit();
        metrics.record_miss();
        metrics.record_eviction();
        metrics.record_invalidation(3);

        metrics.reset();

        assert_eq!(metrics.hits.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.misses.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.evictions.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.invalidations.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_query_cache_metrics_send_sync() {
        fn _check_send_sync<T: Send + Sync>() {}
        _check_send_sync::<QueryCacheMetrics>();
    }
}
