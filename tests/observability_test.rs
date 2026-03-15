#[cfg(test)]
mod tests {
    use sqlrustgo_common::metrics::{DefaultMetrics, Metrics};
    use sqlrustgo_common::metrics_aggregator::MetricsAggregator;
    use sqlrustgo_common::network_metrics::NetworkMetrics;
    use sqlrustgo_executor::ExecutorMetrics;
    use sqlrustgo_server::health::{HealthChecker, HealthReport, HealthStatus};
    use sqlrustgo_server::metrics_endpoint::MetricsRegistry;
    use std::sync::{Arc, RwLock};

    // ==================== Health Check Tests ====================

    #[test]
    fn test_health_checker_live() {
        let checker = HealthChecker::new("1.3.0");
        let status = checker.check_live();
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_checker_ready() {
        let checker = HealthChecker::new("1.3.0");
        let report = checker.check_ready();
        assert_eq!(report.version, "1.3.0");
    }

    #[test]
    fn test_health_checker_comprehensive() {
        let checker = HealthChecker::new("1.3.0");
        let report = checker.check_health();
        assert_eq!(report.status, HealthStatus::Healthy);
        assert_eq!(report.version, "1.3.0");
    }

    #[test]
    fn test_health_checker_uptime() {
        let checker = HealthChecker::new("1.3.0");
        let uptime = checker.uptime_seconds();
        assert!(uptime >= 0);
    }

    #[test]
    fn test_health_report_status_calculation() {
        let healthy = sqlrustgo_server::health::ComponentHealth::new("test", HealthStatus::Healthy);
        let report = HealthReport::new("1.0.0", vec![healthy]);
        assert_eq!(report.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_strings() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.as_str(), "unhealthy");
    }

    // ==================== Metrics Tests ====================

    #[test]
    fn test_default_metrics() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_error();

        assert_eq!(metrics.query_count(), 1);
        assert_eq!(metrics.error_count(), 1);
    }

    #[test]
    fn test_default_metrics_by_type() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("SELECT", 200);
        metrics.record_query("INSERT", 50);

        assert_eq!(metrics.query_count_by_type("SELECT"), 2);
        assert_eq!(metrics.query_count_by_type("INSERT"), 1);
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
    fn test_default_metrics_bytes() {
        let mut metrics = DefaultMetrics::new();
        metrics.record_bytes_read(1024);
        metrics.record_bytes_written(512);

        assert_eq!(metrics.bytes_read(), 1024);
        assert_eq!(metrics.bytes_written(), 512);
    }

    // ==================== Executor Metrics Tests ====================

    #[test]
    fn test_executor_metrics_creation() {
        let metrics = ExecutorMetrics::new();
        assert_eq!(metrics.queries_total(), 0);
        assert_eq!(metrics.queries_failed(), 0);
    }

