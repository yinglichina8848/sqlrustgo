//! Integration tests for 2PC distributed transactions
//!
//! These tests verify the complete 2PC distributed transaction flow including:
//! - Coordinator and Participant interaction
//! - Router table-to-node resolution
//! - Distributed lock management
//! - Recovery and WAL operations

use sqlrustgo_transaction::{
    dtc::{DistributedTransactionState, TransactionContext},
    Coordinator, DistributedLockManager, GlobalTransactionId, NodeId, Participant, Recovery,
    RecoveryReport, Router, TxOutcome, WalEntry,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_coordinator_initialization() {
    let node_id = NodeId(1);
    let coordinator = Coordinator::new(node_id);

    assert_eq!(coordinator.node_id(), node_id);
}

#[tokio::test]
async fn test_global_transaction_id_creation() {
    let node_id = NodeId(1);
    let gid1 = GlobalTransactionId::new(node_id);
    let gid2 = GlobalTransactionId::new(node_id);

    // Each GID should be unique
    assert_ne!(gid1, gid2, "Each GID should be unique");
    assert_eq!(gid1.node_id, node_id);
    assert_eq!(gid2.node_id, node_id);
}

#[tokio::test]
async fn test_coordinator_begin_transaction() {
    let node_id = NodeId(1);
    let coordinator = Coordinator::new(node_id);

    let gid = coordinator.generate_gid();
    coordinator.begin_transaction(gid.clone()).unwrap();

    let gid2 = coordinator.generate_gid();
    assert_ne!(gid, gid2);
}

#[tokio::test]
async fn test_single_node_transaction_flow() {
    // Setup
    let coordinator_node = NodeId(1);
    let coordinator = Arc::new(Coordinator::new(coordinator_node));

    // Begin transaction
    let gid = coordinator.generate_gid();
    coordinator.begin_transaction(gid.clone()).unwrap();

    // Verify initial state
    assert_eq!(
        coordinator.get_state(&gid),
        Some(DistributedTransactionState::Started)
    );

    // Get participants list
    let participants = vec![2];

    // Prepare phase
    let prepare_result = coordinator.prepare(&gid, &participants).await.unwrap();
    assert!(matches!(
        prepare_result,
        sqlrustgo_transaction::PrepareResult::AllCommitted
    ));

    // Commit phase
    let commit_result = coordinator.commit(&gid).await.unwrap();
    assert!(commit_result.success);

    // State should be None after commit (removed from pending)
    assert_eq!(coordinator.get_state(&gid), None);
}

#[tokio::test]
async fn test_multi_node_transaction_flow() {
    // Setup coordinator
    let coordinator_node = NodeId(1);
    let coordinator = Arc::new(Coordinator::new(coordinator_node));

    // Begin transaction involving multiple nodes
    let gid = coordinator.generate_gid();
    coordinator.begin_transaction(gid.clone()).unwrap();

    let participants = vec![2, 3];

    // Prepare phase - all participants vote commit
    let prepare_result = coordinator.prepare(&gid, &participants).await.unwrap();
    assert!(matches!(
        prepare_result,
        sqlrustgo_transaction::PrepareResult::AllCommitted
    ));

    // Commit phase
    let commit_result = coordinator.commit(&gid).await.unwrap();
    assert!(commit_result.success);
}

#[tokio::test]
async fn test_participant_voting() {
    let node_id = NodeId(2);
    let participant = Participant::new(node_id);

    let request = sqlrustgo_transaction::participant::PrepareRequest {
        gid: "1:1:1000".to_string(),
        coordinator_node_id: "1".to_string(),
        changes: vec![],
    };

    let response = participant.handle_prepare(request).await.unwrap();
    assert_eq!(response.vote, 0, "VoteCommit = 0");
    assert_eq!(response.node_id, "2");
}

#[tokio::test]
async fn test_participant_commit_handling() {
    let node_id = NodeId(2);
    let participant = Participant::new(node_id);

    // First prepare the transaction
    let prepare_request = sqlrustgo_transaction::participant::PrepareRequest {
        gid: "1:1:1000".to_string(),
        coordinator_node_id: "1".to_string(),
        changes: vec![],
    };
    participant.handle_prepare(prepare_request).await.unwrap();

    // Then commit
    let request = sqlrustgo_transaction::participant::CommitRequest {
        gid: "1:1:1000".to_string(),
    };

    let response = participant.handle_commit(request).await.unwrap();
    assert!(response.success);
    assert_eq!(response.node_id, "2");
}

#[tokio::test]
async fn test_participant_rollback_handling() {
    let node_id = NodeId(2);
    let participant = Participant::new(node_id);

    // First prepare the transaction
    let prepare_request = sqlrustgo_transaction::participant::PrepareRequest {
        gid: "1:1:1000".to_string(),
        coordinator_node_id: "1".to_string(),
        changes: vec![],
    };
    participant.handle_prepare(prepare_request).await.unwrap();

    // Then rollback
    let request = sqlrustgo_transaction::participant::RollbackRequest {
        gid: "1:1:1000".to_string(),
        reason: "User requested".to_string(),
    };

    let response = participant.handle_rollback(request).await.unwrap();
    assert!(response.success);
    assert_eq!(response.node_id, "2");
}

#[tokio::test]
async fn test_router_table_registration() {
    let mut router = Router::new();

    router.register_table("users", 1);
    router.register_table("orders", 2);
    router.register_table("products", 3);

    assert_eq!(router.get_node_for_table("users"), Some(1));
    assert_eq!(router.get_node_for_table("orders"), Some(2));
    assert_eq!(router.get_node_for_table("products"), Some(3));
}

#[tokio::test]
async fn test_router_single_node_transaction() {
    let mut router = Router::new();
    router.register_table("users", 1);
    router.register_table("orders", 2);

    // Single table transaction
    let tables1 = vec!["users".to_string()];
    assert!(router.is_single_node_transaction(&tables1));

    // Multi-table transaction (same node)
    router.register_table("profiles", 1);
    let tables2 = vec!["users".to_string(), "profiles".to_string()];
    assert!(router.is_single_node_transaction(&tables2));
}

#[tokio::test]
async fn test_router_multi_node_transaction() {
    let mut router = Router::new();
    router.register_table("users", 1);
    router.register_table("orders", 2);

    // Multi-table transaction across different nodes
    let tables = vec!["users".to_string(), "orders".to_string()];
    assert!(!router.is_single_node_transaction(&tables));
}

#[tokio::test]
async fn test_router_resolve_tables() {
    let mut router = Router::new();
    router.register_table("users", 1);
    router.register_table("orders", 2);
    router.register_table("products", 1);

    let tables = vec![
        "users".to_string(),
        "orders".to_string(),
        "products".to_string(),
    ];

    let nodes = router.resolve_tables(&tables).unwrap();
    assert_eq!(nodes.len(), 2, "Should resolve to 2 unique nodes");
    assert!(nodes.contains(&1));
    assert!(nodes.contains(&2));
}

#[tokio::test]
async fn test_router_unknown_table_error() {
    let mut router = Router::new();
    router.register_table("users", 1);

    let tables = vec!["users".to_string(), "unknown".to_string()];
    let result = router.resolve_tables(&tables);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown table"));
}

#[tokio::test]
async fn test_distributed_lock_manager_acquire_release() {
    let lock_mgr = DistributedLockManager::new();
    let gid = GlobalTransactionId::new(NodeId(1));

    let lock_key = sqlrustgo_transaction::LockKey::Table("users".to_string());

    // Acquire lock
    let result = lock_mgr.try_lock(&gid, &lock_key).await;
    assert!(result.is_ok(), "Should acquire lock");

    // Release lock
    let result = lock_mgr.unlock(&gid).await;
    assert!(result.is_ok(), "Should release lock");

    // Verify lock is released
    assert!(!lock_mgr.is_locked(&lock_key).await);
}

#[tokio::test]
async fn test_distributed_lock_manager_conflict() {
    let lock_mgr = DistributedLockManager::new();
    let gid1 = GlobalTransactionId::new(NodeId(1));
    let gid2 = GlobalTransactionId::new(NodeId(2));

    let lock_key = sqlrustgo_transaction::LockKey::Table("users".to_string());

    // First transaction acquires lock
    let result1 = lock_mgr.try_lock(&gid1, &lock_key).await;
    assert!(result1.is_ok());

    // Second transaction tries to acquire same lock
    let result2 = lock_mgr.try_lock(&gid2, &lock_key).await;
    assert!(result2.is_err(), "Should fail due to lock conflict");
}

#[tokio::test]
async fn test_distributed_lock_manager_same_gid_succeeds() {
    let lock_mgr = DistributedLockManager::new();
    let gid = GlobalTransactionId::new(NodeId(1));

    let lock_key = sqlrustgo_transaction::LockKey::Table("users".to_string());

    // Same transaction can acquire lock multiple times
    let result1 = lock_mgr.try_lock(&gid, &lock_key).await;
    assert!(result1.is_ok());

    let result2 = lock_mgr.try_lock(&gid, &lock_key).await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_distributed_lock_manager_row_lock() {
    let lock_mgr = DistributedLockManager::new();
    let gid = GlobalTransactionId::new(NodeId(1));

    let lock_key = sqlrustgo_transaction::LockKey::Row {
        table: "users".to_string(),
        row_key: vec![1, 2, 3],
    };

    let result = lock_mgr.try_lock(&gid, &lock_key).await;
    assert!(result.is_ok());
    assert!(lock_mgr.is_locked(&lock_key).await);
}

#[tokio::test]
async fn test_distributed_lock_manager_get_holder() {
    let lock_mgr = DistributedLockManager::new();
    let gid = GlobalTransactionId::new(NodeId(1));

    let lock_key = sqlrustgo_transaction::LockKey::Table("users".to_string());

    assert!(lock_mgr.get_holder(&lock_key).await.is_none());

    lock_mgr.try_lock(&gid, &lock_key).await.unwrap();
    assert_eq!(lock_mgr.get_holder(&lock_key).await, Some(gid));
}

#[tokio::test]
async fn test_recovery_initialization() {
    let recovery = Recovery::new(NodeId(1));
    let incomplete = recovery.scan_incomplete_transactions().await.unwrap();
    assert!(
        incomplete.is_empty(),
        "New recovery should have no incomplete transactions"
    );
}

#[tokio::test]
async fn test_recovery_empty_wal_recovery() {
    let recovery = Recovery::new(NodeId(1));
    let report = recovery.recover().await.unwrap();

    assert_eq!(report.committed, 0);
    assert_eq!(report.rolled_back, 0);
    assert_eq!(report.terminated, 0);
}

#[tokio::test]
async fn test_recovery_report_default() {
    let report = RecoveryReport::default();
    assert_eq!(report.committed, 0);
    assert_eq!(report.rolled_back, 0);
    assert_eq!(report.terminated, 0);
}

#[tokio::test]
async fn test_wal_entry_serialization() {
    use serde_json;

    let entry = WalEntry::TxBegin {
        gid: GlobalTransactionId::new(NodeId(1)),
        timestamp: 1000,
    };

    let serialized = serde_json::to_string(&entry).unwrap();
    assert!(serialized.contains("TxBegin"));

    let deserialized: WalEntry = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        WalEntry::TxBegin { gid, timestamp } => {
            assert_eq!(gid.node_id, NodeId(1));
            assert_eq!(timestamp, 1000);
        }
        _ => panic!("Deserialized type mismatch"),
    }
}

#[tokio::test]
async fn test_wal_entry_prepare_serialization() {
    use serde_json;

    let entry = WalEntry::TxPrepare {
        gid: GlobalTransactionId::new(NodeId(1)),
        participants: vec![2, 3, 4],
        timestamp: 2000,
    };

    let serialized = serde_json::to_string(&entry).unwrap();
    assert!(serialized.contains("TxPrepare"));

    let deserialized: WalEntry = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        WalEntry::TxPrepare { participants, .. } => {
            assert_eq!(participants.len(), 3);
        }
        _ => panic!("Deserialized type mismatch"),
    }
}

