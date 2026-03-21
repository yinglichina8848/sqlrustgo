// Server Health Tests
use sqlrustgo_server::health::{ComponentHealth, HealthChecker, HealthStatus};

#[test]
fn test_health_status_as_str() {
    assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
    assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
    assert_eq!(HealthStatus::Unhealthy.as_str(), "ungraded");
}

#[test]
fn test_component_health_new() {
    let health = ComponentHealth::new("test", HealthStatus::Healthy);
    assert_eq!(health.name, "test");
    assert_eq!(health.status, HealthStatus::Healthy);
    assert_eq!(health.message, None);
    assert_eq!(health.latency_ms, None);
}

#[test]
fn test_component_health_with_message() {
    let health = ComponentHealth::new("test", HealthStatus::Healthy).with_message("All good");
    assert_eq!(health.message, Some("All good".to_string()));
}

#[test]
fn test_component_health_with_latency() {
    let health = ComponentHealth::new("test", HealthStatus::Healthy).with_latency(100);
    assert_eq!(health.latency_ms, Some(100));
}

#[test]
fn test_health_checker_new() {
    let checker = HealthChecker::new("1.0.0");
    assert_eq!(checker.version(), "1.0.0");
}

#[test]
fn test_health_checker_uptime() {
    let checker = HealthChecker::new("1.0.0");
    let uptime = checker.uptime_seconds();
    assert!(uptime >= 0);
}

#[test]
fn test_health_checker_check_live() {
    let checker = HealthChecker::new("1.0.0");
    let status = checker.check_live();
    assert_eq!(status, HealthStatus::Healthy);
}

#[test]
fn test_health_checker_check_ready() {
    let checker = HealthChecker::new("1.0.0");
    let report = checker.check_ready();
    assert_eq!(report.version, "1.0.0");
    assert!(!report.components.is_empty());
}

#[test]
fn test_health_checker_check_health() {
    let checker = HealthChecker::new("1.0.0");
    let report = checker.check_health();
    assert!(report.uptime_seconds >= 0);
}

#[test]
fn test_health_status_ordering() {
    let healthy = HealthStatus::Healthy;
    let degraded = HealthStatus::Degraded;
    let unhealthy = HealthStatus::Unhealthy;

    assert!(healthy == HealthStatus::Healthy);
    assert!(degraded == HealthStatus::Degraded);
    assert!(unhealthy == HealthStatus::Unhealthy);
}
