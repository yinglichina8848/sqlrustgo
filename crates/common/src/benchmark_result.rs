//! Benchmark Result Module
//!
//! Provides unified benchmark result structure with JSON serialization support.

use crate::latency_stats::LatencyStats;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", now)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub name: String,
    pub version: String,
    pub timestamp: String,
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    pub total_operations: u64,
    pub duration_ms: u64,
    pub ops_per_sec: f64,
    pub mb_per_sec: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total: u64,
    pub by_type: std::collections::HashMap<String, u64>,
}

impl ErrorStats {
    pub fn new() -> Self {
        Self {
            total: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    pub fn record(&mut self, error_type: impl Into<String>) {
        let et = error_type.into();
        *self.by_type.entry(et.clone()).or_insert(0) += 1;
        self.total += 1;
    }
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub metadata: BenchmarkMetadata,
    pub latency: Option<LatencyStats>,
    pub throughput: Option<ThroughputStats>,
    pub errors: ErrorStats,
    pub success: bool,
    pub message: Option<String>,
}

impl BenchmarkResult {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            metadata: BenchmarkMetadata {
                name: name.into(),
                version: version.into(),
                timestamp: current_timestamp(),
                environment: None,
            },
            latency: None,
            throughput: None,
            errors: ErrorStats::new(),
            success: true,
            message: None,
        }
    }

    pub fn with_latency(mut self, latency: LatencyStats) -> Self {
        self.latency = Some(latency);
        self
    }

    pub fn with_throughput(mut self, throughput: ThroughputStats) -> Self {
        self.throughput = Some(throughput);
        self
    }

    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.metadata.environment = Some(env.into());
        self
    }

    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
    }

    pub fn record_error(&mut self, error_type: impl Into<String>) {
        self.errors.record(error_type);
        self.success = false;
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        Self::new("benchmark", "1.0.0")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub results: Vec<BenchmarkResult>,
    pub summary: BenchmarkSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration_ms: u64,
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            summary: BenchmarkSummary {
                total_tests: 0,
                passed: 0,
                failed: 0,
                duration_ms: 0,
            },
        }
    }

    pub fn add_result(&mut self, result: BenchmarkResult) {
        if result.success {
            self.summary.passed += 1;
        } else {
            self.summary.failed += 1;
        }
        self.results.push(result);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_new() {
        let result = BenchmarkResult::new("test", "1.0.0");
        assert_eq!(result.metadata.name, "test");
        assert_eq!(result.metadata.version, "1.0.0");
        assert!(result.success);
    }

    #[test]
    fn test_benchmark_result_with_latency() {
        let mut latency = LatencyStats::new();
        latency.record(100);
        latency.record(200);
        latency.record(300);

        let result = BenchmarkResult::new("test", "1.0.0").with_latency(latency);

        assert!(result.latency.is_some());
    }

    #[test]
    fn test_benchmark_result_json() {
        let result = BenchmarkResult::new("test", "1.0.0");
        let json = result.to_json();
        assert!(json.contains("test"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_error_stats() {
        let mut stats = ErrorStats::new();
        stats.record("timeout");
        stats.record("timeout");
        stats.record("connection_error");

        assert_eq!(stats.total, 3);
        assert_eq!(stats.by_type.get("timeout"), Some(&2));
        assert_eq!(stats.by_type.get("connection_error"), Some(&1));
    }
}
