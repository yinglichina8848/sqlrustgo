use crate::encryption::{AesEncryptionManager, DecryptedPage, EncryptedPage, EncryptionError};
use crate::key_manager::{KeyManager, KeyManagerError};
use rustc_hash::FxHashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::RwLock;

const PAGE_FILE_EXTENSION: &str = ".enc";

#[derive(Debug, thiserror::Error)]
pub enum EncryptedStorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    Encryption(#[from] EncryptionError),

    #[error("Key manager error: {0}")]
    KeyManager(#[from] KeyManagerError),

    #[error("Page not found: {0}")]
    PageNotFound(u32),

    #[error("Invalid page format")]
    InvalidFormat,
}

pub struct EncryptedFileStorage {
    data_dir: PathBuf,
    key_manager: Box<dyn KeyManager>,
    page_cache: RwLock<FxHashMap<u32, DecryptedPage>>,
    table_id: u32,
}

impl EncryptedFileStorage {
    pub fn new(
        data_dir: PathBuf,
        key_manager: Box<dyn KeyManager>,
        table_id: u32,
    ) -> Result<Self, EncryptedStorageError> {
        fs::create_dir_all(&data_dir)?;
        Ok(Self {
            data_dir,
            key_manager,
            page_cache: RwLock::new(FxHashMap::default()),
            table_id,
        })
    }

    fn page_path(&self, page_id: u32) -> PathBuf {
        self.data_dir
            .join(format!("page_{:08}{}", page_id, PAGE_FILE_EXTENSION))
    }

    pub fn write_page(&self, page: &DecryptedPage) -> Result<(), EncryptedStorageError> {
        let dek = self.key_manager.get_dek(self.table_id)?;
        let enc_manager = AesEncryptionManager::new(&dek)?;
        let key_version = self.key_manager.get_key_version(self.table_id);

        let encrypted = enc_manager.encrypt(page.page_id, &page.data, key_version)?;

        let path = self.page_path(page.page_id);
        let mut file = File::create(&path)?;

        file.write_all(&page.page_id.to_le_bytes())?;
        file.write_all(&encrypted.nonce)?;
        file.write_all(&(encrypted.ciphertext.len() as u32).to_le_bytes())?;
        file.write_all(&encrypted.ciphertext)?;
        file.write_all(&encrypted.tag)?;
        file.write_all(&encrypted.key_version.to_le_bytes())?;

        file.flush()?;

        let mut cache = self.page_cache.write().unwrap();
        cache.insert(page.page_id, page.clone());

        Ok(())
    }

    pub fn read_page(&self, page_id: u32) -> Result<DecryptedPage, EncryptedStorageError> {
        {
            let cache = self.page_cache.read().unwrap();
            if let Some(page) = cache.get(&page_id) {
                return Ok(page.clone());
            }
        }

        let path = self.page_path(page_id);
        if !path.exists() {
            return Err(EncryptedStorageError::PageNotFound(page_id));
        }

        let mut file = File::open(&path)?;

        let mut page_id_bytes = [0u8; 4];
        file.read_exact(&mut page_id_bytes)?;
        let _read_page_id = u32::from_le_bytes(page_id_bytes);

        let mut nonce = [0u8; 12];
        file.read_exact(&mut nonce)?;

        let mut ciphertext_len_bytes = [0u8; 4];
        file.read_exact(&mut ciphertext_len_bytes)?;
        let ciphertext_len = u32::from_le_bytes(ciphertext_len_bytes) as usize;

        let mut ciphertext = vec![0u8; ciphertext_len];
        file.read_exact(&mut ciphertext)?;

        let mut tag = [0u8; 16];
        file.read_exact(&mut tag)?;

        let mut key_version_bytes = [0u8; 4];
        file.read_exact(&mut key_version_bytes)?;
        let key_version = u32::from_le_bytes(key_version_bytes);

        let encrypted_page = EncryptedPage {
            page_id,
            nonce,
            ciphertext,
            tag,
            key_version,
        };

        let dek = self.key_manager.get_dek(self.table_id)?;
        let enc_manager = AesEncryptionManager::new(&dek)?;
        let decrypted = enc_manager.decrypt(&encrypted_page)?;

        let mut cache = self.page_cache.write().unwrap();
        cache.insert(page_id, decrypted.clone());

        Ok(decrypted)
    }

    pub fn delete_page(&self, page_id: u32) -> std::io::Result<()> {
        let path = self.page_path(page_id);
        if path.exists() {
            fs::remove_file(path)?;
        }

        let mut cache = self.page_cache.write().unwrap();
        cache.remove(&page_id);

        Ok(())
    }

    pub fn flush(&self) -> Result<(), EncryptedStorageError> {
        let cache = self.page_cache.read().unwrap();
        for page in cache.values() {
            let dek = self.key_manager.get_dek(self.table_id)?;
            let enc_manager = AesEncryptionManager::new(&dek)?;
            let key_version = self.key_manager.get_key_version(self.table_id);

            let encrypted = enc_manager.encrypt(page.page_id, &page.data, key_version)?;

            let path = self.page_path(page.page_id);
            let mut file = File::create(&path)?;

            file.write_all(&page.page_id.to_le_bytes())?;
            file.write_all(&encrypted.nonce)?;
            file.write_all(&(encrypted.ciphertext.len() as u32).to_le_bytes())?;
            file.write_all(&encrypted.ciphertext)?;
            file.write_all(&encrypted.tag)?;
            file.write_all(&encrypted.key_version.to_le_bytes())?;

            file.flush()?;
        }
        Ok(())
    }

    pub fn clear_cache(&self) {
        let mut cache = self.page_cache.write().unwrap();
        cache.clear();
    }

    pub fn get_cached_page(&self, page_id: u32) -> Option<DecryptedPage> {
        let cache = self.page_cache.read().unwrap();
        cache.get(&page_id).cloned()
    }

    pub fn cache_size(&self) -> usize {
        let cache = self.page_cache.read().unwrap();
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_manager::BasicKeyManager;
    use std::fs::remove_dir_all;

    fn create_test_storage(temp_dir: &PathBuf) -> EncryptedFileStorage {
        let master_key = [0x42u8; 32];
        let key_manager = Box::new(BasicKeyManager::new(master_key));
        let table_id = 1;

        EncryptedFileStorage::new(temp_dir.clone(), key_manager, table_id).unwrap()
    }

    #[test]
    fn test_write_and_read_page() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_write");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let page = DecryptedPage {
            page_id: 1,
            data: b"Hello, Encrypted World!".to_vec(),
        };

        storage.write_page(&page).unwrap();

        let read_page = storage.read_page(1).unwrap();
        assert_eq!(read_page.page_id, page.page_id);
        assert_eq!(read_page.data, page.data);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_read_nonexistent_page() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_read_none");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let result = storage.read_page(999);
        assert!(result.is_err());
        matches!(
            result.unwrap_err(),
            EncryptedStorageError::PageNotFound(999)
        );

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_delete_page() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_delete");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let page = DecryptedPage {
            page_id: 1,
            data: b"Data to delete".to_vec(),
        };

        storage.write_page(&page).unwrap();
        assert!(storage.read_page(1).is_ok());

        storage.delete_page(1).unwrap();
        assert!(storage.read_page(1).is_err());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multiple_pages() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_multi");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        for i in 1..=5 {
            let page = DecryptedPage {
                page_id: i,
                data: format!("Page {} data", i).into_bytes(),
            };
            storage.write_page(&page).unwrap();
        }

        for i in 1..=5 {
            let read_page = storage.read_page(i).unwrap();
            assert_eq!(read_page.data, format!("Page {} data", i).into_bytes());
        }

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cache_works() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_cache");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let page = DecryptedPage {
            page_id: 1,
            data: b"Cached data".to_vec(),
        };

        assert_eq!(storage.cache_size(), 0);

        storage.write_page(&page).unwrap();
        assert_eq!(storage.cache_size(), 1);

        let cached = storage.get_cached_page(1);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().data, b"Cached data");

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_clear");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let page = DecryptedPage {
            page_id: 1,
            data: b"Data".to_vec(),
        };

        storage.write_page(&page).unwrap();
        assert_eq!(storage.cache_size(), 1);

        storage.clear_cache();
        assert_eq!(storage.cache_size(), 0);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_flush_persists_to_disk() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_flush");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let page = DecryptedPage {
            page_id: 1,
            data: b"Flush test".to_vec(),
        };

        storage.write_page(&page).unwrap();
        storage.clear_cache();

        assert!(storage.get_cached_page(1).is_none());

        let read_page = storage.read_page(1).unwrap();
        assert_eq!(read_page.data, b"Flush test");

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_key_rotation_affects_new_pages() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_rotation");
        let _ = remove_dir_all(&temp_dir);

        let master_key = [0x42u8; 32];
        let key_manager = Box::new(BasicKeyManager::new(master_key));
        let table_id = 1;

        let storage = EncryptedFileStorage::new(temp_dir.clone(), key_manager, table_id).unwrap();

        let page1 = DecryptedPage {
            page_id: 1,
            data: b"Before rotation".to_vec(),
        };
        storage.write_page(&page1).unwrap();

        storage.key_manager.rotate_dek(table_id).unwrap();

        let page2 = DecryptedPage {
            page_id: 2,
            data: b"After rotation".to_vec(),
        };
        storage.write_page(&page2).unwrap();

        let read_page1 = storage.read_page(1).unwrap();
        assert_eq!(read_page1.data, b"Before rotation");

        let read_page2 = storage.read_page(2).unwrap();
        assert_eq!(read_page2.data, b"After rotation");

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_data_integrity() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_integrity");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let original_data = (0..1000).map(|i| (i % 256) as u8).collect::<Vec<_>>();
        let page = DecryptedPage {
            page_id: 1,
            data: original_data.clone(),
        };

        storage.write_page(&page).unwrap();
        storage.clear_cache();

        let read_page = storage.read_page(1).unwrap();
        assert_eq!(read_page.data, original_data);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_empty_data_page() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_encrypted_empty");
        let _ = remove_dir_all(&temp_dir);

        let storage = create_test_storage(&temp_dir);

        let page = DecryptedPage {
            page_id: 1,
            data: vec![],
        };

        storage.write_page(&page).unwrap();
        storage.clear_cache();

        let read_page = storage.read_page(1).unwrap();
        assert!(read_page.data.is_empty());

        let _ = remove_dir_all(&temp_dir);
    }
}
