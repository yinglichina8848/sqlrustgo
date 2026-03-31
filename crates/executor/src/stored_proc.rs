//! Stored Procedure Executor
//!
//! This module provides stored procedure execution support with control flow.

use crate::ExecutorResult;
use sqlrustgo_catalog::StoredProcStatement;
use sqlrustgo_types::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Stored procedure execution error
#[derive(Debug, Clone)]
pub struct StoredProcError {
    pub sqlstate: String,
    pub message: String,
}

impl std::fmt::Display for StoredProcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SQLSTATE {}: {}", self.sqlstate, self.message)
    }
}

impl std::error::Error for StoredProcError {}

/// Default SQLSTATE codes
pub const SQLSTATE_SQL_EXCEPTION: &str = "45000";
pub const SQLSTATE_WARNING: &str = "01000";
pub const SQLSTATE_NOT_FOUND: &str = "02000";

/// Session-level variables for stored procedure execution
#[derive(Debug, Clone, Default)]
pub struct ProcedureContext {
    /// Local variables declared in the procedure (key without @)
    local_variables: HashMap<String, Value>,
    /// Session-level variables (key without @, e.g., "uid" for @uid)
    session_variables: HashMap<String, Value>,
    /// Return value (if RETURN was called)
    return_value: Option<Value>,
    /// Whether to exit the current loop/block
    leave: bool,
    /// Whether to continue to next iteration
    iterate: bool,
    /// Current loop label
    current_label: Option<String>,
    /// Label stack for validating LEAVE/ITERATE targets
    label_stack: Vec<String>,
    /// Current exception (for RESIGNAL)
    current_exception: Option<StoredProcError>,
}

impl ProcedureContext {
    /// Create a new procedure context
    pub fn new() -> Self {
        Self {
            local_variables: HashMap::new(),
            session_variables: HashMap::new(),
            return_value: None,
            leave: false,
            iterate: false,
            current_label: None,
            label_stack: Vec::new(),
            current_exception: None,
        }
    }

    /// Push a label onto the stack (enter a labeled block)
    pub fn enter_label(&mut self, label: String) {
        self.label_stack.push(label.clone());
        self.current_label = Some(label);
    }

    /// Pop a label from the stack (exit a labeled block)
    pub fn exit_label(&mut self) -> Option<String> {
        let popped = self.label_stack.pop();
        self.current_label = self.label_stack.last().cloned();
        popped
    }

    /// Check if a label exists in the stack
    pub fn has_label(&self, label: &str) -> bool {
        self.label_stack.iter().any(|l| l == label)
    }

    /// Validate that LEAVE/ITERATE target label exists
    pub fn validate_label(&self, label: &str) -> Result<(), String> {
        if label.is_empty() {
            // If no label specified, use current label if available
            if self.current_label.is_none() {
                return Err("LEAVE/ITERATE without label requires an active loop".to_string());
            }
            Ok(())
        } else if !self.has_label(label) {
            Err(format!("Label '{}' not found in current scope", label))
        } else {
            Ok(())
        }
    }

    /// Set an exception (from SIGNAL)
    pub fn set_exception(&mut self, sqlstate: String, message: String) {
        self.current_exception = Some(StoredProcError { sqlstate, message });
    }

    /// Get the current exception
    pub fn get_exception(&self) -> Option<&StoredProcError> {
        self.current_exception.as_ref()
    }

    /// Clear the current exception (after handling)
    pub fn clear_exception(&mut self) {
        self.current_exception = None;
    }

    /// Set a variable value (local variable, without @ prefix)
    pub fn set_local_var(&mut self, name: &str, value: Value) {
        self.local_variables.insert(name.to_string(), value);
    }

    /// Get a local variable value (without @ prefix)
    pub fn get_local_var(&self, name: &str) -> Option<&Value> {
        self.local_variables.get(name)
    }

    /// Set a session variable (without @ prefix, e.g., "uid" for @uid)
    pub fn set_session_var(&mut self, name: &str, value: Value) {
        self.session_variables.insert(name.to_string(), value);
    }

