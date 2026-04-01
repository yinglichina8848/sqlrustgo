//! Performance regression detection
//!
//! This module provides regression detection for CI Gate integration.

use crate::{BenchmarkResult, RegressionReport};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for regression detection thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    /// QPS drop threshold percentage (default 10%).
    pub qps_drop_threshold: f64,
    /// P99 latency increase threshold percentage (default 20%).
    pub p99_increase_threshold: f64,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            qps_drop_threshold: 10.0,
            p99_increase_threshold: 20.0,
        }
    }
}

/// Regression detector for comparing benchmark results against a baseline.
pub struct RegressionDetector {
    config: RegressionConfig,
    baseline: Option<BenchmarkResult>,
}

impl RegressionDetector {
    /// Creates a new RegressionDetector by loading baseline from a file.
    ///
    /// If the baseline file doesn't exist, returns a detector with no baseline
    /// which will always pass regression checks.
    pub fn load_baseline(path: &Path) -> std::io::Result<Self> {
        let baseline = if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Some(serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?)
        } else {
            None
        };
        Ok(Self {
            config: RegressionConfig::default(),
            baseline,
        })
    }

    /// Creates a new RegressionDetector with explicit configuration and baseline.
    pub fn new(config: RegressionConfig, baseline: Option<BenchmarkResult>) -> Self {
        Self { config, baseline }
    }

    /// Compares current benchmark results against the baseline.
    ///
    /// Returns a RegressionReport indicating whether a regression was detected.
    pub fn compare(&self, current: &BenchmarkResult) -> RegressionReport {
        match &self.baseline {
            Some(baseline) => {
                // Calculate QPS drop percentage
                let qps_drop = if baseline.qps > 0.0 {
                    (baseline.qps - current.qps) / baseline.qps * 100.0
                } else {
                    0.0
                };

                // Calculate P99 latency increase percentage
                let p99_increase = if baseline.statement_latency.p99 > 0 {
                    (current.statement_latency.p99 as f64
                        - baseline.statement_latency.p99 as f64)
                        / baseline.statement_latency.p99 as f64
                        * 100.0
                } else {
                    0.0
                };

                // Determine if regression is detected
                let qps_regressed = qps_drop >= self.config.qps_drop_threshold;
                let latency_regressed = p99_increase >= self.config.p99_increase_threshold;

                let detected = qps_regressed || latency_regressed;

                // Build message
                let mut messages = Vec::new();
                if qps_regressed {
                    messages.push(format!("QPS regressed by {:.1}%", qps_drop));
                }
                if latency_regressed {
                    messages.push(format!("P99 latency increased by {:.1}%", p99_increase));
                }

                let description = if messages.is_empty() {
                    String::from("No regression detected")
                } else {
                    messages.join(", ")
                };

                RegressionReport {
                    detected,
                    description,
                }
            }
            None => RegressionReport {
                detected: false,
                description: "No baseline established".to_string(),
            },
        }
    }

    /// Sets a new baseline for future comparisons.
    pub fn set_baseline(&mut self, baseline: BenchmarkResult) {
        self.baseline = Some(baseline);
    }
}

impl Default for RegressionDetector {
    fn default() -> Self {
        Self {
            config: RegressionConfig::default(),
            baseline: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Percentiles;

    fn create_test_result(qps: f64, p99: u64) -> BenchmarkResult {
        BenchmarkResult {
            total_queries: 60000,
            total_transactions: 60000,
            qps,
            qps_stddev: 10.0,
            tps: 100.0,
            statement_latency: Percentiles {
                min: 100,
                avg: 500,
                p50: 400,
                p95: 8000,
                p99,
                max: 200000,
            },
            transaction_latency: Percentiles {
                min: 100,
                avg: 500,
                p50: 400,
                p95: 8000,
                p99,
                max: 200000,
            },
            regression: RegressionReport::default(),
        }
    }

    #[test]
    fn test_regression_detection_no_baseline() {
        let detector = RegressionDetector::default();
        let current = create_test_result(1000.0, 50000);

        let report = detector.compare(&current);
        assert!(!report.detected);
        assert!(report.description.contains("No baseline"));
    }

    #[test]
    fn test_regression_detection_no_regression() {
        let baseline = create_test_result(1000.0, 50000);
        let detector = RegressionDetector::new(
            RegressionConfig::default(),
            Some(baseline),
        );

        // Same results - should pass
        let current = create_test_result(1000.0, 50000);
        let report = detector.compare(&current);
        assert!(!report.detected);
    }

    #[test]
    fn test_regression_detection_qps_drop() {
        let baseline = create_test_result(1000.0, 50000);
        let detector = RegressionDetector::new(
            RegressionConfig::default(),
            Some(baseline),
        );

        // QPS dropped by 15% (more than threshold of 10%)
        let current = create_test_result(850.0, 50000);
        let report = detector.compare(&current);
        assert!(report.detected);
        assert!(report.description.contains("QPS"));
    }

    #[test]
    fn test_regression_detection_p99_increase() {
        let baseline = create_test_result(1000.0, 50000);
        let detector = RegressionDetector::new(
            RegressionConfig::default(),
            Some(baseline),
        );

        // P99 increased by 25% (more than threshold of 20%)
        let current = create_test_result(1000.0, 62500);
        let report = detector.compare(&current);
        assert!(report.detected);
        assert!(report.description.contains("P99"));
    }

    #[test]
    fn test_regression_detection_custom_thresholds() {
        let config = RegressionConfig {
            qps_drop_threshold: 30.0,
            p99_increase_threshold: 50.0,
        };
        let baseline = create_test_result(1000.0, 50000);
        let detector = RegressionDetector::new(config, Some(baseline));

        // QPS dropped by 15% - below custom threshold of 30%
        let current = create_test_result(850.0, 50000);
        let report = detector.compare(&current);
        assert!(!report.detected);
    }

    #[test]
    fn test_load_baseline_nonexistent_file() {
        let result = RegressionDetector::load_baseline(Path::new("/nonexistent/baseline.json"));
        // Result is Ok with no baseline
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_baseline_existing_file() {
        // Create a temporary file with baseline data
        let temp_dir = std::env::temp_dir();
        let baseline_path = temp_dir.join("test_baseline.json");

        let baseline = create_test_result(1000.0, 50000);
        let json = serde_json::to_string(&baseline).unwrap();
        std::fs::write(&baseline_path, json).unwrap();

        let result = RegressionDetector::load_baseline(&baseline_path);
        assert!(result.is_ok());
        assert!(result.unwrap().baseline.is_some());

        // Clean up
        std::fs::remove_file(baseline_path).ok();
    }
}