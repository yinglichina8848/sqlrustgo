//! SQLRustGo Benchmark Library
//!
//! Provides benchmark functionality for SQLRustGo.

pub mod analysis;
pub mod benchmark_runner;
pub mod cli;
pub mod config;
pub mod db;
pub mod dataset;
pub mod distribution;
pub mod memory;
pub mod metrics;
pub mod mysql_config;
pub mod progress;
pub mod report;
pub mod runner;
pub mod workload;

/// Benchmark phase enum representing different phases of a benchmark run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkPhase {
    /// Warmup phase - allows the system to stabilize before measurement.
    Warmup,
    /// Measurement phase - actual benchmark measurements are taken.
    Measurement,
    /// Cooldown phase - allows system to settle after benchmark.
    Cooldown,
}

impl BenchmarkPhase {
    /// Returns the duration for this phase based on the given configuration.
    pub fn duration(&self, config: &BenchmarkConfig) -> u64 {
        match self {
            BenchmarkPhase::Warmup => config.warmup_secs,
            BenchmarkPhase::Measurement => config.duration_secs,
            BenchmarkPhase::Cooldown => config.cooldown_secs,
        }
    }

    /// Returns true if this is the measurement phase.
    pub fn is_measured(&self) -> bool {
        matches!(self, BenchmarkPhase::Measurement)
    }
}

/// Distribution type for generating workload data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distribution {
    /// Uniform distribution - equal probability for all values.
    Uniform,
    /// Zipfian distribution - skewed towards popular values.
    /// theta parameter controls the skew (higher = more skewed, typical value 0.9).
    Zipfian { theta: f64 },
}

impl Default for Distribution {
    fn default() -> Self {
        Distribution::Uniform
    }
}

/// Connection mode for database connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionMode {
    /// Each thread gets its own connection (recommended).
    #[default]
    PerThread,
    /// Threads share a pool of connections.
    SharedPool,
}

/// Configuration for benchmark execution.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkConfig {
    /// Number of threads for the benchmark.
    pub threads: usize,
    /// Warmup duration in seconds.
    pub warmup_secs: u64,
    /// Measurement duration in seconds.
    pub duration_secs: u64,
    /// Cooldown duration in seconds.
    pub cooldown_secs: u64,
    /// Number of tables to use.
    pub tables: usize,
    /// Total dataset size in number of rows.
    pub dataset_size: usize,
    /// Data distribution pattern.
    pub distribution: Distribution,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Connection mode.
    pub connection_mode: ConnectionMode,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            threads: 1,
            warmup_secs: 10,
            duration_secs: 60,
            cooldown_secs: 5,
            tables: 1,
            dataset_size: 1_000_000,
            distribution: Distribution::default(),
            seed: 12345,
            connection_mode: ConnectionMode::default(),
        }
    }
}

/// Latency percentiles for benchmark results.
#[derive(Debug, Clone, Copy, Default)]
pub struct Percentiles {
    /// Minimum latency in microseconds.
    pub min: u64,
    /// Average latency in microseconds.
    pub avg: u64,
    /// 50th percentile latency (median) in microseconds.
    pub p50: u64,
    /// 95th percentile latency in microseconds.
    pub p95: u64,
    /// 99th percentile latency in microseconds.
    pub p99: u64,
    /// Maximum latency in microseconds.
    pub max: u64,
}

/// Regression report for comparing benchmark results.
#[derive(Debug, Clone, Default)]
pub struct RegressionReport {
    /// Whether regression was detected.
    pub detected: bool,
    /// Description of the regression.
    pub description: String,
}

/// Complete benchmark result.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Total number of queries executed.
    pub total_queries: u64,
    /// Total number of transactions executed.
    pub total_transactions: u64,
    /// Queries per second.
    pub qps: f64,
    /// Standard deviation of QPS.
    pub qps_stddev: f64,
    /// Transactions per second.
    pub tps: f64,
    /// Statement latency percentiles.
    pub statement_latency: Percentiles,
    /// Transaction latency percentiles.
    pub transaction_latency: Percentiles,
    /// Regression analysis report.
    pub regression: RegressionReport,
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        BenchmarkResult {
            total_queries: 0,
            total_transactions: 0,
            qps: 0.0,
            qps_stddev: 0.0,
            tps: 0.0,
            statement_latency: Percentiles::default(),
            transaction_latency: Percentiles::default(),
            regression: RegressionReport::default(),
        }
    }
}
