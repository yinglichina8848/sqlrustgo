//! Pipeline Trace Module
//!
//! Provides execution tracing for vectorized query pipelines.
//! This feature differentiates SQLRustGo from MySQL by enabling
//! detailed visualization of query execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Represents a single operator's execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorTrace {
    /// Operator name (e.g., "SeqScan", "HashJoin", "Aggregate")
    pub operator_name: String,
    /// Operator type identifier
    pub operator_type: String,
    /// Unique trace ID for this operator instance
    pub trace_id: String,
    /// Parent trace ID (for tree hierarchy)
    pub parent_trace_id: Option<String>,
    /// Start time relative to query start (nanoseconds)
    pub start_ns: u64,
    /// End time relative to query start (nanoseconds)
    pub end_ns: u64,
    /// Duration in nanoseconds
    pub duration_ns: u64,
    /// Number of rows produced
    pub rows_produced: usize,
    /// Number of batches processed (for vectorized execution)
    pub batches_processed: usize,
    /// Children operator traces
    pub children: Vec<OperatorTrace>,
    /// Custom metadata (operator-specific metrics)
    pub metadata: HashMap<String, String>,
}

impl OperatorTrace {
    pub fn new(operator_name: &str, operator_type: &str) -> Self {
        Self {
            operator_name: operator_name.to_string(),
            operator_type: operator_type.to_string(),
            trace_id: uuid_simple(),
            parent_trace_id: None,
            start_ns: 0,
            end_ns: 0,
            duration_ns: 0,
            rows_produced: 0,
            batches_processed: 0,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_trace_id = Some(parent_id.to_string());
        self
    }

    pub fn start(&mut self, query_start: Instant) {
        self.start_ns = query_start.elapsed().as_nanos() as u64;
    }

    pub fn finish(&mut self, query_start: Instant) {
        self.end_ns = query_start.elapsed().as_nanos() as u64;
        self.duration_ns = self.end_ns.saturating_sub(self.start_ns);
    }

    pub fn add_child(&mut self, child: OperatorTrace) {
        self.children.push(child);
    }

    pub fn record_rows(&mut self, count: usize) {
        self.rows_produced = count;
    }

    pub fn record_batch(&mut self) {
        self.batches_processed += 1;
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Generate ASCII tree representation
    pub fn to_tree_string(&self) -> String {
        self.build_tree(0, true)
    }

    fn build_tree(&self, depth: usize, is_last: bool) -> String {
        let mut result = String::new();

        // Build prefix
        let prefix = if depth == 0 {
            "".to_string()
        } else {
            let indent = if is_last { "└── " } else { "├── " };
            indent.to_string()
        };

        // Duration display
        let duration_str = format_duration(self.duration_ns);

        // Rows display
        let rows_str = format!("{} rows", self.rows_produced);

        // Build the line
        result.push_str(&format!(
            "{}{} ({}) [{}]\n",
            prefix, self.operator_name, duration_str, rows_str
        ));

        // Process children
        for (i, child) in self.children.iter().enumerate() {
            let child_is_last = i == self.children.len() - 1;

            // Add vertical lines for parents
            if depth > 0 {
                // Add proper indentation
                for d in 1..depth {
                    if d == depth - 1 {
                        if is_last {
                            result.push_str("    ");
                        } else {
                            result.push_str("│   ");
                        }
                    } else {
                        result.push_str("│   ");
                    }
                }
            }

            result.push_str(&child.build_tree(depth + 1, child_is_last));
        }

        result
    }

    /// Get total duration including children
    pub fn total_duration_ns(&self) -> u64 {
        let children_total: u64 = self.children.iter().map(|c| c.total_duration_ns()).sum();
        self.duration_ns + children_total
    }

    /// Get operator statistics as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Simple UUID generator for trace IDs
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
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

/// Query execution trace - contains the entire pipeline trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTrace {
    /// Unique query execution ID
    pub query_id: String,
    /// SQL query text
    pub sql: String,
    /// Start timestamp
    pub start_time: String,
    /// Total query duration
    pub total_duration_ns: u64,
    /// Root operator trace
    pub root_trace: OperatorTrace,
    /// Number of operators in the pipeline
    pub operator_count: usize,
    /// Total rows produced
    pub total_rows: usize,
    /// Total batches processed
    pub total_batches: usize,
}

impl QueryTrace {
    pub fn new(sql: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        Self {
            query_id: uuid_simple(),
            sql: sql.to_string(),
            start_time: format!("{}.{:09}", now.as_secs(), now.subsec_nanos()),
            total_duration_ns: 0,
            root_trace: OperatorTrace::new("QueryRoot", "root"),
            operator_count: 0,
            total_rows: 0,
            total_batches: 0,
        }
    }

    pub fn finish(&mut self, duration: Duration) {
        self.total_duration_ns = duration.as_nanos() as u64;
    }

    pub fn set_root(&mut self, root: OperatorTrace) {
        self.root_trace = root;
        self.operator_count = self.count_operators(&self.root_trace);
        self.total_rows = self.root_trace.rows_produced;
        self.total_batches = self.root_trace.batches_processed;
    }

    fn count_operators(&self, trace: &OperatorTrace) -> usize {
        1 + trace
            .children
            .iter()
            .map(|c| self.count_operators(c))
            .sum::<usize>()
    }

    /// Generate ASCII pipeline visualization
    pub fn visualize_pipeline(&self) -> String {
        let mut output = String::new();

        output.push_str("╔══════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║          SQLRustGo 2.0 - Query Pipeline Visualization          ║\n");
        output.push_str("╠══════════════════════════════════════════════════════════════════╣\n");
        output.push_str(&format!(
            "║ Query ID: {}                                        ║\n",
            self.query_id
        ));
        output.push_str(&format!("║ SQL: {}  ║\n", truncate_string(&self.sql, 60)));
        output.push_str(&format!(
            "║ Duration: {}                                             ║\n",
            format_duration(self.total_duration_ns)
        ));
        output.push_str(&format!(
            "║ Operators: {}                                                   ║\n",
            self.operator_count
        ));
        output.push_str(&format!(
            "║ Total Rows: {}                                                 ║\n",
            self.total_rows
        ));
        output.push_str("╚══════════════════════════════════════════════════════════════════╝\n\n");

        output.push_str("Execution Pipeline:\n");
        output.push_str(&self.root_trace.to_tree_string());

        output
    }

    /// Export as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Export as compact JSON (single line)
    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Truncate string with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Pipeline trace collector - thread-safe trace storage
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct TraceCollector {
    traces: Arc<RwLock<Vec<QueryTrace>>>,
    max_traces: usize,
}

impl TraceCollector {
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: Arc::new(RwLock::new(Vec::new())),
            max_traces,
        }
    }

    pub fn record(&self, trace: QueryTrace) {
        if let Ok(mut traces) = self.traces.write() {
            traces.push(trace);
            // Keep only last N traces
            while traces.len() > self.max_traces {
                traces.remove(0);
            }
        }
    }

    pub fn get_traces(&self) -> Vec<QueryTrace> {
        self.traces.read().map(|t| t.clone()).unwrap_or_default()
    }

    pub fn get_trace(&self, query_id: &str) -> Option<QueryTrace> {
        self.traces
            .read()
            .ok()
            .and_then(|traces| traces.iter().find(|t| t.query_id == query_id).cloned())
    }

    pub fn clear(&self) {
        if let Ok(mut traces) = self.traces.write() {
            traces.clear();
        }
    }

    pub fn latest(&self) -> Option<QueryTrace> {
        self.traces.read().ok().and_then(|t| t.last().cloned())
    }
}

lazy_static::lazy_static! {
    #[allow(unused_doc_comments)]
    pub static ref GLOBAL_TRACE_COLLECTOR: TraceCollector = TraceCollector::new(1000);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_trace_creation() {
        let trace = OperatorTrace::new("SeqScan", "scan");
        assert_eq!(trace.operator_name, "SeqScan");
        assert_eq!(trace.operator_type, "scan");
    }

    #[test]
    fn test_operator_trace_timing() {
        let start = Instant::now();
        let mut trace = OperatorTrace::new("SeqScan", "scan");

        trace.start(start);
        std::thread::sleep(Duration::from_millis(1));
        trace.finish(start);

        assert!(trace.duration_ns > 0);
    }

    #[test]
    fn test_operator_trace_tree() {
        let mut root = OperatorTrace::new("HashJoin", "join");
        let mut left = OperatorTrace::new("SeqScan", "scan");
        left.rows_produced = 100;
        let mut right = OperatorTrace::new("SeqScan", "scan");
        right.rows_produced = 50;

        root.add_child(left);
        root.add_child(right);

        let tree = root.to_tree_string();
        assert!(tree.contains("HashJoin"));
        assert!(tree.contains("SeqScan"));
    }

    #[test]
    fn test_query_trace_creation() {
        let trace = QueryTrace::new("SELECT * FROM users");
        assert!(trace.sql.contains("SELECT"));
    }

    #[test]
    fn test_query_trace_pipeline_viz() {
        let mut query = QueryTrace::new("SELECT * FROM users WHERE id > 10");

        let mut root = OperatorTrace::new("Filter", "filter");
        let mut scan = OperatorTrace::new("SeqScan", "scan");
        scan.rows_produced = 1000;
        root.add_child(scan);

        query.set_root(root);
        query.finish(Duration::from_millis(10));

        let viz = query.visualize_pipeline();
        assert!(viz.contains("SQLRustGo"));
        assert!(viz.contains("Query Pipeline"));
    }

    #[test]
    fn test_trace_collector() {
        let collector = TraceCollector::new(10);

        let trace1 = QueryTrace::new("SELECT 1");
        let trace2 = QueryTrace::new("SELECT 2");

        collector.record(trace1);
        collector.record(trace2);

        let traces = collector.get_traces();
        assert_eq!(traces.len(), 2);
    }

    #[test]
    fn test_trace_collector_max_traces() {
        let collector = TraceCollector::new(2);

        for i in 0..5 {
            let trace = QueryTrace::new(&format!("SELECT {}", i));
            collector.record(trace);
        }

        let traces = collector.get_traces();
        assert_eq!(traces.len(), 2);
    }

    #[test]
    fn test_operator_trace_json() {
        let trace = OperatorTrace::new("SeqScan", "scan");
        let json = trace.to_json();
        assert!(json.contains("SeqScan"));
    }

    #[test]
    fn test_query_trace_json() {
        let trace = QueryTrace::new("SELECT * FROM test");
        let json = trace.to_json();
        assert!(json.contains("SELECT * FROM test"));
    }

    #[test]
    fn test_format_duration() {
        assert!(format_duration(500).contains("ns"));
        assert!(format_duration(50_000).contains("µs"));
        assert!(format_duration(5_000_000).contains("ms"));
        assert!(format_duration(5_000_000_000).contains("s"));
    }
}
