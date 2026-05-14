//! L1 Unit Tests for IdempotencyRegistry

use super::*;

#[test]
fn test_registry_new_is_empty() {
    let registry = IdempotencyRegistry::new();
    assert!(registry.get_state("nonexistent").unwrap().is_none());
}

#[test]
fn test_register_new_request() {
    let registry = IdempotencyRegistry::new();
    let hash = [0u8; 32];

    let is_idempotent = registry.check_and_register("txn-1", hash, 1).unwrap();
    assert!(!is_idempotent); // New request, not idempotent
}

#[test]
fn test_same_request_same_hash_is_idempotent() {
    let registry = IdempotencyRegistry::new();
    let hash = [0u8; 32];

    registry.check_and_register("txn-1", hash, 1).unwrap();
    let is_idempotent = registry.check_and_register("txn-1", hash, 2).unwrap();
    assert!(is_idempotent);
}

#[test]
fn test_same_key_different_hash_rejected() {
    let registry = IdempotencyRegistry::new();

    registry.check_and_register("txn-1", [0u8; 32], 1).unwrap();
    let result = registry.check_and_register("txn-1", [1u8; 32], 2);

    assert!(matches!(result, Err(IdempotencyError::HashMismatch(_))));
}

#[test]
fn test_mark_committed() {
    let registry = IdempotencyRegistry::new();
    let hash = [0u8; 32];

    registry.check_and_register("txn-1", hash, 1).unwrap();
    registry.mark_committed("txn-1").unwrap();

    let state = registry.get_state("txn-1").unwrap().unwrap();
    assert!(matches!(state, IdempotencyState::Committed));
}

#[test]
fn test_mark_rejected() {
    let registry = IdempotencyRegistry::new();
    let hash = [0u8; 32];

    registry.check_and_register("txn-1", hash, 1).unwrap();
    registry.mark_rejected("txn-1", "Invalid request").unwrap();

    let state = registry.get_state("txn-1").unwrap().unwrap();
    assert!(matches!(state, IdempotencyState::Rejected { reason } if reason == "Invalid request"));
}

#[test]
fn test_pending_state_is_idempotent() {
    let registry = IdempotencyRegistry::new();
    let hash = [0u8; 32];

    registry.check_and_register("txn-1", hash, 1).unwrap();
    let result = registry.check_and_register("txn-1", hash, 2).unwrap();

    // Pending state returns false (operation in progress, not yet idempotent)
    assert!(!result);
}

#[test]
fn test_multiple_keys_independent() {
    let registry = IdempotencyRegistry::new();
    let hash = [0u8; 32];

    registry.check_and_register("txn-1", hash, 1).unwrap();
    registry.check_and_register("txn-2", hash, 2).unwrap();

    // Both should be registered independently
    assert!(registry.get_state("txn-1").unwrap().is_some());
    assert!(registry.get_state("txn-2").unwrap().is_some());
}

#[test]
fn test_previously_rejected_error() {
    let registry = IdempotencyRegistry::new();
    let hash = [0u8; 32];

    registry.check_and_register("txn-1", hash, 1).unwrap();
    registry.mark_rejected("txn-1", "Bad request").unwrap();

    let result = registry.check_and_register("txn-1", hash, 2);
    assert!(matches!(
        result,
        Err(IdempotencyError::PreviouslyRejected(_))
    ));
}
