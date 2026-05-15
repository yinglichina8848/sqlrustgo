use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct AdaptiveMemoryTracker {
    current_bytes: AtomicU64,
    spill_threshold: usize,
    memory_limit: usize,
    fallback_mode: AtomicBool,
}

impl AdaptiveMemoryTracker {
    pub fn new(memory_limit: usize, spill_threshold: usize) -> Self {
        Self {
            current_bytes: AtomicU64::new(0),
            spill_threshold,
            memory_limit,
            fallback_mode: AtomicBool::new(false),
        }
    }

    pub fn allocate(&self, bytes: usize) -> bool {
        let prev = self.current_bytes.fetch_add(bytes as u64, Ordering::SeqCst);
        let new = prev + bytes as u64;
        new <= self.memory_limit as u64
    }

    pub fn deallocate(&self, bytes: usize) {
        self.current_bytes.fetch_sub(bytes as u64, Ordering::SeqCst);
    }

    pub fn should_spill(&self) -> bool {
        self.current_bytes.load(Ordering::SeqCst) >= self.spill_threshold as u64
    }

    pub fn is_memory_exceeded(&self) -> bool {
        self.current_bytes.load(Ordering::SeqCst) > self.memory_limit as u64
    }

    pub fn enable_fallback(&self) {
        self.fallback_mode.store(true, Ordering::SeqCst);
    }

    pub fn is_fallback_enabled(&self) -> bool {
        self.fallback_mode.load(Ordering::SeqCst)
    }

    pub fn current_usage(&self) -> usize {
        self.current_bytes.load(Ordering::SeqCst) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_allocate() {
        let tracker = AdaptiveMemoryTracker::new(1024, 512);
        assert!(tracker.allocate(100));
        assert_eq!(tracker.current_usage(), 100);
    }

    #[test]
    fn test_memory_tracker_should_spill() {
        let tracker = AdaptiveMemoryTracker::new(1024, 100);
        tracker.allocate(50);
        assert!(!tracker.should_spill());
        tracker.allocate(60);
        assert!(tracker.should_spill());
    }

    #[test]
    fn test_memory_tracker_deallocate() {
        let tracker = AdaptiveMemoryTracker::new(1024, 512);
        tracker.allocate(100);
        tracker.deallocate(50);
        assert_eq!(tracker.current_usage(), 50);
    }
}
