use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct QueryCacheConfig {
    pub max_entries: usize,
    pub max_memory_bytes: usize,
    pub ttl_seconds: u64,
    pub enabled: bool,
    pub benchmark_mode: bool,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        let benchmark_mode = std::env::var("SQLRUSTGO_BENCHMARK_MODE")
            .map(|v| v == "1")
            .unwrap_or(false);

        Self {
            max_entries: 1000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            ttl_seconds: 30,
            enabled: !benchmark_mode,
            benchmark_mode,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub normalized_sql: String,
    pub params_hash: u64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub result: super::ExecutorResult,
    pub tables: Vec<String>,
    pub created_at: Instant,
    pub size_bytes: usize,
}

impl CacheEntry {
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    pub fn estimate_size(&self) -> usize {
        let mut size = 0;
        for row in &self.result.rows {
            for val in row {
                size += val.estimate_memory_size();
            }
        }
        size
    }
}
