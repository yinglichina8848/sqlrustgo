//! Backup Storage Manager - Local and Remote backup storage
//!
//! Provides abstraction for backup storage:
//! - Local filesystem storage
//! - Remote storage (S3-compatible) via HTTP API
//! - Storage backend abstraction

use serde::{Deserialize, Serialize};
use sqlrustgo_types::{SqlError, SqlResult};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum StorageBackend {
    Local(PathBuf),
    Remote(RemoteConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

impl RemoteConfig {
    pub fn new(
        endpoint: &str,
        bucket: &str,
        access_key: &str,
        secret_key: &str,
        region: &str,
    ) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            bucket: bucket.to_string(),
            access_key: access_key.to_string(),
            secret_key: secret_key.to_string(),
            region: region.to_string(),
        }
    }
}

pub trait BackupStorage: Send + Sync {
    fn save(&self, key: &str, data: &[u8]) -> SqlResult<()>;
    fn load(&self, key: &str) -> SqlResult<Vec<u8>>;
    fn delete(&self, key: &str) -> SqlResult<()>;
    fn exists(&self, key: &str) -> SqlResult<bool>;
    fn list(&self, prefix: &str) -> SqlResult<Vec<String>>;
}

pub struct LocalBackupStorage {
    base_path: PathBuf,
}

impl LocalBackupStorage {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(&base_path).ok();
        Self { base_path }
    }

    fn full_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }
}

impl BackupStorage for LocalBackupStorage {
    fn save(&self, key: &str, data: &[u8]) -> SqlResult<()> {
        let path = self.full_path(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SqlError::IoError(e.to_string()))?;
        }
        std::fs::write(&path, data).map_err(|e| SqlError::IoError(e.to_string()))?;
        Ok(())
    }

    fn load(&self, key: &str) -> SqlResult<Vec<u8>> {
        let path = self.full_path(key);
        std::fs::read(&path).map_err(|e| SqlError::IoError(e.to_string()))
    }

    fn delete(&self, key: &str) -> SqlResult<()> {
        let path = self.full_path(key);
        std::fs::remove_file(&path).map_err(|e| SqlError::IoError(e.to_string()))?;
        Ok(())
    }

    fn exists(&self, key: &str) -> SqlResult<bool> {
        let path = self.full_path(key);
        Ok(path.exists())
    }

    fn list(&self, prefix: &str) -> SqlResult<Vec<String>> {
        let mut results = Vec::new();
        let prefix_path = self.base_path.join(prefix);

        if prefix_path.is_dir() {
            for entry in walkdir(&prefix_path) {
                let relative = entry
                    .strip_prefix(&self.base_path)
                    .map_err(|e| SqlError::IoError(e.to_string()))?
                    .to_string_lossy()
                    .to_string();
                results.push(relative);
            }
        }

        Ok(results)
    }
}

fn walkdir(path: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path));
            } else {
                results.push(path);
            }
        }
    }
    results
}

pub struct RemoteBackupStorage {
    config: RemoteConfig,
    client: reqwest::blocking::Client,
}

impl RemoteBackupStorage {
    pub fn new(config: RemoteConfig) -> Self {
        Self {
            config,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn object_url(&self, key: &str) -> String {
        format!(
            "{}/{}/{}",
            self.config.endpoint.trim_end_matches('/'),
            self.config.bucket,
            key
        )
    }
}

impl BackupStorage for RemoteBackupStorage {
    fn save(&self, key: &str, data: &[u8]) -> SqlResult<()> {
        let url = self.object_url(key);

        let response = self
            .client
            .put(&url)
            .header("Content-Type", "application/octet-stream")
            .header("x-amz-acl", "private")
            .header(
                "Authorization",
                format!(
                    "AWS4-HMAC-SHA256 Credential={}/{{}}",
                    self.config.access_key
                ),
            )
            .body(data.to_vec())
            .send()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(SqlError::IoError(format!(
                "Upload failed: {}",
                response.status()
            )))
        }
    }

    fn load(&self, key: &str) -> SqlResult<Vec<u8>> {
        let url = self.object_url(key);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        if response.status().is_success() {
            response
                .bytes()
                .map_err(|e| SqlError::IoError(e.to_string()))
                .map(|b| b.to_vec())
        } else {
            Err(SqlError::IoError(format!(
                "Download failed: {}",
                response.status()
            )))
        }
    }

    fn delete(&self, key: &str) -> SqlResult<()> {
        let url = self.object_url(key);

        let response = self
            .client
            .delete(&url)
            .send()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(SqlError::IoError(format!(
                "Delete failed: {}",
                response.status()
            )))
        }
    }

    fn exists(&self, key: &str) -> SqlResult<bool> {
        let url = self.object_url(key);

        let response = self
            .client
            .head(&url)
            .send()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        Ok(response.status().is_success())
    }

    fn list(&self, prefix: &str) -> SqlResult<Vec<String>> {
        let url = format!(
            "{}/{}?list-type=2&prefix={}",
            self.config.endpoint.trim_end_matches('/'),
            self.config.bucket,
            prefix
        );

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        if response.status().is_success() {
            let body = response
                .text()
                .map_err(|e| SqlError::IoError(e.to_string()))?;
            let keys = parse_s3_list_response(&body);
            Ok(keys)
        } else {
            Err(SqlError::IoError(format!(
                "List failed: {}",
                response.status()
            )))
        }
    }
}

fn parse_s3_list_response(xml: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for line in xml.lines() {
        if line.contains("<Key>") {
            if let Some(start) = line.find("<Key>") {
                let rest = &line[start + 5..];
                if let Some(end) = rest.find("</Key>") {
                    keys.push(rest[..end].to_string());
                }
            }
        }
    }
    keys
}

