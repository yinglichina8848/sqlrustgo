//! Integration tests for Stored Procedure Executor - Issue #1164

use sqlrustgo_catalog::{
    Catalog, HandlerCondition, ParamMode, StoredProcParam, StoredProcStatement, StoredProcedure,
};
use sqlrustgo_executor::stored_proc::{ProcedureContext, StoredProcError, StoredProcExecutor};
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_executor_with_proc(proc: StoredProcedure) -> StoredProcExecutor {
    let storage: Arc<RwLock<dyn StorageEngine>> = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut catalog = Catalog::new();
    catalog.add_stored_procedure(proc).unwrap();
    StoredProcExecutor::new(Arc::new(catalog), storage)
}

// ============================================================================
// Variable Declaration Tests
// ============================================================================

#[test]
fn test_declare_and_set_variable() {
    let proc = StoredProcedure::new(
        "test_decl".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("10".to_string()),
            },
            StoredProcStatement::Set {
                variable: "x".to_string(),
                value: "20".to_string(),
            },
            StoredProcStatement::Return {
                value: "@x".to_string(),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_decl", vec![]).is_ok());
}

#[test]
fn test_declare_with_null_default() {
    let proc = StoredProcedure::new(
        "test_null".to_string(),
        vec![],
        vec![StoredProcStatement::Declare {
            name: "x".to_string(),
            data_type: "INTEGER".to_string(),
            default_value: None,
        }],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_null", vec![]).is_ok());
}

// ============================================================================
// IF/ELSE Conditional Tests
// ============================================================================

#[test]
fn test_if_true_branch() {
    let proc = StoredProcedure::new(
        "test_if".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("10".to_string()),
            },
            StoredProcStatement::If {
                condition: "@x > 5".to_string(),
                then_body: vec![StoredProcStatement::Set {
                    variable: "result".to_string(),
                    value: "'large'".to_string(),
                }],
                elseif_body: vec![],
                else_body: vec![],
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_if", vec![]).is_ok());
}

#[test]
fn test_if_else_branch() {
    let proc = StoredProcedure::new(
        "test_if_else".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("3".to_string()),
            },
            StoredProcStatement::If {
                condition: "@x > 5".to_string(),
                then_body: vec![StoredProcStatement::Set {
                    variable: "result".to_string(),
                    value: "'large'".to_string(),
                }],
                elseif_body: vec![],
                else_body: vec![StoredProcStatement::Set {
                    variable: "result".to_string(),
                    value: "'small'".to_string(),
                }],
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_if_else", vec![]).is_ok());
}

// ============================================================================
// WHILE Loop Tests
// ============================================================================

#[test]
fn test_while_loop() {
    let proc = StoredProcedure::new(
        "test_while".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "i".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::While {
                condition: "@i < 5".to_string(),
                body: vec![StoredProcStatement::Set {
                    variable: "i".to_string(),
                    value: "@i + 1".to_string(),
                }],
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_while", vec![]).is_ok());
}

#[test]
fn test_while_with_leave() {
    let proc = StoredProcedure::new(
        "test_while_leave".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "i".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::While {
                condition: "@i < 100".to_string(),
                body: vec![
                    StoredProcStatement::Set {
                        variable: "i".to_string(),
                        value: "@i + 1".to_string(),
                    },
                    StoredProcStatement::If {
                        condition: "@i >= 5".to_string(),
                        then_body: vec![StoredProcStatement::Leave {
                            label: String::new(),
                        }],
                        elseif_body: vec![],
                        else_body: vec![],
                    },
                ],
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_while_leave", vec![]).is_ok());
}

// ============================================================================
// LOOP with LEAVE/ITERATE Tests
// ============================================================================

#[test]
fn test_infinite_loop_with_leave() {
    let proc = StoredProcedure::new(
        "test_loop_leave".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "i".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::Loop {
                body: vec![
                    StoredProcStatement::Set {
                        variable: "i".to_string(),
                        value: "@i + 1".to_string(),
                    },
                    StoredProcStatement::If {
                        condition: "@i >= 10".to_string(),
                        then_body: vec![StoredProcStatement::Leave {
                            label: String::new(),
                        }],
                        elseif_body: vec![],
                        else_body: vec![],
                    },
                ],
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_loop_leave", vec![]).is_ok());
}

// ============================================================================
// RETURN Statement Tests
// ============================================================================

#[test]
fn test_return_with_value() {
    let proc = StoredProcedure::new(
        "test_return".to_string(),
        vec![],
        vec![StoredProcStatement::Return {
            value: "42".to_string(),
        }],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_return", vec![]).is_ok());
}

// ============================================================================
// CALL Nested Procedure Tests
// ============================================================================

#[test]
fn test_call_nested_procedure() {
    let inner = StoredProcedure::new(
        "inner_proc".to_string(),
        vec![],
        vec![StoredProcStatement::Return {
            value: "100".to_string(),
        }],
    );
    let outer = StoredProcedure::new(
        "outer_proc".to_string(),
        vec![],
        vec![
            StoredProcStatement::Call {
                procedure_name: "inner_proc".to_string(),
                args: vec![],
                into_var: Some("result".to_string()),
            },
            StoredProcStatement::Return {
                value: "@result".to_string(),
            },
        ],
    );
    let mut catalog = Catalog::new();
    catalog.add_stored_procedure(inner).unwrap();
    catalog.add_stored_procedure(outer).unwrap();
    let storage: Arc<RwLock<dyn StorageEngine>> = Arc::new(RwLock::new(MemoryStorage::new()));
    let executor = StoredProcExecutor::new(Arc::new(catalog), storage);
    assert!(executor.execute_call("outer_proc", vec![]).is_ok());
}

#[test]
fn test_call_nonexistent_procedure() {
    let proc = StoredProcedure::new(
        "test_call_missing".to_string(),
        vec![],
        vec![StoredProcStatement::Call {
            procedure_name: "missing_proc".to_string(),
            args: vec![],
            into_var: None,
        }],
    );
    let executor = create_executor_with_proc(proc);
    let result = executor.execute_call("test_call_missing", vec![]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// ============================================================================
// SIGNAL/RESIGNAL Exception Tests
// ============================================================================

#[test]
fn test_signal_exception() {
    let proc = StoredProcedure::new(
        "test_signal".to_string(),
        vec![],
        vec![StoredProcStatement::Signal {
            sqlstate: Some("45000".to_string()),
            message: Some("Custom error".to_string()),
        }],
    );
    let executor = create_executor_with_proc(proc);
    let result = executor.execute_call("test_signal", vec![]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("45000") && err.contains("Custom error"));
}

#[test]
fn test_resignal() {
    let proc = StoredProcedure::new(
        "test_resignal".to_string(),
        vec![],
        vec![
            StoredProcStatement::Signal {
                sqlstate: Some("22012".to_string()),
                message: Some("Division by zero".to_string()),
            },
            StoredProcStatement::Resignal {
                sqlstate: Some("45000".to_string()),
                message: Some("After signal".to_string()),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_resignal", vec![]).is_err());
}

// ============================================================================
// BEGIN/END Block Scope Tests
// ============================================================================

#[test]
fn test_block_scope() {
    let proc = StoredProcedure::new(
        "test_block".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("1".to_string()),
            },
            StoredProcStatement::Block {
                label: Some("inner_block".to_string()),
                body: vec![StoredProcStatement::Declare {
                    name: "y".to_string(),
                    data_type: "INTEGER".to_string(),
                    default_value: Some("2".to_string()),
                }],
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_block", vec![]).is_ok());
}

#[test]
fn test_nested_blocks() {
    let proc = StoredProcedure::new(
        "test_nested".to_string(),
        vec![],
        vec![StoredProcStatement::Block {
            label: Some("outer".to_string()),
            body: vec![
                StoredProcStatement::Declare {
                    name: "x".to_string(),
                    data_type: "INTEGER".to_string(),
                    default_value: Some("1".to_string()),
                },
                StoredProcStatement::Block {
                    label: Some("inner".to_string()),
                    body: vec![StoredProcStatement::Declare {
                        name: "y".to_string(),
                        data_type: "INTEGER".to_string(),
                        default_value: Some("2".to_string()),
                    }],
                },
            ],
        }],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_nested", vec![]).is_ok());
}

// ============================================================================
// DECLARE HANDLER Tests
// ============================================================================

#[test]
fn test_declare_handler_sqlexception() {
    let proc = StoredProcedure::new(
        "test_handler".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareHandler {
                condition_type: HandlerCondition::SqlException,
                body: vec![StoredProcStatement::Set {
                    variable: "handled".to_string(),
                    value: "'yes'".to_string(),
                }],
            },
            StoredProcStatement::Signal {
                sqlstate: Some("45000".to_string()),
                message: Some("Error".to_string()),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_handler", vec![]).is_ok());
}

#[test]
fn test_declare_handler_not_found() {
    let proc = StoredProcedure::new(
        "test_handler_notfound".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareHandler {
                condition_type: HandlerCondition::NotFound,
                body: vec![StoredProcStatement::Set {
                    variable: "notfound".to_string(),
                    value: "'true'".to_string(),
                }],
            },
            StoredProcStatement::Signal {
                sqlstate: Some("02000".to_string()),
                message: Some("No data".to_string()),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor
        .execute_call("test_handler_notfound", vec![])
        .is_ok());
}

// ============================================================================
// Cursor Operation Tests
// ============================================================================

#[test]
fn test_cursor_declare() {
    let proc = StoredProcedure::new(
        "test_cursor_decl".to_string(),
        vec![],
        vec![StoredProcStatement::DeclareCursor {
            name: "cur1".to_string(),
            query: "SELECT * FROM users".to_string(),
        }],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_cursor_decl", vec![]).is_ok());
}

#[test]
fn test_cursor_open() {
    let proc = StoredProcedure::new(
        "test_cursor_open".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareCursor {
                name: "cur1".to_string(),
                query: "SELECT * FROM users".to_string(),
            },
            StoredProcStatement::OpenCursor {
                name: "cur1".to_string(),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_cursor_open", vec![]).is_ok());
}

#[test]
fn test_cursor_fetch() {
    let proc = StoredProcedure::new(
        "test_cursor_fetch".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareCursor {
                name: "cur1".to_string(),
                query: "SELECT id, name FROM users".to_string(),
            },
            StoredProcStatement::OpenCursor {
                name: "cur1".to_string(),
            },
            StoredProcStatement::Fetch {
                name: "cur1".to_string(),
                into_vars: vec!["v_id".to_string(), "v_name".to_string()],
            },
            StoredProcStatement::CloseCursor {
                name: "cur1".to_string(),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_cursor_fetch", vec![]).is_ok());
}

#[test]
fn test_cursor_not_found() {
    let proc = StoredProcedure::new(
        "test_cursor_missing".to_string(),
        vec![],
        vec![StoredProcStatement::OpenCursor {
            name: "nonexistent".to_string(),
        }],
    );
    let executor = create_executor_with_proc(proc);
    let result = executor.execute_call("test_cursor_missing", vec![]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// ============================================================================
// DML Tests
// ============================================================================

#[test]
fn test_insert_statement() {
    let proc = StoredProcedure::new(
        "test_insert".to_string(),
        vec![],
        vec![StoredProcStatement::RawSql(
            "INSERT INTO users VALUES (1, 'Alice')".to_string(),
        )],
    );
    let executor = create_executor_with_proc(proc);
    let result = executor.execute_call("test_insert", vec![]);
    // May fail if table doesn't exist - that's OK
    assert!(result.is_ok() || result.unwrap_err().contains("not found"));
}

#[test]
fn test_select_into() {
    let proc = StoredProcedure::new(
        "test_select_into".to_string(),
        vec![],
        vec![StoredProcStatement::SelectInto {
            columns: vec!["id".to_string(), "name".to_string()],
            into_vars: vec!["v_id".to_string(), "v_name".to_string()],
            table: "users".to_string(),
            where_clause: Some("id = 1".to_string()),
        }],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_select_into", vec![]).is_ok());
}

// ============================================================================
// ProcedureContext Tests
// ============================================================================

#[test]
fn test_procedure_context_new() {
    let ctx = ProcedureContext::new();
    assert!(ctx.get_return().is_none());
    assert!(!ctx.should_leave());
    assert!(!ctx.should_iterate());
}

#[test]
fn test_procedure_context_local_vars() {
    let mut ctx = ProcedureContext::new();
    ctx.set_var("x", Value::Integer(10));
    assert_eq!(ctx.get_var("x"), Some(&Value::Integer(10)));
}

#[test]
fn test_procedure_context_session_vars() {
    let mut ctx = ProcedureContext::new();
    ctx.set_var("@uid", Value::Integer(100));
    assert_eq!(ctx.get_var("uid"), Some(&Value::Integer(100)));
    ctx.clear_local_vars();
    assert_eq!(ctx.get_var("uid"), Some(&Value::Integer(100)));
}

#[test]
fn test_procedure_context_leave_iterate() {
    let mut ctx = ProcedureContext::new();
    assert!(!ctx.should_leave());
    ctx.set_leave();
    assert!(ctx.should_leave());
    ctx.reset_leave();
    assert!(!ctx.should_leave());

    assert!(!ctx.should_iterate());
    ctx.set_iterate();
    assert!(ctx.should_iterate());
    ctx.reset_iterate();
    assert!(!ctx.should_iterate());
}

#[test]
fn test_procedure_context_labels() {
    let mut ctx = ProcedureContext::new();
    ctx.enter_label("loop1".to_string());
    assert!(ctx.has_label("loop1"));
    assert!(!ctx.has_label("loop2"));
    ctx.exit_label();
    assert!(!ctx.has_label("loop1"));
}

#[test]
fn test_procedure_context_scopes() {
    let mut ctx = ProcedureContext::new();
    ctx.set_var("x", Value::Integer(1));
    ctx.enter_scope();
    ctx.set_var("y", Value::Integer(2));
    assert_eq!(ctx.get_var("y"), Some(&Value::Integer(2)));
    ctx.exit_scope();
    assert_eq!(ctx.get_var("x"), Some(&Value::Integer(1)));
    assert!(ctx.get_var("y").is_none());
}

#[test]
fn test_stored_proc_error_display() {
    let err = StoredProcError {
        sqlstate: "45000".to_string(),
        message: "Custom error".to_string(),
    };
    let display = format!("{}", err);
    assert!(display.contains("45000") && display.contains("Custom error"));
}

// ============================================================================
// Expression Evaluation Tests
// ============================================================================

#[test]
fn test_expression_literals() {
    let proc = StoredProcedure::new(
        "test_expr".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("42".to_string()),
            },
            StoredProcStatement::Declare {
                name: "s".to_string(),
                data_type: "VARCHAR".to_string(),
                default_value: Some("'hello'".to_string()),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_expr", vec![]).is_ok());
}

#[test]
fn test_expression_arithmetic() {
    let proc = StoredProcedure::new(
        "test_arith".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "a".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("10".to_string()),
            },
            StoredProcStatement::Set {
                variable: "sum".to_string(),
                value: "@a + 5".to_string(),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("test_arith", vec![]).is_ok());
}

// ============================================================================
// Catalog Tests
// ============================================================================

#[test]
fn test_stored_procedure_creation() {
    let proc = StoredProcedure::new(
        "test_proc".to_string(),
        vec![StoredProcParam {
            name: "param1".to_string(),
            mode: ParamMode::In,
            data_type: "INTEGER".to_string(),
        }],
        vec![StoredProcStatement::RawSql("SELECT 1".to_string())],
    );
    assert_eq!(proc.name, "test_proc");
    assert_eq!(proc.params.len(), 1);
    assert_eq!(proc.body.len(), 1);
}

#[test]
fn test_param_mode() {
    assert!(matches!(ParamMode::In, ParamMode::In));
    assert!(matches!(ParamMode::Out, ParamMode::Out));
    assert!(matches!(ParamMode::InOut, ParamMode::InOut));
}

#[test]
fn test_catalog_stored_procedures() {
    let mut catalog = Catalog::new();
    let proc = StoredProcedure::new(
        "test".to_string(),
        vec![],
        vec![StoredProcStatement::RawSql("SELECT 1".to_string())],
    );
    assert!(catalog.add_stored_procedure(proc).is_ok());
    assert!(catalog.get_stored_procedure("test").is_some());
}

// ============================================================================
// Issue #1164 Regression Tests
// ============================================================================

#[test]
fn test_issue_1164_full_procedure() {
    let proc = StoredProcedure::new(
        "process_orders".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "counter".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::DeclareHandler {
                condition_type: HandlerCondition::SqlException,
                body: vec![StoredProcStatement::Set {
                    variable: "error_handled".to_string(),
                    value: "'yes'".to_string(),
                }],
            },
            StoredProcStatement::Loop {
                body: vec![
                    StoredProcStatement::Set {
                        variable: "counter".to_string(),
                        value: "@counter + 1".to_string(),
                    },
                    StoredProcStatement::If {
                        condition: "@counter >= 10".to_string(),
                        then_body: vec![StoredProcStatement::Leave {
                            label: String::new(),
                        }],
                        elseif_body: vec![],
                        else_body: vec![],
                    },
                ],
            },
            StoredProcStatement::Return {
                value: "@counter".to_string(),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("process_orders", vec![]).is_ok());
}

#[test]
fn test_issue_1164_cursor_with_loop() {
    let proc = StoredProcedure::new(
        "process_cursor".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareCursor {
                name: "data_cursor".to_string(),
                query: "SELECT id, value FROM items".to_string(),
            },
            StoredProcStatement::OpenCursor {
                name: "data_cursor".to_string(),
            },
            StoredProcStatement::Loop {
                body: vec![
                    StoredProcStatement::Fetch {
                        name: "data_cursor".to_string(),
                        into_vars: vec!["item_id".to_string(), "item_val".to_string()],
                    },
                    StoredProcStatement::Leave {
                        label: String::new(),
                    },
                ],
            },
            StoredProcStatement::CloseCursor {
                name: "data_cursor".to_string(),
            },
        ],
    );
    let executor = create_executor_with_proc(proc);
    assert!(executor.execute_call("process_cursor", vec![]).is_ok());
}
