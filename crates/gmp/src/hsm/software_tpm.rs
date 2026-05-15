use std::collections::HashMap;
use std::sync::RwLock;

use sha2::{Digest, Sha256};

use super::{HsmConfig, HsmError, HsmProvider, HsmProviderType};

pub struct SoftwareTpmProvider {
    keys: RwLock<HashMap<String, Vec<u8>>>,
}

impl SoftwareTpmProvider {
    pub fn new(_config: &HsmConfig) -> Result<Self, HsmError> {
        Ok(Self {
            keys: RwLock::new(HashMap::new()),
        })
    }
}

impl HsmProvider for SoftwareTpmProvider {
    fn generate_key(&self, key_id: &str) -> Result<(), HsmError> {
        let key_bytes: [u8; 32] = rand::random();
        let mut keys = self
            .keys
            .write()
            .map_err(|_| HsmError::KeyGenerationFailed("Lock poisoned".to_string()))?;
        keys.insert(key_id.to_string(), key_bytes.to_vec());
        Ok(())
    }

    fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, HsmError> {
        let keys = self
            .keys
            .read()
            .map_err(|_| HsmError::SigningFailed("Lock poisoned".to_string()))?;

        let key = keys
            .get(key_id)
            .ok_or_else(|| HsmError::KeyNotFound(key_id.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(data);
        let result = hasher.finalize();
        Ok(result.to_vec())
    }

    fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> Result<bool, HsmError> {
        let keys = self
            .keys
            .read()
            .map_err(|_| HsmError::SigningFailed("Lock poisoned".to_string()))?;

        let key = keys
            .get(key_id)
            .ok_or_else(|| HsmError::KeyNotFound(key_id.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(data);
        let computed = hasher.finalize();

        Ok(computed.to_vec() == signature)
    }

    fn delete_key(&self, key_id: &str) -> Result<(), HsmError> {
        let mut keys = self
            .keys
            .write()
            .map_err(|_| HsmError::KeyNotFound("Lock poisoned".to_string()))?;
        keys.remove(key_id);
        Ok(())
    }

    fn list_keys(&self) -> Result<Vec<String>, HsmError> {
        let keys = self
            .keys
            .read()
            .map_err(|_| HsmError::NotAvailable("Lock poisoned".to_string()))?;
        Ok(keys.keys().cloned().collect())
    }
}

pub fn create_provider(config: &HsmConfig) -> Result<Box<dyn HsmProvider>, HsmError> {
    match config.provider_type {
        HsmProviderType::SoftwareTpm => {
            let provider = SoftwareTpmProvider::new(config)?;
            Ok(Box::new(provider))
        }
        _ => Err(HsmError::NotAvailable(format!(
            "Provider {:?} not implemented",
            config.provider_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_tpm_generate_and_sign() {
        let config = HsmConfig::software_tpm();
        let provider = SoftwareTpmProvider::new(&config).unwrap();

        provider.generate_key("test_key").unwrap();
        let signature = provider.sign("test_key", b"test data").unwrap();
        assert_eq!(signature.len(), 32);

        let is_valid = provider
            .verify("test_key", b"test data", &signature)
            .unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_software_tpm_key_not_found() {
        let config = HsmConfig::software_tpm();
        let provider = SoftwareTpmProvider::new(&config).unwrap();

        let result = provider.sign("nonexistent", b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_software_tpm() {
        let config = HsmConfig::software_tpm();
        let provider = create_provider(&config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_create_provider_unsupported() {
        let config = HsmConfig::tpm20("/path");
        let provider = create_provider(&config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_software_tpm_delete_key() {
        let config = HsmConfig::software_tpm();
        let provider = SoftwareTpmProvider::new(&config).unwrap();

        provider.generate_key("test_key").unwrap();
        let result = provider.delete_key("test_key");
        assert!(result.is_ok());

        let result = provider.sign("test_key", b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_software_tpm_list_keys() {
        let config = HsmConfig::software_tpm();
        let provider = SoftwareTpmProvider::new(&config).unwrap();

        let keys = provider.list_keys().unwrap();
        assert!(keys.is_empty());

        provider.generate_key("key1").unwrap();
        provider.generate_key("key2").unwrap();

        let keys = provider.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[test]
    fn test_software_tpm_verify_invalid_signature() {
        let config = HsmConfig::software_tpm();
        let provider = SoftwareTpmProvider::new(&config).unwrap();

        provider.generate_key("test_key").unwrap();

        let is_valid = provider
            .verify("test_key", b"test data", &[0u8; 32])
            .unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_software_tpm_verify_wrong_data() {
        let config = HsmConfig::software_tpm();
        let provider = SoftwareTpmProvider::new(&config).unwrap();

        provider.generate_key("test_key").unwrap();
        let signature = provider.sign("test_key", b"original data").unwrap();

        let is_valid = provider
            .verify("test_key", b"different data", &signature)
            .unwrap();
        assert!(!is_valid);
    }
}
