//! Stored Procedure Executor
//!
//! This module provides stored procedure execution support with control flow.
//!
//! # Module Structure
//!
//! - `context` - ProcedureContext, variables, labels, scope management
//! - `cursor` - Cursor management
//! - `handler` - Exception handling
//! - `cte` - CTE and recursive query execution
//! - `expression` - Expression evaluation
//! - `execution` - Statement execution

mod context;
mod cte;
mod cursor;
mod execution;
mod expression;
mod handler;

use crate::ExecutorResult;
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

pub use context::{ExceptionHandler, ProcedureContext, StoredProcError};

/// Stored procedure executor for calling stored procedures
#[derive(Clone)]
pub struct StoredProcExecutor {
    catalog: Arc<sqlrustgo_catalog::Catalog>,
    storage: Arc<RwLock<dyn StorageEngine>>,
}

impl StoredProcExecutor {
    /// Create a new stored procedure executor
    pub fn new(
        catalog: Arc<sqlrustgo_catalog::Catalog>,
        storage: Arc<RwLock<dyn StorageEngine>>,
    ) -> Self {
        Self { catalog, storage }
    }

    #[cfg(test)]
    fn new_for_test(catalog: Arc<sqlrustgo_catalog::Catalog>) -> Self {
        use sqlrustgo_storage::MemoryStorage;
        Self {
            catalog,
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
        }
    }

