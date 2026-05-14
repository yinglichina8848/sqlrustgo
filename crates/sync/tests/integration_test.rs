use sqlrustgo_sync::client_registry::{InMemoryClientRegistry, TransactionStatus};
use sqlrustgo_sync::{
    ClientGtid, ClientRegistry, OTEngine, Operation, OperationType, ResponseResult, SqlOperation,
    SyncRequest, SyncResponse, VectorClock,
};

fn make_cgtid(client_id: &str, seq: u64) -> ClientGtid {
    ClientGtid::new(client_id, seq)
}

fn make_op(sql: &str) -> Operation {
    Operation {
        op: OperationType::Sql(SqlOperation {
            sql: sql.to_string(),
            params: vec![],
        }),
    }
}

#[tokio::test]
async fn test_sync_request_creation() {
    let cgtid = make_cgtid("iphone-23", 1);
    let ops = vec![make_op("INSERT INTO t VALUES (1)")];

    let request = SyncRequest::new(cgtid.clone(), ops.clone())
        .with_device_info("iPhone 15 Pro")
        .with_client_timestamp(1699999999000);

    assert_eq!(request.cgtid, cgtid);
    assert_eq!(request.operations.len(), 1);
    assert!(request.device_info.is_some());
    assert!(request.client_timestamp.is_some());
}

#[tokio::test]
async fn test_sync_response_commit() {
    let cgtid = make_cgtid("iphone-23", 1);
    let gtid = "server-1:1000".to_string();
    let commit_ts = 1699999999000;

    let response = SyncResponse::commit(cgtid.clone(), gtid.clone(), commit_ts);

    assert!(response.is_commit());
    assert!(!response.is_conflict());
    assert!(!response.is_error());

    if let ResponseResult::Commit(commit) = response.result {
        assert_eq!(commit.cgtid, cgtid);
        assert_eq!(commit.gtid, gtid);
        assert_eq!(commit.commit_timestamp, commit_ts);
    } else {
        panic!("Expected Commit result");
    }
}

#[tokio::test]
async fn test_sync_response_conflict() {
    let cgtid = make_cgtid("iphone-23", 1);
    let ops = vec![make_op("UPDATE t SET x = 2 WHERE id = 1")];

    let response = SyncResponse::conflict(
        cgtid.clone(),
        ops.clone(),
        vec!["Concurrent modification".to_string()],
    );

    assert!(!response.is_commit());
    assert!(response.is_conflict());
    assert!(!response.is_error());

    if let ResponseResult::Conflict(conflict) = response.result {
        assert_eq!(conflict.cgtid, cgtid);
        assert_eq!(conflict.transformed_ops.len(), 1);
        assert_eq!(conflict.conflicts.len(), 1);
    } else {
        panic!("Expected Conflict result");
    }
}

#[tokio::test]
async fn test_sync_response_error() {
    let cgtid = make_cgtid("iphone-23", 1);

    let response = SyncResponse::error(cgtid.clone(), "E001", "Duplicate key", true);

    assert!(!response.is_commit());
    assert!(!response.is_conflict());
    assert!(response.is_error());

    if let ResponseResult::Error(error) = response.result {
        assert_eq!(error.cgtid, cgtid);
        assert_eq!(error.error_code, "E001");
        assert_eq!(error.error_message, "Duplicate key");
        assert!(error.retryable);
    } else {
        panic!("Expected Error result");
    }
}

#[tokio::test]
async fn test_client_registry_operations() {
    let mut registry = InMemoryClientRegistry::new();

    let cgtid = make_cgtid("iphone-23", 1);
    let tx = sqlrustgo_sync::client_registry::ClientTransaction {
        cgtid: cgtid.clone(),
        status: TransactionStatus::Processing,
        operations_json: "[]".to_string(),
        sql_statements: None,
        request_hash: vec![],
        response_blob: None,
        vector_clock_json: "{}".to_string(),
        client_ts: 0,
        server_ts: None,
        device_info: None,
    };

    registry.begin_transaction(&tx).unwrap();
    assert!(registry.is_in_progress(&cgtid).unwrap());
    assert!(!registry.is_committed(&cgtid).unwrap());

    let duplicate_result = registry.begin_transaction(&tx);
    assert!(duplicate_result.is_err());

    registry
        .mark_committed(&cgtid, "server-1:100", Some(vec![1, 2, 3]), 1699999999000)
        .unwrap();
    assert!(!registry.is_in_progress(&cgtid).unwrap());
    assert!(registry.is_committed(&cgtid).unwrap());

    let retrieved = registry.get_transaction(&cgtid).unwrap().unwrap();
    assert_eq!(retrieved.cgtid, cgtid);
    assert!(retrieved.response_blob.is_some());
}

