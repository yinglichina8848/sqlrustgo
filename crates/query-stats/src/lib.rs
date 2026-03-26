//! Query Statistics System (pg_stat_statements)
//!
//! Provides query execution statistics tracking similar to PostgreSQL's pg_stat_statements.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub query: String,
    pub calls: u64,
    pub total_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub mean_time_ms: f64,
    pub stddev_time_ms: f64,
    pub rows: u64,
    pub shared_blks_hit: u64,
    pub shared_blks_read: u64,
}

impl QueryStats {
    pub fn new(query: String) -> Self {
        Self {
            query,
            calls: 0,
            total_time_ms: 0.0,
            min_time_ms: f64::MAX,
            max_time_ms: 0.0,
            mean_time_ms: 0.0,
            stddev_time_ms: 0.0,
            rows: 0,
            shared_blks_hit: 0,
            shared_blks_read: 0,
        }
    }

    pub fn record(&mut self, time_ms: f64, rows: u64) {
        self.calls += 1;
        self.total_time_ms += time_ms;
        self.rows += rows;

        if time_ms < self.min_time_ms {
            self.min_time_ms = time_ms;
        }
        if time_ms > self.max_time_ms {
            self.max_time_ms = time_ms;
        }

        self.mean_time_ms = self.total_time_ms / self.calls as f64;
    }
}

pub struct QueryNormalizer {
    replace_numbers: bool,
    replace_strings: bool,
}

impl QueryNormalizer {
    pub fn new() -> Self {
        Self {
            replace_numbers: true,
            replace_strings: true,
        }
    }

    pub fn normalize(&self, query: &str) -> String {
        let mut result = query.to_string();

        if self.replace_numbers {
            result = result
                .split_whitespace()
                .map(|word| {
                    if word.chars().all(|c| c.is_ascii_digit()) {
                        "$N".to_string()
                    } else {
                        word.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
        }

        if self.replace_strings {
            let mut chars: Vec<char> = result.chars().collect();
            let mut in_string = false;
            let mut result_chars = Vec::new();

            for c in chars {
                if c == '\'' && !in_string {
                    in_string = true;
                    result_chars.push('$');
                } else if c == '\'' && in_string {
                    in_string = false;
                    result_chars.push('$');
                } else if in_string {
                    continue;
                } else {
                    result_chars.push(c);
                }
            }

            result = result_chars.into_iter().collect();
        }

        result.trim().to_string()
    }
}

impl Default for QueryNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StatsCollector {
    entries: RwLock<HashMap<String, QueryStats>>,
    normalizer: QueryNormalizer,
    max_entries: usize,
}

impl StatsCollector {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            normalizer: QueryNormalizer::new(),
            max_entries,
        }
    }

    pub fn record(&self, query: &str, time_ms: f64, rows: u64) {
        let normalized = self.normalizer.normalize(query);

        let mut entries = self.entries.write();

        if let Some(stats) = entries.get_mut(&normalized) {
            stats.record(time_ms, rows);
        } else if entries.len() < self.max_entries {
            let mut stats = QueryStats::new(normalized.clone());
            stats.record(time_ms, rows);
            entries.insert(normalized, stats);
        }
    }

    pub fn get_stats(&self) -> Vec<QueryStats> {
        let entries = self.entries.read();
        let mut stats: Vec<_> = entries.values().cloned().collect();
        stats.sort_by(|a, b| b.total_time_ms.partial_cmp(&a.total_time_ms).unwrap());
        stats
    }

    pub fn reset(&self) {
        let mut entries = self.entries.write();
        entries.clear();
    }

    pub fn total_queries(&self) -> usize {
        self.entries.read().len()
    }
}

pub fn create_global_collector() -> Arc<StatsCollector> {
    Arc::new(StatsCollector::new(1000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizer_numbers() {
        let normalizer = QueryNormalizer::new();
        let result = normalizer.normalize("SELECT * FROM users WHERE id = 123");
        assert!(result.contains("$N"));
    }

    #[test]
    fn test_normalizer_strings() {
        let normalizer = QueryNormalizer::new();
        let result = normalizer.normalize("SELECT * FROM users WHERE name = 'alice'");
        assert!(!result.contains("'alice'"));
    }

    #[test]
    fn test_record() {
        let collector = StatsCollector::new(100);
        collector.record("SELECT 1", 10.0, 1);
        collector.record("SELECT 1", 20.0, 1);

        let stats = collector.get_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].calls, 2);
    }

    #[test]
    fn test_reset() {
        let collector = StatsCollector::new(100);
        collector.record("SELECT 1", 10.0, 1);
        collector.reset();

        assert_eq!(collector.total_queries(), 0);
    }
}
