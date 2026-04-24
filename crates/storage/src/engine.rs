//! Storage Engine trait - abstraction for storage backends
//! Supports multiple storage implementations (File, Memory, etc.)

use serde::{Deserialize, Serialize};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::{HashMap, HashSet};

/// Referential action for foreign key constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForeignKeyAction {
    Cascade,
    SetNull,
    Restrict,
    NoAction,
}

/// Foreign key constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyConstraint {
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueConstraint {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

/// Check constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConstraint {
    pub name: Option<String>,
    pub expression: String,
}

/// Evaluate a CHECK constraint expression against a record
/// The expression is stored as a string like "age >= 0" or "name IS NOT NULL"
/// Returns Ok(true) if constraint is satisfied, Ok(false) if not, Err on parse error
pub fn evaluate_check_constraint(
    constraint: &CheckConstraint,
    columns: &[String],
    record: &[Value],
) -> SqlResult<bool> {
    evaluate_sql_expression(&constraint.expression, columns, record)
}

/// Evaluate a SQL expression against a record
/// Supports: comparisons (=, !=, <, >, <=, >=), boolean ops (AND, OR, NOT), IS NULL/IS NOT NULL
fn evaluate_sql_expression(expr: &str, columns: &[String], record: &[Value]) -> SqlResult<bool> {
    let expr = expr.trim();

    // Handle AND/OR
    if let Some(idx) = find_top_level_op(expr, "AND") {
        let left = &expr[..idx];
        let right = &expr[idx + 3..];
        return Ok(evaluate_sql_expression(left.trim(), columns, record)?
            && evaluate_sql_expression(right.trim(), columns, record)?);
    }
    if let Some(idx) = find_top_level_op(expr, "OR") {
        let left = &expr[..idx];
        let right = &expr[idx + 2..];
        return Ok(evaluate_sql_expression(left.trim(), columns, record)?
            || evaluate_sql_expression(right.trim(), columns, record)?);
    }

    // Handle NOT
    if expr.to_uppercase().starts_with("NOT ") {
        let inner = &expr[4..].trim();
        return Ok(!evaluate_sql_expression(inner, columns, record)?);
    }

    // Handle IS NULL / IS NOT NULL
    if let Some(idx) = expr.to_uppercase().find(" IS NULL") {
        let col_name = expr[..idx].trim();
        if let Some(val) = get_column_value(col_name, columns, record) {
            return Ok(matches!(val, Value::Null));
        }
        return Ok(true); // column not found, assume OK
    }
    if let Some(idx) = expr.to_uppercase().find(" IS NOT NULL") {
        let col_name = expr[..idx].trim();
        if let Some(val) = get_column_value(col_name, columns, record) {
            return Ok(!matches!(val, Value::Null));
        }
        return Ok(false);
    }

    // Handle comparisons: column op value
    for (op, check) in &[
        (">=", "gte"),
        ("<=", "lte"),
        ("!=", "neq"),
        ("<>", "neq"),
        ("=", "eq"),
        ("==", "eq"),
        (">", "gt"),
        ("<", "lt"),
    ] {
        if let Some(idx) = expr.find(op) {
            let col_name = expr[..idx].trim();
            let value_str = expr[idx + op.len()..].trim();

            if let Some(col_val) = get_column_value(col_name, columns, record) {
                return compare_values(col_val, value_str, check);
            }
            break;
        }
    }

    // If no comparison found, try to evaluate as a literal boolean or column existence check
    let upper = expr.to_uppercase();
    if upper == "TRUE" || upper == "1" {
        return Ok(true);
    }
    if upper == "FALSE" || upper == "0" {
        return Ok(false);
    }

    // Treat as column name - check if not null
    if let Some(val) = get_column_value(expr, columns, record) {
        return Ok(!matches!(val, Value::Null) && !is_zero_or_empty(val));
    }

    Err(format!("Cannot evaluate CHECK expression: {}", expr).into())
}

