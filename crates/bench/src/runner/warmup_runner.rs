//! Warmup runner for pre-heating the system
//!
//! Executes a warmup phase to stabilize the system before measurement.

use crate::workload::Workload;
use crate::db::Database;
use crate::BenchmarkConfig;
use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::rngs::SmallRng;
use rand::SeedableRng;

/// WarmupRunner - pre-heats the system before benchmarking
///
/// The warmup phase allows buffer pools, JIT compilers, and caches to stabilize
/// before the actual measurement phase begins.
pub struct WarmupRunner {
    config: BenchmarkConfig,
    workload: Arc<dyn Workload>,
}

impl WarmupRunner {
    /// Create a new WarmupRunner
    pub fn new(config: BenchmarkConfig, workload: Arc<dyn Workload>) -> Self {
        Self { config, workload }
    }

    /// Run the warmup phase
    ///
    /// # Arguments
    /// * `db` - Database connection for executing warmup operations
    ///
    /// # Returns
    /// Number of operations executed during warmup
    pub async fn run(&self, db: &dyn Database) -> anyhow::Result<u64> {
        let warmup_duration = Duration::from_secs(self.config.warmup_secs);
        let start = Instant::now();
        let mut count = 0u64;

        tracing::info!(
            "Starting warmup phase for {} seconds",
            self.config.warmup_secs
        );

        // Create a separate RNG for warmup to not affect the measurement RNG state
        let _rng = SmallRng::seed_from_u64(self.config.seed);

        while start.elapsed() < warmup_duration {
            // Execute warmup operations
            if let Err(e) = self.workload.execute(db).await {
                tracing::debug!("Warmup operation error (ignored): {}", e);
            }
            count += 1;

            // Periodically log progress
            if count % 1000 == 0 {
                let elapsed = start.elapsed().as_secs();
                tracing::debug!(
                    "Warmup progress: {} ops in {}s ({:.2} ops/s)",
                    count,
                    elapsed,
                    count as f64 / elapsed.max(1) as f64
                );
            }
        }

        tracing::info!(
            "Warmup completed: {} operations in {:.2}s",
            count,
            start.elapsed().as_secs_f64()
        );

        Ok(count)
    }
}

/// Run a simple warmup without database connection
///
/// This is a simplified version for testing or when no database is available.
pub async fn run_simple_warmup(secs: u64) -> u64 {
    let warmup_duration = Duration::from_secs(secs);
    let start = Instant::now();
    let mut count = 0u64;

    println!("[Warmup] {}s", secs);

    while start.elapsed() < warmup_duration {
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(1)).await;
        count += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workload::create_workload;
    use std::sync::Arc;

    #[test]
    fn test_warmup_runner_creation() {
        let config = BenchmarkConfig::default();
        let workload = create_workload("oltp_point_select", 1000);
        let runner = WarmupRunner::new(config, workload);

        assert_eq!(runner.config.warmup_secs, 10);
    }

    #[test]
    fn test_warmup_runner_config() {
        let config = BenchmarkConfig {
            warmup_secs: 5,
            duration_secs: 60,
            cooldown_secs: 3,
            threads: 4,
            tables: 2,
            dataset_size: 100_000,
            distribution: crate::Distribution::default(),
            seed: 42,
            connection_mode: crate::ConnectionMode::default(),
        };
        let workload = create_workload("oltp_point_select", 1000);
        let runner = WarmupRunner::new(config, workload);

        assert_eq!(runner.config.warmup_secs, 5);
    }

    #[tokio::test]
    async fn test_run_simple_warmup() {
        let count = run_simple_warmup(1).await;
        assert!(count > 0, "Warmup should execute some operations");
    }
}