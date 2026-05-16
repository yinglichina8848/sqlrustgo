use rustc_hash::FxHashMap;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub struct PageAccessInfo {
    pub page_id: u32,
    access_count: AtomicU64,
    last_access_time: AtomicU64,
    last_tier_change: AtomicU64,
    current_tier: Mutex<super::StorageTier>,
}

impl Debug for PageAccessInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PageAccessInfo")
            .field("page_id", &self.page_id)
            .finish()
    }
}

impl PageAccessInfo {
    pub fn new(page_id: u32) -> Self {
        let now = current_time_secs();
        Self {
            page_id,
            access_count: AtomicU64::new(1),
            last_access_time: AtomicU64::new(now),
            last_tier_change: AtomicU64::new(now),
            current_tier: Mutex::new(super::StorageTier::Hot),
        }
    }

    pub fn record_access(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.last_access_time.store(current_time_secs(), Ordering::Relaxed);
    }

    pub fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }

    pub fn last_access_seconds_ago(&self) -> u64 {
        current_time_secs() - self.last_access_time.load(Ordering::Relaxed)
    }

    pub fn current_tier(&self) -> super::StorageTier {
        *self.current_tier.lock().unwrap()
    }

    pub fn update_tier(&self, new_tier: super::StorageTier) {
        *self.current_tier.lock().unwrap() = new_tier;
        self.last_tier_change.store(current_time_secs(), Ordering::Relaxed);
    }

    pub fn minutes_since_last_access(&self) -> u64 {
        self.last_access_seconds_ago() / 60
    }

    pub fn days_since_last_access(&self) -> u64 {
        self.last_access_seconds_ago() / (24 * 60 * 60)
    }
}

fn current_time_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PageAccessTracker {
    access_info: RwLock<FxHashMap<u32, Arc<PageAccessInfo>>>,
    hot_threshold_access_count: u64,
    warm_threshold_access_count: u64,
    warm_threshold_secs: u64,
    cold_threshold_secs: u64,
    archive_threshold_secs: u64,
}

impl Default for PageAccessTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PageAccessTracker {
    pub fn new() -> Self {
        Self {
            access_info: RwLock::new(FxHashMap::default()),
            hot_threshold_access_count: 10,
            warm_threshold_access_count: 5,
            warm_threshold_secs: 30 * 24 * 60 * 60,
            cold_threshold_secs: 90 * 24 * 60 * 60,
            archive_threshold_secs: 365 * 24 * 60 * 60,
        }
    }

    pub fn with_config(
        hot_access: u64,
        warm_access: u64,
        warm_secs: u64,
        cold_secs: u64,
        archive_secs: u64,
    ) -> Self {
        Self {
            access_info: RwLock::new(FxHashMap::default()),
            hot_threshold_access_count: hot_access,
            warm_threshold_access_count: warm_access,
            warm_threshold_secs: warm_secs,
            cold_threshold_secs: cold_secs,
            archive_threshold_secs: archive_secs,
        }
    }

    pub fn record_access(&self, page_id: u32) -> Arc<PageAccessInfo> {
        let info = {
            let mut guard = self.access_info.write().unwrap();
            if let Some(existing) = guard.get(&page_id) {
                existing.record_access();
                Arc::clone(existing)
            } else {
                let info = Arc::new(PageAccessInfo::new(page_id));
                guard.insert(page_id, Arc::clone(&info));
                info
            }
        };
        info
    }

    pub fn get_access_info(&self, page_id: u32) -> Option<Arc<PageAccessInfo>> {
        let guard = self.access_info.read().unwrap();
        guard.get(&page_id).map(Arc::clone)
    }

    pub fn get_recommended_tier(&self, page_id: u32) -> super::StorageTier {
        let info = match self.get_access_info(page_id) {
            Some(i) => i,
            None => return super::StorageTier::Hot,
        };

        let secs = info.last_access_seconds_ago();

        if secs > self.archive_threshold_secs {
            super::StorageTier::Archive
        } else if secs > self.cold_threshold_secs {
            super::StorageTier::Cold
        } else if secs > self.warm_threshold_secs {
            super::StorageTier::Warm
        } else {
            super::StorageTier::Hot
        }
    }

    pub fn get_page_ids_needing_migration(&self, from_tier: super::StorageTier) -> Vec<u32> {
        let guard = self.access_info.read().unwrap();
        let mut result = Vec::new();

        for (&page_id, info) in guard.iter() {
            if info.current_tier() == from_tier {
                let recommended = self.get_recommended_tier_for_info(info.as_ref());
                if recommended != from_tier {
                    result.push(page_id);
                }
            }
        }

        result
    }

    fn get_recommended_tier_for_info(&self, info: &PageAccessInfo) -> super::StorageTier {
        let secs = info.last_access_seconds_ago();

        if secs > self.archive_threshold_secs {
            super::StorageTier::Archive
        } else if secs > self.cold_threshold_secs {
            super::StorageTier::Cold
        } else if secs > self.warm_threshold_secs {
            super::StorageTier::Warm
        } else {
            super::StorageTier::Hot
        }
    }

    pub fn remove_page(&self, page_id: u32) {
        let mut guard = self.access_info.write().unwrap();
        guard.remove(&page_id);
    }

    pub fn clear(&self) {
        let mut guard = self.access_info.write().unwrap();
        guard.clear();
    }

    pub fn len(&self) -> usize {
        let guard = self.access_info.read().unwrap();
        guard.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StorageTier;

    #[test]
    fn test_page_access_info() {
        let info = PageAccessInfo::new(1);
        assert_eq!(info.page_id, 1);
        assert_eq!(info.access_count(), 1);
        info.record_access();
        assert_eq!(info.access_count(), 2);
    }

    #[test]
    fn test_tracker_record_access() {
        let tracker = PageAccessTracker::new();
        let info = tracker.record_access(1);
        assert_eq!(info.access_count(), 1);

        tracker.record_access(1);
        assert_eq!(info.access_count(), 2);

        let retrieved = tracker.get_access_info(1).unwrap();
        assert_eq!(retrieved.page_id, 1);
    }

    #[test]
    fn test_get_recommended_tier() {
        let tracker = PageAccessTracker::new();
        tracker.record_access(42);

        let tier = tracker.get_recommended_tier(42);
        assert_eq!(tier, StorageTier::Hot);

        let tier_unknown = tracker.get_recommended_tier(999);
        assert_eq!(tier_unknown, StorageTier::Hot);
    }

    #[test]
    fn test_page_access_info_tier() {
        let info = PageAccessInfo::new(42);
        assert_eq!(info.current_tier(), StorageTier::Hot);
        info.update_tier(StorageTier::Cold);
        assert_eq!(info.current_tier(), StorageTier::Cold);
    }
}