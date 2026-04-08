//! Operator Profiling Module
//!
//! Provides per-operator performance profiling for query execution.
//! This is a key differentiation feature - MySQL doesn't expose
//! operator-level timing in a user-friendly way.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Represents profiling data for a single operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorProfile {
    /// Operator name
    pub operator_name: String,
    /// Operator type
    pub operator_type: String,
    /// Total time spent in this operator (nanoseconds)
    pub total_time_ns: u64,
    /// Number of times this operator was executed
    pub execution_count: u64,
    /// Average time per execution (nanoseconds)
    pub avg_time_ns: u64,
    /// Minimum execution time (nanoseconds)
    pub min_time_ns: u64,
    /// Maximum execution time (nanoseconds)
    pub max_time_ns: u64,
    /// Total rows processed
    pub rows_processed: u64,
    /// Total batches processed (vectorized)
    pub batches_processed: u64,
    /// Rows per second (calculated)
    pub rows_per_second: f64,
    /// Custom metrics
    pub metrics: HashMap<String, f64>,
}

impl OperatorProfile {
    pub fn new(name: &str, operator_type: &str) -> Self {
        Self {
            operator_name: name.to_string(),
            operator_type: operator_type.to_string(),
            total_time_ns: 0,
            execution_count: 0,
            avg_time_ns: 0,
            min_time_ns: u64::MAX,
            max_time_ns: 0,
            rows_processed: 0,
            batches_processed: 0,
            rows_per_second: 0.0,
            metrics: HashMap::new(),
        }
    }

    /// Record an execution of this operator
    pub fn record_execution(&mut self, duration_ns: u64, rows: usize, batches: usize) {
        self.total_time_ns += duration_ns;
        self.execution_count += 1;

        if duration_ns < self.min_time_ns {
            self.min_time_ns = duration_ns;
        }
        if duration_ns > self.max_time_ns {
            self.max_time_ns = duration_ns;
        }

        if self.execution_count > 0 {
            self.avg_time_ns = self.total_time_ns / self.execution_count;
        }

        self.rows_processed += rows as u64;
        self.batches_processed += batches as u64;

        // Calculate rows per second
        if self.total_time_ns > 0 {
            let seconds = self.total_time_ns as f64 / 1_000_000_000.0;
            self.rows_per_second = self.rows_processed as f64 / seconds;
        }
    }

    /// Add a custom metric
    pub fn add_metric(&mut self, key: &str, value: f64) {
        self.metrics.insert(key.to_string(), value);
    }

    /// Reset profiling data
    pub fn reset(&mut self) {
        self.total_time_ns = 0;
        self.execution_count = 0;
        self.avg_time_ns = 0;
        self.min_time_ns = u64::MAX;
        self.max_time_ns = 0;
        self.rows_processed = 0;
        self.batches_processed = 0;
        self.rows_per_second = 0.0;
        self.metrics.clear();
    }

    /// Get human-readable timing
    pub fn format_total_time(&self) -> String {
        format_duration(self.total_time_ns)
    }

    pub fn format_avg_time(&self) -> String {
        format_duration(self.avg_time_ns)
    }

    pub fn format_min_time(&self) -> String {
        format_duration(self.min_time_ns)
    }

    pub fn format_max_time(&self) -> String {
        format_duration(self.max_time_ns)
    }

