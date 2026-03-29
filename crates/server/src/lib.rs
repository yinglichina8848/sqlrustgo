// SQLRustGo server module

pub mod health;
pub mod http_server;
pub mod metrics_endpoint;
pub mod teaching_endpoints;

pub mod connection_pool;
pub mod security_integration;

pub use connection_pool::{ConnectionPool, PoolConfig, PooledSession};
pub use security_integration::{SecurityGuard, SecurityIntegration, SecurityStats};

pub use health::{ComponentHealth, HealthChecker, HealthComponent, HealthReport, HealthStatus};
pub use http_server::HttpServer;
pub use metrics_endpoint::MetricsRegistry;
pub use teaching_endpoints::{TeachingEndpoints, TeachingHttpServer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_module_exists() {
        let checker = HealthChecker::new("1.3.0");
        assert_eq!(checker.check_live(), HealthStatus::Healthy);
    }

    #[test]
    fn test_health_checker_default() {
        let checker = HealthChecker::default();
        let report = checker.check_ready();
        assert_eq!(report.version, "unknown");
    }

    #[test]
    fn test_health_status_variants() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Unhealthy.as_str(), "unhealthy");
        assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
    }

    #[test]
    fn test_component_health_builder() {
        let component = ComponentHealth::new("test_component", HealthStatus::Healthy)
            .with_message("All good")
            .with_latency(100);
        
        assert_eq!(component.name, "test_component");
        assert_eq!(component.status, HealthStatus::Healthy);
        assert_eq!(component.message.as_deref(), Some("All good"));
        assert_eq!(component.latency_ms, Some(100));
    }

    #[test]
    fn test_health_report_builder() {
        let component = ComponentHealth::new("db", HealthStatus::Healthy);
        let report = HealthReport::new("2.0.0", vec![component]);
        
        assert_eq!(report.version, "2.0.0");
        assert_eq!(report.components.len(), 1);
        assert_eq!(report.components[0].name, "db");
    }

    #[test]
    fn test_connection_pool_config() {
        let config = PoolConfig {
            size: 10,
            timeout_ms: 5000,
        };
        assert_eq!(config.size, 10);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_pooled_session_creation() {
        let session = PooledSession::new();
        assert!(session.is_available());
        assert!(session.transaction_id.is_none());
    }

    #[test]
    fn test_metrics_registry_new() {
        let registry = MetricsRegistry::new();
        let output = registry.to_prometheus_format();
        assert_eq!(output, "");
    }

    #[test]
    fn test_teaching_endpoints_default() {
        let endpoints = TeachingEndpoints::default();
        assert!(endpoints.enable_pipeline_viz);
        assert!(endpoints.enable_profiling);
        assert!(endpoints.enable_trace);
    }

    #[test]
    fn test_http_server_builder() {
        let server = HttpServer::new("127.0.0.1", 8080)
            .with_version("3.0.0");
        assert_eq!(server.get_version(), "3.0.0");
    }
}