pub struct BackupStorageManager {
    local: Arc<LocalBackupStorage>,
    remote: Arc<RwLock<Option<RemoteBackupStorage>>>,
    use_remote: Arc<RwLock<bool>>,
}

impl BackupStorageManager {
    pub fn new(local_path: PathBuf) -> Self {
        Self {
            local: Arc::new(LocalBackupStorage::new(local_path)),
            remote: Arc::new(RwLock::new(None)),
            use_remote: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_remote(&mut self, config: RemoteConfig) {
        *self.remote.write().unwrap() = Some(RemoteBackupStorage::new(config));
    }

    pub fn set_use_remote(&self, use_remote: bool) {
        *self.use_remote.write().unwrap() = use_remote;
    }

    pub fn save(&self, key: &str, data: &[u8]) -> SqlResult<()> {
        self.local.save(key, data)?;
        if *self.use_remote.read().unwrap() {
            if let Some(remote) = self.remote.read().unwrap().as_ref() {
                remote.save(key, data)?;
            }
        }
        Ok(())
    }

    pub fn load(&self, key: &str) -> SqlResult<Vec<u8>> {
        if *self.use_remote.read().unwrap() {
            if let Some(remote) = self.remote.read().unwrap().as_ref() {
                return remote.load(key);
            }
        }
        self.local.load(key)
    }

    pub fn delete(&self, key: &str) -> SqlResult<()> {
        self.local.delete(key)?;
        if *self.use_remote.read().unwrap() {
            if let Some(remote) = self.remote.read().unwrap().as_ref() {
                remote.delete(key)?;
            }
        }
        Ok(())
    }

    pub fn exists(&self, key: &str) -> SqlResult<bool> {
        if *self.use_remote.read().unwrap() {
            if let Some(remote) = self.remote.read().unwrap().as_ref() {
                return remote.exists(key);
            }
        }
        self.local.exists(key)
    }

    pub fn list(&self, prefix: &str) -> SqlResult<Vec<String>> {
        if *self.use_remote.read().unwrap() {
            if let Some(remote) = self.remote.read().unwrap().as_ref() {
                return remote.list(prefix);
            }
        }
        self.local.list(prefix)
    }

    pub fn get_local(&self) -> &LocalBackupStorage {
        &self.local
    }
}

pub struct BackupTransfer {
    manager: BackupStorageManager,
}

impl BackupTransfer {
    pub fn new(manager: BackupStorageManager) -> Self {
        Self { manager }
    }

    pub fn sync_to_remote(&self, key: &str) -> SqlResult<()> {
        let data = self.manager.load(key)?;
        if let Some(remote) = self.manager.remote.read().unwrap().as_ref() {
            remote.save(key, &data)?;
        }
        Ok(())
    }

    pub fn sync_from_remote(&self, key: &str) -> SqlResult<()> {
        if let Some(remote) = self.manager.remote.read().unwrap().as_ref() {
            let data = remote.load(key)?;
            self.manager.local.save(key, &data)?;
        }
        Ok(())
    }

    pub fn full_sync(&self) -> SqlResult<u32> {
        let keys = self.manager.local.list("")?;
        let mut synced = 0;
        for key in keys {
            if self.sync_to_remote(&key).is_ok() {
                synced += 1;
            }
        }
        Ok(synced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_storage_save_load() {
        let temp = std::env::temp_dir().join("backup_storage_test");
        let storage = LocalBackupStorage::new(temp.clone());

        storage.save("test/key1.dat", b"hello world").unwrap();
        assert!(storage.exists("test/key1.dat").unwrap());

        let data = storage.load("test/key1.dat").unwrap();
        assert_eq!(data, b"hello world");

        storage.delete("test/key1.dat").unwrap();
        assert!(!storage.exists("test/key1.dat").unwrap());

        std::fs::remove_dir_all(temp).ok();
    }

    #[test]
    fn test_local_storage_list() {
        let temp = std::env::temp_dir().join("backup_list_test");
        let storage = LocalBackupStorage::new(temp.clone());

        storage
            .save("backups/2024-01-01/data.sql", b"data1")
            .unwrap();
        storage
            .save("backups/2024-01-02/data.sql", b"data2")
            .unwrap();

        let files = storage.list("backups").unwrap();
        assert_eq!(files.len(), 2);

        std::fs::remove_dir_all(temp).ok();
    }

    #[test]
    fn test_backup_storage_manager_local_only() {
        let temp = std::env::temp_dir().join("backup_manager_test");
        let manager = BackupStorageManager::new(temp.clone());

        manager.save("test/key.dat", b"test data").unwrap();
        let data = manager.load("test/key.dat").unwrap();
        assert_eq!(data, b"test data");

        std::fs::remove_dir_all(temp).ok();
    }

    #[test]
    fn test_remote_config_creation() {
        let config = RemoteConfig::new(
            "https://s3.amazonaws.com",
            "my-bucket",
            "access-key",
            "secret-key",
            "us-east-1",
        );

        assert_eq!(config.endpoint, "https://s3.amazonaws.com");
        assert_eq!(config.bucket, "my-bucket");
    }

    #[test]
    fn test_parse_s3_list_response() {
        let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Key>backup1.sql</Key>
            <Key>backup2.sql</Key>
        </ListBucketResult>
        "#;

        let keys = parse_s3_list_response(xml);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], "backup1.sql");
        assert_eq!(keys[1], "backup2.sql");
    }
}