    /// Convert to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Format nanoseconds to human readable string
fn format_duration(ns: u64) -> String {
    if ns < 1_000 {
        format!("{}ns", ns)
    } else if ns < 1_000_000 {
        format!("{:.1}µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2}ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", ns as f64 / 1_000_000_000.0)
    }
}

/// Query-level profiling summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryProfile {
    /// Query ID
    pub query_id: String,
    /// SQL query text
    pub sql: String,
    /// Total query execution time
    pub total_time_ns: u64,
    /// Operator profiles
    pub operators: Vec<OperatorProfile>,
    /// Peak memory usage (estimated)
    pub peak_memory_bytes: u64,
    /// Whether query completed successfully
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl QueryProfile {
    pub fn new(query_id: &str, sql: &str) -> Self {
        Self {
            query_id: query_id.to_string(),
            sql: sql.to_string(),
            total_time_ns: 0,
            operators: Vec::new(),
            peak_memory_bytes: 0,
            success: true,
            error_message: None,
        }
    }

    pub fn add_operator(&mut self, profile: OperatorProfile) {
        self.operators.push(profile);
    }

    pub fn finish(&mut self, duration_ns: u64) {
        self.total_time_ns = duration_ns;
    }

    pub fn mark_failed(&mut self, error: &str) {
        self.success = false;
        self.error_message = Some(error.to_string());
    }

    /// Get the slowest operator
    pub fn slowest_operator(&self) -> Option<&OperatorProfile> {
        self.operators.iter().max_by_key(|p| p.total_time_ns)
    }

    /// Get operators sorted by execution time
    pub fn operators_by_time(&self) -> Vec<&OperatorProfile> {
        let mut ops = self.operators.iter().collect::<Vec<_>>();
        ops.sort_by(|a, b| b.total_time_ns.cmp(&a.total_time_ns));
        ops
    }

    /// Generate profiling report
    #[allow(clippy::useless_format)]
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("╔══════════════════════════════════════════════════════════════════╗\n");
        report.push_str("║          SQLRustGo 2.0 - Query Performance Profile               ║\n");
        report.push_str("╠══════════════════════════════════════════════════════════════════╣\n");
        report.push_str(&format!(
            "║ Query ID: {}                                          ║\n",
            self.query_id
        ));
        report.push_str(&format!(
            "║ SQL: {}                               ║\n",
            truncate_string(&self.sql, 50)
        ));
        report.push_str(&format!(
            "║ Total Time: {}                                                 ║\n",
            format_duration(self.total_time_ns)
        ));
        report.push_str(&format!(
            "║ Status: {}                                                     ║\n",
            if self.success { "SUCCESS" } else { "FAILED" }
        ));
        report.push_str("╚══════════════════════════════════════════════════════════════════╝\n\n");

        report.push_str("Operator Breakdown:\n");
        report.push_str(
            "┌─────────────────────────────────────────────────────────────────────────┐\n",
        );
        report.push_str(
            "│ Operator          │ Executions  │ Avg Time     │ Total Time   │ Rows  │\n",
        );
        report.push_str(
            "├───────────────────┼──────────────┼──────────────┼──────────────┼───────┤\n",
        );

        for op in self.operators_by_time() {
            let name = format!("{:17}", truncate_string(&op.operator_name, 17));
            let execs = format!("{:12}", op.execution_count);
            let avg = format!("{:12}", op.format_avg_time());
            let total = format!("{:12}", op.format_total_time());
            let rows = format!("{:6}", op.rows_processed);

            report.push_str(&format!(
                "│ {} │ {} │ {} │ {} │ {} │\n",
                name, execs, avg, total, rows
            ));
        }
        report.push_str(
            "└─────────────────────────────────────────────────────────────────────────┘\n",
        );

        if let Some(slowest) = self.slowest_operator() {
            report.push_str(&format!(
                "\n⚠️  Slowest Operator: {} ({}ms)\n",
                slowest.operator_name,
                slowest.total_time_ns / 1_000_000
            ));
        }

        report
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Thread-safe profile accumulator using atomic operations
pub struct AtomicProfile {
    total_time_ns: AtomicU64,
    execution_count: AtomicU64,
    rows_processed: AtomicU64,
    batches_processed: AtomicU64,
    min_time_ns: AtomicU64,
    max_time_ns: AtomicU64,
}

impl AtomicProfile {
    pub fn new() -> Self {
        Self {
            total_time_ns: AtomicU64::new(0),
            execution_count: AtomicU64::new(0),
            rows_processed: AtomicU64::new(0),
            batches_processed: AtomicU64::new(0),
            min_time_ns: AtomicU64::new(u64::MAX),
            max_time_ns: AtomicU64::new(0),
        }
    }

    pub fn record(&self, duration_ns: u64, rows: usize, batches: usize) {
        self.total_time_ns.fetch_add(duration_ns, Ordering::Relaxed);
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        self.rows_processed
            .fetch_add(rows as u64, Ordering::Relaxed);
        self.batches_processed
            .fetch_add(batches as u64, Ordering::Relaxed);

        // Update min (try to set lower value)
        loop {
            let current = self.min_time_ns.load(Ordering::Relaxed);
            if duration_ns >= current {
                break;
            }
            if self
                .min_time_ns
                .compare_exchange(current, duration_ns, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        // Update max (try to set higher value)
        loop {
            let current = self.max_time_ns.load(Ordering::Relaxed);
            if duration_ns <= current {
                break;
            }
            if self
                .max_time_ns
                .compare_exchange(current, duration_ns, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub fn total_time_ns(&self) -> u64 {
        self.total_time_ns.load(Ordering::Relaxed)
    }

    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }

    pub fn rows_processed(&self) -> u64 {
        self.rows_processed.load(Ordering::Relaxed)
    }

    pub fn batches_processed(&self) -> u64 {
        self.batches_processed.load(Ordering::Relaxed)
    }

    pub fn min_time_ns(&self) -> u64 {
        let val = self.min_time_ns.load(Ordering::Relaxed);
        if val == u64::MAX {
            0
        } else {
            val
        }
    }

    pub fn max_time_ns(&self) -> u64 {
        self.max_time_ns.load(Ordering::Relaxed)
    }

    pub fn avg_time_ns(&self) -> u64 {
        let count = self.execution_count();
        if count > 0 {
            self.total_time_ns() / count
        } else {
            0
        }
    }
}

impl Default for AtomicProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Scope guard for timing operator execution (requires mutable reference)
/// Use Profiler::start_timer() for thread-safe profiling with GLOBAL_PROFILER
pub struct ProfileTimer<'a> {
    start: Instant,
    profile: &'a mut OperatorProfile,
    rows: usize,
    batches: usize,
}

impl<'a> ProfileTimer<'a> {
    pub fn new(profile: &'a mut OperatorProfile, rows: usize, batches: usize) -> Self {
        Self {
            start: Instant::now(),
            profile,
            rows,
            batches,
        }
    }
}

impl<'a> Drop for ProfileTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.profile
            .record_execution(duration.as_nanos() as u64, self.rows, self.batches);
    }
}

/// RAII timer guard for profiling with GLOBAL_PROFILER (thread-safe)
/// Use this with Profiler::start_timer() for automatic timing collection
pub struct GlobalProfileTimer {
    start: Instant,
    profiler: Arc<RwLock<HashMap<String, OperatorProfile>>>,
    name: String,
    rows: usize,
    batches: usize,
}

impl GlobalProfileTimer {
    /// Create a new global profile timer
    pub fn new(
        profiler: Arc<RwLock<HashMap<String, OperatorProfile>>>,
        name: String,
        rows: usize,
        batches: usize,
    ) -> Self {
        Self {
            start: Instant::now(),
            profiler,
            name,
            rows,
            batches,
        }
    }
}

impl Drop for GlobalProfileTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if let Ok(mut profiles) = self.profiler.write() {
            if let Some(profile) = profiles.get_mut(&self.name) {
                profile.record_execution(duration.as_nanos() as u64, self.rows, self.batches);
            }
        }
    }
}

/// Global profiler for aggregating profiles across queries
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct Profiler {
    profiles: Arc<RwLock<HashMap<String, OperatorProfile>>>,
    query_profiles: Arc<RwLock<Vec<QueryProfile>>>,
    max_query_profiles: usize,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            query_profiles: Arc::new(RwLock::new(Vec::new())),
            max_query_profiles: 100,
        }
    }

    /// Get or create an operator profile
    pub fn get_operator_profile(&self, name: &str, operator_type: &str) -> OperatorProfile {
        self.profiles
            .read()
            .ok()
            .and_then(|p| p.get(name).cloned())
            .unwrap_or_else(|| OperatorProfile::new(name, operator_type))
    }

    /// Record an operator execution
    pub fn record(
        &self,
        name: &str,
        operator_type: &str,
        duration_ns: u64,
        rows: usize,
        batches: usize,
    ) {
        if let Ok(mut profiles) = self.profiles.write() {
            let profile = profiles
                .entry(name.to_string())
                .or_insert_with(|| OperatorProfile::new(name, operator_type));
            profile.record_execution(duration_ns, rows, batches);
        }
    }

    /// Start a timer for profiling an operator execution (RAII pattern)
    /// Returns a GlobalProfileTimer that automatically records execution when dropped
    ///
    /// # Example
    /// ```ignore
    /// {
    ///     let _timer = profiler.start_timer("SeqScan", 1000, 10);
    ///     // ... perform operation ...
    /// } // execution time automatically recorded
    /// ```
    pub fn start_timer(&self, name: &str, rows: usize, batches: usize) -> GlobalProfileTimer {
        // Ensure the profile entry exists before timing starts
        if let Ok(mut profiles) = self.profiles.write() {
            profiles
                .entry(name.to_string())
                .or_insert_with(|| OperatorProfile::new(name, ""));
        }

        GlobalProfileTimer::new(self.profiles.clone(), name.to_string(), rows, batches)
    }

    /// Record a query profile
    pub fn record_query(&self, profile: QueryProfile) {
        if let Ok(mut profiles) = self.query_profiles.write() {
            profiles.push(profile);
            while profiles.len() > self.max_query_profiles {
                profiles.remove(0);
            }
        }
    }

    /// Get all operator profiles
    pub fn get_all_profiles(&self) -> Vec<OperatorProfile> {
        self.profiles
            .read()
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get profiles sorted by total time
    pub fn get_sorted_profiles(&self) -> Vec<OperatorProfile> {
        let mut profiles = self.get_all_profiles();
        profiles.sort_by(|a, b| b.total_time_ns.cmp(&a.total_time_ns));
        profiles
    }

    /// Get all query profiles
    pub fn get_query_profiles(&self) -> Vec<QueryProfile> {
        self.query_profiles
            .read()
            .map(|p| p.clone())
            .unwrap_or_default()
    }

    /// Get the latest query profile
    pub fn latest_query_profile(&self) -> Option<QueryProfile> {
        self.query_profiles
            .read()
            .ok()
            .and_then(|p| p.last().cloned())
    }

    /// Clear all profiling data
    pub fn clear(&self) {
        if let Ok(mut profiles) = self.profiles.write() {
            profiles.clear();
        }
        if let Ok(mut query_profiles) = self.query_profiles.write() {
            query_profiles.clear();
        }
    }

    /// Generate aggregate profiling report
    pub fn generate_report(&self) -> String {
        let profiles = self.get_sorted_profiles();

        let mut report = String::new();

        report.push_str("╔════════════════════════════════════════════════════════════════════╗\n");
        report.push_str("║          SQLRustGo 2.0 - Aggregate Profiling Report                ║\n");
        report.push_str("╠════════════════════════════════════════════════════════════════════╣\n");

        let total_queries = self.query_profiles.read().map(|p| p.len()).unwrap_or(0);
        report.push_str(&format!(
            "║ Total Queries Profiled: {}                                         ║\n",
            total_queries
        ));

        let total_operators = profiles.len();
        report.push_str(&format!(
            "║ Unique Operators: {}                                                 ║\n",
            total_operators
        ));

        let total_time: u64 = profiles.iter().map(|p| p.total_time_ns).sum();
        report.push_str(&format!(
            "║ Total Time: {}                                                     ║\n",
            format_duration(total_time)
        ));

        report
            .push_str("╚════════════════════════════════════════════════════════════════════╝\n\n");

        if !profiles.is_empty() {
            report.push_str("Top Operators by Execution Time:\n");
            report.push_str(
                "┌─────────────────────────────────────────────────────────────────────────┐\n",
            );
            report.push_str(
                "│ Operator          │ Executions  │ Avg Time     │ Total Time   │ Rows  │\n",
            );
            report.push_str(
                "├───────────────────┼──────────────┼──────────────┼──────────────┼───────┤\n",
            );

            for profile in profiles.iter().take(20) {
                let name = format!("{:17}", truncate_string(&profile.operator_name, 17));
                let execs = format!("{:12}", profile.execution_count);
                let avg = format!("{:12}", profile.format_avg_time());
                let total = format!("{:12}", profile.format_total_time());
                let rows = format!("{:6}", profile.rows_processed);

                report.push_str(&format!(
                    "│ {} │ {} │ {} │ {} │ {} │\n",
                    name, execs, avg, total, rows
                ));
            }
            report.push_str(
                "└─────────────────────────────────────────────────────────────────────────┘\n",
            );
        }

        report
    }

    pub fn to_json(&self) -> String {
        let profiles = self.get_sorted_profiles();
        serde_json::to_string_pretty(&profiles).unwrap_or_default()
    }
}

