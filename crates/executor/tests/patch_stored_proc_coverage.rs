//! patch_stored_proc_coverage.rs - Stored Procedure Coverage Killer
//!
//! This file contains targeted tests to boost stored_proc.rs coverage from ~47% to 75%+.
//! Each test is designed to hit specific execute_statement branches.
//!
//! Coverage targets:
//! - execute_statement: 16 branches (If, While, Loop, Repeat, Return, Call, Block, etc.)
//! - execute_sql: DDL/DML paths
//! - execute_statement_storage: INSERT, UPDATE, DELETE paths

use sqlrustgo::ExecutionEngine;
use sqlrustgo_catalog::stored_proc::{
    HandlerCondition, ParamMode, StoredProcParam, StoredProcStatement, StoredProcedure,
};
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

/// Helper to create a stored procedure executor with test catalog and memory storage
fn create_proc_executor(
    catalog: Arc<sqlrustgo_catalog::Catalog>,
) -> sqlrustgo_executor::stored_proc::StoredProcExecutor {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    sqlrustgo_executor::stored_proc::StoredProcExecutor::new(catalog, storage)
}

// =============================================================================
// P0: Control Flow Tests (highest leverage - each hits 3-5% coverage)
// =============================================================================

/// Test IF condition = true (should execute then_body)
#[test]
fn test_sp_if_condition_true() {
    let proc = StoredProcedure::new(
        "sp_if_true".to_string(),
        vec![],
        vec![StoredProcStatement::If {
            condition: "1 = 1".to_string(),
            then_body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "1".to_string(),
            }],
            elseif_body: vec![],
            else_body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "2".to_string(),
            }],
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_if_true", vec![]);
    assert!(result.is_ok());
}

/// Test IF condition = false (should execute else_body)
#[test]
fn test_sp_if_condition_false_goes_to_else() {
    let proc = StoredProcedure::new(
        "sp_if_false".to_string(),
        vec![],
        vec![StoredProcStatement::If {
            condition: "1 = 2".to_string(),
            then_body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "1".to_string(),
            }],
            elseif_body: vec![],
            else_body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "99".to_string(),
            }],
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_if_false", vec![]);
    assert!(result.is_ok());
}

/// Test IF with ELSIF chain
#[test]
fn test_sp_if_elsif_chain() {
    let proc = StoredProcedure::new(
        "sp_if_elsif".to_string(),
        vec![],
        vec![StoredProcStatement::If {
            condition: "1 = 2".to_string(),
            then_body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "1".to_string(),
            }],
            elseif_body: vec![(
                "2 = 2".to_string(),
                vec![StoredProcStatement::Set {
                    variable: "@x".to_string(),
                    value: "2".to_string(),
                }],
            )],
            else_body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "3".to_string(),
            }],
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_if_elsif", vec![]);
    assert!(result.is_ok());
}

