use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

const GCM_NONCE_SIZE: usize = 12;
const GCM_TAG_SIZE: usize = 16;
const AES_KEY_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct EncryptedPage {
    pub page_id: u32,
    pub nonce: [u8; GCM_NONCE_SIZE],
    pub ciphertext: Vec<u8>,
    pub tag: [u8; GCM_TAG_SIZE],
    pub key_version: u32,
}

#[derive(Debug, Clone)]
pub struct DecryptedPage {
    pub page_id: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    #[error("Decryption failed: authentication tag mismatch (data tampered?)")]
    AuthenticationFailed,

    #[error("Ciphertext too short: minimum {min} bytes, got {actual}")]
    CiphertextTooShort { min: usize, actual: usize },

    #[error("Encryption operation failed: {0}")]
    AesError(String),
}

#[derive(Clone)]
pub struct AesEncryptionManager {
    cipher: Aes256Gcm,
}

impl AesEncryptionManager {
    pub fn new(master_key: &[u8]) -> Result<Self, EncryptionError> {
        if master_key.len() != AES_KEY_SIZE {
            return Err(EncryptionError::InvalidKeySize {
                expected: AES_KEY_SIZE,
                actual: master_key.len(),
            });
        }
        let cipher = Aes256Gcm::new_from_slice(master_key)
            .map_err(|e| EncryptionError::AesError(e.to_string()))?;
        Ok(Self { cipher })
    }

    pub fn encrypt(
        &self,
        page_id: u32,
        data: &[u8],
        key_version: u32,
    ) -> Result<EncryptedPage, EncryptionError> {
        let mut nonce_bytes = [0u8; GCM_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::AesError(e.to_string()))?;

        let tag = Self::extract_tag(&ciphertext);
        let ciphertext_without_tag = &ciphertext[..ciphertext.len() - GCM_TAG_SIZE];

        Ok(EncryptedPage {
            page_id,
            nonce: nonce_bytes,
            ciphertext: ciphertext_without_tag.to_vec(),
            tag,
            key_version,
        })
    }

    pub fn decrypt(&self, page: &EncryptedPage) -> Result<DecryptedPage, EncryptionError> {
        let nonce = Nonce::from_slice(&page.nonce);

        let mut combined = page.ciphertext.clone();
        combined.extend_from_slice(&page.tag);

        let plaintext = self
            .cipher
            .decrypt(nonce, combined.as_ref())
            .map_err(|_| EncryptionError::AuthenticationFailed)?;

        Ok(DecryptedPage {
            page_id: page.page_id,
            data: plaintext,
        })
    }

    fn extract_tag(ciphertext_with_tag: &[u8]) -> [u8; GCM_TAG_SIZE] {
        let mut tag = [0u8; GCM_TAG_SIZE];
        let tag_start = ciphertext_with_tag.len() - GCM_TAG_SIZE;
        tag.copy_from_slice(&ciphertext_with_tag[tag_start..]);
        tag
    }

    pub fn encrypt_bytes(
        &self,
        page_id: u32,
        data: &[u8],
        key_version: u32,
    ) -> Result<Vec<u8>, EncryptionError> {
        let encrypted = self.encrypt(page_id, data, key_version)?;
        let mut result =
            Vec::with_capacity(GCM_NONCE_SIZE + encrypted.ciphertext.len() + GCM_TAG_SIZE);
        result.extend_from_slice(&encrypted.nonce);
        result.extend_from_slice(&encrypted.ciphertext);
        result.extend_from_slice(&encrypted.tag);
        Ok(result)
    }

