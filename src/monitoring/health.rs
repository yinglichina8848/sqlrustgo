//! Health Checker Module
//!
//! This module provides health check functionality for monitoring system components.

use std::time::{Duration, Instant};

/// Health status enum representing the health state of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Component is functioning normally
    Healthy,
    /// Component is functioning but with degraded performance
    Degraded,
    /// Component is not functioning properly
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
        }
    }
}

/// Component health information
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Current health status
    pub status: HealthStatus,
    /// Optional message with additional information
    pub message: Option<String>,
    /// Latency in milliseconds (if applicable)
    pub latency_ms: Option<u64>,
}

impl ComponentHealth {
    /// Create a new healthy component
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
        }
    }

    /// Create a new degraded component
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            latency_ms: None,
        }
    }

    /// Create a new unhealthy component
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            latency_ms: None,
        }
    }

    /// Create a new component health with latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

/// Trait for components that can be health checked
pub trait HealthComponent: Send + Sync {
    /// Get the component name
    fn name(&self) -> &str;

    /// Check the component health
    fn check(&self) -> ComponentHealth;
}

/// HealthChecker - main struct for checking health of all registered components
pub struct HealthChecker {
    /// Application start time
    start_time: Instant,
    /// Application version
    version: String,
    /// Registered health components
    #[allow(dead_code)]
    components: Vec<Box<dyn HealthComponent>>,
}

impl std::fmt::Debug for HealthChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthChecker")
            .field("version", &self.version)
            .field("components_count", &self.components.len())
            .finish()
    }
}

impl HealthChecker {
    /// Create a new HealthChecker
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            start_time: Instant::now(),
            version: version.into(),
            components: Vec::new(),
        }
    }

    /// Register a health component
    pub fn register(&mut self, component: Box<dyn HealthComponent>) {
        self.components.push(component);
    }

    /// Check overall health status
    pub fn check(&self) -> HealthReport {
        let mut component_reports = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        for component in &self.components {
            let health = component.check();
            if health.status == HealthStatus::Unhealthy {
                overall_status = HealthStatus::Unhealthy;
            } else if health.status == HealthStatus::Degraded
                && overall_status == HealthStatus::Healthy
            {
                overall_status = HealthStatus::Degraded;
            }
            component_reports.push(ComponentReport {
                name: component.name().to_string(),
                health,
            });
        }

        let uptime = self.start_time.elapsed();

        HealthReport {
            status: overall_status,
            version: self.version.clone(),
            uptime_ms: uptime.as_millis() as u64,
            components: component_reports,
        }
    }

    /// Get the application version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the application uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Health report containing overall system health
#[derive(Debug)]
pub struct HealthReport {
    /// Overall health status
    pub status: HealthStatus,
    /// Application version
    pub version: String,
    /// Uptime in milliseconds
    pub uptime_ms: u64,
    /// Individual component reports
    pub components: Vec<ComponentReport>,
}

impl std::fmt::Display for HealthReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Health Report: {}", self.status)?;
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Uptime: {}ms", self.uptime_ms)?;
        writeln!(f, "Components:")?;
        for component in &self.components {
            writeln!(f, "  - {}: {}", component.name, component.health.status)?;
        }
        Ok(())
    }
}

/// Report for a single component
#[derive(Debug)]
pub struct ComponentReport {
    /// Component name
    pub name: String,
    /// Component health
    pub health: ComponentHealth,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Healthy), "Healthy");
        assert_eq!(format!("{}", HealthStatus::Degraded), "Degraded");
        assert_eq!(format!("{}", HealthStatus::Unhealthy), "Unhealthy");
    }

    #[test]
    fn test_component_health_healthy() {
        let health = ComponentHealth::healthy();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.message, None);
        assert_eq!(health.latency_ms, None);
    }

    #[test]
    fn test_component_health_degraded() {
        let health = ComponentHealth::degraded("High latency");
        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.message, Some("High latency".to_string()));
    }

    #[test]
    fn test_component_health_unhealthy() {
        let health = ComponentHealth::unhealthy("Connection failed");
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.message, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_component_health_with_latency() {
        let health = ComponentHealth::healthy().with_latency(100);
        assert_eq!(health.latency_ms, Some(100));
    }

    #[test]
    fn test_health_checker_new() {
        let checker = HealthChecker::new("1.0.0");
        assert_eq!(checker.version(), "1.0.0");
        assert!(checker.uptime().as_millis() >= 0);
    }

    #[test]
    fn test_health_checker_register() {
        struct TestComponent;

        impl HealthComponent for TestComponent {
            fn name(&self) -> &str {
                "test"
            }
            fn check(&self) -> ComponentHealth {
                ComponentHealth::healthy()
            }
        }

        let mut checker = HealthChecker::new("1.0.0");
        checker.register(Box::new(TestComponent));

        let report = checker.check();
        assert_eq!(report.components.len(), 1);
        assert_eq!(report.components[0].name, "test");
    }

    #[test]
    fn test_health_checker_overall_healthy() {
        struct HealthyComponent;

        impl HealthComponent for HealthyComponent {
            fn name(&self) -> &str {
                "healthy"
            }
            fn check(&self) -> ComponentHealth {
                ComponentHealth::healthy()
            }
        }

        let mut checker = HealthChecker::new("1.0.0");
        checker.register(Box::new(HealthyComponent));

        let report = checker.check();
        assert_eq!(report.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_checker_overall_degraded() {
        struct DegradedComponent;

        impl HealthComponent for DegradedComponent {
            fn name(&self) -> &str {
                "degraded"
            }
            fn check(&self) -> ComponentHealth {
                ComponentHealth::degraded("Slow response")
            }
        }

        let mut checker = HealthChecker::new("1.0.0");
        checker.register(Box::new(DegradedComponent));

        let report = checker.check();
        assert_eq!(report.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_health_checker_overall_unhealthy() {
        struct UnhealthyComponent;

        impl HealthComponent for UnhealthyComponent {
            fn name(&self) -> &str {
                "unhealthy"
            }
            fn check(&self) -> ComponentHealth {
                ComponentHealth::unhealthy("Connection lost")
            }
        }

        let mut checker = HealthChecker::new("1.0.0");
        checker.register(Box::new(UnhealthyComponent));

        let report = checker.check();
        assert_eq!(report.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_checker_priority() {
        // Unhealthy should take precedence over Degraded
        struct UnhealthyComponent;
        struct DegradedComponent;

        impl HealthComponent for UnhealthyComponent {
            fn name(&self) -> &str {
                "unhealthy"
            }
            fn check(&self) -> ComponentHealth {
                ComponentHealth::unhealthy("Failed")
            }
        }

        impl HealthComponent for DegradedComponent {
            fn name(&self) -> &str {
                "degraded"
            }
            fn check(&self) -> ComponentHealth {
                ComponentHealth::degraded("Slow")
            }
        }

        let mut checker = HealthChecker::new("1.0.0");
        checker.register(Box::new(DegradedComponent));
        checker.register(Box::new(UnhealthyComponent));

        let report = checker.check();
        assert_eq!(report.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_report_display() {
        let report = HealthReport {
            status: HealthStatus::Healthy,
            version: "1.0.0".to_string(),
            uptime_ms: 1000,
            components: vec![ComponentReport {
                name: "test".to_string(),
                health: ComponentHealth::healthy(),
            }],
        };

        let output = format!("{}", report);
        assert!(output.contains("Health Report: Healthy"));
        assert!(output.contains("Version: 1.0.0"));
        assert!(output.contains("Uptime: 1000ms"));
        assert!(output.contains("test: Healthy"));
    }
}
