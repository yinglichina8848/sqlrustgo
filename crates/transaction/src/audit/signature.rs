//! Audit Signature Module
//!
//! Digital signature support for audit events (optional for MVP).

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;

pub struct SignatureManager {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl SignatureManager {
    pub fn new() -> Self {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn from_bytes(key_bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(key_bytes);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(data);
        signature.to_vec()
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        let sig = match Signature::from_slice(signature) {
            Ok(s) => s,
            Err(_) => return false,
        };
        self.verifying_key.verify(data, &sig).is_ok()
    }

    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

impl Default for SignatureManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sign_data(signing_key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let key = SigningKey::from_bytes(signing_key);
    let signature = key.sign(data);
    signature.to_vec()
}

pub fn verify_signature(verifying_key: &[u8; 32], data: &[u8], signature: &[u8]) -> bool {
    let key = VerifyingKey::from_bytes(verifying_key).ok();
    let key = match key {
        Some(k) => k,
        None => return false,
    };
    let sig = match Signature::from_slice(signature) {
        Ok(s) => s,
        Err(_) => return false,
    };
    key.verify(data, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_manager() {
        let manager = SignatureManager::new();
        let data = b"test data";

        let signature = manager.sign(data);
        assert!(manager.verify(data, &signature));
    }

    #[test]
    fn test_signature_verification_fails() {
        let manager = SignatureManager::new();
        let data = b"test data";
        let wrong_data = b"wrong data";

        let signature = manager.sign(data);
        assert!(!manager.verify(wrong_data, &signature));
    }

    #[test]
    fn test_sign_and_verify_functions() {
        let manager = SignatureManager::new();
        let signing_key = manager.signing_key_bytes();
        let verifying_key = manager.verifying_key_bytes();

        let data = b"test data";
        let signature = sign_data(&signing_key, data);

        assert!(verify_signature(&verifying_key, data, &signature));
        assert!(!verify_signature(&verifying_key, b"wrong", &signature));
    }

    #[test]
    fn test_key_serialization() {
        let manager = SignatureManager::new();
        let signing_bytes = manager.signing_key_bytes();
        let verifying_bytes = manager.verifying_key_bytes();

        let manager2 = SignatureManager::from_bytes(&signing_bytes);
        assert_eq!(manager2.verifying_key_bytes(), verifying_bytes);
    }
}