#[tokio::test]
async fn test_wal_entry_commit_serialization() {
    use serde_json;

    let entry = WalEntry::TxCommit {
        gid: GlobalTransactionId::new(NodeId(1)),
        timestamp: 3000,
    };

    let serialized = serde_json::to_string(&entry).unwrap();
    let deserialized: WalEntry = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        WalEntry::TxCommit { timestamp, .. } => {
            assert_eq!(timestamp, 3000);
        }
        _ => panic!("Deserialized type mismatch"),
    }
}

#[tokio::test]
async fn test_wal_entry_rollback_serialization() {
    use serde_json;

    let entry = WalEntry::TxRollback {
        gid: GlobalTransactionId::new(NodeId(1)),
        reason: "Insufficient funds".to_string(),
        timestamp: 4000,
    };

    let serialized = serde_json::to_string(&entry).unwrap();
    assert!(serialized.contains("TxRollback"));
    assert!(serialized.contains("Insufficient funds"));

    let deserialized: WalEntry = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        WalEntry::TxRollback { reason, .. } => {
            assert_eq!(reason, "Insufficient funds");
        }
        _ => panic!("Deserialized type mismatch"),
    }
}

#[tokio::test]
async fn test_transaction_context_state_transitions() {
    let node_id = NodeId(1);
    let gid = GlobalTransactionId::new(node_id);
    let mut ctx = TransactionContext::new(gid);

    // Initial state should be Started
    assert!(matches!(ctx.state, DistributedTransactionState::Started));

    // Transition to Preparing
    ctx.state = DistributedTransactionState::Preparing;
    assert!(matches!(ctx.state, DistributedTransactionState::Preparing));

    // Transition to Prepared
    ctx.state = DistributedTransactionState::Prepared;
    assert!(matches!(ctx.state, DistributedTransactionState::Prepared));

    // Transition to Committing
    ctx.state = DistributedTransactionState::Committing;
    assert!(matches!(ctx.state, DistributedTransactionState::Committing));

    // Transition to Committed
    ctx.state = DistributedTransactionState::Committed;
    assert!(matches!(ctx.state, DistributedTransactionState::Committed));
}

