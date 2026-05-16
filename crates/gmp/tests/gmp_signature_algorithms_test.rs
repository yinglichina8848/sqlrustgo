#![allow(deprecated)]

use sqlrustgo_gmp::signature::{
    algorithms::SignatureAlgorithm,
    error::{KeyError, SignatureError},
    keys::PrivateKey,
    local_keys::LocalKeyManager,
    traits::KeyManager,
    Signer,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_signature_algorithm_display_ed25519() {
    let algo = SignatureAlgorithm::Ed25519;
    assert_eq!(format!("{}", algo), "Ed25519");
}

#[test]
fn test_signature_algorithm_display_ecdsa() {
    let algo = SignatureAlgorithm::EcdsaP256;
    assert_eq!(format!("{}", algo), "ECDSA P-256");
}

#[test]
fn test_signature_algorithm_eq() {
    assert_eq!(SignatureAlgorithm::Ed25519, SignatureAlgorithm::Ed25519);
    assert_eq!(SignatureAlgorithm::EcdsaP256, SignatureAlgorithm::EcdsaP256);
    assert_ne!(SignatureAlgorithm::Ed25519, SignatureAlgorithm::EcdsaP256);
}

#[test]
fn test_signature_error_messages() {
    let err = SignatureError::InvalidSignature("test".to_string());
    assert!(err.to_string().contains("Invalid signature"));
    assert!(err.to_string().contains("test"));

    let err = SignatureError::SigningFailed("signing error".to_string());
    assert!(err.to_string().contains("Signing failed"));

    let err = SignatureError::VerificationFailed("verification error".to_string());
    assert!(err.to_string().contains("Verification failed"));

    let err = SignatureError::UnsupportedAlgorithm("unknown".to_string());
    assert!(err.to_string().contains("Unsupported algorithm"));

    let err = SignatureError::InvalidFormat("bad format".to_string());
    assert!(err.to_string().contains("Invalid format"));
}

#[test]
fn test_key_error_messages() {
    let err = KeyError::KeyNotFound("key-123".to_string());
    assert!(err.to_string().contains("Key not found"));
    assert!(err.to_string().contains("key-123"));

    let err = KeyError::InvalidFormat("invalid key format".to_string());
    assert!(err.to_string().contains("Invalid key format"));
}

#[test]
fn test_local_key_manager_new_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");

    {
        let _manager = LocalKeyManager::new(&path).unwrap();
        assert!(path.exists());
    }
}

#[test]
fn test_local_key_manager_new_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");

    fs::create_dir_all(&path).unwrap();

    let _manager = LocalKeyManager::new(&path).unwrap();
    assert!(path.exists());
}

#[test]
fn test_local_key_manager_get_private_key_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");
    let manager = LocalKeyManager::new(&path).unwrap();

    let result = manager.get_private_key("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_local_key_manager_get_public_key_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");
    let manager = LocalKeyManager::new(&path).unwrap();

    let result = manager.get_public_key("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_local_key_manager_get_certificate_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");
    let manager = LocalKeyManager::new(&path).unwrap();

    let result = manager.get_certificate("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_local_key_manager_list_keys_empty() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");
    let manager = LocalKeyManager::new(&path).unwrap();

    let keys = manager.list_keys().unwrap();
    assert!(keys.is_empty());
}

#[test]
fn test_local_key_manager_roundtrip_ed25519() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");

    let manager = LocalKeyManager::new(&path).unwrap();

    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let key_id = "test-ed25519-key";

    let key_dir = path.join(key_id);
    fs::create_dir_all(&key_dir).unwrap();
    fs::write(
        key_dir.join("private_key.pem"),
        private_key.algorithm().to_string().as_bytes(),
    )
    .unwrap();
    fs::write(key_dir.join("public_key.pem"), b"public key data").unwrap();
    fs::write(key_dir.join("certificate.pem"), b"certificate data").unwrap();

    let retrieved = manager.get_private_key(key_id);
    assert!(retrieved.is_ok());

    let public = manager.get_public_key(key_id);
    assert!(public.is_ok());

    let cert = manager.get_certificate(key_id);
    assert!(cert.is_ok());
    assert_eq!(cert.unwrap().subject, key_id);
}

#[test]
fn test_local_key_manager_list_keys_with_entries() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("keys");

    let manager = LocalKeyManager::new(&path).unwrap();

    fs::create_dir_all(path.join("key1")).unwrap();
    fs::create_dir_all(path.join("key2")).unwrap();

    let keys = manager.list_keys().unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"key1".to_string()));
    assert!(keys.contains(&"key2".to_string()));
}

#[test]
fn test_signed_audit_chain_multiple_entries() {
    let mut chain = sqlrustgo_gmp::signature::chain::SignedAuditChain::new();

    for i in 1..=5 {
        let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
        let data = format!("data {}", i);
        let signature = private_key.sign(data.as_bytes(), "test").unwrap();

        let entry = sqlrustgo_gmp::signature::signed_entry::SignedAuditEntry::new(
            i,
            [i as u8; 32],
            chrono_timestamp(),
            "user".to_string(),
            "CREATE".to_string(),
            "table".to_string(),
            signature,
            SignatureAlgorithm::Ed25519,
            "test".to_string(),
        );
        chain.append(entry).unwrap();
    }

    assert_eq!(chain.len(), 5);

    for i in 1..=5 {
        let entry = chain.get_entry(i).unwrap();
        assert_eq!(entry.seq, i as u64);
    }

    assert!(chain.get_entry(100).is_none());
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
