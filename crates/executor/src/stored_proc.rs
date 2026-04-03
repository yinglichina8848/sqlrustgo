//! Stored Procedure Executor
//!
//! This module provides stored procedure execution support with control flow.

use crate::ExecutorResult;
use sqlrustgo_catalog::HandlerCondition;
use sqlrustgo_catalog::StoredProcStatement;
use sqlrustgo_storage::StorageEngine;
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
            sqlrustgo_parser::Statement::Select(select) => {
                let table_name = &select.table;
                let storage = self.storage.read().unwrap();
                let records = storage
                    .scan(table_name)
                    .map_err(|e| format!("Failed to scan table: {}", e))?;

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
                let table_name = &insert.table;
                let mut storage = self.storage.write().unwrap();

                if !storage.has_table(table_name) {
                    return Err(format!("Table '{}' not found", table_name));
                }

                let table_info = storage.get_table_info(table_name).ok();
                let num_columns = table_info.as_ref().map(|i| i.columns.len()).unwrap_or(0);

                for row in &insert.values {
                    let mut new_row: Vec<Value> = vec![Value::Null; num_columns];

                    if insert.columns.is_empty() {
                        for (col_idx, expr) in row.iter().enumerate() {
                            if col_idx < num_columns {
                                new_row[col_idx] = self.expression_to_value(expr, ctx);
                            }
                        }
                    } else {
                        for (value_idx, col_name) in insert.columns.iter().enumerate() {
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

                    storage
                        .insert(table_name, vec![new_row])
                        .map_err(|e| format!("Failed to insert: {}", e))?;
                }

                ctx.set_session_var(
                    "__last_insert_count",
                    Value::Integer(insert.values.len() as i64),
                );
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
            sqlrustgo_parser::Expression::QualifiedColumn(table, col) => {
                Value::Text(format!("{}.{}", table, col))
            }
            sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
                let left_val = self.expression_to_value(left, ctx);
                let right_val = self.expression_to_value(right, ctx);
                self.evaluate_binary_op(&left_val, &right_val, op)
            }
            sqlrustgo_parser::Expression::FunctionCall(name, args) => {
                self.evaluate_function(name, args, ctx)
            }
            _ => Value::Null,
        }
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

    /// Evaluate a function call expression
    fn evaluate_function(
        &self,
        name: &str,
        args: &[sqlrustgo_parser::Expression],
        ctx: &ProcedureContext,
    ) -> Value {
        let name_upper = name.to_uppercase();
        match name_upper.as_str() {
            "COALESCE" => {
                for arg in args {
                    let val = self.expression_to_value(arg, ctx);
                    if val != Value::Null {
                        return val;
                    }
                }
                Value::Null
            }
            "IFNULL" | "NVL" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let val = self.expression_to_value(&args[0], ctx);
                if val == Value::Null {
                    if args.len() > 1 {
                        self.expression_to_value(&args[1], ctx)
                    } else {
                        Value::Null
                    }
                } else {
                    val
                }
            }
            "NULLIF" => {
                if args.len() < 2 {
                    return Value::Null;
                }
                let left = self.expression_to_value(&args[0], ctx);
                let right = self.expression_to_value(&args[1], ctx);
                if left == right {
                    Value::Null
                } else {
                    left
                }
            }
            "CONCAT" => {
                let mut result = String::new();
                for arg in args {
                    let val = self.expression_to_value(arg, ctx);
                    if !result.is_empty() {
                        result.push_str(", ");
                    }
                    result.push_str(&val.to_string());
                }
                Value::Text(result)
            }
            "LENGTH" | "LEN" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let val = self.expression_to_value(&args[0], ctx);
                if let Value::Text(s) = val {
                    Value::Integer(s.len() as i64)
                } else {
                    Value::Integer(0)
                }
            }
            "UPPER" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let val = self.expression_to_value(&args[0], ctx);
                if let Value::Text(s) = val {
                    Value::Text(s.to_uppercase())
                } else {
                    Value::Null
                }
            }
            "LOWER" => {
                if args.is_empty() {
                    return Value::Null;
                }
                let val = self.expression_to_value(&args[0], ctx);
                if let Value::Text(s) = val {
                    Value::Text(s.to_lowercase())
                } else {
                    Value::Null
                }
            }
            "IF" => {
                if args.len() < 3 {
                    return Value::Null;
                }
                let cond = self.expression_to_value(&args[0], ctx);
                if cond != Value::Null && cond != Value::Boolean(false) {
                    self.expression_to_value(&args[1], ctx)
                } else {
                    self.expression_to_value(&args[2], ctx)
                }
            }
            _ => Value::Null,
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
        let executor = StoredProcExecutor::new_for_test(catalog);

        let result = executor.execute_call("non_existent", vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_stored_proc_executor_list_empty() {
        let catalog = Arc::new(Catalog::new());
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
        let catalog = Arc::new(Catalog::new());
        let executor = StoredProcExecutor::new_for_test(catalog);
        let mut ctx = ProcedureContext::new();
        ctx.set_var("x", Value::Integer(10));

        // Note: This test requires the condition to reference the variable properly
        // In practice, we'd need proper variable expansion
    }
}
