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
