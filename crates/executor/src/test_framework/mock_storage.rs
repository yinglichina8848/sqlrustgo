//! Mock Storage for Testing

use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::HashMap;
use std::sync::RwLock;

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
        Self {
            tables: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_table(self, name: &str, columns: Vec<(&str, &str)>, rows: Vec<Vec<Value>>) -> Self {
        let cols = columns
            .into_iter()
            .map(|(n, t)| ColumnInfo {
                name: n.to_string(),
                data_type: t.to_string(),
            })
            .collect();
        let info = TableInfo {
            name: name.to_string(),
            columns: cols,
        };
        self.tables
            .write()
            .unwrap()
            .insert(name.to_string(), TableData { info, rows });
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
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_storage_new() {
        let storage = MockStorage::new();
        assert!(storage.get_table("test").is_none());
    }

    #[test]
    fn test_mock_storage_with_table() {
        let storage = MockStorage::new().with_table(
            "users",
            vec![("id", "INTEGER"), ("name", "TEXT")],
            vec![],
        );
        assert!(storage.get_table("users").is_some());
    }

    #[test]
    fn test_mock_storage_get_table() {
        let storage = MockStorage::new().with_table(
            "users",
            vec![("id", "INTEGER")],
            vec![vec![Value::Integer(1)]],
        );
        let table = storage.get_table("users").unwrap();
        assert_eq!(table.info.name, "users");
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_mock_storage_insert() {
        let storage = MockStorage::new().with_table("users", vec![("id", "INTEGER")], vec![]);
        storage.insert("users", vec![Value::Integer(1)]).unwrap();
        let table = storage.get_table("users").unwrap();
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_mock_storage_insert_nonexistent() {
        let storage = MockStorage::new();
        let result = storage.insert("users", vec![Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_data_clone() {
        let data = TableData {
            info: TableInfo {
                name: "test".to_string(),
                columns: vec![],
            },
            rows: vec![vec![Value::Integer(1)]],
        };
        let cloned = data.clone();
        assert_eq!(cloned.rows.len(), 1);
    }

    #[test]
    fn test_table_info_clone() {
        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnInfo {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
            }],
        };
        let cloned = info.clone();
        assert_eq!(cloned.columns.len(), 1);
    }

    #[test]
    fn test_column_info() {
        let col = ColumnInfo {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
        };
        assert_eq!(col.name, "id");
        assert_eq!(col.data_type, "INTEGER");
    }
}
