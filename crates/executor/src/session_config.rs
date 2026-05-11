use std::env;

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub benchmark_mode: bool,
    pub teaching_mode: bool,
    pub cache_enabled: bool,
    pub stats_enabled: bool,
    pub max_memory_per_query: usize,
    pub spill_to_disk_threshold: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        let benchmark_mode = env::var("SQLRUSTGO_BENCHMARK_MODE")
            .map(|v| v == "1")
            .unwrap_or(false);

        let teaching_mode = env::var("SQLRUSTGO_TEACHING_MODE")
            .map(|v| v == "1")
            .unwrap_or(false);

        let max_memory = env::var("SQLRUSTGO_MAX_MEMORY_PER_QUERY")
            .map(|v| v.parse().unwrap_or(256 * 1024 * 1024))
            .unwrap_or(256 * 1024 * 1024);

        let spill_threshold = env::var("SQLRUSTGO_SPILL_THRESHOLD")
            .map(|v| v.parse().unwrap_or(128 * 1024 * 1024))
            .unwrap_or(128 * 1024 * 1024);

        Self {
            benchmark_mode,
            teaching_mode,
            cache_enabled: !benchmark_mode && !teaching_mode,
            stats_enabled: !benchmark_mode && !teaching_mode,
            max_memory_per_query: max_memory,
            spill_to_disk_threshold: spill_threshold,
        }
    }
}

impl SessionConfig {
    pub fn new(benchmark_mode: bool) -> Self {
        Self {
            benchmark_mode,
            teaching_mode: false,
            cache_enabled: !benchmark_mode,
            stats_enabled: !benchmark_mode,
            max_memory_per_query: 256 * 1024 * 1024,
            spill_to_disk_threshold: 128 * 1024 * 1024,
        }
    }

    pub fn with_teaching_mode(teaching_mode: bool) -> Self {
        Self {
            benchmark_mode: false,
            teaching_mode,
            cache_enabled: !teaching_mode,
            stats_enabled: !teaching_mode,
            max_memory_per_query: 256 * 1024 * 1024,
            spill_to_disk_threshold: 128 * 1024 * 1024,
        }
    }

    pub fn with_memory_limit(mut self, max_memory: usize, spill_threshold: usize) -> Self {
        self.max_memory_per_query = max_memory;
        self.spill_to_disk_threshold = spill_threshold;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_not_benchmark_mode() {
        std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
        std::env::remove_var("SQLRUSTGO_TEACHING_MODE");
        let config = SessionConfig::default();
        // Just verify config can be created
        let _ = config;
    }

    #[test]
    fn test_benchmark_mode_from_env() {
        std::env::remove_var("SQLRUSTGO_TEACHING_MODE");
        std::env::set_var("SQLRUSTGO_BENCHMARK_MODE", "1");
        let config = SessionConfig::default();
        assert!(config.benchmark_mode);
        assert!(!config.teaching_mode);
        assert!(!config.cache_enabled);
        assert!(!config.stats_enabled);
        std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
    }

    #[test]
    fn test_teaching_mode_from_env() {
        std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
        std::env::remove_var("SQLRUSTGO_TEACHING_MODE");
        std::env::set_var("SQLRUSTGO_TEACHING_MODE", "1");
        let config = SessionConfig::default();
        assert!(!config.benchmark_mode);
        assert!(config.teaching_mode);
        assert!(!config.cache_enabled);
        assert!(!config.stats_enabled);
        std::env::remove_var("SQLRUSTGO_TEACHING_MODE");
    }

    #[test]
    fn test_teaching_mode_with_constructor() {
        let config = SessionConfig::with_teaching_mode(true);
        assert!(!config.benchmark_mode);
        assert!(config.teaching_mode);
        assert!(!config.cache_enabled);
        assert!(!config.stats_enabled);
    }

    #[test]
    fn test_explicit_benchmark_mode() {
        let config = SessionConfig::new(true);
        assert!(config.benchmark_mode);
        assert!(!config.teaching_mode);
        assert!(!config.cache_enabled);
        assert!(!config.stats_enabled);
    }

    #[test]
    fn test_explicit_normal_mode() {
        let config = SessionConfig::new(false);
        assert!(!config.benchmark_mode);
        assert!(!config.teaching_mode);
        assert!(config.cache_enabled);
        assert!(config.stats_enabled);
    }
}
