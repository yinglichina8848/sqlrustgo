//! Semi-synchronous Replication Integration Tests

use sqlrustgo_distributed::semisync::{
    SemiSyncMaster, SemiSyncMode, SemiSyncSlave, SemiSyncTimeoutError,
};

#[test]
fn test_semisync_master_basic_lifecycle() {
    let master = SemiSyncMaster::new();
    assert!(!master.is_enabled());

    master.enable();
    assert!(master.is_enabled());

    master.disable();
    assert!(!master.is_enabled());
}

#[test]
fn test_semisync_master_mode_configuration() {
    let master = SemiSyncMaster::new();
    assert_eq!(master.get_mode(), SemiSyncMode::AfterSync);

    master.set_mode(SemiSyncMode::AfterCommit);
    assert_eq!(master.get_mode(), SemiSyncMode::AfterCommit);
}

#[test]
fn test_semisync_master_timeout_configuration() {
    let master = SemiSyncMaster::new();
    assert_eq!(master.get_timeout_ms(), 10000);

    master.set_timeout_ms(5000);
    assert_eq!(master.get_timeout_ms(), 5000);
}

#[test]
fn test_semisync_master_wait_count_configuration() {
    let master = SemiSyncMaster::new();
    assert_eq!(master.get_wait_count(), 1);

    master.set_wait_count(3);
    assert_eq!(master.get_wait_count(), 3);
}

#[test]
fn test_semisync_master_ack_tracking() {
    let master = SemiSyncMaster::new();
    assert_eq!(master.get_ack_count(), 0);

    master.increment_ack();
    assert_eq!(master.get_ack_count(), 1);

    master.increment_ack();
    assert_eq!(master.get_ack_count(), 2);

    master.reset_ack_counter();
    assert_eq!(master.get_ack_count(), 0);
}

#[test]
fn test_semisync_master_transaction_counting() {
    let master = SemiSyncMaster::new();

    master.record_yes_transaction();
    master.record_yes_transaction();
    master.record_no_transaction();

    let status = master.get_status();
    assert_eq!(status.rpl_semi_sync_master_yes_transactions, 2);
    assert_eq!(status.rpl_semi_sync_master_no_transactions, 1);
}

#[test]
fn test_semisync_slave_basic_lifecycle() {
    let slave = SemiSyncSlave::new();
    assert!(!slave.is_enabled());

    slave.enable();
    assert!(slave.is_enabled());

    slave.disable();
    assert!(!slave.is_enabled());
}

#[test]
fn test_semisync_slave_master_uuid() {
    let slave = SemiSyncSlave::new();
    assert!(slave.get_master_uuid().is_none());

    slave.set_master_uuid(Some("test-uuid-123".to_string()));
    assert_eq!(slave.get_master_uuid(), Some("test-uuid-123".to_string()));
}

#[test]
fn test_semisync_slave_ack_management() {
    let slave = SemiSyncSlave::new();
    assert!(!slave.is_ack_sent());

    slave.mark_ack_sent();
    assert!(slave.is_ack_sent());

    slave.reset_ack_sent();
    assert!(!slave.is_ack_sent());
}

#[test]
fn test_semisync_master_with_custom_config() {
    let master = SemiSyncMaster::with_config(15000, 2, SemiSyncMode::AfterCommit);

    assert_eq!(master.get_timeout_ms(), 15000);
    assert_eq!(master.get_wait_count(), 2);
    assert_eq!(master.get_mode(), SemiSyncMode::AfterCommit);
}

#[test]
fn test_semisync_master_status_reflects_enabled_state() {
    let master = SemiSyncMaster::new();

    let status = master.get_status();
    assert!(!status.rpl_semi_sync_master_status);

    master.enable();
    let status = master.get_status();
    assert!(status.rpl_semi_sync_master_status);
}

#[test]
fn test_semisync_slave_status_reflects_enabled_state() {
    let slave = SemiSyncSlave::new();

    let status = slave.get_status();
    assert!(!status.rpl_semi_sync_slave_status);

    slave.enable();
    let status = slave.get_status();
    assert!(status.rpl_semi_sync_slave_status);
}

#[test]
fn test_semisync_timeout_error_contains_details() {
    let err = SemiSyncTimeoutError {
        required: 3,
        received: 1,
        timeout_ms: 10000,
    };

    assert_eq!(err.required, 3);
    assert_eq!(err.received, 1);
    assert_eq!(err.timeout_ms, 10000);
}

#[tokio::test]
async fn test_semisync_master_wait_returns_when_disabled() {
    let master = SemiSyncMaster::new();
    let result = master.wait_for_acks(1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_semisync_master_wait_succeeds_with_sufficient_acks() {
    let master = SemiSyncMaster::new();
    master.enable();
    master.set_wait_count(2);

    master.increment_ack();
    master.increment_ack();

    let result = master.wait_for_acks(2).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_semisync_master_wait_fails_on_timeout() {
    let master = SemiSyncMaster::new();
    master.enable();
    master.set_timeout_ms(50);
    master.set_wait_count(5);

    let result = master.wait_for_acks(5).await;
    assert!(result.is_err());
}

#[test]
fn test_semisync_mode_default_is_aftersync() {
    assert_eq!(SemiSyncMode::default(), SemiSyncMode::AfterSync);
}

#[test]
fn test_semisync_master_default() {
    let master = SemiSyncMaster::default();
    assert!(!master.is_enabled());
    assert_eq!(master.get_mode(), SemiSyncMode::AfterSync);
    assert_eq!(master.get_timeout_ms(), 10000);
    assert_eq!(master.get_wait_count(), 1);
}

#[test]
fn test_semisync_slave_default() {
    let slave = SemiSyncSlave::default();
    assert!(!slave.is_enabled());
    assert!(slave.get_master_uuid().is_none());
}
