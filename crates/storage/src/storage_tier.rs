use sqlrustgo_types::SqlResult;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageTier {
    Hot,
    Cold,
}

#[derive(Debug, Clone)]
pub struct TieringPolicy {
    pub age_threshold_days: u32,
    pub size_threshold_gb: u64,
    pub access_count_threshold: u32,
}

impl Default for TieringPolicy {
    fn default() -> Self {
        Self {
            age_threshold_days: 30,
            size_threshold_gb: 100,
            access_count_threshold: 10,
        }
    }
}

impl TieringPolicy {
    pub fn should_tier_to_cold(&self, age_days: u32, size_gb: u64, access_count: u32) -> bool {
        age_days >= self.age_threshold_days
            || size_gb >= self.size_threshold_gb
            || access_count < self.access_count_threshold
    }
}

pub struct StorageTierManager {
    hot: Arc<dyn super::backup_storage::BackupStorage>,
    cold: Arc<dyn super::backup_storage::BackupStorage>,
    policy: TieringPolicy,
    metadata: Arc<RwLock<HashMap<String, StorageTier>>>,
}

impl StorageTierManager {
    pub fn new(
        hot: Arc<dyn super::backup_storage::BackupStorage>,
        cold: Arc<dyn super::backup_storage::BackupStorage>,
        policy: TieringPolicy,
    ) -> Self {
        Self {
            hot,
            cold,
            policy,
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn save(&self, key: &str, data: &[u8], tier: StorageTier) -> SqlResult<()> {
        match tier {
            StorageTier::Hot => self.hot.save(key, data)?,
            StorageTier::Cold => self.cold.save(key, data)?,
        }

        self.metadata.write().unwrap().insert(key.to_string(), tier);
        Ok(())
    }

    pub fn load(&self, key: &str) -> SqlResult<Vec<u8>> {
        let tier = self.metadata.read().unwrap().get(key).copied();

        match tier {
            Some(StorageTier::Hot) | None => self.hot.load(key).or_else(|_| self.cold.load(key)),
            Some(StorageTier::Cold) => self.cold.load(key),
        }
    }

    pub fn delete(&self, key: &str) -> SqlResult<()> {
        let tier = self.metadata.read().unwrap().get(key).copied();

        match tier {
            Some(StorageTier::Hot) | None => {
                self.hot.delete(key).ok();
                self.cold.delete(key).ok();
            }
            Some(StorageTier::Cold) => {
                self.cold.delete(key)?;
            }
        }

        self.metadata.write().unwrap().remove(key);
        Ok(())
    }

    pub fn exists(&self, key: &str) -> SqlResult<bool> {
        let tier = self.metadata.read().unwrap().get(key).copied();

        match tier {
            Some(StorageTier::Hot) | None => self.hot.exists(key).or_else(|_| self.cold.exists(key)),
            Some(StorageTier::Cold) => self.cold.exists(key),
        }
    }

    pub fn get_tier(&self, key: &str) -> Option<StorageTier> {
        self.metadata.read().unwrap().get(key).copied()
    }

    pub fn run_tiering(&self, items: &[(String, u32, u64, u32)]) -> SqlResult<u32> {
        let mut migrated = 0;

        for (key, age_days, size_gb, access_count) in items {
            if self.policy.should_tier_to_cold(*age_days, *size_gb, *access_count) {
                if let Ok(data) = self.hot.load(key) {
                    if self.cold.save(key, &data).is_ok() {
                        self.metadata.write().unwrap().insert(key.clone(), StorageTier::Cold);
                        migrated += 1;
                    }
                }
            }
        }

        Ok(migrated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup_storage::BackupStorage;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockStorage {
        data: RwLock<HashMap<String, Vec<u8>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: RwLock::new(HashMap::new()),
            }
        }
    }

    impl BackupStorage for MockStorage {
        fn save(&self, key: &str, data: &[u8]) -> SqlResult<()> {
            self.data.write().unwrap().insert(key.to_string(), data.to_vec());
            Ok(())
        }

        fn load(&self, key: &str) -> SqlResult<Vec<u8>> {
            self.data
                .read()
                .unwrap()
                .get(key)
                .cloned()
                .ok_or(sqlrustgo_types::SqlError::IoError(
                    "Key not found".to_string(),
                ))
        }

        fn delete(&self, key: &str) -> SqlResult<()> {
            self.data.write().unwrap().remove(key);
            Ok(())
        }

        fn exists(&self, key: &str) -> SqlResult<bool> {
            Ok(self.data.read().unwrap().contains_key(key))
        }

        fn list(&self, _prefix: &str) -> SqlResult<Vec<String>> {
            Ok(self.data.read().unwrap().keys().cloned().collect())
        }
    }

    #[test]
    fn test_tiering_policy_age() {
        let policy = TieringPolicy::default();
        assert!(policy.should_tier_to_cold(30, 0, 100));
        assert!(!policy.should_tier_to_cold(5, 0, 100));
    }

    #[test]
    fn test_tiering_policy_size() {
        let policy = TieringPolicy::default();
        assert!(policy.should_tier_to_cold(0, 100, 100));
        assert!(!policy.should_tier_to_cold(0, 50, 100));
    }

    #[test]
    fn test_tiering_policy_access_count() {
        let policy = TieringPolicy::default();
        assert!(policy.should_tier_to_cold(0, 0, 5));
        assert!(!policy.should_tier_to_cold(0, 0, 100));
    }

    #[test]
    fn test_storage_tier_manager_save_load() {
        let hot = Arc::new(MockStorage::new());
        let cold = Arc::new(MockStorage::new());
        let manager = StorageTierManager::new(hot.clone(), cold.clone(), TieringPolicy::default());

        manager.save("test_key", b"test_data", StorageTier::Hot).unwrap();

        assert_eq!(manager.load("test_key").unwrap(), b"test_data");
        assert_eq!(manager.get_tier("test_key"), Some(StorageTier::Hot));
    }

    #[test]
    fn test_storage_tier_manager_cold_storage() {
        let hot = Arc::new(MockStorage::new());
        let cold = Arc::new(MockStorage::new());
        let manager = StorageTierManager::new(hot.clone(), cold.clone(), TieringPolicy::default());

        manager.save("cold_key", b"cold_data", StorageTier::Cold).unwrap();

        assert_eq!(manager.load("cold_key").unwrap(), b"cold_data");
        assert_eq!(manager.get_tier("cold_key"), Some(StorageTier::Cold));
    }

    #[test]
    fn test_storage_tier_manager_delete() {
        let hot = Arc::new(MockStorage::new());
        let cold = Arc::new(MockStorage::new());
        let manager = StorageTierManager::new(hot.clone(), cold.clone(), TieringPolicy::default());

        manager.save("delete_key", b"delete_data", StorageTier::Hot).unwrap();
        assert!(manager.exists("delete_key").unwrap());

        manager.delete("delete_key").unwrap();
        assert!(!manager.exists("delete_key").unwrap());
    }

    #[test]
    fn test_run_tiering() {
        let hot = Arc::new(MockStorage::new());
        let cold = Arc::new(MockStorage::new());
        let manager = StorageTierManager::new(hot.clone(), cold.clone(), TieringPolicy::default());

        hot.save("old_key", b"old_data").unwrap();

        let items = vec![("old_key".to_string(), 30_u32, 10_u64, 5_u32)];
        let migrated = manager.run_tiering(&items).unwrap();

        assert_eq!(migrated, 1);
        assert_eq!(manager.get_tier("old_key"), Some(StorageTier::Cold));
        assert!(cold.load("old_key").is_ok());
    }
}