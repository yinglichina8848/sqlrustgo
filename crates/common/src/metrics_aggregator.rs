//! Metrics Aggregator Module
//!
//! Aggregates metrics from multiple sources (Executor, Network, BufferPool, etc.)
//! and provides unified access to all metrics.

use crate::metrics::{MetricValue, Metrics};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct MetricsAggregator {
    sources: HashMap<String, Arc<RwLock<dyn Metrics>>>,
    custom_metrics: HashMap<String, String>,
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            custom_metrics: HashMap::new(),
        }
    }

    pub fn register_source(&mut self, name: String, metrics: Arc<RwLock<dyn Metrics>>) {
        self.sources.insert(name, metrics);
    }

    pub fn add_custom_metric(&mut self, name: String, value: String) {
        self.custom_metrics.insert(name, value);
    }

    pub fn get_all_metrics(&self) -> HashMap<String, HashMap<String, MetricValue>> {
        let mut result = HashMap::new();

        for (source_name, source) in &self.sources {
            if let Ok(metrics) = source.read() {
                let mut source_metrics = HashMap::new();
                for name in metrics.get_metric_names() {
                    if let Some(value) = metrics.get_metric(&name) {
                        source_metrics.insert(name, value);
                    }
                }
                result.insert(source_name.clone(), source_metrics);
            }
        }

        result
    }

    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();

        for (source_name, source) in &self.sources {
            if let Ok(metrics) = source.read() {
                let metric_names = metrics.get_metric_names();
                for name in metric_names {
                    if let Some(value) = metrics.get_metric(&name) {
                        let metric_type = match &value {
                            MetricValue::Counter(_) => "counter",
                            MetricValue::Gauge(_) => "gauge",
                            MetricValue::Histogram(_) => "histogram",
                            MetricValue::Timing(_) => "gauge",
                        };

                        output.push_str(&format!(
                            "# TYPE sqlrustgo_{}_{} {}\n",
                            source_name, name, metric_type
                        ));

                        let metric_value = match &value {
                            MetricValue::Counter(v) => v.to_string(),
                            MetricValue::Gauge(v) => v.to_string(),
                            MetricValue::Histogram(buckets) => {
                                let mut result = String::new();
                                for (i, count) in buckets.iter().enumerate() {
                                    result.push_str(&format!(
                                        "sqlrustgo_{}_{}{{bucket=\"{}\"}} {}\n",
                                        source_name, name, i, count
                                    ));
                                }
                                result
                            }
                            MetricValue::Timing(v) => v.to_string(),
                        };

                        output.push_str(&format!(
                            "sqlrustgo_{}_{} {}\n",
                            source_name, name, metric_value
                        ));
                    }
                }
            }
        }

        for (name, value) in &self.custom_metrics {
            output.push_str(&format!("{} {}\n", name, value));
        }

        output
    }

    pub fn get_summary(&self) -> MetricsSummary {
        let mut total_queries = 0u64;
        let mut total_errors = 0u64;
        let mut total_bytes_read = 0u64;
        let mut total_bytes_written = 0u64;

        for source in self.sources.values() {
            if let Ok(metrics) = source.read() {
                if let Some(MetricValue::Counter(v)) = metrics.get_metric("queries") {
                    total_queries += v;
                }
                if let Some(MetricValue::Counter(v)) = metrics.get_metric("queries_failed") {
                    total_errors += v;
                }
                if let Some(MetricValue::Counter(v)) = metrics.get_metric("bytes_read") {
                    total_bytes_read += v;
                }
                if let Some(MetricValue::Counter(v)) = metrics.get_metric("bytes_written") {
                    total_bytes_written += v;
                }
            }
        }

        MetricsSummary {
            total_queries,
            total_errors,
            total_bytes_read,
            total_bytes_written,
            source_count: self.sources.len(),
        }
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_queries: u64,
    pub total_errors: u64,
    pub total_bytes_read: u64,
    pub total_bytes_written: u64,
    pub source_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::DefaultMetrics;

    #[test]
    fn test_metrics_aggregator_creation() {
        let aggregator = MetricsAggregator::new();
        assert!(aggregator.to_prometheus_format().is_empty());
    }

    #[test]
    fn test_metrics_aggregator_register_source() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));

        let mut aggregator = MetricsAggregator::new();
        aggregator.register_source("executor".to_string(), metrics);

        assert_eq!(aggregator.get_summary().source_count, 1);
    }

    #[test]
    fn test_metrics_aggregator_custom_metrics() {
        let mut aggregator = MetricsAggregator::new();
        aggregator.add_custom_metric("custom_test".to_string(), "100".to_string());

        let output = aggregator.to_prometheus_format();
        assert!(output.contains("custom_test 100"));
    }

    #[test]
    fn test_metrics_aggregator_prometheus_format() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let mut aggregator = MetricsAggregator::new();
        aggregator.register_source("executor".to_string(), metrics);

        let output = aggregator.to_prometheus_format();
        assert!(output.contains("sqlrustgo_executor_queries"));
    }

    #[test]
    fn test_metrics_aggregator_summary() {
        use crate::metrics::DefaultMetrics;

        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
            m.record_error();
        }

        let mut aggregator = MetricsAggregator::new();
        aggregator.register_source("default".to_string(), metrics);

        let summary = aggregator.get_summary();
        assert_eq!(summary.source_count, 1);
    }

    #[test]
    fn test_metrics_aggregator_multiple_sources() {
        let metrics1: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        let metrics2: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));

        let mut aggregator = MetricsAggregator::new();
        aggregator.register_source("executor".to_string(), metrics1);
        aggregator.register_source("network".to_string(), metrics2);

        let summary = aggregator.get_summary();
        assert_eq!(summary.source_count, 2);
    }
}
