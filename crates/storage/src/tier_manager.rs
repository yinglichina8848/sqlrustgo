use crate::page_access_tracker::PageAccessTracker;
use crate::storage_tier::StorageTier;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
#[allow(dead_code)]
pub struct TierManagerConfig {
    pub promotion_interval_secs: u64,
    pub demotion_batch_size: usize,
    pub enable_async_promotion: bool,
    pub enable_sync_demotion: bool,
}

impl Default for TierManagerConfig {
    fn default() -> Self {
        Self {
            promotion_interval_secs: 60,
            demotion_batch_size: 100,
            enable_async_promotion: true,
            enable_sync_demotion: true,
        }
    }
}

#[derive(Debug)]
pub struct TierManager {
    tracker: Arc<PageAccessTracker>,
    #[allow(dead_code)]
    config: TierManagerConfig,
    running: AtomicBool,
    migration_queue: RwLock<VecDeque<u32>>,
}

impl TierManager {
    pub fn new(tracker: Arc<PageAccessTracker>) -> Self {
        Self {
            tracker,
            config: TierManagerConfig::default(),
            running: AtomicBool::new(false),
            migration_queue: RwLock::new(VecDeque::new()),
        }
    }

    pub fn with_config(tracker: Arc<PageAccessTracker>, config: TierManagerConfig) -> Self {
        Self {
            tracker,
            config,
            running: AtomicBool::new(false),
            migration_queue: RwLock::new(VecDeque::new()),
        }
    }

    pub fn record_page_access(&self, page_id: u32) {
        self.tracker.record_access(page_id);
    }

    pub fn get_page_tier(&self, page_id: u32) -> StorageTier {
        self.tracker
            .get_access_info(page_id)
            .map(|info| info.current_tier())
            .unwrap_or(StorageTier::Hot)
    }

    pub fn get_recommended_tier(&self, page_id: u32) -> StorageTier {
        self.tracker.get_recommended_tier(page_id)
    }

    pub fn should_migrate(&self, page_id: u32) -> bool {
        let current = self.get_page_tier(page_id);
        let recommended = self.get_recommended_tier(page_id);
        current != recommended
    }

    pub fn queue_for_migration(&self, page_id: u32) {
        let mut queue = self.migration_queue.write().unwrap();
        if !queue.contains(&page_id) {
            queue.push_back(page_id);
        }
    }

    pub fn get_migration_batch(&self, batch_size: usize) -> Vec<u32> {
        let mut queue = self.migration_queue.write().unwrap();
        let mut batch = Vec::new();
        while batch.len() < batch_size {
            if let Some(page_id) = queue.pop_front() {
                batch.push(page_id);
            } else {
                break;
            }
        }
        batch
    }

    pub fn get_tier_counts(&self) -> Vec<(StorageTier, usize)> {
        vec![
            (StorageTier::Hot, 0),
            (StorageTier::Warm, 0),
            (StorageTier::Cold, 0),
            (StorageTier::Archive, 0),
        ]
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn pending_migrations(&self) -> usize {
        let queue = self.migration_queue.read().unwrap();
        queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_access_tracker::PageAccessTracker;

    fn create_test_manager() -> Arc<TierManager> {
        let tracker = Arc::new(PageAccessTracker::new());
        Arc::new(TierManager::new(tracker))
    }

    #[test]
    fn test_tier_manager_record_access() {
        let manager = create_test_manager();
        manager.record_page_access(1);
        manager.record_page_access(1);
        assert_eq!(manager.get_page_tier(1), StorageTier::Hot);
    }

    #[test]
    fn test_migration_queue() {
        let manager = create_test_manager();
        manager.queue_for_migration(1);
        manager.queue_for_migration(2);
        manager.queue_for_migration(1);

        let batch = manager.get_migration_batch(10);
        assert_eq!(batch.len(), 2);
        assert!(batch.contains(&1));
        assert!(batch.contains(&2));
    }

    #[test]
    fn test_start_stop() {
        let manager = create_test_manager();
        assert!(!manager.is_running());
        manager.start();
        assert!(manager.is_running());
        manager.stop();
        assert!(!manager.is_running());
    }
}
