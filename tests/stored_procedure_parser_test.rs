// Stored Procedure Parser Integration Tests
// Tests for parsing stored procedure statements (CALL and CREATE PROCEDURE)

use sqlrustgo_parser::{parse, Statement, StoredProcParamMode};

#[test]
fn test_parse_call_simple() {
    let sql = "CALL test_proc()";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);
    match result.unwrap() {
        Statement::Call(call) => {
            assert_eq!(call.procedure_name, "test_proc");
            assert!(call.args.is_empty());
        }
        _ => panic!("Expected CALL statement"),
    }
}

#[test]
fn test_parse_call_with_args() {
    let sql = "CALL test_proc(1, 'hello', var1)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);
    match result.unwrap() {
        Statement::Call(call) => {
            assert_eq!(call.procedure_name, "test_proc");
            assert_eq!(call.args.len(), 3);
            assert_eq!(call.args[0], "1");
            assert_eq!(call.args[1], "hello");
            assert_eq!(call.args[2], "var1");
        }
        _ => panic!("Expected CALL statement"),
    }
}

// TODO(v2.6.0): Re-enable when parser supports NULL literal
#[ignore]
#[test]
fn test_parse_call_with_null() {
    let sql = "CALL test_proc(NULL)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);
    match result.unwrap() {
        Statement::Call(call) => {
            assert_eq!(call.procedure_name, "test_proc");
            assert_eq!(call.args.len(), 1);
            assert_eq!(call.args[0], "NULL");
        }
        _ => panic!("Expected CALL statement"),
    }
}

#[test]
fn test_parse_create_procedure_simple() {
    let sql = "CREATE PROCEDURE test_proc() BEGIN SELECT 1 END";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);
    match result.unwrap() {
        Statement::CreateProcedure(p) => {
            assert_eq!(p.name, "test_proc");
            assert!(p.params.is_empty());
            assert_eq!(p.body.len(), 1);
        }
        _ => panic!("Expected CREATE PROCEDURE statement"),
    }
}

// TODO(v2.6.0): Re-enable when parser supports IN/OUT/INOUT parameter modes
#[ignore]
#[test]
fn test_parse_create_procedure_with_in_param() {
    let sql =
        "CREATE PROCEDURE test_proc(IN id INTEGER) BEGIN SELECT * FROM users WHERE id = id END";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::CreateProcedure(p) => {
            assert_eq!(p.name, "test_proc");
            assert_eq!(p.params.len(), 1);
            assert_eq!(p.params[0].name, "id");
            assert_eq!(p.params[0].mode, StoredProcParamMode::In);
            assert_eq!(p.params[0].data_type, "INTEGER");
        }
        _ => panic!("Expected CREATE PROCEDURE statement"),
    }
}

// TODO(v2.6.0): Re-enable when parser supports IN/OUT/INOUT parameter modes
#[ignore]
#[test]
fn test_parse_create_procedure_with_multiple_params() {
    let sql = "CREATE PROCEDURE add_user(IN name TEXT, IN email TEXT) BEGIN INSERT INTO users VALUES (name, email) END";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::CreateProcedure(p) => {
            assert_eq!(p.name, "add_user");
            assert_eq!(p.params.len(), 2);
            assert_eq!(p.params[0].name, "name");
            assert_eq!(p.params[0].data_type, "TEXT");
            assert_eq!(p.params[1].name, "email");
            assert_eq!(p.params[1].data_type, "TEXT");
        }
        _ => panic!("Expected CREATE PROCEDURE statement"),
    }
}

// TODO(v2.6.0): Re-enable when parser supports IN/OUT/INOUT parameter modes
#[ignore]
#[test]
fn test_parse_create_procedure_with_out_param() {
    let sql =
        "CREATE PROCEDURE get_count(OUT cnt INTEGER) BEGIN SELECT COUNT(*) INTO cnt FROM users END";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::CreateProcedure(p) => {
            assert_eq!(p.name, "get_count");
            assert_eq!(p.params.len(), 1);
            assert_eq!(p.params[0].name, "cnt");
            assert_eq!(p.params[0].mode, StoredProcParamMode::Out);
            assert_eq!(p.params[0].data_type, "INTEGER");
        }
        _ => panic!("Expected CREATE PROCEDURE statement"),
    }
}

#[test]
fn test_parse_create_procedure_without_param_mode() {
    // When no mode is specified, defaults to IN
    let sql = "CREATE PROCEDURE test_proc(id INTEGER) BEGIN SELECT * FROM users END";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::CreateProcedure(p) => {
            assert_eq!(p.name, "test_proc");
            assert_eq!(p.params.len(), 1);
            assert_eq!(p.params[0].name, "id");
            assert_eq!(p.params[0].mode, StoredProcParamMode::In);
            assert_eq!(p.params[0].data_type, "INTEGER");
        }
        _ => panic!("Expected CREATE PROCEDURE statement"),
    }
}

// TODO(v2.6.0): Re-enable when parser supports IN/OUT/INOUT parameter modes
#[ignore]
#[test]
fn test_parse_create_procedure_with_inout_param() {
    let sql =
        "CREATE PROCEDURE test_proc(INOUT counter INTEGER) BEGIN SET counter = counter + 1 END";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::CreateProcedure(p) => {
            assert_eq!(p.name, "test_proc");
            assert_eq!(p.params.len(), 1);
            assert_eq!(p.params[0].name, "counter");
            assert_eq!(p.params[0].mode, StoredProcParamMode::InOut);
            assert_eq!(p.params[0].data_type, "INTEGER");
        }
        _ => panic!("Expected CREATE PROCEDURE statement"),
    }
}

// TODO(v2.6.0): Re-enable when parser supports NULL literal
#[ignore]
#[test]
fn test_parse_call_with_mixed_args() {
    let sql = "CALL process_user(1, 'Alice', balance, NULL)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
    match result.unwrap() {
        Statement::Call(call) => {
            assert_eq!(call.procedure_name, "process_user");
            assert_eq!(call.args.len(), 4);
            assert_eq!(call.args[0], "1");
            assert_eq!(call.args[1], "Alice");
            assert_eq!(call.args[2], "balance");
            assert_eq!(call.args[3], "NULL");
        }
        _ => panic!("Expected CALL statement"),
    }
}
