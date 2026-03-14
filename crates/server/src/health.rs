//! Health Check Module
//!
//! Provides health check endpoints for Kubernetes liveness and readiness probes.
//!
//! # Usage
//!
//! ```ignore
//! use sqlrustgo_server::health::{HealthChecker, HealthStatus};
//!
//! let checker = HealthChecker::new("1.3.0");
//! let live_status = checker.check_live();
//! let ready_report = checker.check_ready();
//! ```

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unhealthy => "unhealthy",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

impl ComponentHealth {
    pub fn new(name: impl Into<String>, status: HealthStatus) -> Self {
        Self {
            name: name.into(),
            status,
            message: None,
            latency_ms: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub version: String,
    pub timestamp: u64,
    pub components: Vec<ComponentHealth>,
}

impl HealthReport {
    pub fn new(version: impl Into<String>, components: Vec<ComponentHealth>) -> Self {
        let status = if components.iter().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if components.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            status,
            version: version.into(),
            timestamp,
            components,
        }
    }
}

pub trait HealthComponent: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> ComponentHealth;
}

pub struct HealthChecker {
    version: String,
    start_time: SystemTime,
    components: Vec<Box<dyn HealthComponent>>,
}

impl HealthChecker {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            start_time: SystemTime::now(),
            components: Vec::new(),
        }
    }

    pub fn with_component<C: HealthComponent + 'static>(mut self, component: C) -> Self {
        self.components.push(Box::new(component));
        self
    }

    pub fn check_live(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    pub fn check_ready(&self) -> HealthReport {
        let components: Vec<ComponentHealth> = self
            .components
            .iter()
            .map(|c| c.check())
            .collect();

        HealthReport::new(&self.version, components)
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new("unknown")
    }
}

#[derive(Debug, Clone, Default)]
pub struct SystemHealthComponent;

impl SystemHealthComponent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HealthComponent for SystemHealthComponent {
    fn name(&self) -> &str {
        "system"
    }

    fn check(&self) -> ComponentHealth {
        ComponentHealth::new("system", HealthStatus::Healthy)
            .with_message("System is operational")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new("1.3.0");
        assert_eq!(checker.version(), "1.3.0");
    }

    #[test]
    fn test_liveness_check() {
        let checker = HealthChecker::new("1.3.0");
        let status = checker.check_live();
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_readiness_check_empty() {
        let checker = HealthChecker::new("1.3.0");
        let report = checker.check_ready();
        assert_eq!(report.status, HealthStatus::Healthy);
        assert_eq!(report.components.len(), 0);
    }

    #[test]
    fn test_readiness_check_with_components() {
        let checker = HealthChecker::new("1.3.0")
            .with_component(SystemHealthComponent::new());
        let report = checker.check_ready();
        assert_eq!(report.status, HealthStatus::Healthy);
        assert_eq!(report.components.len(), 1);
    }

    #[test]
    fn test_component_health() {
        let health = ComponentHealth::new("test", HealthStatus::Healthy)
            .with_message("Test message")
            .with_latency(10);

        assert_eq!(health.name, "test");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.message, Some("Test message".to_string()));
        assert_eq!(health.latency_ms, Some(10));
    }

    #[test]
    fn test_health_status_strings() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.as_str(), "unhealthy");
    }

    #[test]
    fn test_health_report_all_healthy() {
        let components = vec![
            ComponentHealth::new("comp1", HealthStatus::Healthy),
            ComponentHealth::new("comp2", HealthStatus::Healthy),
        ];
        let report = HealthReport::new("1.3.0", components);
        assert_eq!(report.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_report_with_degraded() {
        let components = vec![
            ComponentHealth::new("comp1", HealthStatus::Healthy),
            ComponentHealth::new("comp2", HealthStatus::Degraded),
        ];
        let report = HealthReport::new("1.3.0", components);
        assert_eq!(report.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_health_report_with_unhealthy() {
        let components = vec![
            ComponentHealth::new("comp1", HealthStatus::Healthy),
            ComponentHealth::new("comp2", HealthStatus::Unhealthy),
        ];
        let report = HealthReport::new("1.3.0", components);
        assert_eq!(report.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_system_health_component() {
        let component = SystemHealthComponent::new();
        let health = component.check();
        assert_eq!(health.name, "system");
        assert_eq!(health.status, HealthStatus::Healthy);
    }
}