/// Find top-level operator (not inside quotes or parentheses)
fn find_top_level_op(expr: &str, op: &str) -> Option<usize> {
    let upper = expr.to_uppercase();
    let op_upper = op.to_uppercase();
    let mut depth = 0;
    let mut in_string = false;

    for (i, c) in expr.char_indices() {
        match c {
            '(' => {
                depth += 1;
            }
            ')' => {
                depth -= 1;
            }
            '\'' => {
                in_string = !in_string;
            }
            _ if !in_string && depth == 0 => {
                if upper[i..].starts_with(&op_upper) {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Get column value by name (case-insensitive)
fn get_column_value<'a>(
    name: &str,
    columns: &'a [String],
    record: &'a [Value],
) -> Option<&'a Value> {
    // Remove quotes if present
    let name = name.trim().trim_matches(|c| c == '\'' || c == '"');

    for (i, col) in columns.iter().enumerate() {
        if col.eq_ignore_ascii_case(name) {
            return record.get(i);
        }
    }
    None
}

/// Compare column value with string representation
fn compare_values(col_val: &Value, compare_with: &str, op: &str) -> SqlResult<bool> {
    let cmp_str = compare_with.trim().trim_matches(|c| c == '\'' || c == '"');

    match op {
        "eq" => match col_val {
            Value::Null => Ok(false),
            Value::Integer(i) => {
                if let Ok(cmp) = cmp_str.parse::<i64>() {
                    Ok(*i == cmp)
                } else {
                    Ok(false)
                }
            }
            Value::Float(f) => {
                if let Ok(cmp) = cmp_str.parse::<f64>() {
                    Ok(*f == cmp)
                } else {
                    Ok(false)
                }
            }
            Value::Text(s) => Ok(s == cmp_str),
            Value::Boolean(b) => {
                let cmp_bool = cmp_str.eq_ignore_ascii_case("true") || cmp_str == "1";
                Ok(*b == cmp_bool)
            }
            Value::Blob(_) => Ok(false),
        },
        "neq" => Ok(!compare_values(col_val, compare_with, "eq")?),
        "gt" | "gte" | "lt" | "lte" => {
            match col_val {
                Value::Integer(i) => {
                    if let Ok(cmp) = cmp_str.parse::<i64>() {
                        return Ok(match op {
                            "gt" => *i > cmp,
                            "gte" => *i >= cmp,
                            "lt" => *i < cmp,
                            "lte" => *i <= cmp,
                            _ => false,
                        });
                    }
                }
                Value::Float(f) => {
                    if let Ok(cmp) = cmp_str.parse::<f64>() {
                        return Ok(match op {
                            "gt" => *f > cmp,
                            "gte" => *f >= cmp,
                            "lt" => *f < cmp,
                            "lte" => *f <= cmp,
                            _ => false,
                        });
                    }
                }
                Value::Text(s) => {
                    return Ok(match op {
                        "gt" => s.as_str() > cmp_str,
                        "gte" => s.as_str() >= cmp_str,
                        "lt" => s.as_str() < cmp_str,
                        "lte" => s.as_str() <= cmp_str,
                        _ => false,
                    });
                }
                _ => {}
            }
            Err(format!("Cannot compare {} with {}", col_val, cmp_str).into())
        }
        _ => Err(format!("Unknown operator: {}", op).into()),
    }
}

fn is_zero_or_empty(val: &Value) -> bool {
    match val {
        Value::Integer(i) => *i == 0,
        Value::Float(f) => *f == 0.0,
        Value::Text(s) => s.is_empty(),
        Value::Boolean(b) => !*b,
        Value::Null => true,
        Value::Blob(_) => false,
    }
}

/// Trigger timing: BEFORE or AFTER
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerTiming {
    Before,
    After,
}

/// Trigger event: INSERT, UPDATE, or DELETE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
}

/// Trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerInfo {
    pub name: String,
    pub table_name: String,
    pub timing: TriggerTiming,
    pub event: TriggerEvent,
    pub body: String,
}

/// Partition type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartitionType {
    Range,
    List,
    Hash,
}

/// Partition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub partition_type: PartitionType,
    pub column: String,
    pub boundaries: Vec<Value>,
}

impl PartitionInfo {
    pub fn new_range(column: &str, boundaries: Vec<Value>) -> Self {
        Self {
            partition_type: PartitionType::Range,
            column: column.to_string(),
            boundaries,
        }
    }

    pub fn new_list(column: &str, values: Vec<Value>) -> Self {
        Self {
            partition_type: PartitionType::List,
            column: column.to_string(),
            boundaries: values,
        }
    }

    pub fn new_hash(column: &str, num_partitions: u32) -> Self {
        Self {
            partition_type: PartitionType::Hash,
            column: column.to_string(),
            boundaries: vec![Value::Integer(num_partitions as i64)],
        }
    }

    pub fn get_partition_index(&self, value: &Value) -> Option<usize> {
        match self.partition_type {
            PartitionType::Range => self.get_range_partition(value),
            PartitionType::List => self.get_list_partition(value),
            PartitionType::Hash => self.get_hash_partition(value),
        }
    }

