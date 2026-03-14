// SQLRustGo server module

pub mod health;

pub use health::{ComponentHealth, HealthChecker, HealthComponent, HealthReport, HealthStatus};

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