#[tokio::test]
async fn test_full_2pc_flow_with_router_and_locks() {
    // Setup
    let coordinator = Arc::new(Coordinator::new(NodeId(1)));
    let participant1 = Arc::new(Participant::new(NodeId(2)));
    let participant2 = Arc::new(Participant::new(NodeId(3)));

    let lock_mgr = Arc::new(DistributedLockManager::new());

    let mut router = Router::new();
    router.register_table("users", 2);
    router.register_table("orders", 3);

    // Verify multi-node transaction
    let tables = vec!["users".to_string(), "orders".to_string()];
    assert!(!router.is_single_node_transaction(&tables));

    let nodes = router.resolve_tables(&tables).unwrap();
    assert_eq!(nodes.len(), 2);

    // Begin transaction
    let gid = coordinator.generate_gid();
    coordinator.begin_transaction(gid.clone()).unwrap();

    // Prepare phase
    let prepare_result = coordinator.prepare(&gid, &nodes).await.unwrap();
    assert!(matches!(
        prepare_result,
        sqlrustgo_transaction::PrepareResult::AllCommitted
    ));

    // Acquire locks
    let users_lock = sqlrustgo_transaction::LockKey::Table("users".to_string());
    let orders_lock = sqlrustgo_transaction::LockKey::Table("orders".to_string());

    assert!(lock_mgr.try_lock(&gid, &users_lock).await.is_ok());
    assert!(lock_mgr.try_lock(&gid, &orders_lock).await.is_ok());

    // Commit phase
    let commit_result = coordinator.commit(&gid).await.unwrap();
    assert!(commit_result.success);

    // Release locks
    lock_mgr.unlock(&gid).await.unwrap();

    assert!(!lock_mgr.is_locked(&users_lock).await);
    assert!(!lock_mgr.is_locked(&orders_lock).await);
}

