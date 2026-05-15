#![allow(deprecated)]

use sqlrustgo_gmp::signature::{
    chain::SignedAuditChain, keys::PrivateKey, signed_entry::SignedAuditEntry, SignatureAlgorithm,
    Signer,
};

#[test]
fn test_signed_audit_chain_new() {
    let chain = SignedAuditChain::new();
    assert!(chain.is_empty());
    assert_eq!(chain.len(), 0);
}

#[test]
fn test_signed_audit_chain_append() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let entry = create_test_entry(1, private_key);

    let mut chain = SignedAuditChain::new();
    chain.append(entry).unwrap();
    assert_eq!(chain.len(), 1);
    assert!(!chain.is_empty());
}

#[test]
fn test_signed_audit_chain_get_entry() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let entry = create_test_entry(42, private_key);

    let mut chain = SignedAuditChain::new();
    chain.append(entry).unwrap();

    let retrieved = chain.get_entry(42);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().seq, 42);
}

#[test]
fn test_signed_audit_chain_get_entry_not_found() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let entry = create_test_entry(1, private_key);

    let mut chain = SignedAuditChain::new();
    chain.append(entry).unwrap();

    let retrieved = chain.get_entry(999);
    assert!(retrieved.is_none());
}

#[test]
fn test_signed_audit_chain_verify() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let entry = create_test_entry(1, private_key);

    let mut chain = SignedAuditChain::new();
    chain.append(entry).unwrap();

    let result = chain.verify();
    assert!(result.is_valid);
    assert!(result.invalid_entries.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn test_signed_audit_chain_default() {
    let chain = SignedAuditChain::default();
    assert!(chain.is_empty());
}

fn create_test_entry(seq: u64, private_key: PrivateKey) -> SignedAuditEntry {
    let data = format!("test data {}", seq);
    let signature = private_key.sign(data.as_bytes(), "test_signer").unwrap();

    SignedAuditEntry::new(
        seq,
        [0u8; 32],
        chrono_timestamp(),
        "user-001".to_string(),
        "CREATE".to_string(),
        "test_table".to_string(),
        signature,
        SignatureAlgorithm::Ed25519,
        "test_signer".to_string(),
    )
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[test]
fn test_public_key_to_bytes_ed25519() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let public_key = private_key.to_public();
    let bytes = public_key.to_bytes();
    assert!(!bytes.is_empty());
}

#[test]
fn test_public_key_to_bytes_ecdsa() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::EcdsaP256).unwrap();
    let public_key = private_key.to_public();
    let bytes = public_key.to_bytes();
    assert!(!bytes.is_empty());
}

#[test]
fn test_public_key_algorithm_ed25519() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let public_key = private_key.to_public();
    assert_eq!(public_key.algorithm(), SignatureAlgorithm::Ed25519);
}

#[test]
fn test_public_key_algorithm_ecdsa() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::EcdsaP256).unwrap();
    let public_key = private_key.to_public();
    assert_eq!(public_key.algorithm(), SignatureAlgorithm::EcdsaP256);
}

#[test]
fn test_private_key_algorithm() {
    let ed25519_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    assert_eq!(ed25519_key.algorithm(), SignatureAlgorithm::Ed25519);

    let ecdsa_key = PrivateKey::generate(SignatureAlgorithm::EcdsaP256).unwrap();
    assert_eq!(ecdsa_key.algorithm(), SignatureAlgorithm::EcdsaP256);
}

#[test]
fn test_signed_audit_entry_creation() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let data = b"test data";
    let signature = private_key.sign(data, "test_signer").unwrap();

    let entry = SignedAuditEntry::new(
        1,
        [1u8; 32],
        chrono_timestamp(),
        "user-001".to_string(),
        "INSERT".to_string(),
        "documents".to_string(),
        signature,
        SignatureAlgorithm::Ed25519,
        "test_signer".to_string(),
    );

    assert_eq!(entry.seq, 1);
    assert_eq!(entry.user_id, "user-001");
    assert_eq!(entry.action, "INSERT");
    assert_eq!(entry.table_name, "documents");
    assert_eq!(entry.signer_id, "test_signer");
    assert!(!entry.signature.is_empty());
}

#[test]
fn test_signed_audit_entry_with_optional_fields() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let data = b"test data";
    let signature = private_key.sign(data, "test_signer").unwrap();

    let mut entry = SignedAuditEntry::new(
        1,
        [0u8; 32],
        chrono_timestamp(),
        "user-001".to_string(),
        "UPDATE".to_string(),
        "records".to_string(),
        signature,
        SignatureAlgorithm::Ed25519,
        "test_signer".to_string(),
    );

    entry.record_id = Some("rec-123".to_string());
    entry.old_value = Some("old value".to_string());
    entry.new_value = Some("new value".to_string());
    entry.certificate_path = Some("/path/to/cert.pem".to_string());

    assert_eq!(entry.record_id, Some("rec-123".to_string()));
    assert_eq!(entry.old_value, Some("old value".to_string()));
    assert_eq!(entry.new_value, Some("new value".to_string()));
    assert_eq!(
        entry.certificate_path,
        Some("/path/to/cert.pem".to_string())
    );
}