    #[test]
    fn test_executor_metrics_record_query() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);

        assert_eq!(metrics.queries_total(), 1);
        assert_eq!(metrics.execution_count(), 1);
    }

    #[test]
    fn test_executor_metrics_by_type() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_query("INSERT", 50);

        assert_eq!(metrics.queries_by_type("SELECT"), 1);
        assert_eq!(metrics.queries_by_type("INSERT"), 1);
    }

    #[test]
    fn test_executor_metrics_error() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.record_error();

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
    fn test_executor_metrics_rows() {
        let metrics = ExecutorMetrics::new();
        metrics.record_rows(100);

        assert_eq!(metrics.rows_processed(), 100);
    }

    #[test]
    fn test_executor_metrics_reset() {
        let mut metrics = ExecutorMetrics::new();
        metrics.record_query("SELECT", 100);
        metrics.reset();

        assert_eq!(metrics.queries_total(), 0);
    }

    // ==================== Network Metrics Tests ====================

    #[test]
    fn test_network_metrics_creation() {
        let metrics = NetworkMetrics::new();
        assert_eq!(metrics.connections_active(), 0);
    }

    #[test]
    fn test_network_metrics_connection_lifecycle() {
        let metrics = NetworkMetrics::new();

        metrics.record_connection_open();
        assert_eq!(metrics.connections_active(), 1);
        assert_eq!(metrics.connections_total(), 1);

        metrics.record_connection_close();
        assert_eq!(metrics.connections_active(), 0);
        assert_eq!(metrics.connections_closed(), 1);
    }

    #[test]
    fn test_network_metrics_bytes() {
        let metrics = NetworkMetrics::new();
        metrics.record_bytes_sent(1024);
        metrics.record_bytes_received(2048);

        assert_eq!(metrics.bytes_sent(), 1024);
        assert_eq!(metrics.bytes_received(), 2048);
    }

    #[test]
    fn test_network_metrics_packets() {
        let metrics = NetworkMetrics::new();
        metrics.record_packet_sent();
        metrics.record_packet_received();

        assert_eq!(metrics.packets_sent(), 1);
        assert_eq!(metrics.packets_received(), 1);
    }

    #[test]
    fn test_network_metrics_errors() {
        let metrics = NetworkMetrics::new();
        metrics.record_error();
        metrics.record_error();

        assert_eq!(metrics.errors_total(), 2);
    }

    // ==================== Metrics Registry Tests ====================

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
        registry.register_custom_metric("custom_metric".to_string(), "42".to_string());

        let output = registry.to_prometheus_format();
        assert!(output.contains("custom_metric 42"));
    }

    #[test]
    fn test_metrics_registry_prometheus_format() {
        let metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        {
            let mut m = metrics.write().unwrap();
            m.record_query("SELECT", 100);
            m.record_error();
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(metrics);
        let output = registry.to_prometheus_format();

        assert!(output.contains("# TYPE"));
        assert!(output.contains("counter") || output.contains("gauge"));
    }

    // ==================== Metrics Aggregator Tests ====================

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

        let summary = aggregator.get_summary();
        assert_eq!(summary.source_count, 1);
    }

    #[test]
    fn test_metrics_aggregator_custom_metric() {
        let mut aggregator = MetricsAggregator::new();
        aggregator.add_custom_metric("test_metric".to_string(), "100".to_string());

        let output = aggregator.to_prometheus_format();
        assert!(output.contains("test_metric 100"));
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
    fn test_metrics_aggregator_multiple_sources() {
        let metrics1: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(DefaultMetrics::new()));
        let metrics2: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(NetworkMetrics::new()));

        let mut aggregator = MetricsAggregator::new();
        aggregator.register_source("executor".to_string(), metrics1);
        aggregator.register_source("network".to_string(), metrics2);

        let summary = aggregator.get_summary();
        assert_eq!(summary.source_count, 2);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_monitoring_pipeline() {
        let checker = HealthChecker::new("1.3.0");
        let health_report = checker.check_health();
        assert_eq!(health_report.status, HealthStatus::Healthy);

        let executor_metrics: Arc<RwLock<dyn Metrics>> =
            Arc::new(RwLock::new(ExecutorMetrics::new()));
        {
            let mut m = executor_metrics.write().unwrap();
            m.record_query("SELECT", 100);
            m.record_query("INSERT", 50);
        }

        let network_metrics: Arc<RwLock<dyn Metrics>> =
            Arc::new(RwLock::new(NetworkMetrics::new()));
        {
            let mut m = network_metrics.write().unwrap();
            m.record_bytes_read(1024);
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(executor_metrics);
        registry.register_metrics(network_metrics);

        let output = registry.to_prometheus_format();
        assert!(!output.is_empty());
        assert!(output.contains("sqlrustgo_queries"));
        assert!(output.contains("sqlrustgo_bytes"));
    }

    #[test]
    fn test_metrics_aggregation_flow() {
        let exec_metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(ExecutorMetrics::new()));
        {
            let mut m = exec_metrics.write().unwrap();
            m.record_query("SELECT", 100);
            m.record_query("SELECT", 200);
        }

        let net_metrics: Arc<RwLock<dyn Metrics>> = Arc::new(RwLock::new(NetworkMetrics::new()));
        {
            let mut m = net_metrics.write().unwrap();
            m.record_bytes_read(4096);
        }

        let mut aggregator = MetricsAggregator::new();
        aggregator.register_source("executor".to_string(), exec_metrics);
        aggregator.register_source("network".to_string(), net_metrics);

        let summary = aggregator.get_summary();
        assert_eq!(summary.source_count, 2);

        let output = aggregator.to_prometheus_format();
        assert!(output.contains("sqlrustgo_executor"));
        assert!(output.contains("sqlrustgo_network"));
    }

    #[test]
    fn test_health_with_metrics_integration() {
        let executor_metrics: Arc<RwLock<dyn Metrics>> =
            Arc::new(RwLock::new(ExecutorMetrics::new()));
        {
            let mut m = executor_metrics.write().unwrap();
            for _ in 0..100 {
                m.record_query("SELECT", 50);
            }
            for _ in 0..5 {
                m.record_error();
            }
        }

        let mut registry = MetricsRegistry::new();
        registry.register_metrics(executor_metrics);

        let output = registry.to_prometheus_format();
        assert!(output.contains("sqlrustgo_queries"));
        assert!(output.contains("sqlrustgo_queries_failed"));
    }
}
