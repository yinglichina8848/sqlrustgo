//! Network module for SQLRustGo
//!
//! Provides network communication capabilities including DTC (Distributed Transaction Coordinator).

/// DTC (Distributed Transaction Coordinator) module
/// Generated from protobuf definitions
pub mod dtc {
    include!(concat!(env!("OUT_DIR"), "/sqlrustgo.dtc.rs"));
}

#[cfg(test)]
mod tests {
    use super::dtc::*;

    // ============ VoteType Tests ============

    #[test]
    fn test_vote_type_as_str_name() {
        assert_eq!(VoteType::VoteCommit.as_str_name(), "VOTE_COMMIT");
        assert_eq!(VoteType::VoteAbort.as_str_name(), "VOTE_ABORT");
    }

    #[test]
    fn test_vote_type_from_str_name() {
        assert_eq!(
            VoteType::from_str_name("VOTE_COMMIT"),
            Some(VoteType::VoteCommit)
        );
        assert_eq!(
            VoteType::from_str_name("VOTE_ABORT"),
            Some(VoteType::VoteAbort)
        );
        assert_eq!(VoteType::from_str_name("INVALID"), None);
    }

    // ============ ChangeOperation Tests ============

    #[test]
    fn test_change_operation_as_str_name() {
        assert_eq!(ChangeOperation::Insert.as_str_name(), "INSERT");
        assert_eq!(ChangeOperation::Update.as_str_name(), "UPDATE");
        assert_eq!(ChangeOperation::Delete.as_str_name(), "DELETE");
    }

    #[test]
    fn test_change_operation_from_str_name() {
        assert_eq!(
            ChangeOperation::from_str_name("INSERT"),
            Some(ChangeOperation::Insert)
        );
        assert_eq!(
            ChangeOperation::from_str_name("UPDATE"),
            Some(ChangeOperation::Update)
        );
        assert_eq!(
            ChangeOperation::from_str_name("DELETE"),
            Some(ChangeOperation::Delete)
        );
        assert_eq!(ChangeOperation::from_str_name("INVALID"), None);
    }

    // ============ NotificationType Tests ============

    #[test]
    fn test_notification_type_as_str_name() {
        assert_eq!(
            NotificationType::ParticipantDown.as_str_name(),
            "PARTICIPANT_DOWN"
        );
        assert_eq!(
            NotificationType::RecoveryComplete.as_str_name(),
            "RECOVERY_COMPLETE"
        );
        assert_eq!(NotificationType::LockTimeout.as_str_name(), "LOCK_TIMEOUT");
    }

    #[test]
    fn test_notification_type_from_str_name() {
        assert_eq!(
            NotificationType::from_str_name("PARTICIPANT_DOWN"),
            Some(NotificationType::ParticipantDown)
        );
        assert_eq!(
            NotificationType::from_str_name("RECOVERY_COMPLETE"),
            Some(NotificationType::RecoveryComplete)
        );
        assert_eq!(
            NotificationType::from_str_name("LOCK_TIMEOUT"),
            Some(NotificationType::LockTimeout)
        );
        assert_eq!(NotificationType::from_str_name("INVALID"), None);
    }

    // ============ PrepareRequest Tests ============

    #[test]
    fn test_prepare_request() {
        let change = Change {
            table: "users".to_string(),
            operation: ChangeOperation::Insert as i32,
            key: vec![1, 2, 3],
            value: vec![4, 5, 6],
        };
        let req = PrepareRequest {
            gid: "tx-123".to_string(),
            coordinator_node_id: "node-1".to_string(),
            changes: vec![change],
        };
        assert_eq!(req.gid, "tx-123");
        assert_eq!(req.coordinator_node_id, "node-1");
        assert_eq!(req.changes.len(), 1);
        assert_eq!(req.changes[0].table, "users");
    }

    #[test]
    fn test_prepare_request_empty_changes() {
        let req = PrepareRequest {
            gid: "tx-empty".to_string(),
            coordinator_node_id: "node-x".to_string(),
            changes: vec![],
        };
        assert!(req.changes.is_empty());
    }

    // ============ VoteResponse Tests ============