/// Test WHILE loop with zero iterations (condition starts false)
#[test]
fn test_sp_while_zero_iterations() {
    let proc = StoredProcedure::new(
        "sp_while_zero".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "i".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("10".to_string()),
            },
            StoredProcStatement::While {
                condition: "i < 3".to_string(),
                body: vec![StoredProcStatement::Set {
                    variable: "@i".to_string(),
                    value: "@i + 1".to_string(),
                }],
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_while_zero", vec![]);
    assert!(result.is_ok());
}

/// Test WHILE loop with multiple iterations
#[test]
fn test_sp_while_multiple_iterations() {
    let proc = StoredProcedure::new(
        "sp_while_multi".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "i".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::While {
                condition: "@i < 3".to_string(),
                body: vec![StoredProcStatement::Set {
                    variable: "@i".to_string(),
                    value: "@i + 1".to_string(),
                }],
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_while_multi", vec![]);
    assert!(result.is_ok());
}

/// Test LOOP with LEAVE to exit
#[test]
fn test_sp_loop_leave() {
    let proc = StoredProcedure::new(
        "sp_loop_leave".to_string(),
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
                        variable: "@i".to_string(),
                        value: "@i + 1".to_string(),
                    },
                    StoredProcStatement::If {
                        condition: "@i >= 2".to_string(),
                        then_body: vec![StoredProcStatement::Leave {
                            label: "myloop".to_string(),
                        }],
                        elseif_body: vec![],
                        else_body: vec![],
                    },
                ],
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_loop_leave", vec![]);
    assert!(result.is_ok());
}

/// Test LOOP with RETURN to exit
#[test]
fn test_sp_loop_return() {
    let proc = StoredProcedure::new(
        "sp_loop_return".to_string(),
        vec![],
        vec![StoredProcStatement::Loop {
            body: vec![StoredProcStatement::Return {
                value: "42".to_string(),
            }],
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_loop_return", vec![]);
    assert!(result.is_ok());
}

/// Test REPEAT...UNTIL loop (executes at least once)
#[test]
fn test_sp_repeat_until() {
    let proc = StoredProcedure::new(
        "sp_repeat".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "i".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("0".to_string()),
            },
            StoredProcStatement::Repeat {
                body: vec![StoredProcStatement::Set {
                    variable: "@i".to_string(),
                    value: "@i + 1".to_string(),
                }],
                condition: "@i >= 3".to_string(),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_repeat", vec![]);
    assert!(result.is_ok());
}

/// Test RETURN statement
#[test]
fn test_sp_return_value() {
    let proc = StoredProcedure::new(
        "sp_return".to_string(),
        vec![],
        vec![StoredProcStatement::Return {
            value: "42".to_string(),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_return", vec![]);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.rows.len(), 1);
}

/// Test RETURN from nested block
#[test]
fn test_sp_return_from_block() {
    let proc = StoredProcedure::new(
        "sp_return_block".to_string(),
        vec![],
        vec![StoredProcStatement::Block {
            label: None,
            body: vec![StoredProcStatement::Return {
                value: "100".to_string(),
            }],
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_return_block", vec![]);
    assert!(result.is_ok());
}

/// Test Block with label
#[test]
fn test_sp_block_with_label() {
    let proc = StoredProcedure::new(
        "sp_block_label".to_string(),
        vec![],
        vec![StoredProcStatement::Block {
            label: Some("myblock".to_string()),
            body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "1".to_string(),
            }],
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_block_label", vec![]);
    assert!(result.is_ok());
}

/// Test Declare variable
#[test]
fn test_sp_declare() {
    let proc = StoredProcedure::new(
        "sp_declare".to_string(),
        vec![],
        vec![
            StoredProcStatement::Declare {
                name: "x".to_string(),
                data_type: "INTEGER".to_string(),
                default_value: Some("5".to_string()),
            },
            StoredProcStatement::Declare {
                name: "y".to_string(),
                data_type: "TEXT".to_string(),
                default_value: None,
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_declare", vec![]);
    assert!(result.is_ok());
}

/// Test Set variable
#[test]
fn test_sp_set() {
    let proc = StoredProcedure::new(
        "sp_set".to_string(),
        vec![],
        vec![StoredProcStatement::Set {
            variable: "@x".to_string(),
            value: "123".to_string(),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_set", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P1: Nested CALL tests
// =============================================================================

/// Test CALL to another stored procedure with args
#[test]
fn test_sp_call_nested() {
    // Create inner procedure first
    let inner = StoredProcedure::new(
        "inner_proc".to_string(),
        vec![StoredProcParam {
            name: "n".to_string(),
            mode: ParamMode::In,
            data_type: "INTEGER".to_string(),
        }],
        vec![StoredProcStatement::Return {
            value: "@n + 10".to_string(),
        }],
    );

    let outer = StoredProcedure::new(
        "outer_proc".to_string(),
        vec![],
        vec![
            StoredProcStatement::Call {
                procedure_name: "inner_proc".to_string(),
                args: vec!["5".to_string()],
                into_var: Some("@result".to_string()),
            },
            StoredProcStatement::Return {
                value: "@result".to_string(),
            },
        ],
    );

    let catalog = Arc::new(sqlrustgo_catalog::Catalog::new("test"));
    let mut catalog_mut = (*catalog).clone();
    catalog_mut.add_stored_procedure(inner).unwrap();
    catalog_mut.add_stored_procedure(outer).unwrap();

    let executor =
        sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(Arc::new(catalog_mut));
    let result = executor.execute_call("outer_proc", vec![]);
    assert!(result.is_ok());
}

/// Test Call with no into_var (fire and forget)
#[test]
fn test_sp_call_no_into() {
    let inner = StoredProcedure::new(
        "inner_no_return".to_string(),
        vec![],
        vec![StoredProcStatement::Set {
            variable: "@done".to_string(),
            value: "1".to_string(),
        }],
    );

    let outer = StoredProcedure::new(
        "outer_no_into".to_string(),
        vec![],
        vec![StoredProcStatement::Call {
            procedure_name: "inner_no_return".to_string(),
            args: vec![],
            into_var: None,
        }],
    );

    let catalog = Arc::new(sqlrustgo_catalog::Catalog::new("test"));
    let mut catalog_mut = (*catalog).clone();
    catalog_mut.add_stored_procedure(inner).unwrap();
    catalog_mut.add_stored_procedure(outer).unwrap();

    let executor =
        sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(Arc::new(catalog_mut));
    let result = executor.execute_call("outer_no_into", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P2: Exception handling tests
// =============================================================================

/// Test DeclareHandler
#[test]
fn test_sp_declare_handler() {
    let proc = StoredProcedure::new(
        "sp_handler".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareHandler {
                condition_type: HandlerCondition::SqlException,
                body: vec![StoredProcStatement::Set {
                    variable: "@handled".to_string(),
                    value: "1".to_string(),
                }],
            },
            StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "1".to_string(),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_handler", vec![]);
    assert!(result.is_ok());
}

/// Test Signal (raises an exception)
#[test]
fn test_sp_signal() {
    let proc = StoredProcedure::new(
        "sp_signal".to_string(),
        vec![],
        vec![StoredProcStatement::Signal {
            sqlstate: Some("45000".to_string()),
            message: Some("test error".to_string()),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_signal", vec![]);
    assert!(result.is_err()); // Signal should return error
}

/// Test Resignal
#[test]
fn test_sp_resignal() {
    let proc = StoredProcedure::new(
        "sp_resignal".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareHandler {
                condition_type: HandlerCondition::SqlException,
                body: vec![StoredProcStatement::Resignal {
                    sqlstate: Some("45000".to_string()),
                    message: Some("reshaped".to_string()),
                }],
            },
            StoredProcStatement::Signal {
                sqlstate: Some("45000".to_string()),
                message: Some("original".to_string()),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_resignal", vec![]);
    // Should get the reshaped error from resignal
    assert!(result.is_err());
}

// =============================================================================
// P3: CASE statement tests
// =============================================================================

/// Test CASE...WHEN...END
#[test]
fn test_sp_case() {
    let proc = StoredProcedure::new(
        "sp_case".to_string(),
        vec![],
        vec![StoredProcStatement::Case {
            case_value: Some("@x".to_string()),
            when_clauses: vec![
                ("1".to_string(), "10".to_string()),
                ("2".to_string(), "20".to_string()),
            ],
            else_result: Some("0".to_string()),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_case", vec![]);
    assert!(result.is_ok());
}

/// Test CASE WHEN (searched case)
#[test]
fn test_sp_case_when() {
    let proc = StoredProcedure::new(
        "sp_case_when".to_string(),
        vec![],
        vec![StoredProcStatement::CaseWhen {
            when_clauses: vec![
                ("1 = 1".to_string(), "100".to_string()),
                ("1 = 2".to_string(), "200".to_string()),
            ],
            else_result: Some("0".to_string()),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_case_when", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P4: Cursor tests
// =============================================================================

/// Test DeclareCursor, OpenCursor, Fetch, CloseCursor
#[test]
fn test_sp_cursor_lifecycle() {
    let proc = StoredProcedure::new(
        "sp_cursor".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareCursor {
                name: "mycursor".to_string(),
                query: "SELECT 1".to_string(),
            },
            StoredProcStatement::OpenCursor {
                name: "mycursor".to_string(),
            },
            StoredProcStatement::Fetch {
                name: "mycursor".to_string(),
                into_vars: vec!["@v".to_string()],
            },
            StoredProcStatement::CloseCursor {
                name: "mycursor".to_string(),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_cursor", vec![]);
    assert!(result.is_ok());
}

/// Test cursor fetch exhaustion
#[test]
fn test_sp_cursor_fetch_exhausted() {
    let proc = StoredProcedure::new(
        "sp_cursor_exhaust".to_string(),
        vec![],
        vec![
            StoredProcStatement::DeclareCursor {
                name: "c".to_string(),
                query: "SELECT 1".to_string(),
            },
            StoredProcStatement::OpenCursor {
                name: "c".to_string(),
            },
            StoredProcStatement::Fetch {
                name: "c".to_string(),
                into_vars: vec!["@v".to_string()],
            },
            StoredProcStatement::Fetch {
                name: "c".to_string(),
                into_vars: vec!["@v".to_string()],
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_cursor_exhaust", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P5: LEAVE and ITERATE with labels
// =============================================================================

/// Test ITERATE (continue to next iteration)
#[test]
fn test_sp_iterate() {
    let proc = StoredProcedure::new(
        "sp_iterate".to_string(),
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
                        variable: "@i".to_string(),
                        value: "@i + 1".to_string(),
                    },
                    StoredProcStatement::If {
                        condition: "@i < 3".to_string(),
                        then_body: vec![StoredProcStatement::Iterate {
                            label: "mylabel".to_string(),
                        }],
                        elseif_body: vec![],
                        else_body: vec![],
                    },
                    StoredProcStatement::Leave {
                        label: "mylabel".to_string(),
                    },
                ],
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_iterate", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P6: execute_body early exit paths
// =============================================================================

/// Test execute_body exits early when should_leave is true
#[test]
fn test_sp_body_early_exit_leave() {
    let proc = StoredProcedure::new(
        "sp_early_leave".to_string(),
        vec![],
        vec![
            StoredProcStatement::Leave {
                label: "outer".to_string(),
            },
            StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "999".to_string(),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_early_leave", vec![]);
    assert!(result.is_ok());
}

/// Test execute_body with RETURN in first statement
#[test]
fn test_sp_body_return_first() {
    let proc = StoredProcedure::new(
        "sp_return_first".to_string(),
        vec![],
        vec![
            StoredProcStatement::Return {
                value: "1".to_string(),
            },
            StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "2".to_string(),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_return_first", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P7: Parameter binding edge cases
// =============================================================================

/// Test with zero arguments (uses defaults)
#[test]
fn test_sp_zero_args() {
    let proc = StoredProcedure::new(
        "sp_zero_args".to_string(),
        vec![StoredProcParam {
            name: "x".to_string(),
            mode: ParamMode::In,
            data_type: "INTEGER".to_string(),
        }],
        vec![StoredProcStatement::Return {
            value: "42".to_string(),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    // Call with no args - param binding should handle gracefully
    let result = executor.execute_call("sp_zero_args", vec![]);
    assert!(result.is_ok());
}

/// Test with partial arguments
#[test]
fn test_sp_partial_args() {
    let proc = StoredProcedure::new(
        "sp_partial".to_string(),
        vec![
            StoredProcParam {
                name: "x".to_string(),
                mode: ParamMode::In,
                data_type: "INTEGER".to_string(),
            },
            StoredProcParam {
                name: "y".to_string(),
                mode: ParamMode::In,
                data_type: "INTEGER".to_string(),
            },
        ],
        vec![StoredProcStatement::Return {
            value: "42".to_string(),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    // Call with only 1 arg when 2 are declared
    let result = executor.execute_call("sp_partial", vec![Value::Integer(1)]);
    assert!(result.is_ok());
}

/// Test with NULL argument
#[test]
fn test_sp_null_arg() {
    let proc = StoredProcedure::new(
        "sp_null_arg".to_string(),
        vec![StoredProcParam {
            name: "x".to_string(),
            mode: ParamMode::In,
            data_type: "INTEGER".to_string(),
        }],
        vec![StoredProcStatement::Return {
            value: "COALESCE(@x, 0)".to_string(),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_null_arg", vec![Value::Null]);
    assert!(result.is_ok());
}

// =============================================================================
// P8: Not found / error paths
// =============================================================================

/// Test calling non-existent procedure (via Call statement)
#[test]
fn test_sp_call_nonexistent() {
    let proc = StoredProcedure::new(
        "sp_call_missing".to_string(),
        vec![],
        vec![StoredProcStatement::Call {
            procedure_name: "nonexistent_proc".to_string(),
            args: vec![],
            into_var: None,
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_call_missing", vec![]);
    assert!(result.is_err()); // Should error because inner proc doesn't exist
}

/// Test procedure not found in catalog
#[test]
fn test_sp_not_found() {
    let catalog = Arc::new(sqlrustgo_catalog::Catalog::new("test"));
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("totally_missing", vec![]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

// =============================================================================
// P9: Expression evaluation paths
// =============================================================================

/// Test expression with binary operations
#[test]
fn test_sp_expression_binary_op() {
    let proc = StoredProcedure::new(
        "sp_expr_binop".to_string(),
        vec![],
        vec![StoredProcStatement::Set {
            variable: "@result".to_string(),
            value: "10 + 20 * 3".to_string(),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_expr_binop", vec![]);
    assert!(result.is_ok());
}

/// Test expression with NULL
#[test]
fn test_sp_expression_null() {
    let proc = StoredProcedure::new(
        "sp_expr_null".to_string(),
        vec![],
        vec![StoredProcStatement::Set {
            variable: "@result".to_string(),
            value: "COALESCE(NULL, 99)".to_string(),
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_expr_null", vec![]);
    assert!(result.is_ok());
}

/// Test comparison expression
#[test]
fn test_sp_expression_comparison() {
    let proc = StoredProcedure::new(
        "sp_expr_cmp".to_string(),
        vec![],
        vec![StoredProcStatement::If {
            condition: "10 > 5".to_string(),
            then_body: vec![StoredProcStatement::Set {
                variable: "@x".to_string(),
                value: "1".to_string(),
            }],
            elseif_body: vec![],
            else_body: vec![],
        }],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_expr_cmp", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P10: Multiple statement execution
// =============================================================================

/// Test multiple statements in sequence
#[test]
fn test_sp_multi_stmt_sequence() {
    let proc = StoredProcedure::new(
        "sp_sequence".to_string(),
        vec![],
        vec![
            StoredProcStatement::Set {
                variable: "@a".to_string(),
                value: "1".to_string(),
            },
            StoredProcStatement::Set {
                variable: "@b".to_string(),
                value: "2".to_string(),
            },
            StoredProcStatement::Set {
                variable: "@c".to_string(),
                value: "@a + @b".to_string(),
            },
            StoredProcStatement::Return {
                value: "@c".to_string(),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_sequence", vec![]);
    assert!(result.is_ok());
}

/// Test nested blocks
#[test]
fn test_sp_nested_blocks() {
    let proc = StoredProcedure::new(
        "sp_nested".to_string(),
        vec![],
        vec![
            StoredProcStatement::Block {
                label: Some("outer".to_string()),
                body: vec![
                    StoredProcStatement::Set {
                        variable: "@x".to_string(),
                        value: "1".to_string(),
                    },
                    StoredProcStatement::Block {
                        label: Some("inner".to_string()),
                        body: vec![StoredProcStatement::Set {
                            variable: "@y".to_string(),
                            value: "2".to_string(),
                        }],
                    },
                ],
            },
            StoredProcStatement::Return {
                value: "@x + @y".to_string(),
            },
        ],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_nested", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P11: Empty body / edge cases
// =============================================================================

/// Test procedure with empty body
#[test]
fn test_sp_empty_body() {
    let proc = StoredProcedure::new("sp_empty".to_string(), vec![], vec![]);
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_empty", vec![]);
    assert!(result.is_ok());
}

/// Test procedure with only RawSql (empty string)
#[test]
fn test_sp_raw_sql_empty() {
    let proc = StoredProcedure::new(
        "sp_raw_empty".to_string(),
        vec![],
        vec![StoredProcStatement::RawSql("".to_string())],
    );
    let catalog = create_catalog_with_proc(proc);
    let executor = sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(catalog);
    let result = executor.execute_call("sp_raw_empty", vec![]);
    assert!(result.is_ok());
}

// =============================================================================
// P12: Coverage summary test (meta)
// =============================================================================

/// Test list_procedures and has_procedure
#[test]
fn test_sp_catalog_operations() {
    let proc1 = StoredProcedure::new("proc1".to_string(), vec![], vec![]);
    let proc2 = StoredProcedure::new("proc2".to_string(), vec![], vec![]);

    let catalog = Arc::new(sqlrustgo_catalog::Catalog::new("test"));
    let mut catalog_mut = (*catalog).clone();
    catalog_mut.add_stored_procedure(proc1).unwrap();
    catalog_mut.add_stored_procedure(proc2).unwrap();

    let executor =
        sqlrustgo_executor::stored_proc::StoredProcExecutor::new_for_test(Arc::new(catalog_mut));

    assert!(executor.has_procedure("proc1"));
    assert!(executor.has_procedure("proc2"));
    assert!(!executor.has_procedure("nonexistent"));

    let names = executor.list_procedures();
    assert!(names.contains(&"proc1"));
    assert!(names.contains(&"proc2"));
}
