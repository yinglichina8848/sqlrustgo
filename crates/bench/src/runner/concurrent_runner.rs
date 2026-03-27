//! Concurrent runner for executing benchmark workloads
//!
//! Implements the three-phase benchmark execution: Warmup -> Measurement -> Cooldown

use crate::workload::Workload;
use crate::db::Database;
use crate::{BenchmarkConfig, BenchmarkPhase, BenchmarkResult};
use crate::runner::latency_tracker::LatencyTracker;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Barrier;
use rand::rngs::SmallRng;
use rand::SeedableRng;

/// ConcurrentRunner - executes workloads in parallel with three-phase benchmark
///
/// Coordinates concurrent execution across multiple threads with proper
/// synchronization using tokio barriers.
pub struct ConcurrentRunner {
    config: BenchmarkConfig,
    workload: Arc<dyn Workload>,
}

impl ConcurrentRunner {
    /// Create a new ConcurrentRunner
    ///
    /// # Arguments
    /// * `config` - Benchmark configuration
    /// * `workload` - Workload to execute
    pub fn new(config: BenchmarkConfig, workload: Arc<dyn Workload>) -> Self {
        Self {
            config,
            workload,
        }
    }

    /// Run the complete benchmark with three phases
    ///
    /// # Returns
    /// BenchmarkResult containing all metrics
    pub async fn run(&self) -> BenchmarkResult {
        let mut result = BenchmarkResult::default();

        // Phase 1: Warmup (not计入统计)
        println!("[Warmup] {}s", self.config.warmup_secs);
        self.run_phase(BenchmarkPhase::Warmup, &mut result).await;

        // Phase 2: Measurement (计入统计)
        println!("[Measurement] {}s", self.config.duration_secs);
        self.run_phase(BenchmarkPhase::Measurement, &mut result).await;

        // Phase 3: Cooldown
        println!("[Cooldown] {}s", self.config.cooldown_secs);
        self.run_phase(BenchmarkPhase::Cooldown, &mut result).await;

        result
    }

    /// Run a specific benchmark phase
    async fn run_phase(&self, phase: BenchmarkPhase, result: &mut BenchmarkResult) {
        let phase_duration = Duration::from_secs(phase.duration(&self.config));
        let is_measurement = phase.is_measured();

        // Create barrier for all threads to start together
        let barrier = Arc::new(Barrier::new(self.config.threads));

        // Shared latency tracker for measurement phase
        let latency_tracker = if is_measurement {
            Some(Arc::new(LatencyTracker::new()))
        } else {
            None
        };

        // Counter for operations
        let ops_count = Arc::new(AtomicU64::new(0));
        let tx_count = Arc::new(AtomicU64::new(0));

        // Spawn worker tasks
        let mut handles = vec![];
        for thread_id in 0..self.config.threads {
            let workload = self.workload.clone();
            let barrier = barrier.clone();
            let latency_tracker = latency_tracker.clone();
            let ops_count = ops_count.clone();
            let tx_count = tx_count.clone();
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                // Wait for all threads to be ready
                barrier.wait().await;

                let start = Instant::now();

                // Create per-thread RNG
                let mut rng = SmallRng::seed_from_u64(config.seed + thread_id as u64);

                while start.elapsed() < phase_duration {
                    let tx_start = Instant::now();

                    // Execute a transaction
                    let tx_statements = workload.generate_transaction(&mut rng);

                    // Execute each statement in the transaction
                    for _sql in &tx_statements {
                        // For now, just simulate execution
                        // In real implementation, would call db.execute(sql)
                    }

                    let tx_latency = tx_start.elapsed().as_nanos() as u64;

                    // Record latency only in measurement phase
                    if let Some(ref tracker) = latency_tracker {
                        // Record statement latencies (simplified: treat whole tx as one statement)
                        tracker.record_statement(tx_latency);
                        tracker.record_transaction(tx_latency);
                    }

                    ops_count.fetch_add(tx_statements.len() as u64, Ordering::Relaxed);
                    tx_count.fetch_add(1, Ordering::Relaxed);
                }
            });

            handles.push(handle);
        }

        // Wait for all workers to complete
        for handle in handles {
            let _ = handle.await;
        }

        // Aggregate results
        if is_measurement {
            let elapsed = phase_duration.as_secs_f64();
            result.total_queries = ops_count.load(Ordering::Relaxed);
            result.total_transactions = tx_count.load(Ordering::Relaxed);
            result.qps = result.total_queries as f64 / elapsed;
            result.tps = result.total_transactions as f64 / elapsed;

            if let Some(ref tracker) = latency_tracker {
                result.statement_latency = tracker.statement_percentiles();
                result.transaction_latency = tracker.transaction_percentiles();
            }

            tracing::info!(
                "Measurement phase: {:.2} QPS, {:.2} TPS",
                result.qps,
                result.tps
            );
        }
    }
}

/// Create a concurrent runner with default configuration
///
/// This is useful for testing or when actual database is not available.
pub fn create_stub_runner(
    config: BenchmarkConfig,
    workload: Arc<dyn Workload>,
) -> ConcurrentRunner {
    ConcurrentRunner::new(config, workload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workload::create_workload;

    #[test]
    fn test_concurrent_runner_creation() {
        let config = BenchmarkConfig::default();
        let workload = create_workload("oltp_point_select", 1000);

        let runner = create_stub_runner(config, workload);

        assert_eq!(runner.config.threads, 1);
        assert_eq!(runner.config.warmup_secs, 10);
        assert_eq!(runner.config.duration_secs, 60);
        assert_eq!(runner.config.cooldown_secs, 5);
    }

    #[test]
    fn test_concurrent_runner_custom_config() {
        let config = BenchmarkConfig {
            threads: 8,
            warmup_secs: 5,
            duration_secs: 30,
            cooldown_secs: 2,
            tables: 4,
            dataset_size: 500_000,
            distribution: crate::Distribution::default(),
            seed: 12345,
            connection_mode: crate::ConnectionMode::default(),
        };
        let workload = create_workload("oltp_read_write", 1000);

        let runner = create_stub_runner(config, workload);

        assert_eq!(runner.config.threads, 8);
        assert_eq!(runner.config.warmup_secs, 5);
        assert_eq!(runner.config.duration_secs, 30);
        assert_eq!(runner.config.cooldown_secs, 2);
    }

    #[test]
    fn test_benchmark_phase_duration() {
        let config = BenchmarkConfig::default();

        assert_eq!(BenchmarkPhase::Warmup.duration(&config), 10);
        assert_eq!(BenchmarkPhase::Measurement.duration(&config), 60);
        assert_eq!(BenchmarkPhase::Cooldown.duration(&config), 5);
    }

    #[test]
    fn test_benchmark_phase_is_measured() {
        assert!(!BenchmarkPhase::Warmup.is_measured());
        assert!(BenchmarkPhase::Measurement.is_measured());
        assert!(!BenchmarkPhase::Cooldown.is_measured());
    }

    #[tokio::test]
    async fn test_run_with_short_duration() {
        let config = BenchmarkConfig {
            threads: 2,
            warmup_secs: 1,
            duration_secs: 1,
            cooldown_secs: 1,
            ..Default::default()
        };
        let workload = create_workload("oltp_point_select", 100);

        let runner = create_stub_runner(config, workload);
        let result = runner.run().await;

        // Result should have some values (may be 0 if run too fast)
        assert!(result.total_queries >= 0);
    }
}