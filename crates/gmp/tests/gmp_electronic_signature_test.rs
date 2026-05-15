//! GMP Electronic Signature Tests (M5 - 21 CFR Part 11)
//!
//! Tests for electronic signature module including:
//! - SQL parsing (SIGN RECORD, CREATE APPROVAL POLICY)
//! - ElectronicSignature struct
//! - ApprovalPolicy struct
//! - SignatureRequest struct
//! - ApprovalPolicyEvaluator (sequential/parallel workflows)

use sqlrustgo_gmp::electronic_signature::{
    ApprovalPolicy, ApprovalPolicyEvaluator, ElectronicSignature, PolicyStatus, SignatureError,
    SignatureRequest, compute_data_hash, compute_signing_payload, current_timestamp_ms,
};

// ============================================================================
// SQL Parsing Tests
// ============================================================================

#[test]
fn test_sign_record_sql_parsing() {
    use sqlrustgo_parser::parse;
    use sqlrustgo_parser::Statement;

    let sql = "SIGN RECORD FOR batches (batch_id = 'B001') REASON 'Approved for release'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SIGN RECORD: {:?}", result);

    match result.unwrap() {
        Statement::SignRecord(sign) => {
            assert_eq!(sign.table_name, "batches");
            assert_eq!(sign.columns.len(), 1);
            assert_eq!(sign.columns[0], ("batch_id".to_string(), "B001".to_string()));
            assert_eq!(sign.reason, "Approved for release");
        }
        _ => panic!("Expected SignRecord statement"),
    }
}

#[test]
fn test_sign_record_multiple_columns() {
    use sqlrustgo_parser::parse;
    use sqlrustgo_parser::Statement;

    let sql = "SIGN RECORD FOR production_batches (batch_id = 'B001', product_id = 'P123', quantity = 1000) REASON 'Quality check passed'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SIGN RECORD with multiple columns: {:?}", result);

    match result.unwrap() {
        Statement::SignRecord(sign) => {
            assert_eq!(sign.table_name, "production_batches");
            assert_eq!(sign.columns.len(), 3);
            assert_eq!(sign.columns[0], ("batch_id".to_string(), "B001".to_string()));
            assert_eq!(sign.columns[1], ("product_id".to_string(), "P123".to_string()));
            assert_eq!(sign.columns[2], ("quantity".to_string(), "1000".to_string()));
            assert_eq!(sign.reason, "Quality check passed");
        }
        _ => panic!("Expected SignRecord statement"),
    }
}

#[test]
fn test_sign_record_without_columns() {
    use sqlrustgo_parser::parse;
    use sqlrustgo_parser::Statement;

    let sql = "SIGN RECORD FOR inventory REASON 'Stock verified'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SIGN RECORD without columns: {:?}", result);

    match result.unwrap() {
        Statement::SignRecord(sign) => {
            assert_eq!(sign.table_name, "inventory");
            assert!(sign.columns.is_empty());
            assert_eq!(sign.reason, "Stock verified");
        }
        _ => panic!("Expected SignRecord statement"),
    }
}

#[test]
fn test_create_approval_policy_basic() {
    use sqlrustgo_parser::parse;
    use sqlrustgo_parser::Statement;

    let sql = "CREATE APPROVAL POLICY batch_release (required_signatures = 2, required_roles = ('QA_MANAGER', 'PRODUCTION_MANAGER'), sequential = TRUE)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE APPROVAL POLICY: {:?}", result);

    match result.unwrap() {
        Statement::CreateApprovalPolicy(policy) => {
            assert_eq!(policy.name, "batch_release");
            assert_eq!(policy.required_signatures, 2);
            assert_eq!(policy.required_roles.len(), 2);
            assert_eq!(policy.required_roles[0], "QA_MANAGER");
            assert_eq!(policy.required_roles[1], "PRODUCTION_MANAGER");
            assert!(policy.sequential);
        }
        _ => panic!("Expected CreateApprovalPolicy statement"),
    }
}

