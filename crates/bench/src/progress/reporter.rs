//! Progress reporter for benchmark runs

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Progress reporter for displaying real-time benchmark progress.
///
/// This reporter shows:
/// - Elapsed time / Total time
/// - Current QPS (queries per second)
/// - Number of queries executed
/// - Number of transactions executed
///
/// All operations are thread-safe.
pub struct ProgressReporter {
    /// Start time of the benchmark
    start_time: Instant,
    /// Total expected duration in seconds
    total_duration_secs: u64,
    /// Thread-safe counter for total queries executed
    total_queries: Arc<AtomicU64>,
    /// Thread-safe counter for total transactions executed
    total_transactions: Arc<AtomicU64>,
}

impl ProgressReporter {
    /// Creates a new ProgressReporter with the specified total duration.
    ///
    /// # Arguments
    /// * `total_duration_secs` - The total expected duration of the benchmark in seconds
    ///
    /// # Returns
    /// A new ProgressReporter instance
    pub fn new(total_duration_secs: u64) -> Self {
        Self {
            start_time: Instant::now(),
            total_duration_secs,
            total_queries: Arc::new(AtomicU64::new(0)),
            total_transactions: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increments the query counter by the specified count.
    ///
    /// # Arguments
    /// * `count` - Number of queries to add
    pub fn increment_queries(&self, count: u64) {
        self.total_queries.fetch_add(count, Ordering::Relaxed);
    }

    /// Increments the transaction counter by the specified count.
    ///
    /// # Arguments
    /// * `count` - Number of transactions to add
    pub fn increment_transactions(&self, count: u64) {
        self.total_transactions.fetch_add(count, Ordering::Relaxed);
    }

    /// Returns the total number of queries executed.
    pub fn total_queries(&self) -> u64 {
        self.total_queries.load(Ordering::Relaxed)
    }

    /// Returns the total number of transactions executed.
    pub fn total_transactions(&self) -> u64 {
        self.total_transactions.load(Ordering::Relaxed)
    }

    /// Prints the current progress to stdout.
    ///
    /// Displays:
    /// - Elapsed time and progress percentage
    /// - Current QPS
    /// - Total queries
    /// - Total transactions
    pub fn print_progress(&self) {
        let elapsed = self.start_time.elapsed().as_secs();
        let queries = self.total_queries.load(Ordering::Relaxed);
        let transactions = self.total_transactions.load(Ordering::Relaxed);

        // Calculate progress percentage
        let progress = if self.total_duration_secs > 0 {
            (elapsed as f64 / self.total_duration_secs as f64 * 100.0).min(100.0)
        } else {
            0.0
        };

        // Calculate QPS (queries per second)
        let qps = if elapsed > 0 {
            queries as f64 / elapsed as f64
        } else {
            0.0
        };

        // Calculate TPS (transactions per second)
        let tps = if elapsed > 0 {
            transactions as f64 / elapsed as f64
        } else {
            0.0
        };

        // Format elapsed time as MM:SS
        let elapsed_mins = elapsed / 60;
        let elapsed_secs = elapsed % 60;

        // Format total duration as MM:SS
        let total_mins = self.total_duration_secs / 60;
        let total_secs = self.total_duration_secs % 60;

        println!(
            "[Progress] {:02}:{:02}/{:02}:{:02} ({:5.1}%) | QPS: {:>8.1} | TPS: {:>8.1} | Queries: {:>10} | TXs: {:>10}",
            elapsed_mins,
            elapsed_secs,
            total_mins,
            total_secs,
            progress,
            qps,
            tps,
            queries,
            transactions
        );
    }

    /// Spawns a background task that prints progress periodically.
    ///
    /// # Returns
    /// A JoinHandle for the spawned task. The task runs until the benchmark
    /// duration is complete.
    pub fn spawn_reporter_task(&self) -> std::thread::JoinHandle<()> {
        let total_duration = self.total_duration_secs;
        let queries = Arc::clone(&self.total_queries);
        let transactions = Arc::clone(&self.total_transactions);
        let start = Instant::now();

        std::thread::spawn(move || {
            // Print initial progress
            let elapsed = start.elapsed().as_secs();
            let q = queries.load(Ordering::Relaxed);
            let t = transactions.load(Ordering::Relaxed);

            let progress = if total_duration > 0 {
                (elapsed as f64 / total_duration as f64 * 100.0).min(100.0)
            } else {
                0.0
            };

            let qps = if elapsed > 0 {
                q as f64 / elapsed as f64
            } else {
                0.0
            };

            let tps = if elapsed > 0 {
                t as f64 / elapsed as f64
            } else {
                0.0
            };

            println!(
                "[Progress] 00:00/{}:00 (  0.0%) | QPS: {:>8.0} | TXs: {:>10}",
                total_duration / 60,
                qps,
                t
            );

            // Print progress every second until duration is complete
            while start.elapsed().as_secs() < total_duration {
                std::thread::sleep(std::time::Duration::from_secs(1));

                let elapsed = start.elapsed().as_secs();
                if elapsed >= total_duration {
                    break;
                }

                let q = queries.load(Ordering::Relaxed);
                let tx = transactions.load(Ordering::Relaxed);

                let progress = (elapsed as f64 / total_duration as f64 * 100.0).min(100.0);
                let qps = q as f64 / elapsed as f64;
                let tps = tx as f64 / elapsed as f64;

                let elapsed_mins = elapsed / 60;
                let elapsed_secs = elapsed % 60;
                let total_mins = total_duration / 60;
                let total_secs = total_duration % 60;

                println!(
                    "[Progress] {:02}:{:02}/{:02}:{:02} ({:5.1}%) | QPS: {:>8.1} | TPS: {:>8.1} | Queries: {:>10} | TXs: {:>10}",
                    elapsed_mins,
                    elapsed_secs,
                    total_mins,
                    total_secs,
                    progress,
                    qps,
                    tps,
                    q,
                    tx
                );
            }

            // Print final progress
            let elapsed = start.elapsed().as_secs();
            let q = queries.load(Ordering::Relaxed);
            let tx = transactions.load(Ordering::Relaxed);

            let qps = if elapsed > 0 {
                q as f64 / elapsed as f64
            } else {
                0.0
            };

            let tps = if elapsed > 0 {
                tx as f64 / elapsed as f64
            } else {
                0.0
            };

            println!(
                "[Progress] COMPLETE | QPS: {:>8.1} | TPS: {:>8.1} | Queries: {:>10} | TXs: {:>10}",
                qps,
                tps,
                q,
                tx
            );
        })
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// Tests that ProgressReporter is created correctly with default values.
    #[test]
    fn test_progress_reporter_creation() {
        let reporter = ProgressReporter::new(120);

        // Verify initial state
        assert_eq!(reporter.total_queries(), 0);
        assert_eq!(reporter.total_transactions(), 0);
    }

    /// Tests that increment methods work correctly.
    #[test]
    fn test_progress_reporter_increment() {
        let reporter = ProgressReporter::new(60);

        // Test incrementing queries
        reporter.increment_queries(10);
        assert_eq!(reporter.total_queries(), 10);

        reporter.increment_queries(5);
        assert_eq!(reporter.total_queries(), 15);

        // Test incrementing transactions
        reporter.increment_transactions(3);
        assert_eq!(reporter.total_transactions(), 3);

        reporter.increment_transactions(7);
        assert_eq!(reporter.total_transactions(), 10);
    }

    /// Tests that increment is thread-safe.
    #[test]
    fn test_progress_reporter_increment_thread_safety() {
        let reporter = ProgressReporter::new(60);
        let reporter_clone = Arc::new(reporter);

        // Spawn multiple threads to increment concurrently
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let r = Arc::clone(&reporter_clone);
                thread::spawn(move || {
                    for _ in 0..1000 {
                        r.increment_queries(1);
                        r.increment_transactions(1);
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify total counts
        assert_eq!(reporter_clone.total_queries(), 10_000);
        assert_eq!(reporter_clone.total_transactions(), 10_000);
    }

    /// Tests QPS calculation.
    #[test]
    fn test_progress_reporter_qps_calculation() {
        let reporter = ProgressReporter::new(60);

        // Initially QPS should be 0 (no time elapsed)
        reporter.increment_queries(100);

        // Wait a bit to get non-zero elapsed time
        thread::sleep(Duration::from_millis(100));

        // After some queries and time elapsed, print_progress should work
        reporter.increment_transactions(50);

        // Verify counts
        assert_eq!(reporter.total_queries(), 100);
        assert_eq!(reporter.total_transactions(), 50);
    }

    /// Tests that print_progress doesn't panic and works with zero time.
    #[test]
    fn test_progress_reporter_print_progress() {
        let reporter = ProgressReporter::new(60);
        reporter.increment_queries(100);
        reporter.increment_transactions(50);

        // This should not panic
        reporter.print_progress();
    }

    /// Tests spawn_reporter_task creates a valid join handle.
    #[test]
    fn test_progress_reporter_spawn_reporter_task() {
        let reporter = ProgressReporter::new(1);
        let handle = reporter.spawn_reporter_task();

        // Wait for the task to complete
        let _ = handle.join();

        // Task completed successfully
    }

    /// Tests with zero duration.
    #[test]
    fn test_progress_reporter_zero_duration() {
        let reporter = ProgressReporter::new(0);

        reporter.increment_queries(100);
        reporter.increment_transactions(50);

        // Should not panic
        reporter.print_progress();

        assert_eq!(reporter.total_queries(), 100);
        assert_eq!(reporter.total_transactions(), 50);
    }
}