    /// Execute a stored procedure call
    pub fn execute_call(&self, name: &str, args: Vec<Value>) -> Result<ExecutorResult, String> {
        // Look up the stored procedure
        let procedure = self
            .catalog
            .get_stored_procedure(name)
            .ok_or_else(|| format!("Stored procedure '{}' not found", name))?;

        // Execute the procedure body
        let mut ctx = ProcedureContext::new();

        // Bind input parameters to variables
        for (i, param) in procedure.params.iter().enumerate() {
            if i < args.len() {
                ctx.set_var(&param.name, args[i].clone());
            }
        }

        // Execute procedure body
        let result = self.execute_body(&procedure.body, &mut ctx);

        // Check if RETURN was called
        if let Some(value) = ctx.get_return() {
            return Ok(ExecutorResult::new(vec![vec![value]], 1));
        }

        match result {
            Ok(_) => {
                let result_json = ctx
                    .get_session_var("__last_select_result")
                    .and_then(|v| {
                        if let Value::Text(s) = &v {
                            serde_json::from_str(s).ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                let found_rows = ctx
                    .get_session_var("__found_rows")
                    .and_then(|v| {
                        if let Value::Integer(i) = v {
                            Some(*i as usize)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                Ok(ExecutorResult::new(result_json, found_rows))
            }
            Err(e) => Err(e),
        }
    }

    /// Execute a statement with CTE support (public method for ExecutionEngine)
    pub fn execute_with_cte(
        &self,
        statement: &sqlrustgo_parser::Statement,
    ) -> Result<ExecutorResult, String> {
        let mut ctx = ProcedureContext::new();
        self.execute_statement_storage_impl(statement, &mut ctx)?;

        let result_json = ctx
            .get_session_var("__last_select_result")
            .and_then(|v| {
                if let Value::Text(s) = &v {
                    serde_json::from_str(s).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let found_rows = ctx
            .get_session_var("__found_rows")
            .and_then(|v| {
                if let Value::Integer(i) = v {
                    Some(*i as usize)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        Ok(ExecutorResult::new(result_json, found_rows))
    }

    /// Check if a stored procedure exists
    pub fn has_procedure(&self, name: &str) -> bool {
        self.catalog.has_stored_procedure(name)
    }

    /// List all stored procedure names
    pub fn list_procedures(&self) -> Vec<&str> {
        self.catalog.stored_procedure_names()
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;
    use sqlrustgo_catalog::Catalog;
    use sqlrustgo_catalog::HandlerCondition;
    use sqlrustgo_catalog::StoredProcStatement;

    #[test]
    fn test_stored_proc_executor_not_found() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);

        let result = executor.execute_call("non_existent", vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_stored_proc_executor_list_empty() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);

        assert!(executor.list_procedures().is_empty());
        assert!(!executor.has_procedure("test"));
    }

    #[test]
    fn test_procedure_context_set_get_var() {
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(42));
        assert_eq!(ctx.get_var("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_procedure_context_session_vars() {
        let mut ctx = ProcedureContext::new();
        // Set session variable with @ prefix
        ctx.set_var("@uid", Value::Integer(100));
        // Should be accessible both ways
        assert_eq!(ctx.get_var("@uid"), Some(&Value::Integer(100)));
        assert_eq!(ctx.get_var("uid"), Some(&Value::Integer(100)));
        // Session variables persist
        ctx.clear_local_vars();
        assert_eq!(ctx.get_var("uid"), Some(&Value::Integer(100)));
    }

    #[test]
    fn test_procedure_context_leave() {
        let mut ctx = ProcedureContext::new();
        assert!(!ctx.should_leave());
        ctx.set_leave();
        assert!(ctx.should_leave());
        ctx.reset_leave();
        assert!(!ctx.should_leave());
    }

    #[test]
    fn test_procedure_context_return() {
        let mut ctx = ProcedureContext::new();
        assert!(ctx.get_return().is_none());
        ctx.set_return(Value::Integer(100));
        assert_eq!(ctx.get_return(), Some(Value::Integer(100)));
    }

    #[test]
    fn test_evaluate_condition() {
        let catalog = Arc::new(Catalog::new("test"));
        let _executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(10));

        // Note: This test requires the condition to reference the variable properly
        // In practice, we'd need proper variable expansion
    }

    #[test]
    fn test_cursor_operations() {
        let mut ctx = ProcedureContext::new();

        ctx.declare_cursor("cur1".to_string(), "SELECT * FROM t".to_string());
        assert!(ctx.has_cursor("cur1"));

        let result = ctx.open_cursor("cur1");
        assert!(result.is_ok());

        ctx.set_cursor_records(
            "cur1",
            vec![
                vec![Value::Integer(1), Value::Text("a".to_string())],
                vec![Value::Integer(2), Value::Text("b".to_string())],
            ],
        );

        let has_rows1 = ctx.fetch_cursor("cur1", &["v1".to_string(), "v2".to_string()]);
        assert!(has_rows1.is_ok());
        assert!(has_rows1.unwrap());
        assert_eq!(ctx.get_var("v1"), Some(&Value::Integer(1)));
        assert_eq!(ctx.get_var("v2"), Some(&Value::Text("a".to_string())));

        let has_rows2 = ctx.fetch_cursor("cur1", &["v1".to_string(), "v2".to_string()]);
        assert!(has_rows2.is_ok());
        assert!(has_rows2.unwrap());
        assert_eq!(ctx.get_var("v1"), Some(&Value::Integer(2)));
        assert_eq!(ctx.get_var("v2"), Some(&Value::Text("b".to_string())));

        let has_rows3 = ctx.fetch_cursor("cur1", &["v1".to_string()]);
        assert!(has_rows3.is_ok());
        assert!(!has_rows3.unwrap());

        let close_result = ctx.close_cursor("cur1");
        assert!(close_result.is_ok());

        let not_found = ctx.open_cursor("nonexistent");
        assert!(not_found.is_err());
    }

    #[test]
    fn test_exception_handlers() {
        let mut ctx = ProcedureContext::new();

        ctx.push_handler(
            HandlerCondition::SqlException,
            vec![StoredProcStatement::Return {
                value: "1".to_string(),
            }],
        );
        ctx.push_handler(
            HandlerCondition::SqlWarning,
            vec![StoredProcStatement::Return {
                value: "2".to_string(),
            }],
        );

        assert_eq!(ctx.handler_stack.len(), 2);

        let exc_sqlexception = StoredProcError {
            sqlstate: "45000".to_string(),
            message: "test error".to_string(),
        };
        let handler = ctx.find_matching_handler(&exc_sqlexception);
        assert!(handler.is_some());

        ctx.pop_handler();
        assert_eq!(ctx.handler_stack.len(), 1);

        ctx.clear_exception();
        assert!(ctx.get_exception().is_none());
    }

    #[test]
    fn test_exception_handling_flow() {
        let mut ctx = ProcedureContext::new();
        assert!(!ctx.is_handling_exception());

        ctx.set_exception_handling(true);
        assert!(ctx.is_handling_exception());

        ctx.set_exception("45000".to_string(), "test".to_string());
        let exc = ctx.get_exception();
        assert!(exc.is_some());
        assert_eq!(exc.unwrap().sqlstate, "45000");

        ctx.clear_exception();
        ctx.set_exception_handling(false);
        assert!(!ctx.is_handling_exception());
    }

    #[test]
    fn test_label_operations() {
        let mut ctx = ProcedureContext::new();

        ctx.enter_label("loop1".to_string());
        assert!(ctx.has_label("loop1"));

        ctx.exit_label();
        assert!(!ctx.has_label("loop1"));
    }

    #[test]
    fn test_scope_operations() {
        let mut ctx = ProcedureContext::new();

        ctx.set_local_var("x", Value::Integer(10));
        assert_eq!(ctx.get_local_var("x"), Some(&Value::Integer(10)));

        ctx.enter_scope();
        ctx.set_local_var("y", Value::Text("hello".to_string()));
        assert_eq!(
            ctx.get_local_var("y"),
            Some(&Value::Text("hello".to_string()))
        );

        ctx.exit_scope();
        assert!(ctx.get_local_var("y").is_none());
        assert_eq!(ctx.get_local_var("x"), Some(&Value::Integer(10)));
    }

    #[test]
    fn test_iterate_control() {
        let mut ctx = ProcedureContext::new();
        assert!(!ctx.should_iterate());

        ctx.set_iterate();
        assert!(ctx.should_iterate());

        ctx.reset_iterate();
        assert!(!ctx.should_iterate());
    }

    #[test]
    fn test_get_session_vars() {
        let mut ctx = ProcedureContext::new();
        ctx.set_var("@uid", Value::Integer(100));
        ctx.set_var("@name", Value::Text("test".to_string()));

        let vars = ctx.get_session_vars();
        assert_eq!(vars.get("uid"), Some(&Value::Integer(100)));
        assert_eq!(vars.get("name"), Some(&Value::Text("test".to_string())));
    }

    // =====================================================================
    // White-box tests for stored_proc - Coverage: Statement Execution Paths
    // =====================================================================

    // --- ProcedureContext: Label operations ---

    #[test]
    fn test_procedure_context_nested_labels() {
        let mut ctx = ProcedureContext::new();
        ctx.enter_label("outer".to_string());
        ctx.enter_label("inner".to_string());
        assert!(ctx.has_label("outer"));
        assert!(ctx.has_label("inner"));
        ctx.exit_label();
        assert!(ctx.has_label("outer"));
        assert!(!ctx.has_label("inner"));
        ctx.exit_label();
        assert!(!ctx.has_label("outer"));
    }

    #[test]
    fn test_procedure_context_get_label() {
        let mut ctx = ProcedureContext::new();
        assert!(ctx.get_label().is_none());
        ctx.enter_label("test_label".to_string());
        assert_eq!(ctx.get_label(), Some(&"test_label".to_string()));
    }

    #[test]
    fn test_procedure_context_set_label() {
        let mut ctx = ProcedureContext::new();
        ctx.set_label(Some("my_label".to_string()));
        assert_eq!(ctx.get_label(), Some(&"my_label".to_string()));
        ctx.set_label(None);
        assert!(ctx.get_label().is_none());
    }

    #[test]
    fn test_procedure_context_nested_scopes() {
        let mut ctx = ProcedureContext::new();
        ctx.set_local_var("x", Value::Integer(1));
        assert_eq!(ctx.get_local_var("x"), Some(&Value::Integer(1)));
        ctx.enter_scope();
        ctx.set_local_var("y", Value::Text("hello".to_string()));
        assert_eq!(
            ctx.get_local_var("y"),
            Some(&Value::Text("hello".to_string()))
        );
        ctx.exit_scope();
        assert!(ctx.get_local_var("y").is_none());
        assert_eq!(ctx.get_local_var("x"), Some(&Value::Integer(1)));
    }

    #[test]
    fn test_procedure_context_scope_shadowing() {
        let mut ctx = ProcedureContext::new();
        ctx.set_local_var("z", Value::Integer(100));
        ctx.enter_scope();
        ctx.set_local_var("z", Value::Integer(200));
        assert_eq!(ctx.get_local_var("z"), Some(&Value::Integer(200)));
        ctx.exit_scope();
        assert_eq!(ctx.get_local_var("z"), Some(&Value::Integer(100)));
    }

    #[test]
    fn test_procedure_context_local_vs_session_var() {
        let mut ctx = ProcedureContext::new();
        ctx.set_local_var("local_x", Value::Integer(10));
        assert_eq!(ctx.get_local_var("local_x"), Some(&Value::Integer(10)));
        ctx.set_session_var("session_y", Value::Text("test".to_string()));
        assert_eq!(
            ctx.get_session_var("session_y"),
            Some(&Value::Text("test".to_string()))
        );
        assert_eq!(ctx.get_var("local_x"), Some(&Value::Integer(10)));
        assert_eq!(
            ctx.get_var("session_y"),
            Some(&Value::Text("test".to_string()))
        );
    }

    #[test]
    fn test_procedure_context_get_var_strips_prefix() {
        let mut ctx = ProcedureContext::new();
        ctx.set_var("@uid", Value::Integer(999));
        assert_eq!(ctx.get_var("@uid"), Some(&Value::Integer(999)));
        assert_eq!(ctx.get_var("uid"), Some(&Value::Integer(999)));
    }

    #[test]
    fn test_procedure_context_has_var() {
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(1));
        ctx.set_var("@y", Value::Text("a".to_string()));
        assert!(ctx.has_var("x"));
        assert!(ctx.has_var("@x"));
        assert!(ctx.has_var("@y"));
        assert!(ctx.has_var("y"));
        assert!(!ctx.has_var("nonexistent"));
    }

    #[test]
    fn test_procedure_context_clear_local_vars() {
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(1));
        ctx.set_var("@y", Value::Integer(2));
        ctx.clear_local_vars();
        assert!(!ctx.has_var("x"));
        assert!(ctx.has_var("y"));
    }

    #[test]
    fn test_procedure_context_return_value() {
        let mut ctx = ProcedureContext::new();
        assert!(ctx.get_return().is_none());
        ctx.set_return(Value::Integer(42));
        assert_eq!(ctx.get_return(), Some(Value::Integer(42)));
    }

    // --- StoredProcExecutor: evaluate_constant ---

    #[test]
    fn test_evaluate_constant_string_literal() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let result = executor.evaluate_constant("'hello world'");
        assert_eq!(result, Value::Text("hello world".to_string()));
    }

    #[test]
    fn test_evaluate_constant_integer() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(executor.evaluate_constant("42"), Value::Integer(42));
        assert_eq!(executor.evaluate_constant("-10"), Value::Integer(-10));
    }

    #[test]
    fn test_evaluate_constant_float() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(executor.evaluate_constant("3.14"), Value::Float(3.14));
        assert_eq!(executor.evaluate_constant("-2.5"), Value::Float(-2.5));
    }

    #[test]
    fn test_evaluate_constant_null() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(executor.evaluate_constant("NULL"), Value::Null);
        assert_eq!(executor.evaluate_constant("null"), Value::Null);
    }

    #[test]
    fn test_evaluate_constant_boolean() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(executor.evaluate_constant("TRUE"), Value::Boolean(true));
        assert_eq!(executor.evaluate_constant("FALSE"), Value::Boolean(false));
        assert_eq!(executor.evaluate_constant("true"), Value::Boolean(true));
        assert_eq!(executor.evaluate_constant("false"), Value::Boolean(false));
    }

    #[test]
    fn test_evaluate_constant_identifier() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(
            executor.evaluate_constant("some_identifier"),
            Value::Text("some_identifier".to_string())
        );
    }

    // --- StoredProcExecutor: expand_variables_in_sql ---

    #[test]
    fn test_expand_variables_in_sql_no_vars() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let ctx = ProcedureContext::new();
        let sql = "SELECT * FROM users WHERE id = 1";
        assert_eq!(executor.expand_variables_in_sql(sql, &ctx), sql);
    }

    #[test]
    fn test_expand_variables_in_sql_integer_var() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("user_id", Value::Integer(42));
        let sql = "SELECT * FROM users WHERE id = @user_id";
        assert_eq!(
            executor.expand_variables_in_sql(sql, &ctx),
            "SELECT * FROM users WHERE id = 42"
        );
    }

    #[test]
    fn test_expand_variables_in_sql_text_var() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("name", Value::Text("Alice".to_string()));
        let sql = "SELECT * FROM users WHERE name = '@name'";
        assert_eq!(
            executor.expand_variables_in_sql(sql, &ctx),
            "SELECT * FROM users WHERE name = 'Alice'"
        );
    }

