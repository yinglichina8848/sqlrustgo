// TPC-H Streaming Configuration
// Memory-efficient configuration for large dataset processing

/// Streaming configuration for TPC-H benchmark
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StreamingConfig {
    /// Batch size for data loading (number of rows per batch)
    pub batch_size: usize,
    /// Maximum memory usage in bytes (default: 500MB)
    pub max_memory_bytes: usize,
    /// Enable streaming mode
    pub streaming_enabled: bool,
    /// Prefetch batches
    pub prefetch_batches: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            batch_size: 10000,          // 10k rows per batch
            max_memory_bytes: 500 * 1024 * 1024,  // 500MB
            streaming_enabled: true,
            prefetch_batches: 2,
        }
    }
}

#[allow(dead_code)]
impl StreamingConfig {
    pub fn new(batch_size: usize, max_memory_mb: usize) -> Self {
        Self {
            batch_size,
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            streaming_enabled: true,
            prefetch_batches: 2,
        }
    }

    /// SF0.1 configuration (600k rows, ~60MB)
    pub fn sf01() -> Self {
        Self::new(10000, 100)
    }

    /// SF1 configuration (6M rows, ~600MB)
    pub fn sf1() -> Self {
        Self::new(50000, 700)
    }

    /// SF10 configuration (60M rows, ~6GB) - requires streaming
    pub fn sf10() -> Self {
        Self::new(100000, 500)  // Limit to 500MB memory
    }
}

/// Memory usage tracker
#[derive(Debug)]
#[allow(dead_code)]
pub struct MemoryTracker {
    current_bytes: usize,
    max_bytes: usize,
}

#[allow(dead_code)]
impl MemoryTracker {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            current_bytes: 0,
            max_bytes,
        }
    }

    pub fn allocate(&mut self, bytes: usize) -> bool {
        if self.current_bytes + bytes <= self.max_bytes {
            self.current_bytes += bytes;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, bytes: usize) {
        self.current_bytes = self.current_bytes.saturating_sub(bytes);
    }

    pub fn current(&self) -> usize {
        self.current_bytes
    }

    pub fn max(&self) -> usize {
        self.max_bytes
    }

    pub fn usage_percent(&self) -> f64 {
        (self.current_bytes as f64 / self.max_bytes as f64) * 100.0
    }
}
