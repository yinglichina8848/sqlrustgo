use sqlrustgo_parser::parse;

// ============================================================================
// SAVEPOINT Tests
// ============================================================================

#[test]
fn test_savepoint_simple() {
    let result = parse("SAVEPOINT sp1");
    assert!(result.is_ok(), "SAVEPOINT should parse: {:?}", result);
}

#[test]
fn test_savepoint_quoted_name() {
    let result = parse("SAVEPOINT 'my_savepoint'");
    assert!(
        result.is_ok(),
        "SAVEPOINT with quoted name should parse: {:?}",
        result
    );
}

#[test]
fn test_savepoint_underscore_name() {
    let result = parse("SAVEPOINT my_sp_1");
    assert!(
        result.is_ok(),
        "SAVEPOINT with underscore name: {:?}",
        result
    );
}

// ============================================================================
// RELEASE SAVEPOINT Tests
// ============================================================================

#[test]
fn test_release_savepoint_simple() {
    let result = parse("RELEASE SAVEPOINT sp1");
    assert!(
        result.is_ok(),
        "RELEASE SAVEPOINT should parse: {:?}",
        result
    );
}

#[test]
fn test_release_savepoint_quoted() {
    let result = parse("RELEASE SAVEPOINT 'my_sp'");
    assert!(
        result.is_ok(),
        "RELEASE SAVEPOINT with quoted name: {:?}",
        result
    );
}

// ============================================================================
// ROLLBACK TO SAVEPOINT Tests
// ============================================================================

#[test]
fn test_rollback_to_savepoint() {
    let result = parse("ROLLBACK TO SAVEPOINT sp1");
    assert!(
        result.is_ok(),
        "ROLLBACK TO SAVEPOINT should parse: {:?}",
        result
    );
}

#[test]
fn test_rollback_to_savepoint_quoted() {
    let result = parse("ROLLBACK TO SAVEPOINT 'my_savepoint'");
    assert!(
        result.is_ok(),
        "ROLLBACK TO SAVEPOINT with quoted name: {:?}",
        result
    );
}

#[test]
fn test_rollback_to_savepoint_nested() {
    let result = parse("ROLLBACK TO SAVEPOINT level1_nested_sp");
    assert!(result.is_ok(), "ROLLBACK TO SAVEPOINT nested: {:?}", result);
}

// ============================================================================
// SET ROLE Tests
// ============================================================================

#[test]
fn test_set_role_identifier() {
    let result = parse("SET ROLE admin");
    assert!(result.is_ok(), "SET ROLE should parse: {:?}", result);
}

#[test]
fn test_set_role_quoted() {
    let result = parse("SET ROLE 'db_admin'");
    assert!(result.is_ok(), "SET ROLE with quoted name: {:?}", result);
}

#[test]
fn test_set_role_underscore() {
    let result = parse("SET ROLE read_only_user");
    assert!(
        result.is_ok(),
        "SET ROLE with underscore name: {:?}",
        result
    );
}

// ============================================================================
// BEGIN Isolation Level Variations (without ISOLATION LEVEL keyword)
// ============================================================================

#[test]
fn test_begin_serializable_shortcut() {
    let result = parse("BEGIN SERIALIZABLE");
    assert!(
        result.is_ok(),
        "BEGIN SERIALIZABLE should parse: {:?}",
        result
    );
}

#[test]
fn test_begin_repeatable_read_shortcut() {
    let result = parse("BEGIN REPEATABLE READ");
    assert!(
        result.is_ok(),
        "BEGIN REPEATABLE READ should parse: {:?}",
        result
    );
}

#[test]
fn test_begin_read_committed_shortcut() {
    let result = parse("BEGIN READ COMMITTED");
    assert!(
        result.is_ok(),
        "BEGIN READ COMMITTED should parse: {:?}",
        result
    );
}

