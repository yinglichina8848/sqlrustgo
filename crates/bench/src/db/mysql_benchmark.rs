use crate::db::mysql::MySqlDB;
use crate::db::Database;
use crate::metrics::latency::LatencyRecorder;
use serde::{Deserialize, Serialize};

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

pub struct MySqlBenchmark {
    addr: String,
}

#[allow(dead_code)]
impl MySqlBenchmark {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
        }
    }

    pub async fn run_reads(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let mysql = MySqlDB::new(&self.addr).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            mysql.read(i as usize).await?;
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "mysql".to_string(),
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

    pub async fn run_writes(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let mysql = MySqlDB::new(&self.addr).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            mysql.insert(i as usize).await?;
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "mysql".to_string(),
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

    pub async fn run_mixed(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let mysql = MySqlDB::new(&self.addr).await?;
        let latency_recorder = LatencyRecorder::new();
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            match i % 3 {
                0 => mysql.read(i as usize).await?,
                1 => mysql.update(i as usize).await?,
                _ => mysql.insert(i as usize).await?,
            }
            latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "mysql".to_string(),
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
