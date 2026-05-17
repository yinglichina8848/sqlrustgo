//! Audit Trail Integration Test
//!
//! Tests the end-to-end audit trail functionality including:
//! - Audit chain creation and append
//! - Checksum verification
//! - Hash chain linking verification
//!
//! This test validates the GA Gate requirement G-S7.

use sqlrustgo_gmp::audit_chain::{AuditChain, AuditChainEntry, GENESIS_PREV_HASH};
use sqlrustgo_gmp::audit_chain_tamper::{
    incremental_verify, quick_verify, verify_entry_checksum, verify_entry_link,
};

/// Helper: create a valid test audit chain entry
fn create_valid_entry(seq: u64, prev_hash: [u8; 32]) -> AuditChainEntry {
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

// ============================================================================
// Chain Integrity Tests
// ============================================================================

#[test]
fn test_audit_chain_empty_quick_verify() {
    let chain = AuditChain::new();
    assert!(quick_verify(&chain), "Empty chain should pass quick verify");
}

#[test]
fn test_audit_chain_single_entry_verify() {
    let mut chain = AuditChain::new();
    let entry = create_valid_entry(1, GENESIS_PREV_HASH);
    chain.append(entry).unwrap();

    assert!(
        quick_verify(&chain),
        "Single entry chain should pass quick verify"
    );
}

#[test]
fn test_audit_chain_multi_entry_linked() {
    let mut chain = AuditChain::new();

    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    let checksum1 = entry1.checksum;
    chain.append(entry1).unwrap();

    let entry2 = create_valid_entry(2, checksum1);
    let checksum2 = entry2.checksum;
    chain.append(entry2).unwrap();

    let entry3 = create_valid_entry(3, checksum2);
    chain.append(entry3).unwrap();

    assert!(quick_verify(&chain), "Multi-entry linked chain should pass");
}

#[test]
fn test_audit_chain_incremental_verify_correct() {
    let mut chain = AuditChain::new();
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1.clone()).unwrap();

    let entry2 = create_valid_entry(2, entry1.checksum);

    let result = incremental_verify(&chain, &entry2);
    assert!(
        result.is_ok(),
        "Entry with correct prev_hash should pass incremental verify"
    );
}

#[test]
fn test_audit_chain_incremental_verify_wrong_seq() {
    let mut chain = AuditChain::new();
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    chain.append(entry1.clone()).unwrap();

    // Entry with wrong seq (3 instead of 2)
    let entry2 = create_valid_entry(3, entry1.checksum);

    let result = incremental_verify(&chain, &entry2);
    assert!(result.is_ok());
    assert!(
        !result.unwrap().passed,
        "Entry with wrong seq should fail incremental verify"
    );
}

// ============================================================================
// Checksum Verification Tests
// ============================================================================

#[test]
fn test_verify_entry_checksum_valid() {
    let entry = create_valid_entry(1, GENESIS_PREV_HASH);
    let result = verify_entry_checksum(&entry);
    assert!(result.is_ok());
    assert!(
        result.unwrap(),
        "Valid entry should pass checksum verification"
    );
}

#[test]
fn test_verify_entry_link_genesis() {
    let entry = create_valid_entry(1, GENESIS_PREV_HASH);
    assert!(
        verify_entry_link(&entry, GENESIS_PREV_HASH).is_ok(),
        "Genesis entry with correct prev_hash should pass"
    );
}

#[test]
fn test_verify_entry_link_non_genesis() {
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    let entry2 = create_valid_entry(2, entry1.checksum);

    assert!(
        verify_entry_link(&entry2, entry1.checksum).is_ok(),
        "Entry with correct prev_hash should pass link verification"
    );
}

#[test]
fn test_verify_entry_link_wrong_prev_hash() {
    // Create a non-genesis entry (seq=2)
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    let entry2 = create_valid_entry(2, entry1.checksum);

    // entry2.prev_hash = entry1.checksum, but we verify with wrong_prev
    let wrong_prev = [1u8; 32];
    let result = verify_entry_link(&entry2, wrong_prev);
    assert!(
        result.is_err(),
        "Entry with wrong prev_hash should fail link verification"
    );
}

// ============================================================================
// Audit Chain Operations Tests
// ============================================================================

#[test]
fn test_audit_chain_append_and_retrieve() {
    let mut chain = AuditChain::new();
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    let entry2 = create_valid_entry(2, entry1.checksum);

    chain.append(entry1.clone()).unwrap();
    chain.append(entry2.clone()).unwrap();

    let entries = chain.entries();
    assert_eq!(entries.len(), 2, "Chain should have 2 entries");
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[1].seq, 2);
}

