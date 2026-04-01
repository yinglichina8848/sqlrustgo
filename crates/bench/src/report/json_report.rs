//! JSON report generator
//!
//! This module provides machine-readable JSON output for benchmark results.

use crate::{BenchmarkConfig, BenchmarkResult, Distribution};
use serde::{Deserialize, Serialize};

/// JSON schema version for benchmark results.
pub const JSON_SCHEMA_VERSION: &str = "1.0";

/// JSON report generator for machine-readable benchmark results.
pub struct JsonReport;

impl JsonReport {
    /// Creates a new JsonReport instance.
    pub fn new() -> Self {
        Self
    }

    /// Generates a JSON report for the given benchmark result and configuration.
    pub fn generate(
        &self,
        workload_name: &str,
        config: &BenchmarkConfig,
        result: &BenchmarkResult,
    ) -> String {
        let report = JsonBenchmarkReport {
            schema_version: JSON_SCHEMA_VERSION.to_string(),
            workload: workload_name.to_string(),
            config: JsonConfig {
                threads: config.threads,
                warmup_secs: config.warmup_secs,
                duration_secs: config.duration_secs,
                cooldown_secs: config.cooldown_secs,
                tables: config.tables,
                dataset_size: config.dataset_size,
                distribution: match config.distribution {
                    Distribution::Uniform => "uniform".to_string(),
                    Distribution::Zipfian { theta } => format!("zipfian({})", theta),
                },
                seed: config.seed,
            },
            total_queries: result.total_queries,
            total_transactions: result.total_transactions,
            qps: result.qps,
            qps_stddev: result.qps_stddev,
            tps: result.tps,
            statement_latency_ns: JsonLatency {
                min: result.statement_latency.min,
                avg: result.statement_latency.avg,
                p50: result.statement_latency.p50,
                p95: result.statement_latency.p95,
                p99: result.statement_latency.p99,
                max: result.statement_latency.max,
            },
            transaction_latency_ns: JsonLatency {
                min: result.transaction_latency.min,
                avg: result.transaction_latency.avg,
                p50: result.transaction_latency.p50,
                p95: result.transaction_latency.p95,
                p99: result.transaction_latency.p99,
                max: result.transaction_latency.max,
            },
            regression: JsonRegression {
                passed: !result.regression.detected,
                qps_drop: 0.0, // Calculated by detector, stored in result
                p99_increase: 0.0,
            },
        };

        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generates a compact JSON report (single line).
    pub fn generate_compact(
        &self,
        workload_name: &str,
        config: &BenchmarkConfig,
        result: &BenchmarkResult,
    ) -> String {
        let report = JsonBenchmarkReport {
            schema_version: JSON_SCHEMA_VERSION.to_string(),
            workload: workload_name.to_string(),
            config: JsonConfig {
                threads: config.threads,
                warmup_secs: config.warmup_secs,
                duration_secs: config.duration_secs,
                cooldown_secs: config.cooldown_secs,
                tables: config.tables,
                dataset_size: config.dataset_size,
                distribution: match config.distribution {
                    Distribution::Uniform => "uniform".to_string(),
                    Distribution::Zipfian { theta } => format!("zipfian({})", theta),
                },
                seed: config.seed,
            },
            total_queries: result.total_queries,
            total_transactions: result.total_transactions,
            qps: result.qps,
            qps_stddev: result.qps_stddev,
            tps: result.tps,
            statement_latency_ns: JsonLatency {
                min: result.statement_latency.min,
                avg: result.statement_latency.avg,
                p50: result.statement_latency.p50,
                p95: result.statement_latency.p95,
                p99: result.statement_latency.p99,
                max: result.statement_latency.max,
            },
            transaction_latency_ns: JsonLatency {
                min: result.transaction_latency.min,
                avg: result.transaction_latency.avg,
                p50: result.transaction_latency.p50,
                p95: result.transaction_latency.p95,
                p99: result.transaction_latency.p99,
                max: result.transaction_latency.max,
            },
            regression: JsonRegression {
                passed: !result.regression.detected,
                qps_drop: 0.0,
                p99_increase: 0.0,
            },
        };

        serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Writes the JSON report to a file.
    pub fn write_to_file(
        &self,
        workload_name: &str,
        config: &BenchmarkConfig,
        result: &BenchmarkResult,
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        let json = self.generate(workload_name, config, result);
        std::fs::write(path, json)
    }

    /// Parses a JSON benchmark report from a string.
    pub fn parse(json_str: &str) -> Option<JsonBenchmarkReport> {
        serde_json::from_str(json_str).ok()
    }
}

impl Default for JsonReport {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON representation of benchmark configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonConfig {
    threads: usize,
    warmup_secs: u64,
    duration_secs: u64,
    cooldown_secs: u64,
    tables: usize,
    dataset_size: usize,
    distribution: String,
    seed: u64,
}

/// JSON representation of latency metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonLatency {
    min: u64,
    avg: u64,
    p50: u64,
    p95: u64,
    p99: u64,
    max: u64,
}

/// JSON representation of regression status.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRegression {
    passed: bool,
    qps_drop: f64,
    p99_increase: f64,
}

/// Complete JSON benchmark report structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonBenchmarkReport {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    pub workload: String,
    pub config: JsonConfig,
    #[serde(rename = "total_queries")]
    pub total_queries: u64,
    #[serde(rename = "total_transactions")]
    pub total_transactions: u64,
    pub qps: f64,
    #[serde(rename = "qps_stddev")]
    pub qps_stddev: f64,
    pub tps: f64,
    #[serde(rename = "statement_latency_ns")]
    pub statement_latency_ns: JsonLatency,
    #[serde(rename = "transaction_latency_ns")]
    pub transaction_latency_ns: JsonLatency,
    pub regression: JsonRegression,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Percentiles, RegressionReport};

    fn create_test_config() -> BenchmarkConfig {
        BenchmarkConfig {
            threads: 50,
            warmup_secs: 10,
            duration_secs: 60,
            cooldown_secs: 5,
            tables: 1,
            dataset_size: 1_000_000,
            distribution: Distribution::Zipfian { theta: 0.9 },
            seed: 42,
            connection_mode: crate::ConnectionMode::PerThread,
        }
    }

    fn create_test_result() -> BenchmarkResult {
        BenchmarkResult {
            total_queries: 62345,
            total_transactions: 62345,
            qps: 1039.1,
            qps_stddev: 12.4,
            tps: 98.3,
            statement_latency: Percentiles {
                min: 15230,
                avg: 482300,
                p50: 342100,
                p95: 8923400,
                p99: 87654300,
                max: 234567000,
            },
            transaction_latency: Percentiles {
                min: 15230,
                avg: 482300,
                p50: 342100,
                p95: 8923400,
                p99: 87654300,
                max: 234567000,
            },
            regression: RegressionReport {
                detected: false,
                description: "No regression detected".to_string(),
            },
        }
    }

    #[test]
    fn test_json_report_format() {
        let report = JsonReport::new();
        let output = report.generate("oltp_point_select", &create_test_config(), &create_test_result());

        // Check that it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
        assert_eq!(parsed["schema_version"], "1.0");
        assert_eq!(parsed["workload"], "oltp_point_select");
        assert_eq!(parsed["config"]["threads"], 50);
        assert_eq!(parsed["config"]["warmup_secs"], 10);
        assert_eq!(parsed["config"]["duration_secs"], 60);
        assert_eq!(parsed["qps"], 1039.1);
        assert_eq!(parsed["tps"], 98.3);
    }

    #[test]
    fn test_json_report_regression_passed() {
        let report = JsonReport::new();
        let output = report.generate("oltp_point_select", &create_test_config(), &create_test_result());

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
        assert_eq!(parsed["regression"]["passed"], true);
    }

    #[test]
    fn test_json_report_regression_failed() {
        let report = JsonReport::new();
        let mut result = create_test_result();
        result.regression = RegressionReport {
            detected: true,
            description: "QPS regressed by 15.0%".to_string(),
        };

        let output = report.generate("oltp_point_select", &create_test_config(), &result);

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
        assert_eq!(parsed["regression"]["passed"], false);
    }

    #[test]
    fn test_json_report_uniform_distribution() {
        let report = JsonReport::new();
        let mut config = create_test_config();
        config.distribution = Distribution::Uniform;

        let output = report.generate("oltp_point_select", &config, &create_test_result());

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
        assert_eq!(parsed["config"]["distribution"], "uniform");
    }

    #[test]
    fn test_json_report_zipfian_distribution() {
        let report = JsonReport::new();
        let output = report.generate("oltp_point_select", &create_test_config(), &create_test_result());

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
        assert_eq!(parsed["config"]["distribution"], "zipfian(0.9)");
    }

    #[test]
    fn test_json_report_compact() {
        let report = JsonReport::new();
        let compact = report.generate_compact("oltp_point_select", &create_test_config(), &create_test_result());

        // Compact should not contain newlines
        assert!(!compact.contains('\n'));
        assert!(serde_json::from_str::<serde_json::Value>(&compact).is_ok());
    }

    #[test]
    fn test_json_report_parse() {
        let report = JsonReport::new();
        let json = report.generate("oltp_point_select", &create_test_config(), &create_test_result());

        let parsed = JsonReport::parse(&json).expect("Should parse successfully");
        assert_eq!(parsed.schema_version, "1.0");
        assert_eq!(parsed.workload, "oltp_point_select");
        assert_eq!(parsed.qps, 1039.1);
    }

    #[test]
    fn test_json_latency_fields() {
        let report = JsonReport::new();
        let output = report.generate("oltp_point_select", &create_test_config(), &create_test_result());

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("Valid JSON");
        let stmt_latency = &parsed["statement_latency_ns"];
        assert_eq!(stmt_latency["min"], 15230);
        assert_eq!(stmt_latency["avg"], 482300);
        assert_eq!(stmt_latency["p50"], 342100);
        assert_eq!(stmt_latency["p95"], 8923400);
        assert_eq!(stmt_latency["p99"], 87654300);
        assert_eq!(stmt_latency["max"], 234567000);
    }

    #[test]
    fn test_json_write_to_file() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_benchmark_result.json");

        let report = JsonReport::new();
        let result = report.write_to_file(
            "oltp_point_select",
            &create_test_config(),
            &create_test_result(),
            &temp_path,
        );

        assert!(result.is_ok());
        assert!(temp_path.exists());

        // Clean up
        std::fs::remove_file(temp_path).ok();
    }
}