#[test]
fn test_begin_read_uncommitted_shortcut() {
    let result = parse("BEGIN READ UNCOMMITTED");
    assert!(
        result.is_ok(),
        "BEGIN READ UNCOMMITTED should parse: {:?}",
        result
    );
}

// ============================================================================
// START TRANSACTION Variations
// ============================================================================

#[test]
fn test_start_transaction_read_committed() {
    let result = parse("START TRANSACTION ISOLATION LEVEL READ COMMITTED");
    assert!(
        result.is_ok(),
        "START TRANSACTION ISOLATION LEVEL READ COMMITTED: {:?}",
        result
    );
}

#[test]
fn test_start_transaction_read_uncommitted() {
    let result = parse("START TRANSACTION ISOLATION LEVEL READ UNCOMMITTED");
    assert!(
        result.is_ok(),
        "START TRANSACTION ISOLATION LEVEL READ UNCOMMITTED: {:?}",
        result
    );
}

#[test]
fn test_start_transaction_repeatable_read() {
    let result = parse("START TRANSACTION ISOLATION LEVEL REPEATABLE READ");
    assert!(
        result.is_ok(),
        "START TRANSACTION ISOLATION LEVEL REPEATABLE READ: {:?}",
        result
    );
}

// ============================================================================
// Error Path Tests for Transaction Statements
// ============================================================================

#[test]
fn test_savepoint_error_no_name() {
    let result = parse("SAVEPOINT");
    assert!(
        result.is_err(),
        "SAVEPOINT without name should fail: {:?}",
        result
    );
}

#[test]
fn test_release_savepoint_error_no_name() {
    let result = parse("RELEASE SAVEPOINT");
    assert!(
        result.is_err(),
        "RELEASE SAVEPOINT without name should fail: {:?}",
        result
    );
}

#[test]
fn test_rollback_to_savepoint_error_no_name() {
    let result = parse("ROLLBACK TO SAVEPOINT");
    assert!(
        result.is_err(),
        "ROLLBACK TO SAVEPOINT without name should fail: {:?}",
        result
    );
}

#[test]
fn test_set_role_error_no_name() {
    let result = parse("SET ROLE");
    assert!(
        result.is_err(),
        "SET ROLE without name should fail: {:?}",
        result
    );
}

#[test]
fn test_begin_error_invalid_isolation() {
    let result = parse("BEGIN ISOLATION LEVEL INVALID_LEVEL");
    assert!(
        result.is_err(),
        "BEGIN with invalid isolation level should fail: {:?}",
        result
    );
}

#[test]
fn test_begin_error_incomplete() {
    let result = parse("BEGIN ISOLATION LEVEL");
    assert!(
        result.is_err(),
        "BEGIN with incomplete isolation level should fail: {:?}",
        result
    );
}

#[test]
fn test_begin_read_error_incomplete() {
    let result = parse("BEGIN READ");
    assert!(
        result.is_err(),
        "BEGIN READ without completion should fail: {:?}",
        result
    );
}

#[test]
fn test_set_transaction_error_no_level() {
    let result = parse("SET TRANSACTION");
    assert!(
        result.is_err(),
        "SET TRANSACTION without ISOLATION LEVEL should fail: {:?}",
        result
    );
}

#[test]
fn test_set_transaction_error_invalid_level() {
    let result = parse("SET TRANSACTION ISOLATION LEVEL INVALID");
    assert!(
        result.is_err(),
        "SET TRANSACTION with invalid level should fail: {:?}",
        result
    );
}

#[test]
fn test_start_transaction_error_invalid_isolation() {
    let result = parse("START TRANSACTION ISOLATION LEVEL INVALID");
    assert!(
        result.is_err(),
        "START TRANSACTION with invalid isolation should fail: {:?}",
        result
    );
}

#[test]
fn test_release_error_not_savepoint() {
    let result = parse("RELEASE something");
    assert!(
        result.is_err(),
        "RELEASE without SAVEPOINT should fail: {:?}",
        result
    );
}
