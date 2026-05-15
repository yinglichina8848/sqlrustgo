use crate::error::SpillError;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct FallbackManager {
    attempts: AtomicUsize,
    max_attempts: usize,
    #[allow(dead_code)]
    original_memory_limit: usize,
}

impl FallbackManager {
    pub fn new(max_attempts: usize, original_memory_limit: usize) -> Self {
        Self {
            attempts: AtomicUsize::new(0),
            max_attempts,
            original_memory_limit,
        }
    }

    pub fn can_fallback(&self) -> bool {
        self.attempts.load(Ordering::SeqCst) < self.max_attempts
    }

    pub fn try_fallback(&self) -> Result<(), SpillError> {
        if !self.can_fallback() {
            return Err(crate::error::SpillError::FallbackFailed(
                "已达最大降级尝试次数".into(),
            ));
        }
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn attempt_count(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.attempts.store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_manager() {
        let manager = FallbackManager::new(3, 1024);
        assert!(manager.can_fallback());

        manager.try_fallback().unwrap();
        assert_eq!(manager.attempt_count(), 1);

        manager.try_fallback().unwrap();
        manager.try_fallback().unwrap();
        assert!(!manager.can_fallback());
    }
}
