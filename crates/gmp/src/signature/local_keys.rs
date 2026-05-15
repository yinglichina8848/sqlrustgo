use std::path::Path;

use super::error::KeyError;
use super::keys::{Certificate, PrivateKey, PublicKey};
use super::traits::KeyManager;

pub struct LocalKeyManager {
    base_path: std::path::PathBuf,
}

impl LocalKeyManager {
    pub fn new(base_path: &Path) -> Result<Self, KeyError> {
        if !base_path.exists() {
            std::fs::create_dir_all(base_path)?;
        }
        Ok(Self {
            base_path: base_path.to_path_buf(),
        })
    }

    fn key_dir(&self, key_id: &str) -> std::path::PathBuf {
        self.base_path.join(key_id)
    }
}

impl KeyManager for LocalKeyManager {
    fn get_private_key(&self, key_id: &str) -> Result<PrivateKey, KeyError> {
        let path = self.key_dir(key_id).join("private_key.pem");
        let data = std::fs::read(&path).map_err(|_| KeyError::KeyNotFound(key_id.to_string()))?;
        if path.to_string_lossy().contains("ecdsa") || path.to_string_lossy().contains("p256") {
            Ok(PrivateKey::EcdsaP256(data))
        } else {
            Ok(PrivateKey::RsaSha256(data))
        }
    }

    fn get_public_key(&self, key_id: &str) -> Result<PublicKey, KeyError> {
        let path = self.key_dir(key_id).join("public_key.pem");
        let data = std::fs::read(&path).map_err(|_| KeyError::KeyNotFound(key_id.to_string()))?;
        if path.to_string_lossy().contains("ecdsa") || path.to_string_lossy().contains("p256") {
            Ok(PublicKey::EcdsaP256(data))
        } else {
            Ok(PublicKey::RsaSha256(data))
        }
    }

    fn get_certificate(&self, key_id: &str) -> Result<Certificate, KeyError> {
        let path = self.key_dir(key_id).join("certificate.pem");
        let data = std::fs::read(&path).map_err(|_| KeyError::KeyNotFound(key_id.to_string()))?;
        Ok(Certificate {
            data,
            subject: key_id.to_string(),
        })
    }

    fn list_keys(&self) -> Result<Vec<String>, KeyError> {
        let mut keys = Vec::new();
        if self.base_path.is_dir() {
            for entry in std::fs::read_dir(&self.base_path)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    keys.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        Ok(keys)
    }
}
