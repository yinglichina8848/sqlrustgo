//! SQLite Benchmark Runner
//!
//! Provides benchmark functionality for SQLite with comparison support.

use crate::db::sqlite::SqliteDB;
use crate::metrics::latency::LatencyRecorder;
use serde::{Deserialize, Serialize};

use crate::db::Database;

/// Benchmark result for SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub db_name: String,
    pub total_time_ms: u64,
    pub operations: u64,
    pub qps: f64,
    pub latency_stats: LatencyStatsMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStatsMs {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

impl Default for LatencyStatsMs {
    fn default() -> Self {
        Self {
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
        }
    }
}

/// SQLite benchmark runner
pub struct SQLiteBenchmark {
    path: String,
    scale: usize,
}

#[allow(dead_code)]
impl SQLiteBenchmark {
    pub fn new(path: &str, scale: usize) -> Self {
        Self {
            path: path.to_string(),
            scale,
        }
    }

    pub async fn run_reads(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let db = SqliteDB::new(&self.path, self.scale).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            db.read(i as usize).await?;
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "sqlite".to_string(),
            total_time_ms: elapsed,
            operations,
            qps: operations as f64 / (elapsed as f64 / 1000.0),
            latency_stats: LatencyStatsMs {
                p50_ms: stats.p50 as f64 / 1000.0,
                p95_ms: stats.p95 as f64 / 1000.0,
                p99_ms: stats.p99 as f64 / 1000.0,
            },
        })
    }

    pub async fn run_updates(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let db = SqliteDB::new(&self.path, self.scale).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            db.update(i as usize).await?;
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "sqlite".to_string(),
            total_time_ms: elapsed,
            operations,
            qps: operations as f64 / (elapsed as f64 / 1000.0),
            latency_stats: LatencyStatsMs {
                p50_ms: stats.p50 as f64 / 1000.0,
                p95_ms: stats.p95 as f64 / 1000.0,
                p99_ms: stats.p99 as f64 / 1000.0,
            },
        })
    }
}

impl BenchmarkResult {
    #[allow(dead_code)]
    pub fn print(&self) {
        println!("=== {} Benchmark Results ===", self.db_name);
        println!("Total Time: {} ms", self.total_time_ms);
        println!("Operations: {}", self.operations);
        println!("QPS: {:.2}", self.qps);
        println!("Latency (ms):");
        println!("  P50: {:.3}", self.latency_stats.p50_ms);
        println!("  P95: {:.3}", self.latency_stats.p95_ms);
        println!("  P99: {:.3}", self.latency_stats.p99_ms);
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_stats_default() {
        let stats = LatencyStatsMs::default();
        assert_eq!(stats.p50_ms, 0.0);
        assert_eq!(stats.p95_ms, 0.0);
        assert_eq!(stats.p99_ms, 0.0);
    }

    #[test]
    fn test_benchmark_result_serialization() {
        let result = BenchmarkResult {
            db_name: "sqlite".to_string(),
            total_time_ms: 100,
            operations: 10,
            qps: 100.0,
            latency_stats: LatencyStatsMs {
                p50_ms: 1.0,
                p95_ms: 5.0,
                p99_ms: 10.0,
            },
        };

        let json = result.to_json();
        assert!(json.contains("sqlite"));
        assert!(json.contains("100.0"));
    }

    #[test]
    fn test_sqlite_benchmark_creation() {
        let _bench = SQLiteBenchmark::new(":memory:", 100);
        assert!(true);
    }
}
