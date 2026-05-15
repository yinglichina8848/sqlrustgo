use rustc_hash::FxHashMap;
use std::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum KeyManagerError {
    #[error("Table not found: {0}")]
    TableNotFound(u32),

    #[error("Key rotation failed: {0}")]
    RotationFailed(String),

    #[error("Invalid key state")]
    InvalidKeyState,
}

pub trait KeyManager: Send + Sync {
    fn get_master_key(&self) -> &[u8; 32];
    fn get_dek(&self, table_id: u32) -> Result<[u8; 32], KeyManagerError>;
    fn rotate_dek(&self, table_id: u32) -> Result<(), KeyManagerError>;
    fn get_key_version(&self, table_id: u32) -> u32;
}

const MASTER_KEY_SIZE: usize = 32;

fn derive_dek_from_master(
    master_key: &[u8; MASTER_KEY_SIZE],
    table_id: u32,
    version: u32,
) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(master_key);
    hasher.update(table_id.to_le_bytes());
    hasher.update(version.to_le_bytes());
    let result = hasher.finalize();
    let mut dek = [0u8; 32];
    dek.copy_from_slice(&result);
    dek
}

pub struct BasicKeyManager {
    master_key: [u8; MASTER_KEY_SIZE],
    dek_cache: RwLock<FxHashMap<u32, (u32, [u8; 32])>>,
}

impl BasicKeyManager {
    pub fn new(master_key: [u8; MASTER_KEY_SIZE]) -> Self {
        Self {
            master_key,
            dek_cache: RwLock::new(FxHashMap::default()),
        }
    }

    pub fn with_default_key() -> Self {
        Self::new([0x42u8; 32])
    }
}

impl KeyManager for BasicKeyManager {
    fn get_master_key(&self) -> &[u8; 32] {
        &self.master_key
    }

    fn get_dek(&self, table_id: u32) -> Result<[u8; 32], KeyManagerError> {
        let cache = self.dek_cache.read().unwrap();
        if let Some((_version, dek)) = cache.get(&table_id) {
            return Ok(*dek);
        }
        drop(cache);

        let dek = derive_dek_from_master(&self.master_key, table_id, 1);
        let mut cache = self.dek_cache.write().unwrap();
        cache.insert(table_id, (1, dek));
        Ok(dek)
    }

    fn rotate_dek(&self, table_id: u32) -> Result<(), KeyManagerError> {
        let mut cache = self.dek_cache.write().unwrap();
        let current_version = cache.get(&table_id).map(|(v, _)| *v).unwrap_or(0);
        let new_version = current_version + 1;
        let new_dek = derive_dek_from_master(&self.master_key, table_id, new_version);
        cache.insert(table_id, (new_version, new_dek));
        Ok(())
    }

    fn get_key_version(&self, table_id: u32) -> u32 {
        let cache = self.dek_cache.read().unwrap();
        cache.get(&table_id).map(|(v, _)| *v).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_key_manager_new() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);
        assert_eq!(manager.get_master_key(), &key);
    }

    #[test]
    fn test_basic_key_manager_with_default_key() {
        let manager = BasicKeyManager::with_default_key();
        assert_eq!(manager.get_master_key(), &[0x42u8; 32]);
    }

    #[test]
    fn test_get_dek() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        let dek = manager.get_dek(1);
        assert!(dek.is_ok());
        assert_eq!(dek.unwrap().len(), 32);
    }

    #[test]
    fn test_different_tables_different_deks() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        let dek1 = manager.get_dek(1).unwrap();
        let dek2 = manager.get_dek(2).unwrap();
        let dek3 = manager.get_dek(3).unwrap();

        assert_ne!(dek1, dek2);
        assert_ne!(dek2, dek3);
        assert_ne!(dek1, dek3);
    }

    #[test]
    fn test_same_table_same_dek_without_rotation() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        let dek1 = manager.get_dek(1).unwrap();
        let dek2 = manager.get_dek(1).unwrap();

        assert_eq!(dek1, dek2);
    }

    #[test]
    fn test_rotate_dek() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        let dek_before = manager.get_dek(1).unwrap();
        assert_eq!(manager.get_key_version(1), 1);

        manager.rotate_dek(1).unwrap();

        let dek_after = manager.get_dek(1).unwrap();
        assert_ne!(dek_before, dek_after);
        assert_eq!(manager.get_key_version(1), 2);
    }

    #[test]
    fn test_rotate_multiple_times() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        manager.rotate_dek(1).unwrap();
        manager.rotate_dek(1).unwrap();
        manager.rotate_dek(1).unwrap();

        assert_eq!(manager.get_key_version(1), 3);
    }

    #[test]
    fn test_rotate_different_tables() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        manager.rotate_dek(1).unwrap();
        manager.rotate_dek(2).unwrap();

        assert_eq!(manager.get_key_version(1), 1);
        assert_eq!(manager.get_key_version(2), 1);
        assert_ne!(manager.get_dek(1).unwrap(), manager.get_dek(2).unwrap());
    }

    #[test]
    fn test_key_version_initially_zero() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        assert_eq!(manager.get_key_version(999), 0);
    }

    #[test]
    fn test_key_version_after_get_dek() {
        let key = [0x42u8; 32];
        let manager = BasicKeyManager::new(key);

        let _ = manager.get_dek(42).unwrap();

        assert_eq!(manager.get_key_version(42), 1);
    }

    #[test]
    fn test_derived_dek_consistency() {
        let key = [0x42u8; 32];
        let table_id = 1u32;
        let version = 1u32;

        let derived = derive_dek_from_master(&key, table_id, version);

        let manager = BasicKeyManager::new(key);
        let dek = manager.get_dek(table_id).unwrap();

        assert_eq!(derived, dek);
    }
}
