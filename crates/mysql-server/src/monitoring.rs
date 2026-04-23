//! Performance Monitoring Module
//!
//! Provides performance monitoring capabilities:
//! - Slow query log
//! - Connection statistics
//! - Memory usage tracking
//! - Prometheus-compatible metrics endpoint

#![allow(clippy::len_zero, clippy::comparison_to_empty)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Slow query configuration
#[derive(Debug, Clone)]
pub struct SlowQueryConfig {
    /// Threshold in seconds for slow queries
    pub long_query_time: f64,
    /// Whether slow query log is enabled
    pub enabled: bool,
}

impl Default for SlowQueryConfig {
    fn default() -> Self {
        Self {
            long_query_time: 1.0, // 1 second default
            enabled: true,
        }
    }
}

/// A recorded slow query entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQueryEntry {
    /// SQL query text
    pub query: String,
    /// Execution time in seconds
    pub execution_time: f64,
    /// Timestamp when the query was executed
    pub timestamp: String,
    /// Connection ID
    pub connection_id: u32,
}

/// Query execution statistics
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    /// Total number of queries executed
    pub total_queries: u64,
    /// Number of slow queries
    pub slow_queries: u64,
    /// Total execution time in seconds
    pub total_execution_time: f64,
    /// Query count by type
    pub queries_by_type: HashMap<String, u64>,
}

impl QueryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_query(&mut self, query_type: &str, execution_time: f64, is_slow: bool) {
        self.total_queries += 1;
        self.total_execution_time += execution_time;
        if is_slow {
            self.slow_queries += 1;
        }
        *self
            .queries_by_type
            .entry(query_type.to_string())
            .or_insert(0) += 1;
    }

    pub fn avg_execution_time(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.total_execution_time / self.total_queries as f64
        }
    }
}

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Currently active connections
    pub active_connections: u32,
    /// Total connections ever opened
    pub total_connections: u64,
    /// Peak concurrent connections
    pub peak_connections: u32,
    /// Connections by status
    pub connections_by_status: HashMap<String, u32>,
}

impl ConnectionStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connection_opened(&mut self) {
        self.active_connections += 1;
        self.total_connections += 1;
        if self.active_connections > self.peak_connections {
            self.peak_connections = self.active_connections;
        }
    }

    pub fn connection_closed(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_memory: usize,
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Number of allocations
    pub allocation_count: u64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, current: usize) {
        self.current_memory = current;
        if current > self.peak_memory {
            self.peak_memory = current;
        }
    }
}