lazy_static::lazy_static! {
    #[allow(unused_doc_comments)]
    pub static ref GLOBAL_PROFILER: Profiler = Profiler::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_profile_creation() {
        let profile = OperatorProfile::new("SeqScan", "scan");
        assert_eq!(profile.operator_name, "SeqScan");
        assert_eq!(profile.execution_count, 0);
    }

    #[test]
    fn test_operator_profile_record() {
        let mut profile = OperatorProfile::new("SeqScan", "scan");

        profile.record_execution(1000, 100, 2);
        assert_eq!(profile.execution_count, 1);
        assert_eq!(profile.total_time_ns, 1000);
        assert_eq!(profile.rows_processed, 100);

        profile.record_execution(2000, 150, 3);
        assert_eq!(profile.execution_count, 2);
        assert_eq!(profile.total_time_ns, 3000);
        assert_eq!(profile.avg_time_ns, 1500);
    }

    #[test]
    fn test_operator_profile_min_max() {
        let mut profile = OperatorProfile::new("Test", "test");

        profile.record_execution(100, 10, 1);
        profile.record_execution(500, 20, 2);
        profile.record_execution(200, 15, 1);

        assert_eq!(profile.min_time_ns, 100);
        assert_eq!(profile.max_time_ns, 500);
    }

    #[test]
    fn test_query_profile() {
        let mut query = QueryProfile::new("q1", "SELECT * FROM users");

        let mut op1 = OperatorProfile::new("SeqScan", "scan");
        op1.record_execution(1000, 1000, 10);

        let mut op2 = OperatorProfile::new("Filter", "filter");
        op2.record_execution(500, 500, 5);

        query.add_operator(op1);
        query.add_operator(op2);
        query.finish(1500);

        assert_eq!(query.operators.len(), 2);
        assert_eq!(query.total_time_ns, 1500);

        let slowest = query.slowest_operator().unwrap();
        assert_eq!(slowest.operator_name, "SeqScan");
    }

    #[test]
    fn test_query_profile_report() {
        let mut query = QueryProfile::new("q1", "SELECT * FROM users");

        let mut op = OperatorProfile::new("SeqScan", "scan");
        op.record_execution(1_000_000, 1000, 10);

        query.add_operator(op);
        query.finish(1_000_000);

        let report = query.generate_report();
        assert!(report.contains("SQLRustGo"));
        assert!(report.contains("SeqScan"));
    }

    #[test]
    fn test_atomic_profile() {
        let profile = AtomicProfile::new();

        profile.record(1000, 100, 1);
        profile.record(2000, 200, 2);

        assert_eq!(profile.execution_count(), 2);
        assert_eq!(profile.total_time_ns(), 3000);
        assert_eq!(profile.avg_time_ns(), 1500);
    }

    #[test]
    fn test_profiler() {
        let profiler = Profiler::new();

        profiler.record("SeqScan", "scan", 1000, 100, 1);
        profiler.record("SeqScan", "scan", 2000, 200, 2);
        profiler.record("Filter", "filter", 500, 50, 1);

        let profiles = profiler.get_all_profiles();
        assert_eq!(profiles.len(), 2);

        let sorted = profiler.get_sorted_profiles();
        assert_eq!(sorted[0].operator_name, "SeqScan");
    }

    #[test]
    fn test_profiler_query() {
        let profiler = Profiler::new();

        let mut query = QueryProfile::new("q1", "SELECT 1");
        let mut op = OperatorProfile::new("Test", "test");
        op.record_execution(1000, 100, 1);
        query.add_operator(op);
        query.finish(1000);

        profiler.record_query(query);

        let latest = profiler.latest_query_profile().unwrap();
        assert_eq!(latest.query_id, "q1");
    }

    #[test]
    fn test_profiler_report() {
        let profiler = Profiler::new();

        profiler.record("SeqScan", "scan", 1000, 100, 1);
        profiler.record("HashJoin", "join", 2000, 50, 1);

        let report = profiler.generate_report();
        assert!(report.contains("SQLRustGo"));
        assert!(report.contains("SeqScan"));
    }

    #[test]
    fn test_profile_timer() {
        let mut profile = OperatorProfile::new("Test", "test");

        {
            let _timer = ProfileTimer::new(&mut profile, 100, 5);
            std::thread::sleep(Duration::from_millis(1));
        }

        assert_eq!(profile.execution_count, 1);
        assert!(profile.total_time_ns > 0);
    }

    #[test]
    fn test_global_profile_timer() {
        let profiler = Profiler::new();

        // Ensure profile exists (without recording an execution)
        let _ = profiler.get_operator_profile("TestOp", "test");

        {
            let _timer = profiler.start_timer("TestOp", 100, 5);
            std::thread::sleep(Duration::from_millis(1));
        }

        let profiles = profiler.get_all_profiles();
        let test_op = profiles
            .iter()
            .find(|p| p.operator_name == "TestOp")
            .unwrap();
        assert_eq!(test_op.execution_count, 1);
        assert!(test_op.total_time_ns > 0);
    }

    #[test]
    fn test_start_timer_creates_profile() {
        let profiler = Profiler::new();

        // start_timer should create profile if it doesn't exist
        {
            let _timer = profiler.start_timer("NewOp", 50, 2);
            std::thread::sleep(Duration::from_millis(1));
        }

        let profiles = profiler.get_all_profiles();
        let new_op = profiles
            .iter()
            .find(|p| p.operator_name == "NewOp")
            .unwrap();
        assert_eq!(new_op.execution_count, 1);
    }
}
