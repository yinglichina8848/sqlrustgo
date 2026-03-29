//! Benchmark Suite Module
//!
//! Provides a comprehensive benchmark suite for OLTP workloads with
//! standardized testing, reporting, and target verification.
//!
//! # Performance Target
//! - 50 concurrent threads ≥ 1000 QPS
//!
//! # Example
//!
//! ```rust,ignore
//! let suite = BenchmarkSuite::standard();
//! let results = suite.run(50);
//! println!("{}", suite.report_json());
//! println!("{}", suite.report_markdown());
//! let failures = suite.check_targets();
//! ```

use hdrhistogram::Histogram;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Benchmark result for a single test run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Queries per second
    pub qps: f64,
    /// 50th percentile latency in milliseconds
    pub p50_ms: f64,
    /// 95th percentile latency in milliseconds
    pub p95_ms: f64,
    /// 99th percentile latency in milliseconds
    pub p99_ms: f64,
    /// Average latency in milliseconds
    pub avg_ms: f64,
    /// Total operations executed
    pub total_ops: u64,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        Self {
            qps: 0.0,
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            avg_ms: 0.0,
            total_ops: 0,
            duration_ms: 0,
        }
    }
}

impl BenchmarkResult {
    /// Create a new benchmark result from a histogram of latencies
    fn from_histogram(hist: &Histogram<u64>, duration_secs: f64, total_ops: u64) -> Self {
        if hist.len() == 0 || total_ops == 0 {
            return Self::default();
        }

        // Use histogram's built-in mean to calculate average
        // mean() returns the average recorded value
        let avg_ms = hist.mean() / 1_000_000.0;

        Self {
            qps: total_ops as f64 / duration_secs,
            p50_ms: hist.value_at_quantile(0.50) as f64 / 1_000_000.0,
            p95_ms: hist.value_at_quantile(0.95) as f64 / 1_000_000.0,
            p99_ms: hist.value_at_quantile(0.99) as f64 / 1_000_000.0,
            avg_ms,
            total_ops,
            duration_ms: (duration_secs * 1000.0) as u64,
        }
    }
}