#[test]
fn test_create_approval_policy_with_optional_params() {
    use sqlrustgo_parser::parse;
    use sqlrustgo_parser::Statement;

    let sql = "CREATE APPROVAL POLICY simple_approval (required_signatures = 1, sequential = FALSE, timeout_hours = 48, description = 'Simple approval')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE APPROVAL POLICY with optional params: {:?}", result);

    match result.unwrap() {
        Statement::CreateApprovalPolicy(policy) => {
            assert_eq!(policy.name, "simple_approval");
            assert_eq!(policy.required_signatures, 1);
            assert!(!policy.sequential);
            assert_eq!(policy.timeout_hours, Some(48));
            assert_eq!(policy.description, Some("Simple approval".to_string()));
        }
        _ => panic!("Expected CreateApprovalPolicy statement"),
    }
}

#[test]
fn test_create_approval_policy_minimal() {
    use sqlrustgo_parser::parse;
    use sqlrustgo_parser::Statement;

    let sql = "CREATE APPROVAL POLICY minimal_policy (required_signatures = 1)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse minimal CREATE APPROVAL POLICY: {:?}", result);

    match result.unwrap() {
        Statement::CreateApprovalPolicy(policy) => {
            assert_eq!(policy.name, "minimal_policy");
            assert_eq!(policy.required_signatures, 1);
            assert!(policy.required_roles.is_empty());
            assert!(policy.sequential);
            assert_eq!(policy.timeout_hours, None);
            assert_eq!(policy.description, None);
        }
        _ => panic!("Expected CreateApprovalPolicy statement"),
    }
}

// ============================================================================
// Module Functions Tests
// ============================================================================

#[test]
fn test_electronic_signature_module_functions() {
    let ts1 = current_timestamp_ms();
    let ts2 = current_timestamp_ms();
    assert!(ts2 >= ts1);

    let data = b"test batch data";
    let hash = compute_data_hash(data);
    assert_eq!(hash.len(), 32);

    let data_hash = vec![0u8; 32];
    let reason = "Approved for release";
    let timestamp = 1234567890i64;
    let payload = compute_signing_payload(&data_hash, reason, timestamp);
    assert!(!payload.is_empty());

    assert_eq!(PolicyStatus::Pending.as_str(), "PENDING");
    assert_eq!(PolicyStatus::Approved.as_str(), "APPROVED");
    assert_eq!(PolicyStatus::Rejected.as_str(), "REJECTED");
}

// ============================================================================
// Struct Tests
// ============================================================================

#[test]
fn test_electronic_signature_struct() {
    let sig = ElectronicSignature::new(
        1,
        "user1".to_string(),
        Some("session1".to_string()),
        Some("ADMIN".to_string()),
        "Approved for release".to_string(),
        vec![0u8; 32],
        vec![0u8; 64],
        vec![0u8; 32],
        current_timestamp_ms(),
        None,
        None,
        None,
    );

    assert!(!sig.id.is_empty());
    assert_eq!(sig.user_id, "user1");
    assert_eq!(sig.reason, "Approved for release");
    assert_eq!(sig.audit_chain_id, 1);
}

#[test]
fn test_approval_policy_struct() {
    let policy = ApprovalPolicy::new(
        "batch_release".to_string(),
        2,
        vec!["QA_MANAGER".to_string(), "PRODUCTION_MANAGER".to_string()],
        true,
        72,
        Some("Requires two managers to approve".to_string()),
    );

    assert!(!policy.id.is_empty());
    assert_eq!(policy.name, "batch_release");
    assert_eq!(policy.required_signatures, 2);
    assert!(policy.sequential);
    assert_eq!(policy.timeout_hours, 72);
}

#[test]
fn test_signature_request_struct() {
    let mut request = SignatureRequest::new(
        "policy-1".to_string(),
        "batches".to_string(),
        "batch-001".to_string(),
        72,
    );

    assert_eq!(request.status, PolicyStatus::Pending);
    assert!(!request.is_expired());

    assert_eq!(request.policy_id, "policy-1");
    assert_eq!(request.record_table, "batches");
    assert_eq!(request.record_id, "batch-001");
}