    fn get_range_partition(&self, value: &Value) -> Option<usize> {
        if let Value::Integer(n) = value {
            for (i, boundary) in self.boundaries.iter().enumerate() {
                if let Value::Integer(b) = boundary {
                    if n < b {
                        return Some(i);
                    }
                }
            }
            Some(self.boundaries.len())
        } else {
            None
        }
    }

    fn get_list_partition(&self, value: &Value) -> Option<usize> {
        for (i, boundary) in self.boundaries.iter().enumerate() {
            if value == boundary {
                return Some(i);
            }
        }
        None
    }

    fn get_hash_partition(&self, value: &Value) -> Option<usize> {
        if let Value::Integer(n) = value {
            let num_partitions = self.boundaries.first()?.as_integer()? as u64;
            let hash = n.unsigned_abs() % num_partitions;
            Some(hash as usize)
        } else if let Value::Text(s) = value {
            let num_partitions = self.boundaries.first()?.as_integer()? as u32;
            let hash = calculate_hash(s.as_bytes()) % num_partitions;
            Some(hash as usize)
        } else {
            None
        }
    }
}

fn calculate_hash(data: &[u8]) -> u32 {
    data.iter()
        .fold(0u32, |acc, &b| acc.wrapping_add(b as u32).wrapping_mul(31))
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKeyConstraint>,
    #[serde(default)]
    pub unique_constraints: Vec<UniqueConstraint>,
    #[serde(default)]
    pub check_constraints: Vec<CheckConstraint>,
    #[serde(skip)]
    pub partition_info: Option<PartitionInfo>,
}

/// Column definition for table schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub primary_key: bool,
}

impl ColumnDefinition {
    pub fn new(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
            nullable: false,
            primary_key: false,
        }
    }
}

/// Table data - combines metadata and rows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Record>,
}

/// Record type - a single row of values
pub type Record = Vec<Value>;

/// StorageEngine trait - abstraction for table storage
/// Enables multiple storage backends (FileStorage, MemoryStorage, etc.)
pub trait StorageEngine: Send + Sync {
    /// Scan all rows from a table
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>>;

    /// Insert rows into a table
    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()>;

    /// Delete rows matching a filter
    fn delete(&mut self, table: &str, _filters: &[Value]) -> SqlResult<usize>;