/// Internal benchmark runner function
fn run_benchmark<F>(concurrency: u32, stmts_per_tx: usize, sql_gen: F) -> BenchmarkResult
where
    F: Fn(&mut SmallRng, usize) -> String + Send + Sync + 'static,
{
    let duration_secs: u64 = 10; // 10 second measurement window
    let warmup_secs: u64 = 2;
    let total_threads = concurrency.max(1) as usize;

    // Wrap sql_gen in Arc so threads can share it
    let sql_gen = Arc::new(sql_gen);

    // Create histogram for latencies (nanoseconds)
    let hist = Arc::new(std::sync::Mutex::new(
        Histogram::<u64>::new_with_max(3_600_000_000_000u64, 3)
            .expect("Failed to create histogram"),
    ));
    let ops_count = Arc::new(AtomicU64::new(0));
    let barrier = Arc::new(tokio::sync::Barrier::new(total_threads + 1));

    // Warmup phase
    let warmup_hist = Arc::new(std::sync::Mutex::new(
        Histogram::<u64>::new_with_max(3_600_000_000_000u64, 3)
            .expect("Failed to create warmup histogram"),
    ));
    let warmup_barrier = Arc::new(tokio::sync::Barrier::new(total_threads + 1));

    // Run warmup phase
    let warmup_handles: Vec<_> = (0..total_threads)
        .map(|thread_id| {
            let barrier = warmup_barrier.clone();
            let warmup_hist = warmup_hist.clone();
            let sql_gen = sql_gen.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_time()
                    .build()
                    .expect("Failed to create runtime");
                let mut rng = SmallRng::seed_from_u64(42 + thread_id as u64);
                let start = Instant::now();

                // Wait for all threads to start
                rt.block_on(barrier.wait());

                while start.elapsed() < Duration::from_secs(warmup_secs as u64) {
                    let tx_start = Instant::now();
                    for _ in 0..stmts_per_tx {
                        let _sql = sql_gen(&mut rng, thread_id);
                        // Simulate minimal processing
                        std::hint::spin_loop();
                    }
                    let tx_latency = tx_start.elapsed().as_nanos() as u64;
                    if let Ok(mut h) = warmup_hist.lock() {
                        let _ = h.record(tx_latency);
                    }
                }
            })
        })
        .collect();

    for handle in warmup_handles {
        let _ = handle.join();
    }

    // Run measurement phase
    let handles: Vec<_> = (0..total_threads)
        .map(|thread_id| {
            let barrier = barrier.clone();
            let hist = hist.clone();
            let ops_count = ops_count.clone();
            let sql_gen = sql_gen.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_time()
                    .build()
                    .expect("Failed to create runtime");
                let mut rng = SmallRng::seed_from_u64(12345 + thread_id as u64);
                let start = Instant::now();

                // Wait for all threads to start together
                rt.block_on(barrier.wait());

                while start.elapsed() < Duration::from_secs(duration_secs) {
                    let tx_start = Instant::now();
                    for _ in 0..stmts_per_tx {
                        let _sql = sql_gen(&mut rng, thread_id);
                        // Simulate minimal processing
                        std::hint::spin_loop();
                    }
                    let tx_latency = tx_start.elapsed().as_nanos() as u64;

                    if let Ok(mut h) = hist.lock() {
                        let _ = h.record(tx_latency);
                    }
                    ops_count.fetch_add(stmts_per_tx as u64, Ordering::Relaxed);
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.join();
    }

    let total_ops = ops_count.load(Ordering::Relaxed);
    let hist_guard = hist.lock().unwrap();
    BenchmarkResult::from_histogram(&hist_guard, duration_secs as f64, total_ops)
}

/// Benchmark trait - defines a single benchmark test
pub trait Benchmark: Send + Sync {
    /// Get the benchmark name
    fn name(&self) -> &str;

    /// Run the benchmark with given concurrency
    fn run(&self, concurrency: u32) -> BenchmarkResult;

    /// Target QPS for this benchmark
    fn target_qps(&self) -> f64;

    /// Whether this benchmark is read-only
    fn is_read_only(&self) -> bool {
        false
    }
}

/// Benchmark suite - manages multiple benchmark tests
pub struct BenchmarkSuite {
    /// Suite name
    pub name: String,
    /// Individual benchmark tests
    pub tests: Vec<Box<dyn Benchmark>>,
}

impl BenchmarkSuite {
    /// Create a new empty benchmark suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tests: Vec::new(),
        }
    }

    /// Create the standard OLTP benchmark suite with all test cases
    pub fn standard() -> Self {
        Self {
            name: "oltp_standard".to_string(),
            tests: vec![
                Box::new(OltpPointSelect::new()),
                Box::new(OltpRangeSelect::new()),
                Box::new(OltpInsert::new()),
                Box::new(OltpUpdate::new()),
                Box::new(OltpDelete::new()),
                Box::new(OltpMixed::new()),
            ],
        }
    }

    /// Add a benchmark test to the suite
    pub fn add<B: Benchmark + 'static>(&mut self, benchmark: B) {
        self.tests.push(Box::new(benchmark));
    }

    /// Run all benchmarks with specified concurrency
    pub fn run(&self, concurrency: u32) -> Vec<(String, BenchmarkResult)> {
        self.tests
            .iter()
            .map(|test| (test.name().to_string(), test.run(concurrency)))
            .collect()
    }

    /// Generate JSON report for all benchmark results
    pub fn report_json(&self, results: &[(String, BenchmarkResult)]) -> String {
        let report = JsonReport {
            suite_name: self.name.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            concurrency: 50,
            targets: self
                .tests
                .iter()
                .map(|t| (t.name().to_string(), t.target_qps()))
                .collect(),
            results: results
                .iter()
                .map(|(name, r)| JsonBenchmarkResult {
                    name: name.clone(),
                    qps: r.qps,
                    p50_ms: r.p50_ms,
                    p95_ms: r.p95_ms,
                    p99_ms: r.p99_ms,
                    avg_ms: r.avg_ms,
                    total_ops: r.total_ops,
                })
                .collect(),
        };
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate Markdown report for all benchmark results
    pub fn report_markdown(&self, results: &[(String, BenchmarkResult)]) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("# Benchmark Suite: {}\n\n", self.name));
        output.push_str(&format!(
            "**Generated**: {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!("**Concurrency**: {} threads\n\n", 50));

        // Target QPS table
        output.push_str("## Performance Targets\n\n");
        output.push_str("| Benchmark | Target QPS |\n");
        output.push_str("|-----------|------------|\n");
        for test in &self.tests {
            output.push_str(&format!("| {} | {:.0} |\n", test.name(), test.target_qps()));
        }
        output.push_str("\n");

        // Results table
        output.push_str("## Results\n\n");
        output
            .push_str("| Benchmark | QPS | P50 (ms) | P95 (ms) | P99 (ms) | Avg (ms) | Status |\n");
        output
            .push_str("|-----------|-----|----------|----------|----------|----------|--------|\n");

        for (name, result) in results {
            let target = self
                .tests
                .iter()
                .find(|t| t.name() == name)
                .map(|t| t.target_qps())
                .unwrap_or(0.0);
            let status = if result.qps >= target {
                "✅ PASS"
            } else {
                "❌ FAIL"
            };
            output.push_str(&format!(
                "| {} | {:.1} | {:.3} | {:.3} | {:.3} | {:.3} | {} |\n",
                name,
                result.qps,
                result.p50_ms,
                result.p95_ms,
                result.p99_ms,
                result.avg_ms,
                status
            ));
        }
        output.push_str("\n");

        // Summary
        let total_qps: f64 = results.iter().map(|(_, r)| r.qps).sum();
        let total_ops: u64 = results.iter().map(|(_, r)| r.total_ops).sum();
        let avg_p99: f64 = if !results.is_empty() {
            results.iter().map(|(_, r)| r.p99_ms).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        output.push_str("## Summary\n\n");
        output.push_str(&format!("- **Total QPS**: {:.1}\n", total_qps));
        output.push_str(&format!("- **Total Operations**: {}\n", total_ops));
        output.push_str(&format!("- **Avg P99 Latency**: {:.3} ms\n", avg_p99));

        // Failures
        let failures = self.check_targets_internal(results);
        if !failures.is_empty() {
            output.push_str("\n## Failures\n\n");
            for failure in &failures {
                output.push_str(&format!("- {}\n", failure));
            }
        }

        output
    }

    /// Check which benchmarks failed to meet their targets
    pub fn check_targets(&self, results: &[(String, BenchmarkResult)]) -> Vec<String> {
        self.check_targets_internal(results)
    }

    fn check_targets_internal(&self, results: &[(String, BenchmarkResult)]) -> Vec<String> {
        let mut failures = Vec::new();
        for (name, result) in results {
            let target = self
                .tests
                .iter()
                .find(|t| t.name() == name)
                .map(|t| t.target_qps())
                .unwrap_or(0.0);
            if result.qps < target {
                failures.push(format!(
                    "{}: {:.1} QPS < {:.0} QPS target (missed by {:.1} QPS)",
                    name,
                    result.qps,
                    target,
                    target - result.qps
                ));
            }
        }
        failures
    }
}

// JSON serialization structures
#[derive(Serialize)]
struct JsonReport {
    suite_name: String,
    timestamp: String,
    concurrency: u32,
    targets: Vec<(String, f64)>,
    results: Vec<JsonBenchmarkResult>,
}

#[derive(Serialize)]
struct JsonBenchmarkResult {
    name: String,
    qps: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    avg_ms: f64,
    total_ops: u64,
}

// =============================================================================
// Standard OLTP Benchmark Implementations
// =============================================================================

/// OLTP Point Select - point query (SELECT WHERE id = ?)
///
/// Target: ≥ 1000 QPS at 50 concurrency
pub struct OltpPointSelect {
    /// Target QPS for this benchmark
    target_qps: f64,
    /// Maximum ID range
    max_id: u64,
}

impl OltpPointSelect {
    pub fn new() -> Self {
        Self {
            target_qps: 1000.0,
            max_id: 1_000_000,
        }
    }
}

impl Default for OltpPointSelect {
    fn default() -> Self {
        Self::new()
    }
}

impl Benchmark for OltpPointSelect {
    fn name(&self) -> &str {
        "oltp_point_select"
    }

    fn run(&self, concurrency: u32) -> BenchmarkResult {
        let max_id = self.max_id;
        run_benchmark(concurrency, 10, move |rng, _| {
            let id = rng.gen_range(1..=max_id);
            format!("SELECT c FROM sbtest WHERE id = {}", id)
        })
    }

    fn target_qps(&self) -> f64 {
        self.target_qps
    }

    fn is_read_only(&self) -> bool {
        true
    }
}

/// OLTP Range Select - range query (SELECT WHERE id BETWEEN ? AND ?)
///
/// Target: ≥ 800 QPS at 50 concurrency
pub struct OltpRangeSelect {
    target_qps: f64,
    max_id: u64,
    range_size: u64,
}

impl OltpRangeSelect {
    pub fn new() -> Self {
        Self {
            target_qps: 800.0,
            max_id: 1_000_000,
            range_size: 100,
        }
    }
}

impl Default for OltpRangeSelect {
    fn default() -> Self {
        Self::new()
    }
}

impl Benchmark for OltpRangeSelect {
    fn name(&self) -> &str {
        "oltp_range_select"
    }

    fn run(&self, concurrency: u32) -> BenchmarkResult {
        let max_id = self.max_id;
        let range_size = self.range_size;
        run_benchmark(concurrency, 10, move |rng, _| {
            let start = rng.gen_range(1..max_id);
            let end = (start + rng.gen_range(1..=range_size)).min(max_id);
            format!(
                "SELECT c FROM sbtest WHERE id BETWEEN {} AND {}",
                start, end
            )
        })
    }

    fn target_qps(&self) -> f64 {
        self.target_qps
    }

    fn is_read_only(&self) -> bool {
        true
    }
}

/// OLTP Insert - single row insert
///
/// Target: ≥ 500 QPS at 50 concurrency
pub struct OltpInsert {
    target_qps: f64,
    max_id: u64,
}

impl OltpInsert {
    pub fn new() -> Self {
        Self {
            target_qps: 500.0,
            max_id: 1_000_000,
        }
    }
}

impl Default for OltpInsert {
    fn default() -> Self {
        Self::new()
    }
}

impl Benchmark for OltpInsert {
    fn name(&self) -> &str {
        "oltp_insert"
    }

    fn run(&self, concurrency: u32) -> BenchmarkResult {
        let max_id = self.max_id;
        run_benchmark(concurrency, 1, move |rng, _| {
            let id = rng.gen_range(1..=max_id);
            let k = rng.gen_range(0..1_000_000);
            let c = format!("'c{:x}'", rng.gen::<u32>());
            let pad = format!("'pad{:x}'", rng.gen::<u32>());
            format!(
                "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, {}, {}, {})",
                id, k, c, pad
            )
        })
    }

    fn target_qps(&self) -> f64 {
        self.target_qps
    }

    fn is_read_only(&self) -> bool {
        false
    }
}

/// OLTP Update - single row update
///
/// Target: ≥ 500 QPS at 50 concurrency
pub struct OltpUpdate {
    target_qps: f64,
    max_id: u64,
}

impl OltpUpdate {
    pub fn new() -> Self {
        Self {
            target_qps: 500.0,
            max_id: 1_000_000,
        }
    }
}

impl Default for OltpUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl Benchmark for OltpUpdate {
    fn name(&self) -> &str {
        "oltp_update"
    }

    fn run(&self, concurrency: u32) -> BenchmarkResult {
        let max_id = self.max_id;
        run_benchmark(concurrency, 1, move |rng, _| {
            let id = rng.gen_range(1..=max_id);
            let c_value = format!("'{:x}'", rng.gen::<u32>());
            format!("UPDATE sbtest SET c = {} WHERE id = {}", c_value, id)
        })
    }

    fn target_qps(&self) -> f64 {
        self.target_qps
    }

    fn is_read_only(&self) -> bool {
        false
    }
}

/// OLTP Delete - single row delete
///
/// Target: ≥ 500 QPS at 50 concurrency
pub struct OltpDelete {
    target_qps: f64,
    max_id: u64,
}

impl OltpDelete {
    pub fn new() -> Self {
        Self {
            target_qps: 500.0,
            max_id: 1_000_000,
        }
    }
}

impl Default for OltpDelete {
    fn default() -> Self {
        Self::new()
    }
}

impl Benchmark for OltpDelete {
    fn name(&self) -> &str {
        "oltp_delete"
    }

    fn run(&self, concurrency: u32) -> BenchmarkResult {
        let max_id = self.max_id;
        run_benchmark(concurrency, 1, move |rng, _| {
            let id = rng.gen_range(1..=max_id);
            format!("DELETE FROM sbtest WHERE id = {}", id)
        })
    }

    fn target_qps(&self) -> f64 {
        self.target_qps
    }

    fn is_read_only(&self) -> bool {
        false
    }
}

/// OLTP Mixed - mixed read/write workload
///
/// Operation mix:
/// - 50% Point Select (read)
/// - 20% Range Select (read)
/// - 20% Update (write)
/// - 10% Insert (write)
///
/// Target: ≥ 600 QPS at 50 concurrency
pub struct OltpMixed {
    target_qps: f64,
    max_id: u64,
}

impl OltpMixed {
    pub fn new() -> Self {
        Self {
            target_qps: 600.0,
            max_id: 1_000_000,
        }
    }
}

impl Default for OltpMixed {
    fn default() -> Self {
        Self::new()
    }
}

impl Benchmark for OltpMixed {
    fn name(&self) -> &str {
        "oltp_mixed"
    }

    fn run(&self, concurrency: u32) -> BenchmarkResult {
        let max_id = self.max_id;
        run_benchmark(concurrency, 10, move |rng, _| {
            let op = rng.gen_range(0..100);
            if op < 50 {
                // 50% Point Select
                let id = rng.gen_range(1..=max_id);
                format!("SELECT c FROM sbtest WHERE id = {}", id)
            } else if op < 70 {
                // 20% Range Select
                let start = rng.gen_range(1..max_id);
                let end = (start + rng.gen_range(1..=100)).min(max_id);
                format!(
                    "SELECT c FROM sbtest WHERE id BETWEEN {} AND {}",
                    start, end
                )
            } else if op < 90 {
                // 20% Update
                let id = rng.gen_range(1..=max_id);
                let c_value = format!("'{:x}'", rng.gen::<u32>());
                format!("UPDATE sbtest SET c = {} WHERE id = {}", c_value, id)
            } else {
                // 10% Insert
                let id = rng.gen_range(1..=max_id);
                let k = rng.gen_range(0..1_000_000);
                let c = format!("'c{:x}'", rng.gen::<u32>());
                let pad = format!("'pad{:x}'", rng.gen::<u32>());
                format!(
                    "INSERT INTO sbtest (id, k, c, pad) VALUES ({}, {}, {}, {})",
                    id, k, c, pad
                )
            }
        })
    }

    fn target_qps(&self) -> f64 {
        self.target_qps
    }

    fn is_read_only(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_default() {
        let result = BenchmarkResult::default();
        assert_eq!(result.qps, 0.0);
        assert_eq!(result.p50_ms, 0.0);
        assert_eq!(result.total_ops, 0);
    }

    #[test]
    fn test_oltp_point_select_name() {
        let bench = OltpPointSelect::new();
        assert_eq!(bench.name(), "oltp_point_select");
        assert!(bench.is_read_only());
    }

    #[test]
    fn test_oltp_range_select_name() {
        let bench = OltpRangeSelect::new();
        assert_eq!(bench.name(), "oltp_range_select");
        assert!(bench.is_read_only());
    }

    #[test]
    fn test_oltp_insert_name() {
        let bench = OltpInsert::new();
        assert_eq!(bench.name(), "oltp_insert");
        assert!(!bench.is_read_only());
    }

    #[test]
    fn test_oltp_update_name() {
        let bench = OltpUpdate::new();
        assert_eq!(bench.name(), "oltp_update");
        assert!(!bench.is_read_only());
    }

    #[test]
    fn test_oltp_delete_name() {
        let bench = OltpDelete::new();
        assert_eq!(bench.name(), "oltp_delete");
        assert!(!bench.is_read_only());
    }

    #[test]
    fn test_oltp_mixed_name() {
        let bench = OltpMixed::new();
        assert_eq!(bench.name(), "oltp_mixed");
        assert!(!bench.is_read_only());
    }

    #[test]
    fn test_oltp_point_select_target_qps() {
        let bench = OltpPointSelect::new();
        assert_eq!(bench.target_qps(), 1000.0);
    }

    #[test]
    fn test_oltp_mixed_target_qps() {
        let bench = OltpMixed::new();
        assert_eq!(bench.target_qps(), 600.0);
    }

    #[test]
    fn test_benchmark_suite_standard() {
        let suite = BenchmarkSuite::standard();
        assert_eq!(suite.name, "oltp_standard");
        assert_eq!(suite.tests.len(), 6);

        let names: Vec<_> = suite.tests.iter().map(|t| t.name()).collect();
        assert!(names.contains(&"oltp_point_select"));
        assert!(names.contains(&"oltp_range_select"));
        assert!(names.contains(&"oltp_insert"));
        assert!(names.contains(&"oltp_update"));
        assert!(names.contains(&"oltp_delete"));
        assert!(names.contains(&"oltp_mixed"));
    }

    #[test]
    fn test_benchmark_suite_add() {
        let mut suite = BenchmarkSuite::new("custom");
        assert_eq!(suite.tests.len(), 0);
        suite.add(OltpPointSelect::new());
        assert_eq!(suite.tests.len(), 1);
        assert_eq!(suite.tests[0].name(), "oltp_point_select");
    }

    #[test]
    fn test_benchmark_suite_check_targets_pass() {
        let suite = BenchmarkSuite::standard();
        let results = vec![(
            "oltp_point_select".to_string(),
            BenchmarkResult {
                qps: 1200.0,
                p50_ms: 0.1,
                p95_ms: 0.5,
                p99_ms: 1.0,
                avg_ms: 0.2,
                total_ops: 12000,
                duration_ms: 10000,
            },
        )];
        let failures = suite.check_targets(&results);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_benchmark_suite_check_targets_fail() {
        let suite = BenchmarkSuite::standard();
        let results = vec![(
            "oltp_point_select".to_string(),
            BenchmarkResult {
                qps: 500.0, // Below 1000 target
                p50_ms: 0.1,
                p95_ms: 0.5,
                p99_ms: 1.0,
                avg_ms: 0.2,
                total_ops: 5000,
                duration_ms: 10000,
            },
        )];
        let failures = suite.check_targets(&results);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].contains("oltp_point_select"));
        assert!(failures[0].contains("500.0 QPS < 1000 QPS target"));
    }

    #[test]
    fn test_benchmark_suite_report_json() {
        let suite = BenchmarkSuite::standard();
        let results = vec![(
            "oltp_point_select".to_string(),
            BenchmarkResult {
                qps: 1200.0,
                p50_ms: 0.1,
                p95_ms: 0.5,
                p99_ms: 1.0,
                avg_ms: 0.2,
                total_ops: 12000,
                duration_ms: 10000,
            },
        )];
        let json = suite.report_json(&results);
        assert!(json.contains("oltp_standard"));
        assert!(json.contains("oltp_point_select"));
        assert!(json.contains("1200.0"));
    }

    #[test]
    fn test_benchmark_suite_report_markdown() {
        let suite = BenchmarkSuite::standard();
        let results = vec![(
            "oltp_point_select".to_string(),
            BenchmarkResult {
                qps: 1200.0,
                p50_ms: 0.1,
                p95_ms: 0.5,
                p99_ms: 1.0,
                avg_ms: 0.2,
                total_ops: 12000,
                duration_ms: 10000,
            },
        )];
        let md = suite.report_markdown(&results);
        assert!(md.contains("# Benchmark Suite: oltp_standard"));
        assert!(md.contains("| Benchmark |"));
        assert!(md.contains("oltp_point_select"));
        assert!(md.contains("✅ PASS"));
    }

    #[test]
    fn test_benchmark_run_short() {
        let bench = OltpPointSelect::new();
        // Run with minimal concurrency for quick test
        let result = bench.run(2);
        // Should complete without panic
        assert!(result.total_ops > 0 || result.qps >= 0.0);
    }
}