    #[test]
    fn test_vote_response_commit() {
        let resp = VoteResponse {
            gid: "tx-456".to_string(),
            node_id: "node-2".to_string(),
            vote: VoteType::VoteCommit as i32,
            reason: "OK".to_string(),
        };
        assert_eq!(resp.gid, "tx-456");
        assert_eq!(resp.vote, VoteType::VoteCommit as i32);
    }

    #[test]
    fn test_vote_response_abort() {
        let resp = VoteResponse {
            gid: "tx-789".to_string(),
            node_id: "node-3".to_string(),
            vote: VoteType::VoteAbort as i32,
            reason: "Lock conflict".to_string(),
        };
        assert_eq!(resp.vote, VoteType::VoteAbort as i32);
        assert_eq!(resp.reason, "Lock conflict");
    }

    // ============ CommitRequest Tests ============

    #[test]
    fn test_commit_request() {
        let req = CommitRequest {
            gid: "tx-commit-1".to_string(),
        };
        assert_eq!(req.gid, "tx-commit-1");
    }

    // ============ RollbackRequest Tests ============

    #[test]
    fn test_rollback_request() {
        let req = RollbackRequest {
            gid: "tx-rollback-1".to_string(),
            reason: "User cancelled".to_string(),
        };
        assert_eq!(req.gid, "tx-rollback-1");
        assert_eq!(req.reason, "User cancelled");
    }

    #[test]
    fn test_rollback_request_empty_reason() {
        let req = RollbackRequest {
            gid: "tx-rollback-2".to_string(),
            reason: String::new(),
        };
        assert!(req.reason.is_empty());
    }

    // ============ ExecutionResponse Tests ============

    #[test]
    fn test_execution_response_success() {
        let resp = ExecutionResponse {
            gid: "tx-exec-1".to_string(),
            node_id: "node-primary".to_string(),
            success: true,
            affected_rows: 42,
            error: String::new(),
        };
        assert!(resp.success);
        assert_eq!(resp.affected_rows, 42);
        assert!(resp.error.is_empty());
    }

    #[test]
    fn test_execution_response_failure() {
        let resp = ExecutionResponse {
            gid: "tx-exec-2".to_string(),
            node_id: "node-replica".to_string(),
            success: false,
            affected_rows: 0,
            error: "Constraint violation".to_string(),
        };
        assert!(!resp.success);
        assert_eq!(resp.affected_rows, 0);
        assert_eq!(resp.error, "Constraint violation");
    }

    // ============ Change Tests ============

    #[test]
    fn test_change_insert() {
        let change = Change {
            table: "orders".to_string(),
            operation: ChangeOperation::Insert as i32,
            key: vec![10],
            value: vec![1, 2, 3],
        };
        assert_eq!(change.table, "orders");
        assert_eq!(change.operation, ChangeOperation::Insert as i32);
    }

    #[test]
    fn test_change_update() {
        let change = Change {
            table: "products".to_string(),
            operation: ChangeOperation::Update as i32,
            key: vec![99],
            value: vec![9, 9],
        };
        assert_eq!(change.operation, ChangeOperation::Update as i32);
    }

    #[test]
    fn test_change_delete() {
        let change = Change {
            table: "sessions".to_string(),
            operation: ChangeOperation::Delete as i32,
            key: vec![1, 0],
            value: vec![],
        };
        assert_eq!(change.operation, ChangeOperation::Delete as i32);
        assert!(change.value.is_empty());
    }

    // ============ Notification Tests ============

