//! Metrics Endpoint Module
//!
//! Provides /metrics endpoint for Prometheus-compatible metrics exposition.

use actix_web::{web, HttpResponse, Responder};
use sqlrustgo_common::metrics::{MetricValue, Metrics};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct MetricsRegistry {
    metrics_collectors: Vec<Arc<RwLock<dyn Metrics>>>,
    custom_metrics: HashMap<String, String>,
    help_texts: HashMap<String, String>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            metrics_collectors: Vec::new(),
            custom_metrics: HashMap::new(),
            help_texts: HashMap::new(),
        }
    }

    pub fn register_metrics(&mut self, metrics: Arc<RwLock<dyn Metrics>>) {
        self.metrics_collectors.push(metrics);
    }

    pub fn register_custom_metric(&mut self, name: String, value: String) {
        self.custom_metrics.insert(name, value);
    }

    pub fn register_help(&mut self, name: String, help: String) {
        self.help_texts.insert(name, help);
    }

    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::with_capacity(4096);

        for collector in &self.metrics_collectors {
            if let Ok(metrics) = collector.read() {
                let metric_names = metrics.get_metric_names();
                for name in metric_names {
                    if let Some(value) = metrics.get_metric(&name) {
                        if let Some(help) = self.help_texts.get(&name) {
                            use std::fmt::Write;
                            let _ = writeln!(output, "# HELP sqlrustgo_{} {}", name, help);
                        }

                        let metric_type = match &value {
                            MetricValue::Counter(_) => "counter",
                            MetricValue::Gauge(_) => "gauge",
                            MetricValue::Histogram(_) => "histogram",
                            MetricValue::Timing(_) => "gauge",
                        };

                        use std::fmt::Write;
                        let _ = writeln!(output, "# TYPE sqlrustgo_{} {}", name, metric_type);

                        let metric_value = match &value {
                            MetricValue::Counter(v) => v.to_string(),
                            MetricValue::Gauge(v) => v.to_string(),
                            MetricValue::Histogram(buckets) => {
                                let mut result = String::new();
                                for (i, count) in buckets.iter().enumerate() {
                                    use std::fmt::Write;
                                    let _ = writeln!(
                                        result,
                                        "sqlrustgo_{}{{bucket=\"{}\"}} {}",
                                        name, i, count
                                    );
                                }
                                result
                            }
                            MetricValue::Timing(v) => v.to_string(),
                        };

                        let _ = writeln!(output, "sqlrustgo_{} {}", name, metric_value);
                    }
                }
            }
        }

        for (name, value) in &self.custom_metrics {
            use std::fmt::Write;
            let _ = writeln!(output, "{} {}", name, value);
        }

        output
    }
}

pub async fn metrics_handler(data: web::Data<Arc<RwLock<MetricsRegistry>>>) -> impl Responder {
    let registry = data.read().unwrap();
    let output = registry.to_prometheus_format();

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(output)
}

pub fn configure_metrics_scope(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/metrics").route(web::get().to(metrics_handler)));
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
    fn test_metrics_registry_with_help_text() {
        let mut registry = MetricsRegistry::new();
        registry.register_help("queries".to_string(), "Total number of queries".to_string());

        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }
        registry.register_metrics(metrics);

        let output = registry.to_prometheus_format();

        assert!(output.contains("# HELP sqlrustgo_queries"));
        assert!(output.contains("Total number of queries"));
    }

    #[test]
    fn test_metrics_registry_clone() {
        let mut registry = MetricsRegistry::new();
        registry.register_custom_metric("test_metric".to_string(), "1".to_string());

        let cloned = registry.clone();
        let output = cloned.to_prometheus_format();

        assert!(output.contains("test_metric"));
    }

    #[test]
    fn test_custom_metrics_registration() {
        let mut registry = MetricsRegistry::new();
        registry.register_custom_metric("build_info".to_string(), "version=\"1.4.0\"".to_string());

        let output = registry.to_prometheus_format();

        assert!(output.contains("build_info"));
        assert!(output.contains("version=\"1.4.0\""));
    }

    #[actix_web::test]
    async fn test_metrics_endpoint_handler() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);

        let registry = Arc::new(RwLock::new(registry));

        let app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(web::Data::new(registry))
                .configure(configure_metrics_scope),
        )
        .await;

        let req = actix_web::test::TestRequest::get()
            .uri("/metrics")
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = actix_web::test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("sqlrustgo"));
    }
}
