#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub size: usize,
    pub timeout_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            size: num_cpus::get(),
            timeout_ms: 5000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.size, num_cpus::get());
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_pool_config_new() {
        let config = PoolConfig {
            size: 10,
            timeout_ms: 1000,
        };
        assert_eq!(config.size, 10);
        assert_eq!(config.timeout_ms, 1000);
    }
}