    #[test]
    fn test_notification_participant_down() {
        let notif = Notification {
            gid: "tx-001".to_string(),
            node_id: "node-failed".to_string(),
            r#type: NotificationType::ParticipantDown as i32,
            details: "Heartbeat timeout".to_string(),
        };
        assert_eq!(notif.r#type, NotificationType::ParticipantDown as i32);
    }

    #[test]
    fn test_notification_recovery_complete() {
        let notif = Notification {
            gid: "tx-002".to_string(),
            node_id: "node-recovered".to_string(),
            r#type: NotificationType::RecoveryComplete as i32,
            details: "All logs replayed".to_string(),
        };
        assert_eq!(notif.r#type, NotificationType::RecoveryComplete as i32);
    }

    #[test]
    fn test_notification_lock_timeout() {
        let notif = Notification {
            gid: "tx-003".to_string(),
            node_id: "node-1".to_string(),
            r#type: NotificationType::LockTimeout as i32,
            details: "Waited 30s for lock".to_string(),
        };
        assert_eq!(notif.r#type, NotificationType::LockTimeout as i32);
    }

    // ============ NodePing / NodePong Tests ============

    #[test]
    fn test_node_ping() {
        let ping = NodePing {
            node_id: "node-alpha".to_string(),
        };
        assert_eq!(ping.node_id, "node-alpha");
    }

    #[test]
    fn test_node_pong_healthy() {
        let pong = NodePong {
            node_id: "node-alpha".to_string(),
            healthy: true,
        };
        assert!(pong.healthy);
    }

    #[test]
    fn test_node_pong_unhealthy() {
        let pong = NodePong {
            node_id: "node-beta".to_string(),
            healthy: false,
        };
        assert!(!pong.healthy);
    }

    // ============ Empty Tests ============

    #[test]
    fn test_empty_struct() {
        let _empty = Empty {};
    }

    // ============ Round-trip Serialize Tests ============

    #[test]
    fn test_change_serialize_roundtrip() {
        let original = Change {
            table: "test_table".to_string(),
            operation: ChangeOperation::Update as i32,
            key: vec![0xDE, 0xAD, 0xBE, 0xEF],
            value: vec![0xFE, 0xED],
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: Change = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.table, original.table);
        assert_eq!(decoded.operation, original.operation);
        assert_eq!(decoded.key, original.key);
        assert_eq!(decoded.value, original.value);
    }

    #[test]
    fn test_prepare_request_roundtrip() {
        let original = PrepareRequest {
            gid: "gid-12345".to_string(),
            coordinator_node_id: "coord-1".to_string(),
            changes: vec![
                Change {
                    table: "t1".to_string(),
                    operation: ChangeOperation::Insert as i32,
                    key: vec![1],
                    value: vec![2],
                },
                Change {
                    table: "t2".to_string(),
                    operation: ChangeOperation::Delete as i32,
                    key: vec![3],
                    value: vec![],
                },
            ],
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: PrepareRequest = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.gid, original.gid);
        assert_eq!(decoded.coordinator_node_id, original.coordinator_node_id);
        assert_eq!(decoded.changes.len(), 2);
    }

    #[test]
    fn test_vote_response_roundtrip() {
        let original = VoteResponse {
            gid: "gid-vote".to_string(),
            node_id: "node-v".to_string(),
            vote: VoteType::VoteAbort as i32,
            reason: "Deadlock detected".to_string(),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: VoteResponse = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.gid, original.gid);
        assert_eq!(decoded.vote, original.vote);
        assert_eq!(decoded.reason, original.reason);
    }

    #[test]
    fn test_execution_response_roundtrip() {
        let original = ExecutionResponse {
            gid: "gid-exec".to_string(),
            node_id: "node-e".to_string(),
            success: true,
            affected_rows: 100,
            error: String::new(),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: ExecutionResponse = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.success, original.success);
        assert_eq!(decoded.affected_rows, original.affected_rows);
    }

    // ============ Edge Cases ============

    #[test]
    fn test_change_empty_table_name() {
        let change = Change {
            table: String::new(),
            operation: ChangeOperation::Insert as i32,
            key: vec![],
            value: vec![],
        };
        assert!(change.table.is_empty());
    }

    #[test]
    fn test_prepare_request_large_gid() {
        let long_gid = "x".repeat(1000);
        let req = PrepareRequest {
            gid: long_gid.clone(),
            coordinator_node_id: "node".to_string(),
            changes: vec![],
        };
        assert_eq!(req.gid, long_gid);
    }

    #[test]
    fn test_all_vote_types_exhaustive() {
        let variants = [VoteType::VoteCommit, VoteType::VoteAbort];
        for v in variants {
            let name = v.as_str_name();
            let recovered = VoteType::from_str_name(name);
            assert!(recovered.is_some(), "Failed to round-trip {:?}", v);
        }
    }

    #[test]
    fn test_all_change_operations_exhaustive() {
        let variants = [
            ChangeOperation::Insert,
            ChangeOperation::Update,
            ChangeOperation::Delete,
        ];
        for v in variants {
            let name = v.as_str_name();
            let recovered = ChangeOperation::from_str_name(name);
            assert!(recovered.is_some(), "Failed to round-trip {:?}", v);
        }
    }

    #[test]
    fn test_all_notification_types_exhaustive() {
        let variants = [
            NotificationType::ParticipantDown,
            NotificationType::RecoveryComplete,
            NotificationType::LockTimeout,
        ];
        for v in variants {
            let name = v.as_str_name();
            let recovered = NotificationType::from_str_name(name);
            assert!(recovered.is_some(), "Failed to round-trip {:?}", v);
        }
    }

    // ============ ExecutionResponse Failure Roundtrip ============

    #[test]
    fn test_execution_response_failure_roundtrip() {
        let original = ExecutionResponse {
            gid: "gid-err".to_string(),
            node_id: "node-err".to_string(),
            success: false,
            affected_rows: 0,
            error: "Foreign key constraint".to_string(),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: ExecutionResponse = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.error, original.error);
    }

    // ============ Notification Roundtrip ============

    #[test]
    fn test_notification_roundtrip() {
        let original = Notification {
            gid: "gid-notif".to_string(),
            node_id: "node-notif".to_string(),
            r#type: NotificationType::LockTimeout as i32,
            details: "Lock wait timeout".to_string(),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: Notification = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.gid, original.gid);
        assert_eq!(decoded.r#type, original.r#type);
        assert_eq!(decoded.details, original.details);
    }

    // ============ Commit/Rollback Roundtrip ============

    #[test]
    fn test_commit_request_roundtrip() {
        let original = CommitRequest {
            gid: "gid-commit".to_string(),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: CommitRequest = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.gid, original.gid);
    }

    #[test]
    fn test_rollback_request_roundtrip() {
        let original = RollbackRequest {
            gid: "gid-rollback".to_string(),
            reason: "Timeout".to_string(),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: RollbackRequest = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.gid, original.gid);
        assert_eq!(decoded.reason, original.reason);
    }

    // ============ Multi-change PrepareRequest ============

    #[test]
    fn test_prepare_request_multiple_changes() {
        let changes = vec![
            Change {
                table: "a".to_string(),
                operation: ChangeOperation::Insert as i32,
                key: vec![1],
                value: vec![10],
            },
            Change {
                table: "b".to_string(),
                operation: ChangeOperation::Update as i32,
                key: vec![2],
                value: vec![20],
            },
            Change {
                table: "c".to_string(),
                operation: ChangeOperation::Delete as i32,
                key: vec![3],
                value: vec![],
            },
        ];
        let req = PrepareRequest {
            gid: "multi-tx".to_string(),
            coordinator_node_id: "coordinator-1".to_string(),
            changes,
        };
        assert_eq!(req.changes.len(), 3);
        assert_eq!(req.changes[0].table, "a");
        assert_eq!(req.changes[1].table, "b");
        assert_eq!(req.changes[2].table, "c");
    }

    // ============ NodePing/Pong Roundtrip ============

    #[test]
    fn test_node_ping_roundtrip() {
        let original = NodePing {
            node_id: "node-ping".to_string(),
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: NodePing = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.node_id, original.node_id);
    }

    #[test]
    fn test_node_pong_healthy_roundtrip() {
        let original = NodePong {
            node_id: "node-pong".to_string(),
            healthy: true,
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: NodePong = prost::Message::decode(&buf[..]).unwrap();
        assert_eq!(decoded.node_id, original.node_id);
        assert_eq!(decoded.healthy, original.healthy);
    }

    #[test]
    fn test_node_pong_unhealthy_roundtrip() {
        let original = NodePong {
            node_id: "node-dead".to_string(),
            healthy: false,
        };
        let mut buf = Vec::new();
        prost::Message::encode(&original, &mut buf).unwrap();
        let decoded: NodePong = prost::Message::decode(&buf[..]).unwrap();
        assert!(!decoded.healthy);
    }
}
