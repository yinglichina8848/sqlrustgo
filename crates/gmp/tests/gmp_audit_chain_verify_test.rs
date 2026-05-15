use sqlrustgo_gmp::audit_chain::{AuditChain, AuditChainEntry, GENESIS_PREV_HASH};
use sqlrustgo_gmp::audit_chain_tamper::{
    detect_tamper, incremental_verify, quick_verify, verify_entry_checksum, verify_entry_link,
};

fn create_test_entry(seq: u64, prev_hash: [u8; 32]) -> AuditChainEntry {
    AuditChainEntry::new(
        seq,
        prev_hash,
        1000 + seq as i64,
        format!("user{}", seq),
        Some(format!("session{}", seq)),
        "CREATE".to_string(),
        "test_table".to_string(),
        Some(format!("record{}", seq)),
        None,
        Some(r#"{"data":"value"}"#.to_string()),
        seq,
        Some("192.168.1.1".to_string()),
    )
}

#[test]
fn test_verify_entry_checksum_valid() {
    let entry = create_test_entry(1, GENESIS_PREV_HASH);
    assert!(verify_entry_checksum(&entry).is_ok());
    assert!(verify_entry_checksum(&entry).unwrap());
}

#[test]
fn test_verify_entry_checksum_tampered() {
    let mut entry = create_test_entry(1, GENESIS_PREV_HASH);
    entry.user_id = "tampered".to_string();
    assert!(verify_entry_checksum(&entry).is_err());
}

#[test]
fn test_verify_entry_link_genesis() {
    let entry = create_test_entry(1, GENESIS_PREV_HASH);
    assert!(verify_entry_link(&entry, GENESIS_PREV_HASH).is_ok());
}

#[test]
fn test_verify_entry_link_non_genesis() {
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    let entry2 = create_test_entry(2, entry1.checksum);
    assert!(verify_entry_link(&entry2, entry1.checksum).is_ok());
}

#[test]
fn test_quick_verify_empty_chain() {
    let chain = AuditChain::new();
    assert!(quick_verify(&chain));
}

#[test]
fn test_quick_verify_valid_chain() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1).unwrap();
    assert!(quick_verify(&chain));
}

#[test]
fn test_detect_tamper_no_tamper() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1).unwrap();
    assert!(detect_tamper(&chain).is_none());
}

#[test]
fn test_detect_tamper_with_chain() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    let hash1 = entry1.checksum;
    chain.append(entry1).unwrap();

    let entry2 = create_test_entry(2, hash1);
    chain.append(entry2).unwrap();

    assert!(detect_tamper(&chain).is_none());
}

#[test]
fn test_incremental_verify_success() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1.clone()).unwrap();

    let entry2 = create_test_entry(2, entry1.checksum);
    let result = incremental_verify(&chain, &entry2).unwrap();
    assert!(result.passed);
}

#[test]
fn test_incremental_verify_wrong_seq() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1).unwrap();

    let entry_wrong = create_test_entry(5, GENESIS_PREV_HASH);
    let result = incremental_verify(&chain, &entry_wrong).unwrap();
    assert!(!result.passed);
}

#[test]
fn test_incremental_verify_wrong_prev_hash() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1).unwrap();

    let entry_wrong = create_test_entry(2, [0u8; 32]);
    let result = incremental_verify(&chain, &entry_wrong).unwrap();
    assert!(!result.passed);
}

#[test]
fn test_chain_full_verify_success() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    let hash1 = entry1.checksum;
    chain.append(entry1).unwrap();

    let entry2 = create_test_entry(2, hash1);
    let hash2 = entry2.checksum;
    chain.append(entry2).unwrap();

    let entry3 = create_test_entry(3, hash2);
    chain.append(entry3).unwrap();

    assert!(chain.verify_chain().is_ok());
    assert!(chain.verify_chain().unwrap());
}

#[test]
fn test_chain_full_verify_empty_fails() {
    let chain = AuditChain::new();
    assert!(chain.verify_chain().is_err());
}

#[test]
fn test_chain_full_verify_tampered() {
    let mut chain = AuditChain::new();
    let entry1 = create_test_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1.clone()).unwrap();

    let entry2 = create_test_entry(2, entry1.checksum);
    let hash2 = entry2.checksum;
    chain.append(entry2).unwrap();

    let entry3 = create_test_entry(3, hash2);
    chain.append(entry3).unwrap();

    assert!(chain.verify_chain().is_ok());

    let broken_entry = create_test_entry(4, [0u8; 32]);
    let result = chain.append(broken_entry);
    assert!(result.is_err());
}

#[test]
fn test_report_structure() {
    let chain = AuditChain::new();
    let state = chain.get_state();

    assert_eq!(state.next_seq, 1);
    assert_eq!(state.length, 0);
    assert_eq!(state.last_hash, GENESIS_PREV_HASH);
}

#[test]
fn test_entry_seq_validation() {
    let mut chain = AuditChain::new();

    let entry_wrong_seq = create_test_entry(2, GENESIS_PREV_HASH);
    let result = chain.append(entry_wrong_seq);
    assert!(result.is_err());
}

#[test]
fn test_entry_prev_hash_validation() {
    let mut chain = AuditChain::new();

    let entry_wrong_hash = create_test_entry(1, [1u8; 32]);
    let result = chain.append(entry_wrong_hash);
    assert!(result.is_err());
}