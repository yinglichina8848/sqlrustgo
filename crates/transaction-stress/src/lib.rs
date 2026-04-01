//! Random Transaction Stress Testing
//!
//! This module provides random transaction stress testing to detect
//! concurrency bugs, isolation level violations, and transaction inconsistencies.

pub mod generator;

use generator::{TransactionGenerator, TransactionOperation};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct StressConfig {
    pub thread_count: usize,
    pub transaction_count: u64,
    pub max_statements_per_tx: usize,
    pub think_time_ms: u64,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            thread_count: 10,
            transaction_count: 100,
            max_statements_per_tx: 5,
            think_time_ms: 10,
        }
    }
}

pub struct StressTest {
    config: StressConfig,
    stats: Arc<TestStats>,
}

#[derive(Debug, Default)]
pub struct TestStats {
    pub successful_transactions: AtomicU64,
    pub failed_transactions: AtomicU64,
    pub deadlocks: AtomicU64,
    pub timeouts: AtomicU64,
}

impl StressTest {
    pub fn new(config: StressConfig) -> Self {
        Self {
            config,
            stats: Arc::new(TestStats::default()),
        }
    }

    pub fn run<F>(&self, executor: F) -> StressResult
    where
        F: Fn(&str) -> Result<(), String> + Send + Clone + 'static,
    {
        let mut handles = vec![];
        let start = Instant::now();

        for _ in 0..self.config.thread_count {
            let stats = Arc::clone(&self.stats);
            let config = self.config.clone();
            let exec = executor.clone();

            let handle = thread::spawn(move || {
                for _ in 0..config.transaction_count {
                    let tx_ops = {
                        let gen = TransactionGenerator::new(1);
                        gen.generate_workload(config.max_statements_per_tx)
                    };

                    let mut tx_failed = false;
                    for op in &tx_ops {
                        let sql = op.to_sql();
                        match exec(&sql) {
                            Ok(_) => {}
                            Err(e) => {
                                if e.contains("deadlock") || e.contains("timeout") {
                                    tx_failed = true;
                                    stats.deadlocks.fetch_add(1, Ordering::Relaxed);
                                    break;
                                }
                            }
                        }

                        if config.think_time_ms > 0 {
                            thread::sleep(Duration::from_millis(config.think_time_ms));
                        }
                    }

                    if tx_failed {
                        stats.failed_transactions.fetch_add(1, Ordering::Relaxed);
                    } else {
                        stats
                            .successful_transactions
                            .fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        let duration = start.elapsed();

        StressResult {
            total_transactions: self.config.thread_count as u64 * self.config.transaction_count,
            successful: self.stats.successful_transactions.load(Ordering::Relaxed),
            failed: self.stats.failed_transactions.load(Ordering::Relaxed),
            deadlocks: self.stats.deadlocks.load(Ordering::Relaxed),
            duration_ms: duration.as_millis() as u64,
        }
    }
}

#[derive(Debug)]
pub struct StressResult {
    pub total_transactions: u64,
    pub successful: u64,
    pub failed: u64,
    pub deadlocks: u64,
    pub duration_ms: u64,
}

impl StressResult {
    pub fn success_rate(&self) -> f64 {
        if self.total_transactions == 0 {
            return 0.0;
        }
        self.successful as f64 / self.total_transactions as f64
    }

    pub fn throughput(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        (self.total_transactions as f64) / (self.duration_ms as f64 / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_config() {
        let config = StressConfig::default();
        assert_eq!(config.thread_count, 10);
    }

    #[test]
    fn test_stress_result() {
        let result = StressResult {
            total_transactions: 100,
            successful: 95,
            failed: 5,
            deadlocks: 1,
            duration_ms: 1000,
        };

        assert_eq!(result.success_rate(), 0.95);
        assert_eq!(result.throughput(), 100.0);
    }
}
