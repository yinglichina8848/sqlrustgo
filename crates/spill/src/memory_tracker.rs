use parking_lot::Mutex;
use std::sync::Arc;

pub struct AdaptiveMemoryTracker {
    memory_limit: usize,
    current_usage: Arc<Mutex<u64>>,
}

impl AdaptiveMemoryTracker {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            memory_limit,
            current_usage: Arc::new(Mutex::new(0)),
        }
    }

    pub fn current_usage(&self) -> u64 {
        *self.current_usage.lock()
    }

    pub fn memory_limit(&self) -> usize {
        self.memory_limit
    }
}
