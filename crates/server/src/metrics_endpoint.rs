//! Metrics Endpoint Module
//!
//! Provides /metrics endpoint for Prometheus-compatible metrics exposition.

use sqlrustgo_common::metrics::{MetricValue, Metrics};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct MetricsRegistry {
    metrics_collectors: Vec<Arc<RwLock<dyn Metrics>>>,
    custom_metrics: HashMap<String, String>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            metrics_collectors: Vec::new(),
            custom_metrics: HashMap::new(),
        }
    }

    pub fn register_metrics(&mut self, metrics: Arc<RwLock<dyn Metrics>>) {
        self.metrics_collectors.push(metrics);
    }

    pub fn register_custom_metric(&mut self, name: String, value: String) {
        self.custom_metrics.insert(name, value);
    }

    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();

        for collector in &self.metrics_collectors {
            if let Ok(metrics) = collector.read() {
                let metric_names = metrics.get_metric_names();
                for name in metric_names {
                    if let Some(value) = metrics.get_metric(&name) {
                        let metric_type = match &value {
                            MetricValue::Counter(_) => "counter",
                            MetricValue::Gauge(_) => "gauge",
                            MetricValue::Histogram(_) => "histogram",
                            MetricValue::Timing(_) => "gauge",
                        };

                        output.push_str(&format!("# TYPE sqlrustgo_{} {}\n", name, metric_type));

                        let metric_value = match &value {
                            MetricValue::Counter(v) => v.to_string(),
                            MetricValue::Gauge(v) => v.to_string(),
                            MetricValue::Histogram(buckets) => {
                                let mut result = String::new();
                                for (i, count) in buckets.iter().enumerate() {
                                    result.push_str(&format!(
                                        "sqlrustgo_{}{{bucket=\"{}\"}} {}\n",
                                        name, i, count
                                    ));
                                }
                                result
                            }
                            MetricValue::Timing(v) => v.to_string(),
                        };

                        output.push_str(&format!("sqlrustgo_{} {}\n", name, metric_value));
                    }
                }
            }
        }

        for (name, value) in &self.custom_metrics {
            output.push_str(&format!("{} {}\n", name, value));
        }

        output
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_common::metrics::DefaultMetrics;

    #[test]
    fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.to_prometheus_format().is_empty());
    }

    #[test]
    fn test_metrics_registry_with_default_metrics() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_queries"));
    }

    #[test]
    fn test_metrics_registry_custom_metric() {
        let mut registry = MetricsRegistry::new();
        registry.register_custom_metric("sqlrustgo_custom_metric".to_string(), "42".to_string());

        let output = registry.to_prometheus_format();
        assert!(output.contains("sqlrustgo_custom_metric 42"));
    }

    #[test]
    fn test_metrics_registry_multiple_collectors() {
        let metrics1: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics1.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let metrics2: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics2.write().unwrap();
            m.record_query("INSERT", 50);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics1);
        registry.register_metrics(metrics2);

        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_queries"));
    }

    #[test]
    fn test_metrics_registry_prometheus_format() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        assert!(output.contains("# TYPE"));
    }

    #[test]
    fn test_metrics_registry_with_multiple_default_metrics() {
        let metrics1: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics1.write().unwrap();
            m.record_query("SELECT", 100);
            m.record_query("INSERT", 50);
        }

        let metrics2: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics2.write().unwrap();
            m.record_query("UPDATE", 75);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics1);
        registry.register_metrics(metrics2);

        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_queries"));
    }

    #[test]
    fn test_metrics_registry_empty_output() {
        let registry = MetricsRegistry::new();
        let output = registry.to_prometheus_format();

        assert_eq!(output, "");
    }

    #[test]
    fn test_metrics_registry_with_errors() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
            m.record_error();
            m.record_error();
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_errors"));
    }

    #[test]
    fn test_metrics_registry_with_bytes() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_bytes_read(1024);
            m.record_bytes_written(512);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_bytes_read"));
        assert!(output.contains("sqlrustgo_bytes_written"));
    }

    #[test]
    fn test_metrics_registry_with_cache() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_cache_hit();
            m.record_cache_hit();
            m.record_cache_miss();
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_cache_hits"));
        assert!(output.contains("sqlrustgo_cache_misses"));
    }

    #[test]
    fn test_metrics_registry_multiple_custom_metrics() {
        let mut registry = MetricsRegistry::new();
        registry.register_custom_metric("sqlrustgo_custom_a".to_string(), "10".to_string());
        registry.register_custom_metric("sqlrustgo_custom_b".to_string(), "20".to_string());
        registry.register_custom_metric("sqlrustgo_custom_c".to_string(), "30".to_string());

        let output = registry.to_prometheus_format();

        assert!(output.contains("sqlrustgo_custom_a 10"));
        assert!(output.contains("sqlrustgo_custom_b 20"));
        assert!(output.contains("sqlrustgo_custom_c 30"));
    }

    #[test]
    fn test_metrics_registry_prometheus_type_comments() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        let counter_count = output.matches("counter").count();
        let gauge_count = output.matches("gauge").count();

        assert!(counter_count > 0 || gauge_count > 0);
    }
}
