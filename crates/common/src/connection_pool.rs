#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub size: usize,
    pub timeout_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            size: 50,
            timeout_ms: 5000,
        }
    }
}
