//! Text report generator
//!
//! This module provides human-readable text output for benchmark results.

use crate::{BenchmarkConfig, BenchmarkResult, Distribution};

/// Text report generator for human-readable benchmark results.
pub struct TextReport;

impl TextReport {
    /// Creates a new TextReport instance.
    pub fn new() -> Self {
        Self
    }

    /// Generates a text report for the given benchmark result and configuration.
    pub fn generate(
        &self,
        workload_name: &str,
        config: &BenchmarkConfig,
        result: &BenchmarkResult,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "SQLRustGo Benchmark: {}\n",
            workload_name
        ));
        output.push_str(&"=".repeat(39 + workload_name.len()));
        output.push('\n');

        // Config section
        output.push_str("Config:\n");
        output.push_str(&format!("  Threads:       {}\n", config.threads));
        output.push_str(&format!("  Warmup:        {}s\n", config.warmup_secs));
        output.push_str(&format!("  Duration:      {}s\n", config.duration_secs));
        output.push_str(&format!("  Cooldown:      {}s\n", config.cooldown_secs));
        output.push_str(&format!("  Tables:        {}\n", config.tables));
        // Dataset
        let dataset_str = format!("{:}", config.dataset_size);
        output.push_str(&format!("  Dataset:       {} rows\n", dataset_str));

        // Distribution
        let dist_str = match config.distribution {
            Distribution::Uniform => "uniform".to_string(),
            Distribution::Zipfian { theta } => format!("zipfian (theta={})", theta),
        };
        output.push_str(&format!("  Distribution:  {}\n", dist_str));
        output.push_str(&format!("  Seed:          {}\n", config.seed));
        output.push('\n');

        // Results section
        output.push_str("Results:\n");
        output.push_str(&format!(
            "  Total queries:     {}\n",
            result.total_queries
        ));
        output.push_str(&format!(
            "  QPS:               {:.1} \u{00B1} {:.1}\n",
            result.qps, result.qps_stddev
        ));
        output.push_str(&format!(
            "  TPS:               {:.1}\n",
            result.tps
        ));
        output.push('\n');

        // Statement latency
        output.push_str("Statement Latency (ms):\n");
        output.push_str("  min     avg     p50     p95     p99     max\n");
        output.push_str(&format!(
            "  {:.2}    {:.2}    {:.2}    {:.2}    {:.2}   {:.2}\n",
            nanos_to_millis(result.statement_latency.min),
            nanos_to_millis(result.statement_latency.avg),
            nanos_to_millis(result.statement_latency.p50),
            nanos_to_millis(result.statement_latency.p95),
            nanos_to_millis(result.statement_latency.p99),
            nanos_to_millis(result.statement_latency.max)
        ));
        output.push('\n');

        // Transaction latency
        output.push_str("Transaction Latency (ms):\n");
        output.push_str("  min     avg     p50     p95     p99     max\n");
        output.push_str(&format!(
            "  {:.2}    {:.2}    {:.2}    {:.2}    {:.2}   {:.2}\n",
            nanos_to_millis(result.transaction_latency.min),
            nanos_to_millis(result.transaction_latency.avg),
            nanos_to_millis(result.transaction_latency.p50),
            nanos_to_millis(result.transaction_latency.p95),
            nanos_to_millis(result.transaction_latency.p99),
            nanos_to_millis(result.transaction_latency.max)
        ));
        output.push('\n');

        // Regression status
        let regression_status = if result.regression.detected {
            "FAILED"
        } else {
            "PASSED"
        };
        output.push_str(&format!("Regression: {}\n", regression_status));

        // Regression details if detected
        if result.regression.detected && !result.regression.description.is_empty() {
            output.push_str(&format!("  {}\n", result.regression.description));
        }

        output
    }

    /// Prints the text report directly to stdout.
    pub fn print(
        &self,
        workload_name: &str,
        config: &BenchmarkConfig,
        result: &BenchmarkResult,
    ) {
        print!("{}", self.generate(workload_name, config, result));
    }
}

impl Default for TextReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts nanoseconds to milliseconds with appropriate precision.
fn nanos_to_millis(ns: u64) -> f64 {
    ns as f64 / 1_000_000.0
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
    fn test_text_report_format() {
        let report = TextReport::new();
        let output = report.generate("oltp_point_select", &create_test_config(), &create_test_result());

        // Check header
        assert!(output.contains("SQLRustGo Benchmark: oltp_point_select"));

        // Check config section
        assert!(output.contains("Threads:       50"));
        assert!(output.contains("Warmup:        10s"));
        assert!(output.contains("Duration:      60s"));
        assert!(output.contains("Distribution:  zipfian (theta=0.9)"));
        assert!(output.contains("Seed:          42"));

        // Check results section
        assert!(output.contains("Total queries:     62345"));
        assert!(output.contains("QPS:               1039.1"));
        assert!(output.contains("TPS:               98.3"));

        // Check latency section
        assert!(output.contains("Statement Latency (ms):"));
        assert!(output.contains("Transaction Latency (ms):"));

        // Check regression status
        assert!(output.contains("Regression: PASSED"));
    }

    #[test]
    fn test_text_report_regression_failed() {
        let report = TextReport::new();
        let mut result = create_test_result();
        result.regression = RegressionReport {
            detected: true,
            description: "QPS regressed by 15.0%".to_string(),
        };

        let output = report.generate("oltp_point_select", &create_test_config(), &result);

        assert!(output.contains("Regression: FAILED"));
        assert!(output.contains("QPS regressed by 15.0%"));
    }

    #[test]
    fn test_text_report_uniform_distribution() {
        let report = TextReport::new();
        let mut config = create_test_config();
        config.distribution = Distribution::Uniform;

        let output = report.generate("oltp_point_select", &config, &create_test_result());

        assert!(output.contains("Distribution:  uniform"));
    }

    #[test]
    fn test_nanos_to_millis() {
        assert!((nanos_to_millis(1_000_000) - 1.0).abs() < 0.001);
        assert!((nanos_to_millis(500_000) - 0.5).abs() < 0.001);
        assert!((nanos_to_millis(1_000_000_000) - 1000.0).abs() < 0.001);
    }
}