    /// Update rows matching a filter
    fn update(
        &mut self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize>;

    /// Create a new table
    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()>;

    /// Drop a table
    fn drop_table(&mut self, table: &str) -> SqlResult<()>;

    /// Get table metadata
    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo>;

    /// Check if table exists
    fn has_table(&self, table: &str) -> bool;

    /// List all tables
    fn list_tables(&self) -> Vec<String>;

    /// Create an index on a table
    fn create_index(&mut self, table: &str, column: &str, column_index: usize) -> SqlResult<()>;

    /// Drop an index from a table
    fn drop_index(&mut self, table: &str, column: &str) -> SqlResult<()>;

    /// Add a column to an existing table
    fn add_column(&mut self, table: &str, column: ColumnDefinition) -> SqlResult<()>;

    /// Rename a table
    fn rename_table(&mut self, table: &str, new_name: &str) -> SqlResult<()>;

    /// Create a trigger on a table
    fn create_trigger(&mut self, info: TriggerInfo) -> SqlResult<()>;

    /// Drop a trigger by name
    fn drop_trigger(&mut self, name: &str) -> SqlResult<()>;

    /// Get a trigger by name
    fn get_trigger(&self, name: &str) -> Option<TriggerInfo>;

    /// List all triggers for a table
    fn list_triggers(&self, table: &str) -> Vec<TriggerInfo>;

    /// List all indexes for a table, returns Vec of (column_name, index_name)
    fn list_indexes(&self, table: &str) -> Vec<(String, String)>;

    /// Check if a view exists
    fn has_view(&self, name: &str) -> bool;

    /// Create a database
    fn create_database(&mut self, name: &str) -> SqlResult<()>;

    /// List all databases
    fn list_databases(&self) -> Vec<String>;

    /// Use a database
    fn use_database(&mut self, name: &str) -> SqlResult<()>;
}

/// In-memory storage implementation for testing and caching
pub struct MemoryStorage {
    tables: HashMap<String, Vec<Record>>,
    table_infos: HashMap<String, TableInfo>,
    triggers: HashMap<String, TriggerInfo>,
    views: HashSet<String>,
    current_database: String,
    databases: HashSet<String>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        let mut databases = HashSet::new();
        databases.insert("default".to_string());
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
            triggers: HashMap::new(),
            views: HashSet::new(),
            current_database: "default".to_string(),
            databases,
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for MemoryStorage {
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>> {
        Ok(self.tables.get(table).cloned().unwrap_or_default())
    }

    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()> {
        self.tables
            .entry(table.to_string())
            .or_default()
            .extend(records);
        Ok(())
    }

    fn delete(&mut self, table: &str, _filters: &[Value]) -> SqlResult<usize> {
        let mut count = 0;
        if let Some(records) = self.tables.get_mut(table) {
            count = records.len();
            records.clear();
        }
        Ok(count)
    }

    fn update(
        &mut self,
        table: &str,
        filters: &[Value],
        updates: &[(usize, Value)],
    ) -> SqlResult<usize> {
        let Some(records) = self.tables.get_mut(table) else {
            return Ok(0);
        };

        let mut count = 0;

        // If filters is empty, update all rows
        // If filters has values, match first column against first filter value
        if filters.is_empty() {
            for record in records.iter_mut() {
                for &(col_idx, ref new_val) in updates {
                    if col_idx < record.len() {
                        record[col_idx] = new_val.clone();
                    }
                }
                count += 1;
            }
        } else if let Some(filter_val) = filters.first() {
            for record in records.iter_mut() {
                // Check if first column matches filter value
                let matches = record.first().map(|v| v == filter_val).unwrap_or(false);
                if matches {
                    for &(col_idx, ref new_val) in updates {
                        if col_idx < record.len() {
                            record[col_idx] = new_val.clone();
                        }
                    }
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()> {
        self.table_infos.insert(info.name.clone(), info.clone());
        self.tables.entry(info.name.clone()).or_default();
        Ok(())
    }

    fn drop_table(&mut self, table: &str) -> SqlResult<()> {
        self.tables.remove(table);
        self.table_infos.remove(table);
        Ok(())
    }

    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo> {
        self.table_infos
            .get(table)
            .cloned()
            .ok_or_else(|| SqlError::ExecutionError(format!("Table not found: {}", table)))
    }

    fn has_table(&self, table: &str) -> bool {
        self.table_infos.contains_key(table)
    }

    fn list_tables(&self) -> Vec<String> {
        self.table_infos.keys().cloned().collect()
    }

    fn create_index(&mut self, _table: &str, _column: &str, _column_index: usize) -> SqlResult<()> {
        Ok(())
    }

    fn drop_index(&mut self, _table: &str, _column: &str) -> SqlResult<()> {
        Ok(())
    }

    fn add_column(&mut self, table: &str, column: ColumnDefinition) -> SqlResult<()> {
        if let Some(info) = self.table_infos.get_mut(table) {
            info.columns.push(column);
            Ok(())
        } else {
            Err(SqlError::ExecutionError(format!(
                "Cannot add column: table {} not found",
                table
            )))
        }
    }

    fn rename_table(&mut self, table: &str, new_name: &str) -> SqlResult<()> {
        let info = self.table_infos.remove(table);
        let records = self.tables.remove(table);
        if let (Some(info), Some(records)) = (info, records) {
            let mut new_info = info;
            new_info.name = new_name.to_string();
            self.table_infos.insert(new_name.to_string(), new_info);
            self.tables.insert(new_name.to_string(), records);
            Ok(())
        } else {
            Err(SqlError::ExecutionError(format!(
                "Cannot rename table: table {} not found",
                table
            )))
        }
    }

    fn create_trigger(&mut self, info: TriggerInfo) -> SqlResult<()> {
        self.triggers.insert(info.name.clone(), info);
        Ok(())
    }

    fn drop_trigger(&mut self, name: &str) -> SqlResult<()> {
        self.triggers
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| SqlError::ExecutionError(format!("Trigger not found: {}", name)))
    }

    fn get_trigger(&self, name: &str) -> Option<TriggerInfo> {
        self.triggers.get(name).cloned()
    }

    fn list_triggers(&self, table: &str) -> Vec<TriggerInfo> {
        self.triggers
            .values()
            .filter(|t| t.table_name == table)
            .cloned()
            .collect()
    }

    fn has_view(&self, name: &str) -> bool {
        self.views.contains(name)
    }

    fn list_indexes(&self, _table: &str) -> Vec<(String, String)> {
        Vec::new()
    }

    fn create_database(&mut self, name: &str) -> SqlResult<()> {
        if self.databases.contains(name) {
            return Err(SqlError::ExecutionError(format!(
                "Database '{}' already exists",
                name
            )));
        }
        self.databases.insert(name.to_string());
        Ok(())
    }

    fn list_databases(&self) -> Vec<String> {
        self.databases.iter().cloned().collect()
    }

    fn use_database(&mut self, name: &str) -> SqlResult<()> {
        if !self.databases.contains(name) {
            return Err(SqlError::ExecutionError(format!(
                "Unknown database '{}'",
                name
            )));
        }
        self.current_database = name.to_string();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that StorageEngine trait is defined correctly
    #[test]
    fn test_storage_engine_trait_exists() {
        fn _check_trait(_engine: &dyn StorageEngine) {}
    }

    #[test]
    fn test_memory_storage_new() {
        let storage = MemoryStorage::new();
        assert!(storage.list_tables().is_empty());
    }

    #[test]
    fn test_memory_storage_has_table() {
        let storage = MemoryStorage::new();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_memory_storage_create_and_drop() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };
        storage.create_table(&info).unwrap();
        let tables = storage.list_tables();
        assert!(tables.contains(&"users".to_string()));
    }

    #[test]
    fn test_memory_storage_scan_empty() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);
        let result = storage.scan("users").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_memory_storage_insert_and_scan() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
            ],
        );
        let result = storage.scan("users").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_storage_engine_send_sync() {
        fn _check<T: Send + Sync>() {}
        _check::<MemoryStorage>();
    }

    #[test]
    fn test_storage_engine_create_and_drop_table() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };

