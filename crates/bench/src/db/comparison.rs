//! Database Benchmark Comparison Module
//!
//! Provides comparison functionality between different databases.

use crate::db::postgres_benchmark::PostgresBenchmark;
use crate::db::sqlite_benchmark::SQLiteBenchmark;
use serde::{Deserialize, Serialize};

/// Comparison result between two databases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub sqlite_result: Option<crate::db::sqlite_benchmark::BenchmarkResult>,
    pub postgres_result: Option<crate::db::postgres_benchmark::BenchmarkResult>,
    pub winner: String,
    pub speedup: f64,
}

/// Compare SQLite and PostgreSQL benchmarks
#[allow(dead_code)]
pub struct BenchmarkComparison {
    sqlite_path: String,
    sqlite_scale: usize,
    pg_conn_str: String,
}

#[allow(dead_code)]
impl BenchmarkComparison {
    pub fn new(sqlite_path: &str, sqlite_scale: usize, pg_conn_str: &str) -> Self {
        Self {
            sqlite_path: sqlite_path.to_string(),
            sqlite_scale,
            pg_conn_str: pg_conn_str.to_string(),
        }
    }

    pub async fn compare_reads(&self, operations: u64) -> anyhow::Result<ComparisonResult> {
        let sqlite_bench = SQLiteBenchmark::new(&self.sqlite_path, self.sqlite_scale);
        let postgres_bench = PostgresBenchmark::new(&self.pg_conn_str);

        let sqlite_result = sqlite_bench.run_reads(operations).await.ok();
        let postgres_result = postgres_bench.run_reads(operations).await.ok();

        let (winner, speedup) = if let (Some(sqlite), Some(pg)) = (&sqlite_result, &postgres_result) {
            if sqlite.qps > 0.0 && pg.qps > 0.0 {
                let speedup = pg.qps / sqlite.qps;
                if speedup > 1.0 {
                    ("postgres".to_string(), speedup)
                } else {
                    ("sqlite".to_string(), 1.0 / speedup)
                }
            } else {
                ("unknown".to_string(), 0.0)
            }
        } else {
            ("unknown".to_string(), 0.0)
        };

        Ok(ComparisonResult {
            sqlite_result,
            postgres_result,
            winner,
            speedup,
        })
    }
}

impl ComparisonResult {
    #[allow(dead_code)]
    pub fn print(&self) {
        println!("=== Benchmark Comparison ===");
        if let Some(ref sqlite) = self.sqlite_result {
            println!("SQLite:");
            println!("  QPS: {:.2}", sqlite.qps);
            println!("  Latency P99: {:.3} ms", sqlite.latency_stats.p99_ms);
        }
        if let Some(ref pg) = self.postgres_result {
            println!("PostgreSQL:");
            println!("  QPS: {:.2}", pg.qps);
            println!("  Latency P99: {:.3} ms", pg.latency_stats.p99_ms);
        }
        println!("Winner: {}", self.winner);
        if self.speedup > 0.0 {
            println!("Speedup: {:.2}x", self.speedup);
        }
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
    fn test_comparison_result_default() {
        let result = ComparisonResult {
            sqlite_result: None,
            postgres_result: None,
            winner: "unknown".to_string(),
            speedup: 0.0,
        };
        assert_eq!(result.winner, "unknown");
    }

    #[tokio::test]
    async fn test_sqlite_benchmark_only() {
        let bench = SQLiteBenchmark::new(":memory:", 100);
        let result = bench.run_reads(10).await.unwrap();
        assert_eq!(result.db_name, "sqlite");
        assert_eq!(result.operations, 10);
    }

    #[test]
    fn test_comparison_with_sqlite_only() {
        use crate::db::sqlite_benchmark::BenchmarkResult;
        
        let sqlite_result = BenchmarkResult {
            db_name: "sqlite".to_string(),
            total_time_ms: 100,
            operations: 10,
            qps: 100.0,
            latency_stats: crate::db::sqlite_benchmark::LatencyStatsMs {
                p50_ms: 1.0,
                p95_ms: 5.0,
                p99_ms: 10.0,
            },
        };

        let comparison = ComparisonResult {
            sqlite_result: Some(sqlite_result),
            postgres_result: None,
            winner: "sqlite".to_string(),
            speedup: 1.0,
        };

        assert_eq!(comparison.winner, "sqlite");
    }
}