/// Global performance monitor
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Slow query configuration
    slow_query_config: RwLock<SlowQueryConfig>,
    /// Query execution statistics
    pub query_stats: RwLock<QueryStats>,
    /// Connection statistics
    pub connection_stats: RwLock<ConnectionStats>,
    /// Memory statistics
    pub memory_stats: RwLock<MemoryStats>,
    /// Slow query log entries
    pub slow_query_log: RwLock<Vec<SlowQueryEntry>>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            slow_query_config: RwLock::new(SlowQueryConfig::default()),
            query_stats: RwLock::new(QueryStats::new()),
            connection_stats: RwLock::new(ConnectionStats::new()),
            memory_stats: RwLock::new(MemoryStats::new()),
            slow_query_log: RwLock::new(Vec::new()),
        }
    }

    /// Set slow query threshold (in seconds)
    pub fn set_long_query_time(&self, seconds: f64) {
        let mut config = self.slow_query_config.write().unwrap();
        config.long_query_time = seconds;
    }

    /// Enable or disable slow query logging
    pub fn set_slow_query_enabled(&self, enabled: bool) {
        let mut config = self.slow_query_config.write().unwrap();
        config.enabled = enabled;
    }

    /// Record a query execution
    pub fn record_query(
        &self,
        query: &str,
        query_type: &str,
        execution_time: f64,
        connection_id: u32,
    ) {
        let (threshold, enabled) = {
            let config = self.slow_query_config.read().unwrap();
            (config.long_query_time, config.enabled)
        };
        let is_slow = execution_time >= threshold && enabled;

        // Update query stats
        {
            let mut stats = self.query_stats.write().unwrap();
            stats.record_query(query_type, execution_time, is_slow);
        }

        // Record slow query if applicable
        if is_slow {
            let entry = SlowQueryEntry {
                query: query.to_string(),
                execution_time,
                timestamp: chrono_now(),
                connection_id,
            };

            let mut log = self.slow_query_log.write().unwrap();
            log.push(entry);

            // Keep only last 1000 slow queries
            if log.len() > 1000 {
                log.remove(0);
            }
        }
    }

    /// Record a connection opened event
    pub fn record_connection_opened(&self) {
        let mut stats = self.connection_stats.write().unwrap();
        stats.connection_opened();
    }

    /// Record a connection closed event
    pub fn record_connection_closed(&self) {
        let mut stats = self.connection_stats.write().unwrap();
        stats.connection_closed();
    }

    /// Update memory statistics
    pub fn update_memory_stats(&self, bytes: usize) {
        let mut stats = self.memory_stats.write().unwrap();
        stats.update(bytes);
    }

    /// Get slow query log entries
    pub fn get_slow_queries(&self, limit: usize) -> Vec<SlowQueryEntry> {
        let log = self.slow_query_log.read().unwrap();
        log.iter().rev().take(limit).cloned().collect()
    }

    /// Generate Prometheus-compatible metrics output
    pub fn prometheus_metrics(&self) -> String {
        let query_stats = self.query_stats.read().unwrap();
        let conn_stats = self.connection_stats.read().unwrap();
        let mem_stats = self.memory_stats.read().unwrap();

        let mut output = String::new();

        // Query metrics
        output.push_str("# TYPE sqlrustgo_queries_total counter\n");
        output.push_str(&format!(
            "sqlrustgo_queries_total {}\n",
            query_stats.total_queries
        ));

        output.push_str("# TYPE sqlrustgo_slow_queries_total counter\n");
        output.push_str(&format!(
            "sqlrustgo_slow_queries_total {}\n",
            query_stats.slow_queries
        ));

        output.push_str("# TYPE sqlrustgo_query_duration_seconds summary\n");
        output.push_str(&format!(
            "sqlrustgo_query_duration_seconds_sum {}\n",
            query_stats.total_execution_time
        ));
        output.push_str(&format!(
            "sqlrustgo_query_duration_seconds_count {}\n",
            query_stats.total_queries
        ));

        output.push_str("# TYPE sqlrustgo_avg_query_duration_seconds gauge\n");
        output.push_str(&format!(
            "sqlrustgo_avg_query_duration_seconds {}\n",
            query_stats.avg_execution_time()
        ));

        // Connection metrics
        output.push_str("# TYPE sqlrustgo_active_connections gauge\n");
        output.push_str(&format!(
            "sqlrustgo_active_connections {}\n",
            conn_stats.active_connections
        ));

        output.push_str("# TYPE sqlrustgo_total_connections counter\n");
        output.push_str(&format!(
            "sqlrustgo_total_connections {}\n",
            conn_stats.total_connections
        ));

        output.push_str("# TYPE sqlrustgo_peak_connections gauge\n");
        output.push_str(&format!(
            "sqlrustgo_peak_connections {}\n",
            conn_stats.peak_connections
        ));

        // Memory metrics
        output.push_str("# TYPE sqlrustgo_memory_usage_bytes gauge\n");
        output.push_str(&format!(
            "sqlrustgo_memory_usage_bytes {}\n",
            mem_stats.current_memory
        ));

        output.push_str("# TYPE sqlrustgo_peak_memory_usage_bytes gauge\n");
        output.push_str(&format!(
            "sqlrustgo_peak_memory_usage_bytes {}\n",
            mem_stats.peak_memory
        ));

        // Slow query threshold info
        output.push_str("# TYPE sqlrustgo_slow_query_threshold_seconds gauge\n");
        let threshold = {
            let config = self.slow_query_config.read().unwrap();
            config.long_query_time
        };
        output.push_str(&format!(
            "sqlrustgo_slow_query_threshold_seconds {}\n",
            threshold
        ));

        output
    }

    /// Get JSON stats for /stats endpoint
    pub fn json_stats(&self) -> serde_json::Value {
        let query_stats = self.query_stats.read().unwrap();
        let conn_stats = self.connection_stats.read().unwrap();
        let mem_stats = self.memory_stats.read().unwrap();
        let slow_config = self.slow_query_config.read().unwrap();

        serde_json::json!({
            "queries": {
                "total": query_stats.total_queries,
                "slow": query_stats.slow_queries,
                "avg_duration_seconds": query_stats.avg_execution_time(),
                "by_type": query_stats.queries_by_type,
            },
            "connections": {
                "active": conn_stats.active_connections,
                "total": conn_stats.total_connections,
                "peak": conn_stats.peak_connections,
            },
            "memory": {
                "current_bytes": mem_stats.current_memory,
                "peak_bytes": mem_stats.peak_memory,
            },
            "slow_query_config": {
                "enabled": slow_config.enabled,
                "threshold_seconds": slow_config.long_query_time,
            }
        })
    }
}

/// Helper function to get current timestamp
fn chrono_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    format!("{}", now.as_secs())
}

/// Shared pointer to performance monitor
pub type SharedMonitor = Arc<PerformanceMonitor>;

/// Create a new shared performance monitor
pub fn create_monitor() -> SharedMonitor {
    Arc::new(PerformanceMonitor::new())
}

/// Timer for measuring query execution time
pub struct QueryTimer {
    start: Instant,
    query: String,
    query_type: String,
    connection_id: u32,
    monitor: SharedMonitor,
}

impl QueryTimer {
    pub fn new(
        query: String,
        query_type: String,
        connection_id: u32,
        monitor: SharedMonitor,
    ) -> Self {
        Self {
            start: Instant::now(),
            query,
            query_type,
            connection_id,
            monitor,
        }
    }

