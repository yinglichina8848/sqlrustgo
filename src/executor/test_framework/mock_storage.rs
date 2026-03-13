//! Mock Storage for Testing
//!
//! Provides an in-memory storage implementation for testing executors

use crate::types::{SqlError, SqlResult, Value};
use std::collections::HashMap;
use std::sync::RwLock;

/// Mock storage for testing - simple in-memory implementation
pub struct MockStorage {
    tables: RwLock<HashMap<String, TableData>>,
}

#[derive(Clone, Debug)]
pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Vec<Value>>,
}

#[derive(Clone, Debug)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Clone, Debug)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

impl MockStorage {
    pub fn new() -> Self {
        Self { tables: RwLock::new(HashMap::new()) }
    }

    pub fn with_table(mut self, name: &str, columns: Vec<(&str, &str)>, rows: Vec<Vec<Value>>) -> Self {
        let cols = columns.into_iter().map(|(n, t)| ColumnInfo { name: n.to_string(), data_type: t.to_string() }).collect();
        let info = TableInfo { name: name.to_string(), columns: cols };
        self.tables.write().unwrap().insert(name.to_string(), TableData { info, rows });
        self
    }

    pub fn get_table(&self, name: &str) -> Option<TableData> {
        self.tables.read().unwrap().get(name).cloned()
    }

    pub fn insert(&self, table: &str, row: Vec<Value>) -> SqlResult<()> {
        let mut tables = self.tables.write().unwrap();
        if let Some(data) = tables.get_mut(table) {
            data.rows.push(row);
            Ok(())
        } else {
            Err(SqlError::TableNotFound(table.to_string()))
        }
    }
}

impl Default for MockStorage {
    fn default() -> Self { Self::new() }
}