#[test]
fn test_signature_request_expiry() {
    let mut request = SignatureRequest::new(
        "policy-1".to_string(),
        "batches".to_string(),
        "batch-001".to_string(),
        1,
    );

    assert!(!request.is_expired());

    request.expires_at = request.created_at - 1000;
    assert!(request.is_expired());
}

// ============================================================================
// ApprovalPolicyEvaluator Tests (Workflow)
// ============================================================================

#[test]
fn test_sequential_signature_flow() {
    let policy = ApprovalPolicy::new(
        "batch_release".to_string(),
        2,
        vec!["QA_MANAGER".to_string(), "PRODUCTION_MANAGER".to_string()],
        true,
        72,
        Some("Requires QA and Production manager approval".to_string()),
    );

    let mut evaluator = ApprovalPolicyEvaluator::new(&policy, "req-1".to_string());

    assert_eq!(evaluator.current_status(), PolicyStatus::Pending);

    let eval1 = evaluator.add_signature("user1", "QA_MANAGER").unwrap();
    assert_eq!(eval1.status, PolicyStatus::Pending);
    assert_eq!(eval1.current_signatures, 1);
    assert_eq!(eval1.missing_roles, vec!["PRODUCTION_MANAGER"]);

    let eval2 = evaluator.add_signature("user2", "PRODUCTION_MANAGER").unwrap();
    assert_eq!(eval2.status, PolicyStatus::Approved);
    assert_eq!(eval2.current_signatures, 2);
    assert!(eval2.is_complete);
}

#[test]
fn test_parallel_signature_flow() {
    let policy = ApprovalPolicy::new(
        "batch_release".to_string(),
        2,
        vec!["QA_MANAGER".to_string(), "PRODUCTION_MANAGER".to_string()],
        false,
        72,
        None,
    );

    let mut evaluator = ApprovalPolicyEvaluator::new(&policy, "req-2".to_string());

    assert_eq!(evaluator.current_status(), PolicyStatus::Pending);

    let eval1 = evaluator.add_signature("user1", "PRODUCTION_MANAGER").unwrap();
    assert_eq!(eval1.status, PolicyStatus::Pending);
    assert_eq!(eval1.current_signatures, 1);

    let eval2 = evaluator.add_signature("user2", "QA_MANAGER").unwrap();
    assert_eq!(eval2.status, PolicyStatus::Approved);
    assert_eq!(eval2.current_signatures, 2);
    assert!(eval2.is_complete);
}

#[test]
fn test_sequential_order_violation() {
    let policy = ApprovalPolicy::new(
        "batch_release".to_string(),
        2,
        vec!["QA_MANAGER".to_string(), "PRODUCTION_MANAGER".to_string()],
        true,
        72,
        None,
    );

    let mut evaluator = ApprovalPolicyEvaluator::new(&policy, "req-3".to_string());

    let result = evaluator.add_signature("user1", "PRODUCTION_MANAGER");
    assert!(matches!(result, Err(SignatureError::SequentialOrderViolation { .. })));
}

#[test]
fn test_duplicate_signature_rejection() {
    let policy = ApprovalPolicy::new(
        "batch_release".to_string(),
        2,
        vec!["QA_MANAGER".to_string(), "PRODUCTION_MANAGER".to_string()],
        false,
        72,
        None,
    );

    let mut evaluator = ApprovalPolicyEvaluator::new(&policy, "req-4".to_string());

    evaluator.add_signature("user1", "QA_MANAGER").unwrap();
    let result = evaluator.add_signature("user1", "PRODUCTION_MANAGER");
    assert!(matches!(result, Err(SignatureError::SignatureAlreadyExists { .. })));
}

#[test]
fn test_insufficient_permissions_rejection() {
    let policy = ApprovalPolicy::new(
        "batch_release".to_string(),
        2,
        vec!["QA_MANAGER".to_string(), "PRODUCTION_MANAGER".to_string()],
        false,
        72,
        None,
    );

    let mut evaluator = ApprovalPolicyEvaluator::new(&policy, "req-5".to_string());

    let result = evaluator.add_signature("user1", "ADMIN");
    assert!(matches!(result, Err(SignatureError::InsufficientPermissions { .. })));
}