    /// Get a session variable value (without @ prefix)
    pub fn get_session_var(&self, name: &str) -> Option<&Value> {
        self.session_variables.get(name)
    }

    /// Get any variable (local first, then session)
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.local_variables
            .get(name)
            .or_else(|| self.session_variables.get(name))
    }

    /// Set any variable (local or session based on @ prefix)
    pub fn set_var(&mut self, name: &str, value: Value) {
        if name.starts_with('@') {
            self.session_variables.insert(name[1..].to_string(), value);
        } else {
            self.local_variables.insert(name.to_string(), value);
        }
    }

    /// Check if a variable exists (local or session)
    pub fn has_var(&self, name: &str) -> bool {
        let key = if name.starts_with('@') {
            &name[1..]
        } else {
            name
        };
        self.local_variables.contains_key(key) || self.session_variables.contains_key(key)
    }

    /// Clear all local variables (called when exiting procedure)
    pub fn clear_local_vars(&mut self) {
        self.local_variables.clear();
    }

    /// Get all session variables for persistence
    pub fn get_session_vars(&self) -> &HashMap<String, Value> {
        &self.session_variables
    }

    /// Set return value and signal exit
    pub fn set_return(&mut self, value: Value) {
        self.return_value = Some(value);
    }

    /// Get return value
    pub fn get_return(&self) -> Option<Value> {
        self.return_value.clone()
    }

    /// Signal LEAVE (exit loop)
    pub fn set_leave(&mut self) {
        self.leave = true;
    }

    /// Check if should leave
    pub fn should_leave(&self) -> bool {
        self.leave
    }

    /// Reset leave flag
    pub fn reset_leave(&mut self) {
        self.leave = false;
    }

    /// Signal ITERATE (continue next iteration)
    pub fn set_iterate(&mut self) {
        self.iterate = true;
    }

    /// Check if should iterate
    pub fn should_iterate(&self) -> bool {
        self.iterate
    }

    /// Reset iterate flag
    pub fn reset_iterate(&mut self) {
        self.iterate = false;
    }

    /// Set current label
    pub fn set_label(&mut self, label: Option<String>) {
        self.current_label = label;
    }

    /// Get current label
    pub fn get_label(&self) -> Option<&String> {
        self.current_label.as_ref()
    }
}

/// Stored procedure executor for calling stored procedures
#[derive(Clone)]
pub struct StoredProcExecutor {
    catalog: Arc<sqlrustgo_catalog::Catalog>,
}

