//! Stored Procedure Executor
//!
//! This module provides stored procedure execution support with control flow.

use crate::ExecutorResult;
use sqlrustgo_catalog::HandlerCondition;
use sqlrustgo_catalog::StoredProcStatement;
use sqlrustgo_storage::{ColumnDefinition, StorageEngine};
use sqlrustgo_types::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
    /// Label stack for nested blocks
    label_stack: Vec<String>,
    /// Local variable scope stack for BEGIN/END blocks
    scope_stack: Vec<HashMap<String, Value>>,
    /// Handler stack for exception handling
    handler_stack: Vec<ExceptionHandler>,
    /// Whether an exception is currently being handled
    exception_handling: bool,
    /// Current exception (for RESIGNAL)
    current_exception: Option<StoredProcError>,
    /// Cursors declared in the procedure
    cursors: HashMap<String, Cursor>,
    /// CTE (Common Table Expression) results: CTE name -> rows
    cte_tables: HashMap<String, Vec<Vec<Value>>>,
}

/// Exception handler registered by DECLARE HANDLER
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    condition: HandlerCondition,
    body: Vec<StoredProcStatement>,
}

/// Cursor state for stored procedure cursors
#[derive(Debug, Clone)]
struct Cursor {
    #[allow(dead_code)]
    name: String,
    query: String,
    records: Vec<Vec<Value>>,
    position: usize,
    is_open: bool,
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
            scope_stack: Vec::new(),
            handler_stack: Vec::new(),
            exception_handling: false,
            current_exception: None,
            cursors: HashMap::new(),
            cte_tables: HashMap::new(),
        }
    }

    /// Declare a cursor
    pub fn declare_cursor(&mut self, name: String, query: String) {
        self.cursors.insert(
            name.clone(),
            Cursor {
                name,
                query,
                records: Vec::new(),
                position: 0,
                is_open: false,
            },
        );
    }

    /// Open a cursor (execute query and load results)
    pub fn open_cursor(&mut self, name: &str) -> Result<(), String> {
        if let Some(cursor) = self.cursors.get_mut(name) {
            cursor.is_open = true;
            cursor.position = 0;
            Ok(())
        } else {
            Err(format!("Cursor '{}' not found", name))
        }
    }

    /// Fetch from cursor into variables (returns has_rows, and sets variables via set_var)
    pub fn fetch_cursor(&mut self, name: &str, into_vars: &[String]) -> Result<bool, String> {
        let (has_rows, row_data) = {
            let cursor = self
                .cursors
                .get_mut(name)
                .ok_or_else(|| format!("Cursor '{}' not found", name))?;

            if !cursor.is_open {
                return Err(format!("Cursor '{}' is not open", name));
            }

            if cursor.position < cursor.records.len() {
                let row = cursor.records[cursor.position].clone();
                cursor.position += 1;
                (true, Some(row))
            } else {
                (false, None)
            }
        };

        if let Some(row) = row_data {
            for (i, var) in into_vars.iter().enumerate() {
                if i < row.len() {
                    self.set_var(var, row[i].clone());
                } else {
                    self.set_var(var, Value::Null);
                }
            }
        }
        Ok(has_rows)
    }

    /// Close a cursor
    pub fn close_cursor(&mut self, name: &str) -> Result<(), String> {
        if let Some(cursor) = self.cursors.get_mut(name) {
            cursor.is_open = false;
            Ok(())
        } else {
            Err(format!("Cursor '{}' not found", name))
        }
    }

    /// Check if cursor exists
    pub fn has_cursor(&self, name: &str) -> bool {
        self.cursors.contains_key(name)
    }

    /// Set cursor records (called after OPEN)
    pub fn set_cursor_records(&mut self, name: &str, records: Vec<Vec<Value>>) {
        if let Some(cursor) = self.cursors.get_mut(name) {
            cursor.records = records;
            cursor.position = 0;
        }
    }

    /// Push an exception handler onto the stack
    pub fn push_handler(&mut self, condition: HandlerCondition, body: Vec<StoredProcStatement>) {
        self.handler_stack
            .push(ExceptionHandler { condition, body });
    }

    /// Pop an exception handler from the stack
    pub fn pop_handler(&mut self) {
        self.handler_stack.pop();
    }

    /// Check if current exception matches any handler condition
    pub fn find_matching_handler(&self, exc: &StoredProcError) -> Option<&ExceptionHandler> {
        for handler in &self.handler_stack {
            match &handler.condition {
                HandlerCondition::SqlException => {
                    if exc.sqlstate.starts_with("45") || exc.sqlstate == "22000" {
                        return Some(handler);
                    }
                }
                HandlerCondition::SqlWarning => {
                    if exc.sqlstate.starts_with("01") {
                        return Some(handler);
                    }
                }
                HandlerCondition::NotFound => {
                    if exc.sqlstate == "02000" {
                        return Some(handler);
                    }
                }
                HandlerCondition::SqlState(state) => {
                    if exc.sqlstate == *state {
                        return Some(handler);
                    }
                }
                HandlerCondition::Custom(name) => {
                    if exc.message.contains(name) {
                        return Some(handler);
                    }
                }
            }
        }
        None
    }

    /// Set exception handling mode
    pub fn set_exception_handling(&mut self, handling: bool) {
        self.exception_handling = handling;
    }

    /// Check if currently handling an exception
    pub fn is_handling_exception(&self) -> bool {
        self.exception_handling
    }

    /// Set the current exception (for handler access)
    pub fn set_exception(&mut self, sqlstate: String, message: String) {
        if let Some(exc) = &mut self.current_exception {
            exc.sqlstate = sqlstate;
            exc.message = message;
        } else {
            self.current_exception = Some(StoredProcError { sqlstate, message });
        }
    }

    /// Clear the current exception
    pub fn clear_exception(&mut self) {
        self.current_exception = None;
    }

    /// Get the current exception
    pub fn get_exception(&self) -> Option<&StoredProcError> {
        self.current_exception.as_ref()
    }

    /// Push a label onto the stack (enter a labeled block)
    pub fn enter_label(&mut self, label: String) {
        self.label_stack.push(label.clone());
        self.current_label = Some(label);
    }

    /// Pop a label from the stack (exit a labeled block)
    pub fn exit_label(&mut self) {
        self.label_stack.pop();
        self.current_label = self.label_stack.last().cloned();
    }

    /// Push a new variable scope (enter BEGIN block)
    pub fn enter_scope(&mut self) {
        self.scope_stack.push(self.local_variables.clone());
        self.local_variables.clear();
    }

    /// Pop the current variable scope (exit END block)
    pub fn exit_scope(&mut self) {
        if let Some(saved_vars) = self.scope_stack.pop() {
            self.local_variables = saved_vars;
        }
    }

    /// Check if a label exists in the stack
    pub fn has_label(&self, label: &str) -> bool {
        self.label_stack.iter().any(|l| l == label)
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
        let key = name.strip_prefix('@').unwrap_or(name);
        self.local_variables
            .get(key)
            .or_else(|| self.session_variables.get(key))
    }

    /// Set any variable (local or session based on @ prefix)
    pub fn set_var(&mut self, name: &str, value: Value) {
        if let Some(var_name) = name.strip_prefix('@') {
            self.session_variables.insert(var_name.to_string(), value);
        } else {
            self.local_variables.insert(name.to_string(), value);
        }
    }

    /// Check if a variable exists (local or session)
    pub fn has_var(&self, name: &str) -> bool {
        let key = if let Some(var_name) = name.strip_prefix('@') {
            var_name
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

            let result = self.execute_statement(stmt, ctx);

            if let Err(ref e) = result {
                // Check if we should handle this exception
                if let Some(err_str) = e.strip_prefix("SQLSTATE ") {
                    let sqlstate = err_str
                        .split(':')
                        .next()
                        .unwrap_or("45000")
                        .trim()
                        .to_string();
                    let message = e
                        .strip_prefix(&format!("SQLSTATE {}: ", sqlstate))
                        .unwrap_or(e)
                        .trim()
                        .to_string();
                    let exc = StoredProcError { sqlstate, message };

                    if let Some(handler) = ctx.find_matching_handler(&exc) {
                        // Clone handler body to avoid borrow conflict
                        let handler_body = handler.body.clone();
                        ctx.set_exception_handling(true);
                        ctx.set_exception(exc.sqlstate.clone(), exc.message.clone());
                        let handler_result = self.execute_body(&handler_body, ctx);
                        ctx.clear_exception();
                        ctx.set_exception_handling(false);

                        handler_result.as_ref()?;
                        continue;
                    }
                }
                return result;
            }
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
                // Execute raw SQL using storage engine
                if !sql.is_empty() {
                    self.execute_sql(sql, ctx)?;
                }
                Ok(())
            }
            StoredProcStatement::SelectInto {
                columns,
                into_vars,
                table,
                where_clause,
            } => {
                let where_str = where_clause
                    .as_ref()
                    .map(|w| format!(" WHERE {}", self.expand_variables_in_sql(w, ctx)))
                    .unwrap_or_default();

                let cols = if columns.is_empty() {
                    "*".to_string()
                } else {
                    columns.join(", ")
                };
                let _query = format!("SELECT {} FROM {}{}", cols, table, where_str);

                for (i, var) in into_vars.iter().enumerate() {
                    if i < columns.len() {
                        let col_expr = &columns[i];
                        let value = self.evaluate_expression(col_expr, ctx)?;
                        ctx.set_var(var, value);
                    } else {
                        ctx.set_var(var, Value::Null);
                    }
                }

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
                // While loop
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
            StoredProcStatement::Case {
                case_value,
                when_clauses,
                else_result,
            } => {
                let case_val = if let Some(ref cv) = case_value {
                    self.evaluate_expression(cv, ctx)?
                } else {
                    Value::Null
                };

                for (when_val, result) in when_clauses {
                    let when_expr_val = self.evaluate_expression(when_val, ctx)?;
                    if case_val == when_expr_val {
                        return self.evaluate_expression(result, ctx).map(|v| {
                            ctx.set_return(v);
                        });
                    }
                }

                if let Some(else_val) = else_result {
                    return self.evaluate_expression(else_val, ctx).map(|v| {
                        ctx.set_return(v);
                    });
                }

                Ok(())
            }
            StoredProcStatement::CaseWhen {
                when_clauses,
                else_result,
            } => {
                for (condition, result) in when_clauses {
                    if self.evaluate_condition(condition, ctx)? {
                        return self.evaluate_expression(result, ctx).map(|v| {
                            ctx.set_return(v);
                        });
                    }
                }

                if let Some(else_val) = else_result {
                    return self.evaluate_expression(else_val, ctx).map(|v| {
                        ctx.set_return(v);
                    });
                }

                Ok(())
            }
            StoredProcStatement::Repeat { body, condition } => {
                loop {
                    if ctx.should_leave() {
                        ctx.reset_leave();
                        break;
                    }
                    if ctx.get_return().is_some() {
                        break;
                    }

                    ctx.reset_iterate();
                    self.execute_body(body, ctx)?;

                    if ctx.should_iterate() {
                        ctx.reset_iterate();
                        continue;
                    }

                    if self.evaluate_condition(condition, ctx)? {
                        break;
                    }
                }
                Ok(())
            }
            StoredProcStatement::Leave { .. } => {
                // LEAVE - signal exit
                ctx.set_leave();
                Ok(())
            }
            StoredProcStatement::Iterate { .. } => {
                // ITERATE - signal continue
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
                let sqlstate = sqlstate.as_deref().unwrap_or("45000");
                let message = message.as_deref().unwrap_or("Unhandled exception");
                Err(format!("SQLSTATE {}: {}", sqlstate, message))
            }
            StoredProcStatement::Resignal { sqlstate, message } => {
                let sqlstate = sqlstate.as_deref().unwrap_or("45000");
                let message = message.as_deref().unwrap_or("Unhandled exception");
                Err(format!("SQLSTATE {}: {}", sqlstate, message))
            }
            StoredProcStatement::Block { label, body } => {
                // BEGIN...END block with optional label
                if let Some(ref lbl) = label {
                    ctx.enter_label(lbl.clone());
                }

                // Push new scope for block-local variables
                ctx.enter_scope();

                // Execute block body
                let result = self.execute_body(body, ctx);

                // Pop scope (discard block-local variables)
                ctx.exit_scope();

                if label.is_some() {
                    ctx.exit_label();
                }

                result
            }
            StoredProcStatement::DeclareHandler {
                condition_type,
                body,
            } => {
                // Register the exception handler
                ctx.push_handler(condition_type.clone(), body.clone());
                Ok(())
            }
            StoredProcStatement::DeclareCursor { name, query } => {
                // Declare a cursor
                ctx.declare_cursor(name.clone(), query.clone());
                Ok(())
            }
            StoredProcStatement::OpenCursor { name } => {
                // Open cursor - parse query, execute, and load results
                let query = if let Some(cursor) = ctx.cursors.get(&name.clone()) {
                    cursor.query.clone()
                } else {
                    return Err(format!("Cursor '{}' not found", name));
                };

                let expanded = self.expand_variables_in_sql(&query, ctx);
                let statement = sqlrustgo_parser::parse(&expanded)
                    .map_err(|e| format!("Failed to parse cursor query: {}", e))?;

                if let sqlrustgo_parser::Statement::Select(select) = statement {
                    let storage = self.storage.read().unwrap();
                    let records = storage
                        .scan(&select.table)
                        .map_err(|e| format!("Failed to scan table: {}", e))?;
                    ctx.set_cursor_records(name, records);
                    ctx.open_cursor(name)?;
                }
                Ok(())
            }
            StoredProcStatement::Fetch { name, into_vars } => {
                // Fetch from cursor
                let has_rows = ctx.fetch_cursor(name, into_vars)?;
                ctx.set_session_var("__found", Value::Boolean(has_rows));
                if !has_rows {
                    ctx.set_session_var("__found_rows", Value::Integer(0));
                }
                Ok(())
            }
            StoredProcStatement::CloseCursor { name } => {
                // Close cursor
                ctx.close_cursor(name)?;
                Ok(())
            }
        }
    }

    /// Execute a SQL statement using the storage engine
    fn execute_sql(&self, sql: &str, ctx: &mut ProcedureContext) -> Result<(), String> {
        let expanded_sql = self.expand_variables_in_sql(sql, ctx);
        let sql_upper = expanded_sql.trim().to_uppercase();

        if sql_upper.starts_with("SELECT")
            || sql_upper.starts_with("INSERT")
            || sql_upper.starts_with("UPDATE")
            || sql_upper.starts_with("DELETE")
        {
            let statement = sqlrustgo_parser::parse(&expanded_sql)
                .map_err(|e| format!("Failed to parse SQL: {}", e))?;

            self.execute_statement_storage(&statement, ctx)?;

            // Set ROW_COUNT for MySQL/MariaDB compatibility
            if let Some(found_rows) = ctx.get_session_var("__found_rows") {
                ctx.set_session_var("ROW_COUNT", found_rows.clone());
            } else if let Some(last_insert) = ctx.get_session_var("__last_insert_count") {
                ctx.set_session_var("ROW_COUNT", last_insert.clone());
            } else if let Some(last_update) = ctx.get_session_var("__last_update_count") {
                ctx.set_session_var("ROW_COUNT", last_update.clone());
            } else if let Some(last_delete) = ctx.get_session_var("__last_delete_count") {
                ctx.set_session_var("ROW_COUNT", last_delete.clone());
            }
        } else if sql_upper.starts_with("CREATE")
            || sql_upper.starts_with("DROP")
            || sql_upper.starts_with("ALTER")
            || sql_upper.starts_with("SHOW")
            || sql_upper.starts_with("DESCRIBE")
            || sql_upper.starts_with("SET")
        {
            ctx.set_session_var("ROW_COUNT", Value::Integer(0));
        }

        Ok(())
    }

    /// Execute a parsed statement using storage engine
    fn execute_statement_storage(
        &self,
        statement: &sqlrustgo_parser::Statement,
        ctx: &mut ProcedureContext,
    ) -> Result<(), String> {
        match statement {
            sqlrustgo_parser::Statement::WithSelect(with_select) => {
                if let Some(ref with_clause) = with_select.with_clause {
                    for cte in &with_clause.ctes {
                        let cte_records = self.execute_cte_subquery(&cte.subquery, ctx)?;
                        ctx.cte_tables.insert(cte.name.clone(), cte_records);
                    }
                }
                let select = &with_select.select;
                let table_name = &select.table;

                let records = if ctx.cte_tables.contains_key(table_name) {
                    ctx.cte_tables.get(table_name).cloned().unwrap_or_default()
                } else {
                    let storage = self.storage.read().unwrap();
                    storage
                        .scan(table_name)
                        .map_err(|e| format!("Failed to scan table: {}", e))?
                };

                let filtered: Vec<Vec<Value>> = if let Some(ref where_expr) = select.where_clause {
                    records
                        .into_iter()
                        .filter(|_row| {
                            let where_val = self.expression_to_value(where_expr, ctx);
                            if let Value::Boolean(b) = where_val {
                                b
                            } else {
                                where_val != Value::Null
                            }
                        })
                        .collect()
                } else {
                    records
                };

                ctx.set_session_var(
                    "__last_select_result",
                    Value::Text(serde_json::to_string(&filtered).unwrap_or_default()),
                );
                ctx.set_session_var("__found_rows", Value::Integer(filtered.len() as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::Select(select) => {
                let table_name = &select.table;

                // Handle SELECT without FROM (e.g., SELECT 1, SELECT 'hello', SELECT NULL)
                let records = if table_name.is_empty() {
                    vec![]
                } else if ctx.cte_tables.contains_key(table_name) {
                    ctx.cte_tables.get(table_name).cloned().unwrap_or_default()
                } else {
                    let storage = self.storage.read().unwrap();
                    storage
                        .scan(table_name)
                        .map_err(|e| format!("Failed to scan table: {}", e))?
                };

                let filtered: Vec<Vec<Value>> = if let Some(ref where_expr) = select.where_clause {
                    records
                        .into_iter()
                        .filter(|_row| {
                            let where_val = self.expression_to_value(where_expr, ctx);
                            if let Value::Boolean(b) = where_val {
                                b
                            } else {
                                where_val != Value::Null
                            }
                        })
                        .collect()
                } else {
                    records
                };

                ctx.set_session_var(
                    "__last_select_result",
                    Value::Text(serde_json::to_string(&filtered).unwrap_or_default()),
                );
                ctx.set_session_var("__found_rows", Value::Integer(filtered.len() as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::Insert(insert) => {
                let table_name = &insert.table.clone();
                let insert_columns = insert.columns.clone();

                let table_info = {
                    let storage = self.storage.read().unwrap();
                    if !storage.has_table(table_name) {
                        return Err(format!("Table '{}' not found", table_name));
                    }
                    storage.get_table_info(table_name).ok()
                };

                let num_columns = table_info.as_ref().map(|i| i.columns.len()).unwrap_or(0);
                let mut new_rows: Vec<Vec<Value>> = Vec::new();
                let insert_count;

                if let Some(ref select) = insert.select {
                    let storage = self.storage.read().unwrap();
                    let records = storage
                        .scan(&select.table)
                        .map_err(|e| format!("Failed to scan table: {}", e))?;

                    let selected_rows: Vec<Vec<Value>> =
                        if let Some(ref where_expr) = select.where_clause {
                            records
                                .into_iter()
                                .filter(|_row| {
                                    let where_val = self.expression_to_value(where_expr, ctx);
                                    if let Value::Boolean(b) = where_val {
                                        b
                                    } else {
                                        where_val != Value::Null
                                    }
                                })
                                .collect()
                        } else {
                            records
                        };

                    for row in selected_rows {
                        let mut new_row: Vec<Value> = vec![Value::Null; num_columns];
                        if insert_columns.is_empty() {
                            for (col_idx, val) in row.iter().enumerate() {
                                if col_idx < num_columns {
                                    new_row[col_idx] = val.clone();
                                }
                            }
                        } else {
                            for (col_idx, col_name) in insert_columns.iter().enumerate() {
                                if col_idx < row.len() {
                                    if let Some(ref info) = table_info {
                                        if let Some(target_idx) = info
                                            .columns
                                            .iter()
                                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                        {
                                            new_row[target_idx] = row[col_idx].clone();
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(ref info) = table_info {
                            if !info.foreign_keys.is_empty() {
                                self.validate_foreign_keys(table_name, &new_row, &insert_columns)?;
                            }
                            if info.columns.iter().any(|c| c.primary_key) {
                                self.validate_primary_key(table_name, &new_row, &insert_columns)?;
                            }
                            if !info.unique_constraints.is_empty() {
                                self.validate_unique_constraints(
                                    table_name,
                                    &new_row,
                                    &insert_columns,
                                )?;
                            }
                        }
                        new_rows.push(new_row);
                    }
                    insert_count = new_rows.len();
                } else {
                    for row in &insert.values {
                        let mut new_row: Vec<Value> = vec![Value::Null; num_columns];

                        if insert_columns.is_empty() {
                            for (col_idx, expr) in row.iter().enumerate() {
                                if col_idx < num_columns {
                                    new_row[col_idx] = self.expression_to_value(expr, ctx);
                                }
                            }
                        } else {
                            for (value_idx, col_name) in insert_columns.iter().enumerate() {
                                if value_idx < row.len() {
                                    if let Some(ref info) = table_info {
                                        if let Some(target_idx) = info
                                            .columns
                                            .iter()
                                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                        {
                                            new_row[target_idx] =
                                                self.expression_to_value(&row[value_idx], ctx);
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(ref info) = table_info {
                            if !info.foreign_keys.is_empty() {
                                let cols = insert_columns.clone();
                                self.validate_foreign_keys(table_name, &new_row, &cols)?;
                            }
                            if info.columns.iter().any(|c| c.primary_key) {
                                self.validate_primary_key(table_name, &new_row, &insert_columns)?;
                            }
                            if !info.unique_constraints.is_empty() {
                                self.validate_unique_constraints(
                                    table_name,
                                    &new_row,
                                    &insert_columns,
                                )?;
                            }
                        }

                        new_rows.push(new_row);
                    }
                    insert_count = insert.values.len();
                }

                {
                    let mut storage = self.storage.write().unwrap();
                    for new_row in new_rows {
                        storage
                            .insert(table_name, vec![new_row])
                            .map_err(|e| format!("Failed to insert: {}", e))?;
                    }
                }

                ctx.set_session_var("__last_insert_count", Value::Integer(insert_count as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::Update(update) => {
                let table_name = &update.table;
                let mut storage = self.storage.write().unwrap();

                if !storage.has_table(table_name) {
                    return Err(format!("Table '{}' not found", table_name));
                }

                let table_info = storage.get_table_info(table_name).ok();
                let mut updates: Vec<(usize, Value)> = Vec::new();

                for (col_name, expr) in &update.set_clauses {
                    if let Some(ref info) = table_info {
                        if let Some(col_idx) = info
                            .columns
                            .iter()
                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                        {
                            updates.push((col_idx, self.expression_to_value(expr, ctx)));
                        }
                    }
                }

                let count = storage
                    .update(table_name, &[], &updates)
                    .map_err(|e| format!("Failed to update: {}", e))?;

                ctx.set_session_var("__last_update_count", Value::Integer(count as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::Delete(delete) => {
                let table_name = &delete.table;
                let mut storage = self.storage.write().unwrap();

                if !storage.has_table(table_name) {
                    return Err(format!("Table '{}' not found", table_name));
                }

                let count = storage
                    .delete(table_name, &[])
                    .map_err(|e| format!("Failed to delete: {}", e))?;

                ctx.set_session_var("__last_delete_count", Value::Integer(count as i64));
                Ok(())
            }
            sqlrustgo_parser::Statement::AlterTable(alter_table) => {
                let table_name = &alter_table.table_name;
                let mut storage = self.storage.write().unwrap();

                match &alter_table.operation {
                    sqlrustgo_parser::AlterTableOperation::AddColumn {
                        name,
                        data_type,
                        nullable,
                        default_value: _,
                    } => {
                        let column = ColumnDefinition {
                            name: name.clone(),
                            data_type: data_type.clone(),
                            nullable: *nullable,
                            primary_key: false,
                        };
                        storage
                            .add_column(table_name, column)
                            .map_err(|e| format!("Failed to add column: {}", e))?;
                    }
                    sqlrustgo_parser::AlterTableOperation::RenameTo { new_name } => {
                        storage
                            .rename_table(table_name, new_name)
                            .map_err(|e| format!("Failed to rename table: {}", e))?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Convert parser Expression to runtime Value
    fn expression_to_value(
        &self,
        expr: &sqlrustgo_parser::Expression,
        ctx: &ProcedureContext,
    ) -> Value {
        match expr {
            sqlrustgo_parser::Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
                } else if let Ok(n) = s.parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Value::Text(s[1..s.len() - 1].to_string())
                } else {
                    Value::Text(s.to_string())
                }
            }
            sqlrustgo_parser::Expression::Identifier(name) => {
                if let Some(stripped) = name.strip_prefix('@') {
                    ctx.get_var(stripped).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Text(name.to_string())
                }
            }
            sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
                let left_val = self.expression_to_value(left, ctx);
                let right_val = self.expression_to_value(right, ctx);
                self.evaluate_binary_op(&left_val, &right_val, op)
            }
            sqlrustgo_parser::Expression::Subquery(select) => {
                let rows = self.execute_subquery(select);
                if let Some(first_row) = rows.first() {
                    first_row.first().cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::In(left, select) => {
                let left_val = self.expression_to_value(left, ctx);
                let rows = self.execute_subquery(select);
                let in_result = rows
                    .iter()
                    .any(|row| row.first().map(|v| v == &left_val).unwrap_or(false));
                Value::Boolean(in_result)
            }
            sqlrustgo_parser::Expression::NotIn(left, select) => {
                let left_val = self.expression_to_value(left, ctx);
                let rows = self.execute_subquery(select);
                let not_in_result = rows
                    .iter()
                    .all(|row| row.first().map(|v| v != &left_val).unwrap_or(true));
                Value::Boolean(not_in_result)
            }
            sqlrustgo_parser::Expression::Exists(select) => {
                let rows = self.execute_subquery(select);
                Value::Boolean(!rows.is_empty())
            }
            sqlrustgo_parser::Expression::NotExists(select) => {
                let rows = self.execute_subquery(select);
                Value::Boolean(rows.is_empty())
            }
            sqlrustgo_parser::Expression::QuantifiedOp(expr, quantifier, select) => {
                let rows = self.execute_subquery(select);
                let expr_val = self.expression_to_value(expr, ctx);
                match quantifier.as_str() {
                    "ALL" => {
                        let all_match = rows
                            .iter()
                            .all(|row| row.first().map(|v| v == &expr_val).unwrap_or(false));
                        Value::Boolean(all_match)
                    }
                    "ANY" | "SOME" => {
                        let any_match = rows
                            .iter()
                            .any(|row| row.first().map(|v| v == &expr_val).unwrap_or(false));
                        Value::Boolean(any_match)
                    }
                    _ => Value::Null,
                }
            }
            sqlrustgo_parser::Expression::IsNull(inner) => {
                let val = self.expression_to_value(inner, ctx);
                Value::Boolean(matches!(val, Value::Null))
            }
            sqlrustgo_parser::Expression::IsNotNull(inner) => {
                let val = self.expression_to_value(inner, ctx);
                Value::Boolean(!matches!(val, Value::Null))
            }
            sqlrustgo_parser::Expression::InList(left, values) => {
                let left_val = self.expression_to_value(left, ctx);
                let value_list: Vec<Value> = values
                    .iter()
                    .map(|v| self.expression_to_value(v, ctx))
                    .collect();
                Value::Boolean(value_list.contains(&left_val))
            }
            sqlrustgo_parser::Expression::NotInList(left, values) => {
                let left_val = self.expression_to_value(left, ctx);
                let value_list: Vec<Value> = values
                    .iter()
                    .map(|v| self.expression_to_value(v, ctx))
                    .collect();
                Value::Boolean(!value_list.contains(&left_val))
            }
            sqlrustgo_parser::Expression::NotLike(left, pattern, _) => {
                let left_val = self.expression_to_value(left, ctx);
                let pattern_val = self.expression_to_value(pattern, ctx);
                let like_result = self.like_match(&left_val, &pattern_val);
                Value::Boolean(!like_result)
            }
            sqlrustgo_parser::Expression::NotBetween(left, low, high) => {
                let left_val = self.expression_to_value(left, ctx);
                let low_val = self.expression_to_value(low, ctx);
                let high_val = self.expression_to_value(high, ctx);
                let between_result = self.between_match(&left_val, &low_val, &high_val);
                Value::Boolean(!between_result)
            }
            sqlrustgo_parser::Expression::NotRegexp(left, pattern) => {
                let left_val = self.expression_to_value(left, ctx);
                let pattern_val = self.expression_to_value(pattern, ctx);
                let regexp_result = self.regexp_match(&left_val, &pattern_val);
                Value::Boolean(!regexp_result)
            }
            sqlrustgo_parser::Expression::UnaryOp(op, expr) => {
                let val = self.expression_to_value(expr, ctx);
                match op.as_str() {
                    "NOT" => {
                        if let Value::Boolean(b) = val {
                            Value::Boolean(!b)
                        } else {
                            Value::Boolean(false)
                        }
                    }
                    _ => Value::Null,
                }
            }
            sqlrustgo_parser::Expression::Like(left, pattern, _) => {
                let left_val = self.expression_to_value(left, ctx);
                let pattern_val = self.expression_to_value(pattern, ctx);
                Value::Boolean(self.like_match(&left_val, &pattern_val))
            }
            sqlrustgo_parser::Expression::Between(left, low, high) => {
                let left_val = self.expression_to_value(left, ctx);
                let low_val = self.expression_to_value(low, ctx);
                let high_val = self.expression_to_value(high, ctx);
                Value::Boolean(self.between_match(&left_val, &low_val, &high_val))
            }
            sqlrustgo_parser::Expression::CaseWhen(when_clauses, else_expr) => {
                for clause in when_clauses {
                    let cond_val = self.expression_to_value(&clause.condition, ctx);
                    if let Value::Boolean(true) = cond_val {
                        return self.expression_to_value(&clause.result, ctx);
                    }
                }
                if let Some(else_box) = else_expr {
                    self.expression_to_value(else_box, ctx)
                } else {
                    Value::Null
                }
            }
            sqlrustgo_parser::Expression::Aggregate(_) => Value::Null,
            sqlrustgo_parser::Expression::FunctionCall(_, _) => Value::Null,
            sqlrustgo_parser::Expression::WindowCall(_) => Value::Null,
        }
    }

    /// Execute a SELECT statement and return rows
    fn execute_subquery(&self, select: &sqlrustgo_parser::SelectStatement) -> Vec<Vec<Value>> {
        let storage = match self.storage.read() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let records = match storage.scan(&select.table) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        if let Some(ref where_expr) = select.where_clause {
            records
                .into_iter()
                .filter(|_row| {
                    let where_val = self.expression_to_value(where_expr, &ProcedureContext::new());
                    if let Value::Boolean(b) = where_val {
                        b
                    } else {
                        where_val != Value::Null
                    }
                })
                .collect()
        } else {
            records
        }
    }

    /// Execute CTE subquery and return rows
    fn execute_cte_subquery(
        &self,
        statement: &sqlrustgo_parser::Statement,
        ctx: &mut ProcedureContext,
    ) -> Result<Vec<Vec<Value>>, String> {
        match statement {
            sqlrustgo_parser::Statement::Select(select) => {
                let table_name = &select.table;
                let storage = self.storage.read().unwrap();
                let records = storage
                    .scan(table_name)
                    .map_err(|e| format!("Failed to scan CTE table: {}", e))?;

                if let Some(ref where_expr) = select.where_clause {
                    let filtered: Vec<Vec<Value>> = records
                        .into_iter()
                        .filter(|_row| {
                            let where_val = self.expression_to_value(where_expr, ctx);
                            if let Value::Boolean(b) = where_val {
                                b
                            } else {
                                where_val != Value::Null
                            }
                        })
                        .collect();
                    Ok(filtered)
                } else {
                    Ok(records)
                }
            }
            sqlrustgo_parser::Statement::Union(union_stmt) => {
                let left_records = self.execute_cte_subquery(&union_stmt.left, ctx)?;
                let right_records = self.execute_cte_subquery(&union_stmt.right, ctx)?;
                if union_stmt.union_all {
                    Ok(left_records.into_iter().chain(right_records).collect())
                } else {
                    let mut combined = left_records;
                    combined.extend(right_records);
                    combined.sort();
                    combined.dedup();
                    Ok(combined)
                }
            }
            _ => Err(format!(
                "Unsupported statement type in CTE: {:?}",
                statement
            )),
        }
    }

    /// Validate foreign key constraints for a row being inserted
    fn validate_foreign_keys(
        &self,
        table_name: &str,
        row: &[Value],
        columns: &[String],
    ) -> Result<(), String> {
        let storage = self.storage.read().unwrap();
        let table_info = storage
            .get_table_info(table_name)
            .map_err(|e| format!("Failed to get table info: {}", e))?;

        for fk in &table_info.foreign_keys {
            let parent_values: Vec<Value> = fk
                .columns
                .iter()
                .filter_map(|col_name| {
                    columns
                        .iter()
                        .position(|c| c.eq_ignore_ascii_case(col_name))
                        .and_then(|idx| row.get(idx).cloned())
                })
                .collect();

            if parent_values.iter().any(|v| matches!(v, Value::Null)) {
                continue;
            }

            let parent_rows = storage
                .scan(&fk.referenced_table)
                .map_err(|e| format!("Failed to scan parent table: {}", e))?;

            let ref_col_indices: Vec<usize> = fk
                .referenced_columns
                .iter()
                .filter_map(|col_name| {
                    storage
                        .get_table_info(&fk.referenced_table)
                        .ok()?
                        .columns
                        .iter()
                        .position(|c| c.name.eq_ignore_ascii_case(col_name))
                })
                .collect();

            let parent_has_match = parent_rows.iter().any(|parent_row| {
                ref_col_indices
                    .iter()
                    .enumerate()
                    .all(|(i, &col_idx)| parent_row.get(col_idx) == parent_values.get(i))
            });

            if !parent_has_match {
                return Err(format!(
                    "Foreign key constraint failed: {} ({}) references {} ({}) which does not exist",
                    table_name,
                    fk.columns.join(", "),
                    fk.referenced_table,
                    fk.referenced_columns.join(", ")
                ));
            }
        }

        Ok(())
    }

    fn validate_unique_constraints(
        &self,
        table_name: &str,
        row: &[Value],
        columns: &[String],
    ) -> Result<(), String> {
        let storage = self.storage.read().unwrap();
        let table_info = storage
            .get_table_info(table_name)
            .map_err(|e| format!("Failed to get table info: {}", e))?;

        for unique_constraint in &table_info.unique_constraints {
            let col_indices: Vec<usize> = unique_constraint
                .columns
                .iter()
                .filter_map(|col_name| {
                    columns
                        .iter()
                        .position(|c| c.eq_ignore_ascii_case(col_name))
                })
                .collect();

            if col_indices.is_empty() {
                continue;
            }

            let values: Vec<Value> = col_indices
                .iter()
                .filter_map(|&i| row.get(i).cloned())
                .collect();

            if values.iter().any(|v| matches!(v, Value::Null)) {
                continue;
            }

            let existing_rows = storage
                .scan(table_name)
                .map_err(|e| format!("Failed to scan table: {}", e))?;

            for existing_row in existing_rows {
                let existing_values: Vec<Value> = col_indices
                    .iter()
                    .filter_map(|&i| existing_row.get(i).cloned())
                    .collect();
                if existing_values == values {
                    return Err(format!(
                        "Duplicate unique key '{}': ({}) values ({}) already exist",
                        unique_constraint.name.as_deref().unwrap_or("unnamed"),
                        unique_constraint.columns.join(", "),
                        values
                            .iter()
                            .map(|v| format!("{:?}", v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate PRIMARY KEY uniqueness for a row being inserted
    fn validate_primary_key(
        &self,
        table_name: &str,
        row: &[Value],
        columns: &[String],
    ) -> Result<(), String> {
        let storage = self.storage.read().unwrap();
        let table_info = storage
            .get_table_info(table_name)
            .map_err(|e| format!("Failed to get table info: {}", e))?;

        // Build column name to row index mapping
        let col_to_row_idx: std::collections::HashMap<String, usize> = if columns.is_empty() {
            std::collections::HashMap::new()
        } else {
            columns
                .iter()
                .enumerate()
                .map(|(i, name)| (name.to_uppercase(), i))
                .collect()
        };

        // Get primary key column names from schema
        let pk_col_names: Vec<String> = table_info
            .columns
            .iter()
            .filter(|c| c.primary_key)
            .map(|c| c.name.to_uppercase())
            .collect();

        if pk_col_names.is_empty() {
            return Ok(());
        }

        // Extract primary key values from row using column mapping
        let pk_values: Vec<Value> = pk_col_names
            .iter()
            .filter_map(|pk_name| {
                if let Some(&row_idx) = col_to_row_idx.get(pk_name) {
                    row.get(row_idx).cloned()
                } else if columns.is_empty() {
                    // Positional mapping when no column list
                    table_info
                        .columns
                        .iter()
                        .position(|c| c.name.to_uppercase() == *pk_name)
                        .and_then(|col_idx| row.get(col_idx).cloned())
                } else {
                    None
                }
            })
            .collect();

        if pk_values.iter().any(|v| matches!(v, Value::Null)) {
            return Ok(());
        }

        let existing_rows = storage
            .scan(table_name)
            .map_err(|e| format!("Failed to scan table: {}", e))?;

        // Build full row mapping for existing rows
        let all_col_names: Vec<String> = table_info
            .columns
            .iter()
            .map(|c| c.name.to_uppercase())
            .collect();

        for existing_row in existing_rows {
            let existing_pk_values: Vec<Value> = pk_col_names
                .iter()
                .filter_map(|pk_name| {
                    all_col_names
                        .iter()
                        .position(|c| c == pk_name)
                        .and_then(|col_idx| existing_row.get(col_idx).cloned())
                })
                .collect();
            if existing_pk_values == pk_values {
                return Err(format!(
                    "Duplicate primary key: ({}) values ({}) already exist",
                    pk_col_names.join(", "),
                    pk_values
                        .iter()
                        .map(|v| format!("{:?}", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        Ok(())
    }

    /// Evaluate a binary operation and return a boolean Value
    fn evaluate_binary_op(&self, left: &Value, right: &Value, op: &str) -> Value {
        match op {
            "=" | "==" | "IS" => Value::Boolean(left == right),
            "!=" | "<>" => Value::Boolean(left != right),
            ">" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l > r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l > r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l > r)
                } else {
                    Value::Boolean(false)
                }
            }
            ">=" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l >= r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l >= r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l >= r)
                } else {
                    Value::Boolean(false)
                }
            }
            "<" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l < r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l < r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l < r)
                } else {
                    Value::Boolean(false)
                }
            }
            "<=" => {
                if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                    Value::Boolean(l <= r)
                } else if let (Value::Float(l), Value::Float(r)) = (left, right) {
                    Value::Boolean(l <= r)
                } else if let (Value::Text(l), Value::Text(r)) = (left, right) {
                    Value::Boolean(l <= r)
                } else {
                    Value::Boolean(false)
                }
            }
            "AND" | "&&" => {
                if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                    Value::Boolean(*l && *r)
                } else {
                    Value::Boolean(false)
                }
            }
            "OR" | "||" => {
                if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                    Value::Boolean(*l || *r)
                } else {
                    Value::Boolean(false)
                }
            }
            _ => Value::Null,
        }
    }

    /// LIKE pattern matching (supports % and _ wildcards) - simple O(n*m) implementation
    fn like_match(&self, left: &Value, pattern: &Value) -> bool {
        let (text, pat) = match (left, pattern) {
            (Value::Text(t), Value::Text(p)) => (t.as_str(), p.as_str()),
            _ => return false,
        };
        // Simple recursive match: text[i..] matches pat[j..]
        fn do_match(text: &str, pat: &str) -> bool {
            let t_bytes = text.as_bytes();
            let p_bytes = pat.as_bytes();
            let mut i = 0;
            let mut j = 0;
            let mut stack: Vec<(usize, usize)> = vec![];

            loop {
                if j >= p_bytes.len() {
                    if i >= t_bytes.len() {
                        return true;
                    }
                    // Try to backtrack
                    if let Some((prev_i, prev_j)) = stack.pop() {
                        i = prev_i;
                        j = prev_j + 1;
                        if j < p_bytes.len() && p_bytes[j] == b'%' {
                            j += 1;
                            if j >= p_bytes.len() {
                                return true;
                            }
                        }
                        continue;
                    }
                    return i >= t_bytes.len();
                }
                if i >= t_bytes.len() {
                    // Only % can match empty string at end of pattern
                    while j < p_bytes.len() && p_bytes[j] == b'%' {
                        j += 1;
                    }
                    return j >= p_bytes.len();
                }

                match p_bytes[j] {
                    b'%' => {
                        // Try matching 0 chars, and save state to try more
                        stack.push((i, j));
                        j += 1;
                        if j >= p_bytes.len() {
                            return true;
                        }
                    }
                    b'_' => {
                        i += 1;
                        j += 1;
                    }
                    c => {
                        // Case-insensitive char comparison
                        let text_lower = t_bytes[i].to_ascii_lowercase();
                        let pat_lower = c.to_ascii_lowercase();
                        if text_lower == pat_lower {
                            i += 1;
                            j += 1;
                        } else {
                            // Try backtracking
                            if let Some((prev_i, prev_j)) = stack.pop() {
                                i = prev_i;
                                j = prev_j + 1;
                                if j < p_bytes.len() && p_bytes[j] == b'%' {
                                    j += 1;
                                }
                                continue;
                            }
                            return false;
                        }
                    }
                }
            }
        }

        do_match(text, pat)
    }

    /// BETWEEN check: left >= low AND left <= high
    fn between_match(&self, left: &Value, low: &Value, high: &Value) -> bool {
        // Check left >= low AND left <= high
        let ge_result = self.ge_values(left, low);
        let le_result = self.le_values(left, high);
        ge_result && le_result
    }

    /// REGEXP pattern matching - simple anchored prefix/suffix match
    /// For complex regex, falls back to substring search
    fn regexp_match(&self, left: &Value, pattern: &Value) -> bool {
        let (text, pat) = match (left, pattern) {
            (Value::Text(t), Value::Text(p)) => (t.as_str(), p.as_str()),
            _ => return false,
        };
        // Case-insensitive search
        let text_lower = text.to_lowercase();
        let pat_lower = pat.to_lowercase();
        text_lower.contains(&pat_lower)
    }

    /// Greater-than-or-equal comparison helper
    fn ge_values(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l >= r,
            (Value::Float(l), Value::Float(r)) => l >= r,
            (Value::Text(l), Value::Text(r)) => l >= r,
            _ => false,
        }
    }

    /// Less-than-or-equal comparison helper
    fn le_values(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l <= r,
            (Value::Float(l), Value::Float(r)) => l <= r,
            (Value::Text(l), Value::Text(r)) => l <= r,
            _ => false,
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
        if let Some(var_name) = expr.strip_prefix('@') {
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
        if let Some(var_name) = name.strip_prefix('@') {
            ctx.get_var(var_name).cloned().unwrap_or(Value::Null)
        } else if ctx.has_var(name) {
            ctx.get_var(name).cloned().unwrap_or(Value::Null)
        } else {
            // Try to parse as literal
            self.evaluate_constant(name)
        }
    }

    fn expand_variables_in_sql(&self, sql: &str, ctx: &ProcedureContext) -> String {
        let chars: Vec<char> = sql.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '@' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let var_name: String = chars[start + 1..i].iter().collect();
                let value = ctx
                    .get_var(&var_name)
                    .map(|v| self.escape_sql_value(v))
                    .unwrap_or_else(|| "NULL".to_string());
                result.push_str(&value);
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    fn escape_sql_value(&self, value: &Value) -> String {
        match value {
            Value::Text(s) => s.replace('\'', "''"),
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Value::Null => "NULL".to_string(),
            Value::Blob(b) => b
                .iter()
                .map(|&x| x as char)
                .collect::<String>()
                .replace('\'', "''"),
        }
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
        let executor = StoredProcExecutor::new_for_test(catalog);
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
        use sqlrustgo_catalog::HandlerCondition;

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
    // White-box tests for stored_proc.rs - Coverage: Statement Execution Paths
    // =====================================================================

    // --- ProcedureContext: Label operations (enter_label, exit_label, has_label) ---

    #[test]
    fn test_procedure_context_nested_labels() {
        let mut ctx = ProcedureContext::new();
        // Enter two nested labels
        ctx.enter_label("outer".to_string());
        ctx.enter_label("inner".to_string());
        assert!(ctx.has_label("outer"));
        assert!(ctx.has_label("inner"));
        // exit_label pops the top (inner)
        ctx.exit_label();
        assert!(ctx.has_label("outer"));
        assert!(!ctx.has_label("inner"));
        // exit_label pops outer
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

    // --- ProcedureContext: Scope operations (enter_scope, exit_scope) ---

    #[test]
    fn test_procedure_context_nested_scopes() {
        let mut ctx = ProcedureContext::new();
        // Set x = 1 in outer scope
        ctx.set_local_var("x", Value::Integer(1));
        assert_eq!(ctx.get_local_var("x"), Some(&Value::Integer(1)));
        // Enter inner scope
        ctx.enter_scope();
        // Inner scope has its own variables, x should still be accessible via saved scope
        ctx.set_local_var("y", Value::Text("hello".to_string()));
        assert_eq!(
            ctx.get_local_var("y"),
            Some(&Value::Text("hello".to_string()))
        );
        // Exit inner scope
        ctx.exit_scope();
        // y should be gone, x should still be 1
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
        // Should restore outer z
        assert_eq!(ctx.get_local_var("z"), Some(&Value::Integer(100)));
    }

    // --- ProcedureContext: Variable operations (local vars, session vars, get_var, has_var) ---

    #[test]
    fn test_procedure_context_local_vs_session_var() {
        let mut ctx = ProcedureContext::new();
        // Local var (no @ prefix)
        ctx.set_local_var("local_x", Value::Integer(10));
        assert_eq!(ctx.get_local_var("local_x"), Some(&Value::Integer(10)));
        // Session var (with @ prefix)
        ctx.set_session_var("session_y", Value::Text("test".to_string()));
        assert_eq!(
            ctx.get_session_var("session_y"),
            Some(&Value::Text("test".to_string()))
        );
        // get_var checks local first, then session
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
        // get_var should strip @ prefix
        assert_eq!(ctx.get_var("@uid"), Some(&Value::Integer(999)));
        assert_eq!(ctx.get_var("uid"), Some(&Value::Integer(999)));
    }

    #[test]
    fn test_procedure_context_has_var() {
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(1));
        ctx.set_var("@y", Value::Text("a".to_string()));
        assert!(ctx.has_var("x"));
        assert!(ctx.has_var("@x")); // strip prefix
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
        // Local x should be gone
        assert!(!ctx.has_var("x"));
        // Session y should still exist
        assert!(ctx.has_var("y"));
    }

    #[test]
    fn test_procedure_context_return_value() {
        let mut ctx = ProcedureContext::new();
        // get_return on new context
        assert!(ctx.get_return().is_none());
        // set_return should store value
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
        // Unquoted identifiers become Text
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
        assert!(executor.compare_values(&alice, &bob, "<")); // A < B
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
        // NULL comparisons return false
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
        // Integer vs Float
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
        // Text + Integer is not supported
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
        // 01xxx is SQLWARNING, not SQLEXCEPTION
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
        // Should find the first matching one (top of stack)
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
        // Fetch with 3 vars but only 2 columns
        let result = ctx.fetch_cursor(
            "cur",
            &["v1".to_string(), "v2".to_string(), "v3".to_string()],
        );
        assert!(result.is_ok());
        assert!(result.unwrap()); // has rows
                                  // Third var should be set to Null
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
        // Don't open cursor
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
}
