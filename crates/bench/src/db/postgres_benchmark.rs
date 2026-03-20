//! PostgreSQL Benchmark Runner
//!
//! Provides benchmark functionality for PostgreSQL with comparison support.

use crate::db::postgres::PostgresDB;
use crate::db::Database;
use crate::metrics::latency::LatencyRecorder;
use serde::{Deserialize, Serialize};

/// Benchmark result for a single database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Database name
    pub db_name: String,
    /// Total execution time in milliseconds
    pub total_time_ms: u64,
    /// Number of operations
    pub operations: u64,
    /// Queries per second
    pub qps: f64,
    /// Latency statistics in milliseconds
    pub latency_stats: LatencyStatsMs,
}

/// Latency statistics in milliseconds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStatsMs {
    /// 50th percentile in milliseconds
    pub p50_ms: f64,
    /// 95th percentile in milliseconds
    pub p95_ms: f64,
    /// 99th percentile in milliseconds
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

/// PostgreSQL benchmark runner
pub struct PostgresBenchmark {
    conn_str: String,
}

#[allow(dead_code)]
impl PostgresBenchmark {
    /// Create a new PostgreSQL benchmark runner
    pub fn new(conn_str: &str) -> Self {
        Self {
            conn_str: conn_str.to_string(),
        }
    }

    /// Run read benchmark
    pub async fn run_reads(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let pg = PostgresDB::new(&self.conn_str).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            pg.read(i as usize).await?;
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "postgres".to_string(),
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

    /// Run update benchmark
    pub async fn run_updates(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let pg = PostgresDB::new(&self.conn_str).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            pg.update(i as usize).await?;
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "postgres".to_string(),
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

    /// Run mixed workload benchmark
    pub async fn run_mixed(&self, operations: u64, read_ratio: f64) -> anyhow::Result<BenchmarkResult> {
        let pg = PostgresDB::new(&self.conn_str).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            if (i as f64 / operations as f64) < read_ratio {
                pg.read(i as usize).await?;
            } else {
                pg.update(i as usize).await?;
            }
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "postgres".to_string(),
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
            db_name: "test".to_string(),
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
        assert!(json.contains("test"));
        assert!(json.contains("100.0"));
    }

    #[test]
    fn test_postgres_benchmark_creation() {
        let _bench = PostgresBenchmark::new("host=localhost user=test");
        assert!(true);
    }
}