    pub fn finish(self) {
        let elapsed = self.start.elapsed().as_secs_f64();
        self.monitor
            .record_query(&self.query, &self.query_type, elapsed, self.connection_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_query_detection() {
        let monitor = create_monitor();
        monitor.set_long_query_time(0.5);

        // Record a fast query
        monitor.record_query("SELECT 1", "SELECT", 0.1, 1);

        // Record a slow query
        monitor.record_query("SELECT sleep(1)", "SELECT", 1.0, 1);

        let slow_queries = monitor.get_slow_queries(10);
        assert_eq!(slow_queries.len(), 1);
        assert!(slow_queries[0].query.contains("sleep"));
    }

    #[test]
    fn test_connection_stats() {
        let monitor = create_monitor();

        monitor.record_connection_opened();
        monitor.record_connection_opened();
        monitor.record_connection_closed();

        let stats = monitor.connection_stats.read().unwrap();
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.total_connections, 2);
        assert_eq!(stats.peak_connections, 2);
    }

    #[test]
    fn test_prometheus_metrics() {
        let monitor = create_monitor();
        monitor.record_query("SELECT 1", "SELECT", 0.5, 1);

        let metrics = monitor.prometheus_metrics();
        assert!(metrics.contains("sqlrustgo_queries_total 1"));
        assert!(metrics.contains("sqlrustgo_query_duration_seconds_count 1"));
    }

    #[test]
    fn test_memory_stats_update() {
        let mut stats = MemoryStats::new();
        assert_eq!(stats.current_memory, 0);
        assert_eq!(stats.peak_memory, 0);

        stats.update(100);
        assert_eq!(stats.current_memory, 100);
        assert_eq!(stats.peak_memory, 100);

        stats.update(50);
        assert_eq!(stats.current_memory, 50);
        assert_eq!(stats.peak_memory, 100);

        stats.update(200);
        assert_eq!(stats.current_memory, 200);
        assert_eq!(stats.peak_memory, 200);
    }

    #[test]
    fn test_memory_stats_default() {
        let stats = MemoryStats::default();
        assert_eq!(stats.current_memory, 0);
        assert_eq!(stats.peak_memory, 0);
        assert_eq!(stats.allocation_count, 0);
    }

    #[test]
    fn test_slow_query_config_default() {
        let config = SlowQueryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.long_query_time, 1.0);
    }

    #[test]
    fn test_slow_query_entry() {
        let entry = SlowQueryEntry {
            query: "SELECT 1".to_string(),
            execution_time: 0.5,
            timestamp: 1234567890,
            connection_id: 1,
        };
        assert_eq!(entry.query, "SELECT 1");
        assert_eq!(entry.execution_time, 0.5);
    }

    #[test]
    fn test_query_stats_new() {
        let stats = QueryStats::new();
        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.slow_queries, 0);
    }

    #[test]
    fn test_query_stats_record_query() {
        let stats = QueryStats::new();
        stats.record_query("SELECT", 0.5, false);
        stats.record_query("INSERT", 2.0, true);
        assert_eq!(stats.total_queries, 2);
        assert_eq!(stats.slow_queries, 1);
    }

    #[test]
    fn test_connection_stats_new() {
        let stats = ConnectionStats::new();
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.peak_connections, 0);
    }

    #[test]
    fn test_performance_monitor_new() {
        let monitor = PerformanceMonitor::new();
        assert!(monitor.query_stats.read().unwrap().total_queries == 0);
        assert!(monitor.connection_stats.read().unwrap().active_connections == 0);
    }

    #[test]
    fn test_set_slow_query_enabled() {
        let monitor = create_monitor();
        monitor.set_long_query_time(0.5);

        monitor.set_slow_query_enabled(false);
        monitor.record_query("SELECT sleep(1)", "SELECT", 1.0, 1);
        let slow_queries = monitor.get_slow_queries(10);
        assert_eq!(slow_queries.len(), 0);

        monitor.set_slow_query_enabled(true);
        monitor.record_query("SELECT sleep(1)", "SELECT", 1.0, 1);
        let slow_queries = monitor.get_slow_queries(10);
        assert_eq!(slow_queries.len(), 1);
    }

    #[test]
    fn test_get_slow_queries_limit() {
        let monitor = create_monitor();
        monitor.set_long_query_time(0.0);

        for i in 0..100 {
            monitor.record_query(&format!("SELECT {}", i), "SELECT", 1.0, 1);
        }

        let slow_queries = monitor.get_slow_queries(10);
        assert_eq!(slow_queries.len(), 10);
    }

    #[test]
    fn test_reset_stats() {
        let monitor = create_monitor();
        monitor.record_query("SELECT 1", "SELECT", 0.5, 1);
        monitor.record_connection_opened();

        monitor.reset_stats();

        assert_eq!(monitor.query_stats.read().unwrap().total_queries, 0);
        assert_eq!(monitor.connection_stats.read().unwrap().total_connections, 0);
    }

    #[test]
    fn test_memory_stats_debug() {
        let stats = MemoryStats::new();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("MemoryStats"));
    }
}