    pub fn decrypt_bytes(
        &self,
        page_id: u32,
        encrypted_data: &[u8],
        key_version: u32,
    ) -> Result<Vec<u8>, EncryptionError> {
        if encrypted_data.len() < GCM_NONCE_SIZE + GCM_TAG_SIZE {
            return Err(EncryptionError::CiphertextTooShort {
                min: GCM_NONCE_SIZE + GCM_TAG_SIZE,
                actual: encrypted_data.len(),
            });
        }

        let mut nonce = [0u8; GCM_NONCE_SIZE];
        nonce.copy_from_slice(&encrypted_data[..GCM_NONCE_SIZE]);

        let ciphertext_len = encrypted_data.len() - GCM_NONCE_SIZE - GCM_TAG_SIZE;
        let ciphertext_end = GCM_NONCE_SIZE + ciphertext_len;
        let ciphertext = &encrypted_data[GCM_NONCE_SIZE..ciphertext_end];

        let mut tag = [0u8; GCM_TAG_SIZE];
        tag.copy_from_slice(&encrypted_data[ciphertext_end..]);

        let encrypted_page = EncryptedPage {
            page_id,
            nonce,
            ciphertext: ciphertext.to_vec(),
            tag,
            key_version,
        };

        self.decrypt(&encrypted_page).map(|d| d.data)
    }
}

pub trait Crypt: Send + Sync {
    fn encrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32) -> Vec<u8>;
    fn decrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32) -> Vec<u8>;
    fn current_key_version(&self) -> u32;
    fn is_enabled(&self) -> bool;
}

impl Crypt for AesEncryptionManager {
    fn encrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32) -> Vec<u8> {
        self.encrypt_bytes(page_id, data, key_version)
            .expect("encryption should not fail")
    }

    fn decrypt_bytes(&self, page_id: u32, data: &[u8], key_version: u32) -> Vec<u8> {
        self.decrypt_bytes(page_id, data, key_version)
            .expect("decryption should not fail")
    }

    fn current_key_version(&self) -> u32 {
        1
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encryption_manager_new() {
        let key = [0u8; 32];
        let manager = AesEncryptionManager::new(&key);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_aes_encryption_manager_invalid_key_size() {
        let short_key = [0u8; 16];
        let result = AesEncryptionManager::new(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let page_id = 1;
        let data = b"Hello, World! This is test data for encryption.";
        let key_version = 1;

        let encrypted = manager.encrypt(page_id, data, key_version).unwrap();

        assert_eq!(encrypted.page_id, page_id);
        assert_eq!(encrypted.key_version, key_version);
        assert_ne!(encrypted.ciphertext, data);
        assert_eq!(encrypted.nonce.len(), GCM_NONCE_SIZE);
        assert_eq!(encrypted.tag.len(), GCM_TAG_SIZE);

        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted.page_id, page_id);
        assert_eq!(decrypted.data, data);
    }

    #[test]
    fn test_gcm_tag_verification() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let encrypted = manager.encrypt(1, b"test data", 1).unwrap();

        let mut tampered = encrypted.clone();
        tampered.ciphertext[0] ^= 0xFF;

        let result = manager.decrypt(&tampered);
        assert!(result.is_err());
        matches!(result.unwrap_err(), EncryptionError::AuthenticationFailed);
    }

    #[test]
    fn test_different_nonces_for_same_data() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let data = b"Same data";
        let encrypted1 = manager.encrypt(1, data, 1).unwrap();
        let encrypted2 = manager.encrypt(1, data, 1).unwrap();

        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_different_pages_different_output() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let encrypted1 = manager.encrypt(1, b"Page 1 data", 1).unwrap();
        let encrypted2 = manager.encrypt(2, b"Page 2 data", 1).unwrap();

        assert_eq!(encrypted1.page_id, 1);
        assert_eq!(encrypted2.page_id, 2);
    }

    #[test]
    fn test_key_version_preserved() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let encrypted = manager.encrypt(1, b"data", 5).unwrap();
        assert_eq!(encrypted.key_version, 5);

        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted.data, b"data");
    }

    #[test]
    fn test_empty_data() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let encrypted = manager.encrypt(1, &[], 1).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert!(decrypted.data.is_empty());
    }

    #[test]
    fn test_large_data() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let large_data = vec![0xAB; 65536];
        let encrypted = manager.encrypt(1, &large_data, 1).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted.data, large_data);
    }

    #[test]
    fn test_encrypt_decrypt_bytes_roundtrip() {
        let key = [0x42u8; 32];
        let manager = AesEncryptionManager::new(&key).unwrap();

        let data = b"Test data for byte-level encryption.";
        let encrypted = manager.encrypt_bytes(1, data, 1).unwrap();
        assert_ne!(encrypted.as_slice(), data);

        let decrypted = manager.decrypt_bytes(1, &encrypted, 1).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }
}