    #[test]
    fn test_expand_variables_in_sql_undefined_var() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let ctx = ProcedureContext::new();
        let sql = "SELECT * FROM users WHERE id = @undefined_var";
        assert_eq!(
            executor.expand_variables_in_sql(sql, &ctx),
            "SELECT * FROM users WHERE id = NULL"
        );
    }

    #[test]
    fn test_expand_variables_in_sql_multiple_vars() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("a", Value::Integer(1));
        ctx.set_var("b", Value::Integer(2));
        let sql = "WHERE x = @a AND y = @b";
        assert_eq!(
            executor.expand_variables_in_sql(sql, &ctx),
            "WHERE x = 1 AND y = 2"
        );
    }

    #[test]
    fn test_expand_variables_in_sql_alphanumeric_var() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("user123", Value::Integer(100));
        let sql = "SELECT * FROM users WHERE user_id = @user123";
        assert_eq!(
            executor.expand_variables_in_sql(sql, &ctx),
            "SELECT * FROM users WHERE user_id = 100"
        );
    }

    // --- StoredProcExecutor: compare_values ---

    #[test]
    fn test_compare_values_equality() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let int42 = Value::Integer(42);
        let int42b = Value::Integer(42);
        let int100 = Value::Integer(100);
        assert!(executor.compare_values(&int42, &int42b, "="));
        assert!(executor.compare_values(&int42, &int42b, "=="));
        assert!(!executor.compare_values(&int42, &int100, "="));
    }

    #[test]
    fn test_compare_values_inequality() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let a = Value::Integer(10);
        let b = Value::Integer(20);
        assert!(executor.compare_values(&a, &b, "!="));
        assert!(executor.compare_values(&a, &b, "<>"));
        assert!(!executor.compare_values(&a, &b, "="));
    }

    #[test]
    fn test_compare_values_greater_than() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let big = Value::Integer(100);
        let small = Value::Integer(50);
        assert!(executor.compare_values(&big, &small, ">"));
        assert!(!executor.compare_values(&small, &big, ">"));
    }

    #[test]
    fn test_compare_values_greater_than_or_equal() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let a = Value::Integer(10);
        let b = Value::Integer(10);
        let c = Value::Integer(5);
        assert!(executor.compare_values(&a, &b, ">="));
        assert!(executor.compare_values(&a, &c, ">="));
        assert!(!executor.compare_values(&c, &a, ">="));
    }

    #[test]
    fn test_compare_values_less_than() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let small = Value::Integer(5);
        let big = Value::Integer(100);
        assert!(executor.compare_values(&small, &big, "<"));
        assert!(!executor.compare_values(&big, &small, "<"));
    }

    #[test]
    fn test_compare_values_less_than_or_equal() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let a = Value::Integer(10);
        let b = Value::Integer(10);
        let c = Value::Integer(15);
        assert!(executor.compare_values(&a, &b, "<="));
        assert!(executor.compare_values(&a, &c, "<="));
        assert!(!executor.compare_values(&c, &a, "<="));
    }

    #[test]
    fn test_compare_values_text() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let alice = Value::Text("Alice".to_string());
        let bob = Value::Text("Bob".to_string());
        assert!(executor.compare_values(&alice, &bob, "<"));
        assert!(executor.compare_values(&alice, &alice, "="));
    }

    #[test]
    fn test_compare_values_boolean() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let t = Value::Boolean(true);
        let f = Value::Boolean(false);
        assert!(executor.compare_values(&t, &t, "="));
        assert!(executor.compare_values(&t, &f, "!="));
    }

    #[test]
    fn test_compare_values_null() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let null = Value::Null;
        let val = Value::Integer(1);
        assert!(!executor.compare_values(&null, &val, "="));
        assert!(!executor.compare_values(&val, &null, "="));
    }

    // --- StoredProcExecutor: partial_cmp ---

    #[test]
    fn test_partial_cmp_integer() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let a = Value::Integer(5);
        let b = Value::Integer(10);
        assert_eq!(executor.partial_cmp(&a, &b), Some(std::cmp::Ordering::Less));
        assert_eq!(
            executor.partial_cmp(&b, &a),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            executor.partial_cmp(&a, &a),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_partial_cmp_float() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let a = Value::Float(1.5);
        let b = Value::Float(2.5);
        assert_eq!(executor.partial_cmp(&a, &b), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_partial_cmp_text() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let a = Value::Text("abc".to_string());
        let b = Value::Text("def".to_string());
        assert_eq!(executor.partial_cmp(&a, &b), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_partial_cmp_boolean() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let t = Value::Boolean(true);
        let f = Value::Boolean(false);
        assert_eq!(executor.partial_cmp(&f, &t), Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_partial_cmp_null_left() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(executor.partial_cmp(&Value::Null, &Value::Integer(1)), None);
        assert_eq!(executor.partial_cmp(&Value::Integer(1), &Value::Null), None);
    }

    #[test]
    fn test_partial_cmp_mixed_types() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(
            executor.partial_cmp(&Value::Integer(1), &Value::Float(1.0)),
            None
        );
    }

    // --- StoredProcExecutor: arithmetic_op ---

    #[test]
    fn test_arithmetic_op_integer_add() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(
            executor.arithmetic_op(&Value::Integer(1), &Value::Integer(2), "+"),
            Ok(Value::Integer(3))
        );
    }

    #[test]
    fn test_arithmetic_op_integer_subtract() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(
            executor.arithmetic_op(&Value::Integer(10), &Value::Integer(3), "-"),
            Ok(Value::Integer(7))
        );
    }

    #[test]
    fn test_arithmetic_op_integer_multiply() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(
            executor.arithmetic_op(&Value::Integer(6), &Value::Integer(7), "*"),
            Ok(Value::Integer(42))
        );
    }

    #[test]
    fn test_arithmetic_op_integer_divide() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(
            executor.arithmetic_op(&Value::Integer(20), &Value::Integer(4), "/"),
            Ok(Value::Integer(5))
        );
    }

    #[test]
    fn test_arithmetic_op_integer_divide_by_zero() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert!(executor
            .arithmetic_op(&Value::Integer(1), &Value::Integer(0), "/")
            .is_err());
    }

    #[test]
    fn test_arithmetic_op_float_operations() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert_eq!(
            executor.arithmetic_op(&Value::Float(1.5), &Value::Float(2.5), "+"),
            Ok(Value::Float(4.0))
        );
        assert_eq!(
            executor.arithmetic_op(&Value::Float(5.0), &Value::Float(2.0), "-"),
            Ok(Value::Float(3.0))
        );
        assert_eq!(
            executor.arithmetic_op(&Value::Float(3.0), &Value::Float(2.0), "*"),
            Ok(Value::Float(6.0))
        );
        assert_eq!(
            executor.arithmetic_op(&Value::Float(6.0), &Value::Float(2.0), "/"),
            Ok(Value::Float(3.0))
        );
    }

    #[test]
    fn test_arithmetic_op_float_divide_by_zero() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert!(executor
            .arithmetic_op(&Value::Float(1.0), &Value::Float(0.0), "/")
            .is_err());
    }

    #[test]
    fn test_arithmetic_op_unsupported() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert!(executor
            .arithmetic_op(&Value::Text("a".to_string()), &Value::Integer(1), "+")
            .is_err());
    }

    // --- HandlerCondition matching ---

    #[test]
    fn test_find_matching_handler_sql_exception_45xxx() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::SqlException,
            vec![StoredProcStatement::Return {
                value: "1".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "45000".to_string(),
            message: "test".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_some());
    }

    #[test]
    fn test_find_matching_handler_sql_exception_22000() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::SqlException,
            vec![StoredProcStatement::Return {
                value: "1".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "22000".to_string(),
            message: "data error".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_some());
    }

    #[test]
    fn test_find_matching_handler_sql_exception_not_matched() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::SqlException,
            vec![StoredProcStatement::Return {
                value: "1".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "01000".to_string(),
            message: "warning".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_none());
    }

    #[test]
    fn test_find_matching_handler_sql_warning() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::SqlWarning,
            vec![StoredProcStatement::Return {
                value: "2".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "01000".to_string(),
            message: "warning".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_some());
    }

    #[test]
    fn test_find_matching_handler_not_found() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::NotFound,
            vec![StoredProcStatement::Return {
                value: "3".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "02000".to_string(),
            message: "no rows".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_some());
    }

    #[test]
    fn test_find_matching_handler_sqlstate_specific() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::SqlState("23000".to_string()),
            vec![StoredProcStatement::Return {
                value: "4".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "23000".to_string(),
            message: "constraint".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_some());
    }

    #[test]
    fn test_find_matching_handler_custom_by_message() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::Custom("deadlock".to_string()),
            vec![StoredProcStatement::Return {
                value: "5".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "40001".to_string(),
            message: "deadlock detected".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_some());
    }

    #[test]
    fn test_find_matching_handler_multiple_handlers() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(HandlerCondition::SqlWarning, vec![]);
        ctx.push_handler(HandlerCondition::SqlException, vec![]);
        ctx.push_handler(HandlerCondition::NotFound, vec![]);
        let exc = StoredProcError {
            sqlstate: "45000".to_string(),
            message: "error".to_string(),
        };
        let handler = ctx.find_matching_handler(&exc);
        assert!(handler.is_some());
    }

    #[test]
    fn test_find_matching_handler_no_match() {
        let mut ctx = ProcedureContext::new();
        ctx.push_handler(
            HandlerCondition::SqlState("99999".to_string()),
            vec![StoredProcStatement::Return {
                value: "1".to_string(),
            }],
        );
        let exc = StoredProcError {
            sqlstate: "00000".to_string(),
            message: "unknown".to_string(),
        };
        assert!(ctx.find_matching_handler(&exc).is_none());
    }

    // --- Cursor: edge cases ---

    #[test]
    fn test_cursor_fetch_into_more_vars_than_columns() {
        let mut ctx = ProcedureContext::new();
        ctx.declare_cursor("cur".to_string(), "SELECT a, b FROM t".to_string());
        ctx.set_cursor_records(
            "cur",
            vec![vec![Value::Integer(1), Value::Text("x".to_string())]],
        );
        ctx.open_cursor("cur").unwrap();
        let result = ctx.fetch_cursor(
            "cur",
            &["v1".to_string(), "v2".to_string(), "v3".to_string()],
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(ctx.get_var("v3"), Some(&Value::Null));
    }

    #[test]
    fn test_cursor_close_nonexistent() {
        let mut ctx = ProcedureContext::new();
        let result = ctx.close_cursor("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_cursor_fetch_when_not_open() {
        let mut ctx = ProcedureContext::new();
        ctx.declare_cursor("cur".to_string(), "SELECT 1".to_string());
        let result = ctx.fetch_cursor("cur", &["v1".to_string()]);
        assert!(result.is_err());
    }

    // --- Exception handling flow ---

    #[test]
    fn test_exception_handling_flow_full() {
        let mut ctx = ProcedureContext::new();
        assert!(!ctx.is_handling_exception());
        ctx.set_exception_handling(true);
        assert!(ctx.is_handling_exception());
        ctx.set_exception("45000".to_string(), "test error".to_string());
        let exc = ctx.get_exception();
        assert!(exc.is_some());
        assert_eq!(exc.unwrap().sqlstate, "45000");
        ctx.clear_exception();
        assert!(ctx.get_exception().is_none());
        ctx.set_exception_handling(false);
        assert!(!ctx.is_handling_exception());
    }

    #[test]
    fn test_exception_handling_update_existing() {
        let mut ctx = ProcedureContext::new();
        ctx.set_exception("11111".to_string(), "first".to_string());
        ctx.set_exception("22222".to_string(), "second".to_string());
        let exc = ctx.get_exception().unwrap();
        assert_eq!(exc.sqlstate, "22222");
        assert_eq!(exc.message, "second");
    }

    // --- StoredProcError ---

    #[test]
    fn test_stored_proc_error_display() {
        let err = StoredProcError {
            sqlstate: "45000".to_string(),
            message: "test error".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("45000"));
        assert!(display.contains("test error"));
    }

    // --- StoredProcExecutor: procedure existence checks ---

    #[test]
    fn test_has_procedure_false() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        assert!(!executor.has_procedure("nonexistent"));
    }

    // --- expand_variables_in_sql: edge cases ---

    #[test]
    fn test_expand_variables_in_sql_var_with_underscore() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("user_name", Value::Text("Bob".to_string()));
        let sql = "SELECT * FROM users WHERE name = '@user_name'";
        assert_eq!(
            executor.expand_variables_in_sql(sql, &ctx),
            "SELECT * FROM users WHERE name = 'Bob'"
        );
    }

    #[test]
    fn test_expand_variables_in_sql_at_end() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("id", Value::Integer(5));
        let sql = "SELECT @id";
        assert_eq!(executor.expand_variables_in_sql(sql, &ctx), "SELECT 5");
    }

    #[test]
    fn test_expand_variables_in_sql_consecutive_vars() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("a", Value::Integer(1));
        ctx.set_var("b", Value::Integer(2));
        let sql = "@a@b";
        assert_eq!(executor.expand_variables_in_sql(sql, &ctx), "12");
    }

    #[test]
    fn test_expand_variables_in_sql_only_var() {
        let catalog = Arc::new(Catalog::new("test"));
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Null);
        let sql = "@x";
        assert_eq!(executor.expand_variables_in_sql(sql, &ctx), "NULL");
    }

    // ── like_match tests ─────────────────────────────────────────

    #[test]
    fn test_like_match_exact() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("hello".into()), &Value::Text("hello".into())));
    }

    #[test]
    fn test_like_match_percent_prefix() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("hello".into()), &Value::Text("h%".into())));
    }

    #[test]
    fn test_like_match_percent_suffix() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("hello".into()), &Value::Text("%o".into())));
    }

    #[test]
    fn test_like_match_percent_middle() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("hello".into()), &Value::Text("%ell%".into())));
    }

    #[test]
    fn test_like_match_underscore() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("hello".into()), &Value::Text("h_llo".into())));
    }

    #[test]
    fn test_like_match_underscore_too_long() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(!executor.like_match(&Value::Text("hello".into()), &Value::Text("h____o".into())));
    }

    #[test]
    fn test_like_match_non_text() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(!executor.like_match(&Value::Integer(42), &Value::Text("42".into())));
    }

    #[test]
    fn test_like_match_case_insensitive() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("HELLO".into()), &Value::Text("hello".into())));
    }

    #[test]
    fn test_like_match_no_match() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(!executor.like_match(&Value::Text("hello".into()), &Value::Text("world".into())));
    }

    #[test]
    fn test_like_match_multi_percent() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(
            &Value::Text("a quick brown fox".into()),
            &Value::Text("%quick%brown%".into()),
        ));
    }

    #[test]
    fn test_like_match_only_percent() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("anything".into()), &Value::Text("%".into())));
    }

    #[test]
    fn test_like_match_backtrack() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("aXbYb".into()), &Value::Text("a%b".into())));
    }

    #[test]
    fn test_like_match_empty_text() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.like_match(&Value::Text("".into()), &Value::Text("".into())));
        assert!(executor.like_match(&Value::Text("".into()), &Value::Text("%".into())));
        assert!(!executor.like_match(&Value::Text("".into()), &Value::Text("_".into())));
    }

    // ── between_match tests ──────────────────────────────────────

    #[test]
    fn test_between_match_in_range() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.between_match(
            &Value::Integer(5),
            &Value::Integer(1),
            &Value::Integer(10)
        ));
    }

    #[test]
    fn test_between_match_below_range() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(!executor.between_match(
            &Value::Integer(0),
            &Value::Integer(1),
            &Value::Integer(10)
        ));
    }

    #[test]
    fn test_between_match_above_range() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(!executor.between_match(
            &Value::Integer(15),
            &Value::Integer(1),
            &Value::Integer(10)
        ));
    }

    #[test]
    fn test_between_match_boundary() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.between_match(
            &Value::Integer(1),
            &Value::Integer(1),
            &Value::Integer(10)
        ));
        assert!(executor.between_match(
            &Value::Integer(10),
            &Value::Integer(1),
            &Value::Integer(10)
        ));
    }

    #[test]
    fn test_between_match_float() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.between_match(&Value::Float(3.5), &Value::Float(1.0), &Value::Float(5.0)));
        assert!(!executor.between_match(
            &Value::Float(7.5),
            &Value::Float(1.0),
            &Value::Float(5.0)
        ));
    }

    // ── regexp_match tests ───────────────────────────────────────

    #[test]
    fn test_regexp_match_basic() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.regexp_match(
            &Value::Text("hello world".into()),
            &Value::Text("hello".into())
        ));
    }

    #[test]
    fn test_regexp_match_no_match() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(!executor.regexp_match(&Value::Text("hello".into()), &Value::Text("xyz".into())));
    }

    #[test]
    fn test_regexp_match_non_text() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(!executor.regexp_match(&Value::Integer(42), &Value::Text("42".into())));
    }

    #[test]
    fn test_regexp_match_case_insensitive() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.regexp_match(&Value::Text("HELLO".into()), &Value::Text("hello".into())));
    }

    #[test]
    fn test_regexp_match_partial() {
        let executor = StoredProcExecutor::new_for_test(Arc::new(Catalog::new("test")));
        assert!(executor.regexp_match(
            &Value::Text("the quick brown fox".into()),
            &Value::Text("brown".into())
        ));
    }
}
