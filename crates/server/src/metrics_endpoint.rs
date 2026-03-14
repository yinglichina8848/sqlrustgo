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
}
