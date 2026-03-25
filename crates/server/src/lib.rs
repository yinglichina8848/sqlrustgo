// SQLRustGo server module

pub mod health;
pub mod http_server;
pub mod metrics_endpoint;
pub mod teaching_endpoints;

pub mod connection_pool;
pub use connection_pool::{ConnectionPool, PoolConfig, PooledSession};

pub mod connection_pool;
pub use connection_pool::{ConnectionPool, PooledSession, PoolConfig};

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
}