#[test]
fn test_audit_chain_genesis_hash() {
    let entry = create_valid_entry(1, GENESIS_PREV_HASH);
    assert_eq!(
        entry.prev_hash, GENESIS_PREV_HASH,
        "First entry should have genesis prev_hash"
    );
}

#[test]
fn test_audit_chain_order_preserved() {
    let mut chain = AuditChain::new();
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    let checksum1 = entry1.checksum;
    let entry2 = create_valid_entry(2, checksum1);

    chain.append(entry1).unwrap();
    chain.append(entry2).unwrap();

    let entries = chain.entries();
    assert!(
        entries[0].seq < entries[1].seq,
        "Entries should be in seq order"
    );
}

// ============================================================================
// Performance/Scalability Tests
// ============================================================================

#[test]
fn test_audit_chain_100_entries() {
    let mut chain = AuditChain::new();
    let mut prev_hash = GENESIS_PREV_HASH;

    for i in 1..=100 {
        let entry = create_valid_entry(i, prev_hash);
        prev_hash = entry.checksum;
        chain.append(entry).unwrap();
    }

    assert_eq!(chain.entries().len(), 100, "Chain should have 100 entries");
    assert!(
        quick_verify(&chain),
        "100-entry chain should pass quick verify"
    );
}

#[test]
fn test_incremental_verify_performance() {
    let mut chain = AuditChain::new();
    let mut prev_hash = GENESIS_PREV_HASH;

    for i in 1..=50 {
        let entry = create_valid_entry(i, prev_hash);
        prev_hash = entry.checksum;
        chain.append(entry).unwrap();
    }

    let new_entry = create_valid_entry(51, prev_hash);
    let result = incremental_verify(&chain, &new_entry);
    assert!(result.is_ok(), "Valid chain + entry should pass");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_audit_chain_special_characters_in_data() {
    let entry = AuditChainEntry::new(
        1,
        GENESIS_PREV_HASH,
        1000,
        "user<script>".to_string(),
        Some("session\"with\"quotes".to_string()),
        "CREATE".to_string(),
        "table".to_string(),
        Some("record".to_string()),
        Some(r#"{"key":"value with 'quotes' and \"double quotes\""}"#.to_string()),
        Some(r#"{"new":"data\nwith\nnewlines"}"#.to_string()),
        1,
        Some("192.168.1.1".to_string()),
    );

    assert!(verify_entry_checksum(&entry).unwrap());
    assert!(verify_entry_link(&entry, GENESIS_PREV_HASH).is_ok());
}

#[test]
fn test_audit_chain_empty_optional_fields() {
    // Entry with None for optional fields
    let entry = AuditChainEntry::new(
        1,
        GENESIS_PREV_HASH,
        1000,
        "user1".to_string(),
        None, // No session
        "CREATE".to_string(),
        "test_table".to_string(),
        None, // No record_id
        None, // No old_value
        None, // No new_value
        1,
        None, // No IP
    );

    assert!(verify_entry_checksum(&entry).unwrap());
    assert!(verify_entry_link(&entry, GENESIS_PREV_HASH).is_ok());
}

#[test]
fn test_audit_chain_get_entry() {
    let mut chain = AuditChain::new();
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    let entry2 = create_valid_entry(2, entry1.checksum);

    chain.append(entry1.clone()).unwrap();
    chain.append(entry2.clone()).unwrap();

    let retrieved = chain.get_entry(1);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().seq, 1);

    let retrieved = chain.get_entry(2);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().seq, 2);

    let retrieved = chain.get_entry(99); // Out of range
    assert!(retrieved.is_none());
}

#[test]
fn test_audit_chain_get_recent() {
    let mut chain = AuditChain::new();
    let entry1 = create_valid_entry(1, GENESIS_PREV_HASH);
    let entry2 = create_valid_entry(2, entry1.checksum);
    let entry3 = create_valid_entry(3, entry2.checksum);

    chain.append(entry1).unwrap();
    chain.append(entry2).unwrap();
    chain.append(entry3).unwrap();

    let recent = chain.get_recent(2);
    assert_eq!(recent.len(), 2, "Should return last 2 entries");
    assert_eq!(recent[0].seq, 2);
    assert_eq!(recent[1].seq, 3);
}
