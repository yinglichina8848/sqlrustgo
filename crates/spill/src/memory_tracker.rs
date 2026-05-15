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

    pub fn allocate(&self, bytes: u64) -> bool {
        loop {
            let current = self.current_bytes.load(Ordering::SeqCst);
            let new = current + bytes;
            if new > self.memory_limit as u64 {
                return false;
            }
            if self.current_bytes
                .compare_exchange_weak(current, new, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    pub fn deallocate(&self, bytes: u64) {
        loop {
            let current = self.current_bytes.load(Ordering::SeqCst);
            if current < bytes {
                self.current_bytes.fetch_sub(bytes, Ordering::SeqCst);
                return;
            }
            if self
                .current_bytes
                .compare_exchange_weak(current, current - bytes, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return;
            }
        }
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

    #[test]
    fn test_is_memory_exceeded() {
        let tracker = AdaptiveMemoryTracker::new(100, 50);
        assert!(!tracker.is_memory_exceeded());
        assert!(tracker.allocate(80));
        assert!(!tracker.is_memory_exceeded());
        assert!(!tracker.allocate(30));
        assert_eq!(tracker.current_usage(), 80);
        assert!(!tracker.is_memory_exceeded());
        tracker.deallocate(80);
        assert!(!tracker.is_memory_exceeded());
    }
}