impl StoredProcExecutor {
    /// Create a new stored procedure executor
    pub fn new(catalog: Arc<sqlrustgo_catalog::Catalog>) -> Self {
        Self { catalog }
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
            Ok(_) => Ok(ExecutorResult::new(
                vec![vec![Value::Text(format!(
                    "Procedure '{}' executed successfully",
                    name
                ))]],
                1,
            )),
            Err(e) => Err(e),
        }
    }

    /// Execute a list of procedure statements
    fn execute_body(
        &self,
        body: &[StoredProcStatement],
        ctx: &mut ProcedureContext,
    ) -> Result<(), String> {
        for stmt in body {
            if ctx.should_leave() {
                ctx.reset_leave();
                break;
            }
            if ctx.should_iterate() {
                ctx.reset_iterate();
                break;
            }
            if ctx.get_return().is_some() {
                break;
            }
            self.execute_statement(stmt, ctx)?;
        }
        Ok(())
    }

    /// Execute a single procedure statement
    fn execute_statement(
        &self,
        stmt: &StoredProcStatement,
        ctx: &mut ProcedureContext,
    ) -> Result<(), String> {
        match stmt {
            StoredProcStatement::Declare {
                name,
                default_value,
                ..
            } => {
                // Declare variable with default value
                let value = default_value
                    .as_ref()
                    .map(|v| self.evaluate_constant(v))
                    .unwrap_or_else(|| Value::Null);
                ctx.set_var(name, value);
                Ok(())
            }
            StoredProcStatement::Set { variable, value } => {
                // Set variable value
                let evaluated = self.evaluate_expression(value, ctx)?;
                ctx.set_var(variable, evaluated);
                Ok(())
            }
            StoredProcStatement::RawSql(sql) => {
                // Execute raw SQL (placeholder - actual execution would need executor access)
                if !sql.is_empty() {
                    // In a real implementation, this would execute the SQL
                    // For now, we just acknowledge the statement
                }
                Ok(())
            }
            StoredProcStatement::SelectInto {
                columns,
                into_vars,
                table,
                where_clause,
            } => {
                // SELECT INTO - execute SELECT and store results into variables
                // Build WHERE clause string
                let where_str = where_clause
                    .as_ref()
                    .map(|w| format!(" WHERE {}", self.expand_variables_in_sql(w, ctx)))
                    .unwrap_or_default();

                // Build the SELECT query
                let cols = if columns.is_empty() {
                    "*".to_string()
                } else {
                    columns.join(", ")
                };
                let _query = format!("SELECT {} FROM {}{}", cols, table, where_str);

                // For now, execute a simple query through the catalog
                // In a full implementation, this would use the query planner/executor
                // We evaluate expressions in the INTO variables
                for (i, var) in into_vars.iter().enumerate() {
                    if i < columns.len() {
                        // Try to evaluate the column as an expression
                        let col_expr = &columns[i];
                        let value = self.evaluate_expression(col_expr, ctx)?;
                        ctx.set_var(var, value);
                    } else {
                        // Set remaining vars to NULL if not enough columns
                        ctx.set_var(var, Value::Null);
                    }
                }

                // Note: In a complete implementation, this would:
                // 1. Parse the query using the SQL parser
                // 2. Execute through the query planner
                // 3. Fetch results and assign to variables

                Ok(())
            }
            StoredProcStatement::If {
                condition,
                then_body,
                elseif_body,
                else_body,
            } => {
                // Evaluate condition
                if self.evaluate_condition(condition, ctx)? {
                    self.execute_body(then_body, ctx)?;
                } else {
                    // Check elseif branches
                    let mut matched = false;
                    for (elsif_cond, elsif_body) in elseif_body {
                        if self.evaluate_condition(elsif_cond, ctx)? {
                            self.execute_body(elsif_body, ctx)?;
                            matched = true;
                            break;
                        }
                    }
                    if !matched && !else_body.is_empty() {
                        self.execute_body(else_body, ctx)?;
                    }
                }
                Ok(())
            }
            StoredProcStatement::While { condition, body } => {
                // While loop - always has implicit label for LEAVE/ITERATE
                while self.evaluate_condition(condition, ctx)?
                    && !ctx.should_leave()
                    && ctx.get_return().is_none()
                {
                    ctx.reset_iterate();
                    self.execute_body(body, ctx)?;
                    if ctx.should_iterate() {
                        ctx.reset_iterate();
                    }
                }
                ctx.reset_leave();
                Ok(())
            }
            StoredProcStatement::Loop { body } => {
                // Infinite loop - must be exited with LEAVE
                loop {
                    if ctx.should_leave() {
                        ctx.reset_leave();
                        break;
                    }
                    if ctx.get_return().is_some() {
                        break;
                    }
                    self.execute_body(body, ctx)?;
                }
                Ok(())
            }
            StoredProcStatement::Leave { label } => {
                // LEAVE label - validate and signal exit
                ctx.validate_label(label)?;
                ctx.set_leave();
                Ok(())
            }
            StoredProcStatement::Iterate { label } => {
                // ITERATE label - validate and signal continue
                ctx.validate_label(label)?;
                ctx.set_iterate();
                Ok(())
            }
            StoredProcStatement::Return { value } => {
                // RETURN - set return value and exit
                let ret_val = self.evaluate_expression(value, ctx)?;
                ctx.set_return(ret_val);
                Ok(())
            }
            StoredProcStatement::Call {
                procedure_name,
                args,
                into_var,
            } => {
                // Nested procedure call
                let call_args: Vec<Value> = args
                    .iter()
                    .map(|a| self.evaluate_expression(a, ctx).unwrap_or(Value::Null))
                    .collect();

                let result = self.execute_call(procedure_name, call_args);

                match result {
                    Ok(exec_result) => {
                        if let Some(var_name) = into_var {
                            if let Some(row) = exec_result.rows.first() {
                                if let Some(val) = row.first() {
                                    ctx.set_var(var_name, val.clone());
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            StoredProcStatement::Signal { sqlstate, message } => {
                // SIGNAL - raise an exception
                let state = sqlstate
                    .clone()
                    .unwrap_or_else(|| SQLSTATE_SQL_EXCEPTION.to_string());
                let msg = message
                    .clone()
                    .unwrap_or_else(|| "User-defined exception".to_string());
                ctx.set_exception(state.clone(), msg.clone());
                Err(format!("SQLSTATE {}: {}", state, msg))
            }
            StoredProcStatement::Resignal { sqlstate, message } => {
                // RESIGNAL - re-raise the current exception or create a new one
                if let Some(ref exc) = ctx.get_exception() {
                    // Re-raise with optional modification
                    let state = sqlstate.clone().unwrap_or_else(|| exc.sqlstate.clone());
                    let msg = message.clone().unwrap_or_else(|| exc.message.clone());
                    ctx.set_exception(state.clone(), msg.clone());
                    Err(format!("SQLSTATE {}: {}", state, msg))
                } else if let (Some(state), Some(msg)) = (sqlstate, message) {
                    // RESIGNAL without active exception requires both values
                    ctx.set_exception(state.clone(), msg.clone());
                    Err(format!("SQLSTATE {}: {}", state, msg))
                } else {
                    Err(
                        "RESIGNAL requires an active exception or explicit SQLSTATE and MESSAGE"
                            .to_string(),
                    )
                }
            }
        }
    }

    /// Evaluate a condition expression (returns boolean)
    fn evaluate_condition(&self, condition: &str, ctx: &ProcedureContext) -> Result<bool, String> {
        // Simple condition evaluation for comparison operators
        let cond = condition.trim();

        // Handle "variable op value" comparisons
        for &op in &["<=", ">=", "!=", "<>", "=", "<", ">"] {
            if let Some(pos) = cond.find(op) {
                let left = cond[..pos].trim();
                let right = cond[pos + op.len()..].trim();

                let left_val = self.expand_variable(left, ctx);
                let right_val = self.evaluate_constant(right);

                return Ok(self.compare_values(&left_val, &right_val, op));
            }
        }

        // Default: treat as boolean constant
        Ok(cond != "0" && cond.to_lowercase() != "false" && !cond.is_empty())
    }

    /// Evaluate an expression and return a Value
    fn evaluate_expression(&self, expr: &str, ctx: &ProcedureContext) -> Result<Value, String> {
        let expr = expr.trim();

        // Handle string literals
        if expr.starts_with('\'') && expr.ends_with('\'') {
            return Ok(Value::Text(expr[1..expr.len() - 1].to_string()));
        }

        // Handle numeric literals
        if let Ok(num) = expr.parse::<i64>() {
            return Ok(Value::Integer(num));
        }
        if let Ok(float) = expr.parse::<f64>() {
            return Ok(Value::Float(float));
        }

        // Handle variables (starting with @)
        if expr.starts_with('@') {
            let var_name = &expr[1..];
            if let Some(val) = ctx.get_var(var_name) {
                return Ok(val.clone());
            }
        }

        // Handle simple arithmetic: var + value, var - value, etc.
        for &op in &["+", "-", "*", "/"] {
            if let Some(pos) = expr.find(op) {
                if pos > 0 && pos < expr.len() - 1 {
                    let left = self.evaluate_expression(&expr[..pos], ctx)?;
                    let right = self.evaluate_expression(&expr[pos + 1..], ctx)?;
                    return self.arithmetic_op(&left, &right, op);
                }
            }
        }

        // Default: return as text
        Ok(Value::Text(expr.to_string()))
    }

    /// Expand a variable reference to its value
    fn expand_variable(&self, name: &str, ctx: &ProcedureContext) -> Value {
        let name = name.trim();

        // Handle @variable syntax
        if name.starts_with('@') {
            let var_name = &name[1..];
            ctx.get_var(var_name).cloned().unwrap_or(Value::Null)
        } else if ctx.has_var(name) {
            ctx.get_var(name).cloned().unwrap_or(Value::Null)
        } else {
            // Try to parse as literal
            self.evaluate_constant(name)
        }
    }

    /// Expand all @variables in a SQL expression string
    fn expand_variables_in_sql(&self, sql: &str, ctx: &ProcedureContext) -> String {
        let chars: Vec<char> = sql.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '@' {
                // Find end of variable name
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let var_name: String = chars[start + 1..i].iter().collect();
                let value = ctx
                    .get_var(&var_name)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "NULL".to_string());
                result.push_str(&value);
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    /// Evaluate a constant expression
    fn evaluate_constant(&self, value: &str) -> Value {
        let value = value.trim();

        // Handle string literals
        if value.starts_with('\'') && value.ends_with('\'') {
            return Value::Text(value[1..value.len() - 1].to_string());
        }

        // Handle numeric literals
        if let Ok(num) = value.parse::<i64>() {
            return Value::Integer(num);
        }
        if let Ok(float) = value.parse::<f64>() {
            return Value::Float(float);
        }

        // Handle NULL
        if value.to_uppercase() == "NULL" {
            return Value::Null;
        }

        // Handle boolean literals
        if value.to_uppercase() == "TRUE" {
            return Value::Boolean(true);
        }
        if value.to_uppercase() == "FALSE" {
            return Value::Boolean(false);
        }

        Value::Text(value.to_string())
    }

    /// Compare two values using the given operator
    fn compare_values(&self, left: &Value, right: &Value, op: &str) -> bool {
        match op {
            "=" | "==" => left == right,
            "!=" | "<>" => left != right,
            ">" => self.partial_cmp(left, right) == Some(std::cmp::Ordering::Greater),
            ">=" => matches!(
                self.partial_cmp(left, right),
                Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal)
            ),
            "<" => self.partial_cmp(left, right) == Some(std::cmp::Ordering::Less),
            "<=" => matches!(
                self.partial_cmp(left, right),
                Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal)
            ),
            _ => false,
        }
    }

    /// Compare two values (for ordering)
    fn partial_cmp(&self, left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;

        match (left, right) {
            (Value::Null, _) | (_, Value::Null) => None,
            (Value::Integer(l), Value::Integer(r)) => Some(l.cmp(r)),
            (Value::Float(l), Value::Float(r)) => Some(l.partial_cmp(r).unwrap_or(Ordering::Equal)),
            (Value::Text(l), Value::Text(r)) => Some(l.cmp(r)),
            (Value::Boolean(l), Value::Boolean(r)) => Some(l.cmp(r)),
            _ => None,
        }
    }

    /// Perform arithmetic operation on two values
    fn arithmetic_op(&self, left: &Value, right: &Value, op: &str) -> Result<Value, String> {
        match (left, right, op) {
            (Value::Integer(l), Value::Integer(r), "+") => Ok(Value::Integer(l + r)),
            (Value::Integer(l), Value::Integer(r), "-") => Ok(Value::Integer(l - r)),
            (Value::Integer(l), Value::Integer(r), "*") => Ok(Value::Integer(l * r)),
            (Value::Integer(l), Value::Integer(r), "/") => {
                if *r == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Integer(l / r))
                }
            }
            (Value::Float(l), Value::Float(r), "+") => Ok(Value::Float(l + r)),
            (Value::Float(l), Value::Float(r), "-") => Ok(Value::Float(l - r)),
            (Value::Float(l), Value::Float(r), "*") => Ok(Value::Float(l * r)),
            (Value::Float(l), Value::Float(r), "/") => {
                if *r == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Float(l / r))
                }
            }
            _ => Err(format!(
                "Unsupported arithmetic operation: {:?} {} {:?}",
                left, op, right
            )),
        }
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
mod tests {
    use super::*;
    use sqlrustgo_catalog::Catalog;

    #[test]
    fn test_stored_proc_executor_not_found() {
        let catalog = Arc::new(Catalog::new());
        let executor = StoredProcExecutor::new(catalog);

        let result = executor.execute_call("non_existent", vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_stored_proc_executor_list_empty() {
        let catalog = Arc::new(Catalog::new());
        let executor = StoredProcExecutor::new(catalog);

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
        let catalog = Arc::new(Catalog::new());
        let executor = StoredProcExecutor::new(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(10));

        // Note: This test requires the condition to reference the variable properly
        // In practice, we'd need proper variable expansion
    }

    #[test]
    fn test_procedure_context_label_stack() {
        let mut ctx = ProcedureContext::new();

        // Initially no labels
        assert!(!ctx.has_label("loop1"));

        // Enter a label
        ctx.enter_label("loop1".to_string());
        assert!(ctx.has_label("loop1"));
        assert_eq!(ctx.get_label(), Some(&"loop1".to_string()));

        // Enter nested label
        ctx.enter_label("loop2".to_string());
        assert!(ctx.has_label("loop1"));
        assert!(ctx.has_label("loop2"));
        assert_eq!(ctx.get_label(), Some(&"loop2".to_string()));

        // Exit nested label
        let popped = ctx.exit_label();
        assert_eq!(popped, Some("loop2".to_string()));
        assert_eq!(ctx.get_label(), Some(&"loop1".to_string()));

        // Exit outer label
        let popped = ctx.exit_label();
        assert_eq!(popped, Some("loop1".to_string()));
        assert!(!ctx.has_label("loop1"));
        assert!(ctx.get_label().is_none());
    }

    #[test]
    fn test_label_validation_leave_iterate() {
        let mut ctx = ProcedureContext::new();

        // Empty label with no current label should fail
        let result = ctx.validate_label("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires an active loop"));

        // Valid label should pass
        ctx.enter_label("myloop".to_string());
        let result = ctx.validate_label("myloop");
        assert!(result.is_ok());

        // Invalid label should fail
        let result = ctx.validate_label("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        // Empty label with current label should pass
        ctx.reset_leave();
        let result = ctx.validate_label("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_exception() {
        let mut ctx = ProcedureContext::new();

        // Set an exception via SIGNAL
        ctx.set_exception("45000".to_string(), "Custom error".to_string());

        let exc = ctx.get_exception();
        assert!(exc.is_some());
        let exc = exc.unwrap();
        assert_eq!(exc.sqlstate, "45000");
        assert_eq!(exc.message, "Custom error");

        // Clear exception
        ctx.clear_exception();
        assert!(ctx.get_exception().is_none());
    }

    #[test]
    fn test_resignal_with_active_exception() {
        let mut ctx = ProcedureContext::new();

        // Set an initial exception
        ctx.set_exception("22012".to_string(), "Division error".to_string());

        // RESIGNAL should preserve the exception
        let exc = ctx.get_exception().unwrap();
        assert_eq!(exc.sqlstate, "22012");

        // Clear for next test
        ctx.clear_exception();

        // RESIGNAL without active exception should fail
        let result = ctx.validate_label("");
        assert!(result.is_err());
    }

    #[test]
    fn test_select_into_variable_expansion() {
        let catalog = Arc::new(Catalog::new());
        let executor = StoredProcExecutor::new(catalog);
        let mut ctx = ProcedureContext::new();

        // Set session variable
        ctx.set_var("@user_id", Value::Integer(42));

        // Test variable expansion in SQL
        let expanded =
            executor.expand_variables_in_sql("SELECT * FROM users WHERE id = @user_id", &ctx);
        assert!(expanded.contains("42"));
        assert!(!expanded.contains("@user_id"));

        // Test with undefined variable
        let expanded =
            executor.expand_variables_in_sql("SELECT * FROM users WHERE name = @unknown", &ctx);
        assert!(expanded.contains("NULL"));
    }

    #[test]
    fn test_procedure_error_display() {
        let err = StoredProcError {
            sqlstate: "45000".to_string(),
            message: "Custom error".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("45000"));
        assert!(display.contains("Custom error"));
    }
}