        storage.create_table(&info).unwrap();
        assert!(storage.has_table("users"));
        assert_eq!(storage.list_tables(), vec!["users"]);

        storage.drop_table("users").unwrap();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_storage_engine_get_table_info() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: true,
            }],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };

        storage.create_table(&info).unwrap();
        let retrieved = storage.get_table_info("users").unwrap();
        assert_eq!(retrieved.name, "users");
        assert_eq!(retrieved.columns.len(), 1);
    }

    #[test]
    fn test_storage_engine_insert_records() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);

        storage
            .insert("users", vec![vec![Value::Integer(1)]])
            .unwrap();
        let records = storage.scan("users").unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_storage_engine_delete_all() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
        );

        let deleted = storage.delete("users", &[]).unwrap();
        assert_eq!(deleted, 2);
    }

    #[test]
    fn test_storage_engine_update_values() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
        );

        let updated = storage
            .update("users", &[], &[(1, Value::Text("Bob".to_string()))][..])
            .unwrap();
        assert_eq!(updated, 1);
    }

    #[test]
    fn test_storage_engine_table_operations() {
        let mut storage = MemoryStorage::new();
        let info1 = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };
        let info2 = TableInfo {
            name: "orders".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };
        storage.create_table(&info1).unwrap();
        storage.create_table(&info2).unwrap();

        let tables = storage.list_tables();
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"orders".to_string()));
    }

    #[test]
    fn test_storage_engine_has_table_check() {
        let mut storage = MemoryStorage::new();
        assert!(!storage.has_table("users"));

        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        };
        storage.create_table(&info).unwrap();
        assert!(storage.has_table("users"));
    }

    #[test]
    fn test_storage_engine_table_not_found() {
        let storage = MemoryStorage::new();
        let result = storage.get_table_info("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_sql_expression_integer_comparison() {
        let columns = vec!["age".to_string(), "name".to_string()];
        let record = vec![Value::Integer(25), Value::Text("Alice".to_string())];

        // age > 18
        let result = evaluate_sql_expression("age > 18", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // age >= 25
        let result = evaluate_sql_expression("age >= 25", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // age < 18
        let result = evaluate_sql_expression("age < 18", &columns, &record);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // age = 25
        let result = evaluate_sql_expression("age = 25", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // age <> 30
        let result = evaluate_sql_expression("age <> 30", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_sql_expression_boolean_and_logical_ops() {
        let columns = vec!["age".to_string(), "active".to_string()];
        let record = vec![Value::Integer(25), Value::Boolean(true)];

        // age > 18 AND active = true
        let result = evaluate_sql_expression("age > 18 AND active = true", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // age < 18 OR active = true
        let result = evaluate_sql_expression("age < 18 OR active = true", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // NOT active = false
        let result = evaluate_sql_expression("NOT active = false", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_sql_expression_null_handling() {
        let columns = vec!["age".to_string(), "email".to_string()];
        let record = vec![Value::Null, Value::Text("test@test.com".to_string())];

        // age IS NOT NULL should be false
        let result = evaluate_sql_expression("age IS NOT NULL", &columns, &record);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // email IS NOT NULL should be true
        let result = evaluate_sql_expression("email IS NOT NULL", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_sql_expression_text_comparison() {
        let columns = vec!["name".to_string()];
        let record = vec![Value::Text("Alice".to_string())];

        // name = 'Alice'
        let result = evaluate_sql_expression("name = 'Alice'", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // name <> 'Bob'
        let result = evaluate_sql_expression("name <> 'Bob'", &columns, &record);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
