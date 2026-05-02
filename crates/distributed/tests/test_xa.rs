//! Integration tests for XA transaction coordinator
//!
//! These tests verify the MySQL 5.7 compatible XA transaction flow:
//! - XA START 'xid' - Begin XA transaction
//! - XA END 'xid' - Mark end of XA transaction
//! - XA PREPARE 'xid' - Prepare for commit (phase 1)
//! - XA COMMIT 'xid' - Commit the XA transaction (phase 2)
//! - XA ROLLBACK 'xid' - Rollback the XA transaction
//! - XA RECOVER - List prepared but not committed XA transactions

use sqlrustgo_distributed::{XaCoordinator, XaError, XaState, Xid};

fn create_xid(format_id: i32, gtrid: &str, bqual: &str) -> Xid {
    Xid::new(
        format_id,
        gtrid.as_bytes().to_vec(),
        bqual.as_bytes().to_vec(),
    )
}

#[test]
fn test_xa_start_and_state() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();

    let state = coordinator.get_state(&xid).unwrap();
    assert_eq!(state, XaState::Active);
}

#[test]
fn test_xa_transaction_lifecycle_commit() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_end(&xid).unwrap();
    coordinator.xa_prepare(&xid).unwrap();
    coordinator.xa_commit(&xid).unwrap();

    let state = coordinator.get_state(&xid).unwrap();
    assert_eq!(state, XaState::Committed);
}

#[test]
fn test_xa_transaction_lifecycle_rollback() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_end(&xid).unwrap();
    coordinator.xa_prepare(&xid).unwrap();
    coordinator.xa_rollback(&xid).unwrap();

    let state = coordinator.get_state(&xid).unwrap();
    assert_eq!(state, XaState::RolledBack);
}

#[test]
fn test_xa_rollback_from_active() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_rollback(&xid).unwrap();

    let state = coordinator.get_state(&xid).unwrap();
    assert_eq!(state, XaState::RolledBack);
}

#[test]
fn test_xa_rollback_from_idle() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_end(&xid).unwrap();
    coordinator.xa_rollback(&xid).unwrap();

    let state = coordinator.get_state(&xid).unwrap();
    assert_eq!(state, XaState::RolledBack);
}

#[test]
fn test_xa_recover_empty() {
    let coordinator = XaCoordinator::new();
    let recovered = coordinator.xa_recover().unwrap();
    assert!(recovered.is_empty());
}

#[test]
fn test_xa_recover_after_prepare() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_end(&xid).unwrap();
    coordinator.xa_prepare(&xid).unwrap();

    let recovered = coordinator.xa_recover().unwrap();
    assert_eq!(recovered.len(), 1);
}

#[test]
fn test_xa_recover_after_commit() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_end(&xid).unwrap();
    coordinator.xa_prepare(&xid).unwrap();
    coordinator.xa_commit(&xid).unwrap();

    let recovered = coordinator.xa_recover().unwrap();
    assert!(recovered.is_empty());
}

#[test]
fn test_xa_error_xid_exists() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    let result = coordinator.xa_start(xid.clone());

    assert!(matches!(result, Err(XaError::XidExists(_))));
}

#[test]
fn test_xa_error_xid_not_found() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    let result = coordinator.xa_end(&xid);
    assert!(matches!(result, Err(XaError::XidNotFound(_))));
}

#[test]
fn test_xa_error_invalid_state_end_from_idle() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_end(&xid).unwrap();

    let result = coordinator.xa_end(&xid);
    assert!(matches!(
        result,
        Err(XaError::InvalidStateTransition { .. })
    ));
}

#[test]
fn test_xa_error_commit_not_prepared() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();

    let result = coordinator.xa_commit(&xid);
    assert!(matches!(
        result,
        Err(XaError::InvalidStateTransition { .. })
    ));
}

#[test]
fn test_xa_error_prepare_from_active() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();

    let result = coordinator.xa_prepare(&xid);
    assert!(matches!(
        result,
        Err(XaError::InvalidStateTransition { .. })
    ));
}

#[test]
fn test_xa_error_rollback_committed() {
    let coordinator = XaCoordinator::new();
    let xid = create_xid(0, "gtrid1", "bqual1");

    coordinator.xa_start(xid.clone()).unwrap();
    coordinator.xa_end(&xid).unwrap();
    coordinator.xa_prepare(&xid).unwrap();
    coordinator.xa_commit(&xid).unwrap();

    let result = coordinator.xa_rollback(&xid);
    assert!(matches!(result, Err(XaError::XidTerminal(_))));
}

#[test]
fn test_multiple_xa_transactions() {
    let coordinator = XaCoordinator::new();
    let xids: Vec<Xid> = (0..3)
        .map(|i| create_xid(0, &format!("gtrid{}", i), "bqual"))
        .collect();

    for xid in &xids {
        coordinator.xa_start(xid.clone()).unwrap();
    }

    assert_eq!(coordinator.num_active_transactions(), 3);

    for xid in &xids {
        coordinator.xa_end(xid).unwrap();
        coordinator.xa_prepare(xid).unwrap();
    }

    assert_eq!(coordinator.xa_recover().unwrap().len(), 3);

    for xid in &xids {
        coordinator.xa_commit(xid).unwrap();
    }

    assert!(coordinator.xa_recover().unwrap().is_empty());
}

#[test]
fn test_xa_coordinator_cleanup() {
    let coordinator = XaCoordinator::new();
    let xid1 = create_xid(0, "gtrid1", "bqual1");
    let xid2 = create_xid(0, "gtrid2", "bqual2");

    coordinator.xa_start(xid1.clone()).unwrap();
    coordinator.xa_end(&xid1).unwrap();
    coordinator.xa_prepare(&xid1).unwrap();
    coordinator.xa_commit(&xid1).unwrap();

    coordinator.xa_start(xid2.clone()).unwrap();
    coordinator.xa_end(&xid2).unwrap();
    coordinator.xa_prepare(&xid2).unwrap();

    assert_eq!(coordinator.num_active_transactions(), 1);

    coordinator.cleanup_completed();

    assert_eq!(coordinator.num_active_transactions(), 1);
}

#[test]
fn test_xid_from_string() {
    let xid = Xid::from_string("0:gar1:bar1").unwrap();
    assert_eq!(xid.format_id, 0);
    assert_eq!(xid.gtrid, b"gar1");
    assert_eq!(xid.bqual, b"bar1");
}

#[test]
fn test_xid_display_string() {
    let xid = create_xid(0, "abc", "def");
    let s = xid.to_display_string();
    assert!(s.contains("abc"));
}

#[test]
fn test_xa_state_display() {
    assert_eq!(format!("{}", XaState::Active), "ACTIVE");
    assert_eq!(format!("{}", XaState::Idle), "IDLE");
    assert_eq!(format!("{}", XaState::Prepared), "PREPARED");
    assert_eq!(format!("{}", XaState::Committed), "COMMITTED");
    assert_eq!(format!("{}", XaState::RolledBack), "ROLLEDBACK");
}

#[test]
fn test_xa_error_display() {
    let xid = create_xid(0, "gtrid1", "bqual1");
    let err = XaError::XidNotFound(xid.clone());
    assert!(err.to_string().contains("not found"));
}