#[tokio::test]
async fn test_client_registry_get_last_committed_seq() {
    let mut registry = InMemoryClientRegistry::new();

    for seq in 1..=5 {
        let cgtid = make_cgtid("iphone-23", seq);
        let tx = sqlrustgo_sync::client_registry::ClientTransaction {
            cgtid: cgtid.clone(),
            status: TransactionStatus::Processing,
            operations_json: "[]".to_string(),
            sql_statements: None,
            request_hash: vec![],
            response_blob: None,
            vector_clock_json: "{}".to_string(),
            client_ts: 0,
            server_ts: None,
            device_info: None,
        };
        registry.begin_transaction(&tx).unwrap();
        registry
            .mark_committed(
                &cgtid,
                &format!("server-1:{}", seq * 100),
                None,
                seq as i64 * 1000,
            )
            .unwrap();
    }

    assert_eq!(
        registry.get_last_committed_seq("iphone-23").unwrap(),
        Some(5)
    );
    assert_eq!(
        registry.get_last_committed_seq("unknown-device").unwrap(),
        None
    );
}

#[tokio::test]
async fn test_ot_engine_concurrent_detection() {
    let clock1 = VectorClock::new()
        .with_entry("node-a", 1)
        .with_entry("node-b", 2);
    let clock2 = VectorClock::new()
        .with_entry("node-a", 2)
        .with_entry("node-b", 1);

    assert!(clock1.concurrent_with(&clock2));
    assert!(!clock1.happens_before(&clock2));
}

#[tokio::test]
async fn test_ot_engine_happens_before() {
    let clock1 = VectorClock::new()
        .with_entry("node-a", 1)
        .with_entry("node-b", 1);
    let clock2 = VectorClock::new()
        .with_entry("node-a", 2)
        .with_entry("node-b", 1);

    assert!(clock1.happens_before(&clock2));
    assert!(!clock2.happens_before(&clock1));
}

#[tokio::test]
async fn test_ot_engine_record_commit() {
    let mut engine = OTEngine::new();
    let cgtid = make_cgtid("iphone-23", 1);

    engine.record_commit(&cgtid);
    engine
        .check_dependencies(
            &make_cgtid("iphone-23", 2),
            &VectorClock::new().with_entry("iphone-23", 1),
        )
        .unwrap();
}

#[tokio::test]
async fn test_ot_engine_causality_violation() {
    let mut engine = OTEngine::new();

    engine.record_commit(&make_cgtid("node-a", 1));

    let clock_with_uncommitted = VectorClock::new()
        .with_entry("node-a", 2)
        .with_entry("node-b", 1);

    let result = engine.check_dependencies(&make_cgtid("node-b", 2), &clock_with_uncommitted);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ot_engine_transform_operations() {
    let engine = OTEngine::new();

    let local_ops = vec![make_op("UPDATE t SET x = 1 WHERE id = 1")];
    let remote_ops = vec![make_op("UPDATE t SET x = 2 WHERE id = 1")];

    let clock_local = VectorClock::new().with_entry("client-1", 1);
    let clock_remote = VectorClock::new().with_entry("client-2", 1);

    let result = engine.transform_operations(&local_ops, &remote_ops, &clock_local, &clock_remote);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);
}

#[tokio::test]
async fn test_vector_clock_merge() {
    let mut clock1 = VectorClock::new()
        .with_entry("node-a", 1)
        .with_entry("node-b", 1);
    let clock2 = VectorClock::new()
        .with_entry("node-b", 2)
        .with_entry("node-c", 1);

    clock1.merge(&clock2);

    assert_eq!(clock1.get("node-a"), 1);
    assert_eq!(clock1.get("node-b"), 2);
    assert_eq!(clock1.get("node-c"), 1);
}

#[tokio::test]
async fn test_request_hash() {
    let cgtid1 = make_cgtid("iphone-23", 1);
    let ops1 = vec![make_op("SELECT * FROM t")];

    let cgtid2 = make_cgtid("iphone-23", 1);
    let ops2 = vec![make_op("SELECT * FROM t")];

    let request1 = SyncRequest::new(cgtid1, ops1);
    let request2 = SyncRequest::new(cgtid2, ops2);

    assert_eq!(request1.request_hash(), request2.request_hash());
}

#[tokio::test]
async fn test_different_ops_different_hash() {
    let cgtid1 = make_cgtid("iphone-23", 1);
    let ops1 = vec![make_op("SELECT * FROM t1")];

    let cgtid2 = make_cgtid("iphone-23", 1);
    let ops2 = vec![make_op("SELECT * FROM t2")];

    let request1 = SyncRequest::new(cgtid1, ops1);
    let request2 = SyncRequest::new(cgtid2, ops2);

    assert_ne!(request1.request_hash(), request2.request_hash());
}