#[tokio::test]
async fn test_coordinator_rollback() {
    let node_id = NodeId(1);
    let coordinator = Coordinator::new(node_id);

    let gid = coordinator.generate_gid();
    coordinator.begin_transaction(gid.clone()).unwrap();
    let participants = vec![2];

    // Prepare
    coordinator.prepare(&gid, &participants).await.unwrap();

    // Rollback
    let result = coordinator.rollback(&gid, "User cancelled").await;
    assert!(result.is_ok());

    // State should be None after rollback (removed from pending)
    assert_eq!(coordinator.get_state(&gid), None);
}

#[tokio::test]
async fn test_tx_outcome_variants() {
    use sqlrustgo_transaction::TxOutcome;

    let committed = TxOutcome::Committed;
    let rolled_back = TxOutcome::RolledBack;
    let unknown = TxOutcome::Unknown;

    // All variants should be constructable
    assert!(matches!(committed, TxOutcome::Committed));
    assert!(matches!(rolled_back, TxOutcome::RolledBack));
    assert!(matches!(unknown, TxOutcome::Unknown));
}

#[tokio::test]
async fn test_router_get_routes() {
    let mut router = Router::new();
    router.register_table("users", 1);
    router.register_table("orders", 2);

    let routes = router.get_routes();
    assert_eq!(routes.len(), 2);
}

#[tokio::test]
async fn test_node_id_equality() {
    let node_id1 = NodeId(1);
    let node_id2 = NodeId(1);
    let node_id3 = NodeId(2);

    assert_eq!(node_id1, node_id2);
    assert_ne!(node_id1, node_id3);
}
