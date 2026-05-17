use sqlrustgo_catalog::HandlerCondition;
use sqlrustgo_catalog::StoredProcStatement;
use sqlrustgo_types::Value;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    pub(crate) condition: HandlerCondition,
    pub(crate) body: Vec<StoredProcStatement>,
}

#[derive(Debug, Clone)]
pub(crate) struct Cursor {
    #[allow(dead_code)]
    name: String,
    pub(crate) query: String,
    records: Vec<Vec<Value>>,
    position: usize,
    is_open: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProcedureContext {
    local_variables: HashMap<String, Value>,
    session_variables: HashMap<String, Value>,
    return_value: Option<Value>,
    leave: bool,
    iterate: bool,
    current_label: Option<String>,
    label_stack: Vec<String>,
    scope_stack: Vec<HashMap<String, Value>>,
    pub(crate) handler_stack: Vec<ExceptionHandler>,
    exception_handling: bool,
    current_exception: Option<StoredProcError>,
    pub(crate) cursors: HashMap<String, Cursor>,
    pub(crate) cte_tables: HashMap<String, Vec<Vec<Value>>>,
}

impl ProcedureContext {
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

    pub fn open_cursor(&mut self, name: &str) -> Result<(), String> {
        if let Some(cursor) = self.cursors.get_mut(name) {
            cursor.is_open = true;
            cursor.position = 0;
            Ok(())
        } else {
            Err(format!("Cursor '{}' not found", name))
        }
    }

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

    pub fn close_cursor(&mut self, name: &str) -> Result<(), String> {
        if let Some(cursor) = self.cursors.get_mut(name) {
            cursor.is_open = false;
            Ok(())
        } else {
            Err(format!("Cursor '{}' not found", name))
        }
    }

    pub fn has_cursor(&self, name: &str) -> bool {
        self.cursors.contains_key(name)
    }

    pub fn set_cursor_records(&mut self, name: &str, records: Vec<Vec<Value>>) {
        if let Some(cursor) = self.cursors.get_mut(name) {
            cursor.records = records;
            cursor.position = 0;
        }
    }

    pub fn push_handler(&mut self, condition: HandlerCondition, body: Vec<StoredProcStatement>) {
        self.handler_stack
            .push(ExceptionHandler { condition, body });
    }

    pub fn pop_handler(&mut self) {
        self.handler_stack.pop();
    }

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

    pub fn set_exception_handling(&mut self, handling: bool) {
        self.exception_handling = handling;
    }

    pub fn is_handling_exception(&self) -> bool {
        self.exception_handling
    }

    pub fn set_exception(&mut self, sqlstate: String, message: String) {
        if let Some(exc) = &mut self.current_exception {
            exc.sqlstate = sqlstate;
            exc.message = message;
        } else {
            self.current_exception = Some(StoredProcError { sqlstate, message });
        }
    }

    pub fn clear_exception(&mut self) {
        self.current_exception = None;
    }

    pub fn get_exception(&self) -> Option<&StoredProcError> {
        self.current_exception.as_ref()
    }

    pub fn enter_label(&mut self, label: String) {
        self.label_stack.push(label.clone());
        self.current_label = Some(label);
    }

    pub fn exit_label(&mut self) {
        self.label_stack.pop();
        self.current_label = self.label_stack.last().cloned();
    }

    pub fn enter_scope(&mut self) {
        self.scope_stack.push(self.local_variables.clone());
        self.local_variables.clear();
    }

    pub fn exit_scope(&mut self) {
        if let Some(saved_vars) = self.scope_stack.pop() {
            self.local_variables = saved_vars;
        }
    }

    pub fn has_label(&self, label: &str) -> bool {
        self.label_stack.iter().any(|l| l == label)
    }

    pub fn set_local_var(&mut self, name: &str, value: Value) {
        self.local_variables.insert(name.to_string(), value);
    }

    pub fn get_local_var(&self, name: &str) -> Option<&Value> {
        self.local_variables.get(name)
    }

    pub fn set_session_var(&mut self, name: &str, value: Value) {
        self.session_variables.insert(name.to_string(), value);
    }

    pub fn get_session_var(&self, name: &str) -> Option<&Value> {
        self.session_variables.get(name)
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        let key = name.strip_prefix('@').unwrap_or(name);
        self.local_variables
            .get(key)
            .or_else(|| self.session_variables.get(key))
    }

    pub fn set_var(&mut self, name: &str, value: Value) {
        if let Some(var_name) = name.strip_prefix('@') {
            self.session_variables.insert(var_name.to_string(), value);
        } else {
            self.local_variables.insert(name.to_string(), value);
        }
    }

    pub fn has_var(&self, name: &str) -> bool {
        let key = if let Some(var_name) = name.strip_prefix('@') {
            var_name
        } else {
            name
        };
        self.local_variables.contains_key(key) || self.session_variables.contains_key(key)
    }

    pub fn clear_local_vars(&mut self) {
        self.local_variables.clear();
    }

    pub fn get_session_vars(&self) -> &HashMap<String, Value> {
        &self.session_variables
    }

    pub fn set_return(&mut self, value: Value) {
        self.return_value = Some(value);
    }

    pub fn get_return(&self) -> Option<Value> {
        self.return_value.clone()
    }

    pub fn set_leave(&mut self) {
        self.leave = true;
    }

    pub fn should_leave(&self) -> bool {
        self.leave
    }

    pub fn reset_leave(&mut self) {
        self.leave = false;
    }

    pub fn set_iterate(&mut self) {
        self.iterate = true;
    }

    pub fn should_iterate(&self) -> bool {
        self.iterate
    }

    pub fn reset_iterate(&mut self) {
        self.iterate = false;
    }

    pub fn set_label(&mut self, label: Option<String>) {
        self.current_label = label;
    }

    pub fn get_label(&self) -> Option<&String> {
        self.current_label.as_ref()
    }
}
