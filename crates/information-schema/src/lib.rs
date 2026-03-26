//! INFORMATION_SCHEMA Implementation
//!
//! Provides standard SQL INFORMATION_SCHEMA views for metadata access.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRow {
    pub schema_name: String,
    pub schema_owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub table_schema: String,
    pub table_name: String,
    pub table_type: String,
    pub is_insertable_into: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnRow {
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub column_default: Option<String>,
    pub is_nullable: String,
    pub data_type: String,
    pub character_maximum_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRow {
    pub table_schema: String,
    pub table_name: String,
    pub index_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub is_unique: bool,
    pub is_primary: bool,
}

pub struct InformationSchema {
    current_schema: String,
}

impl InformationSchema {
    pub fn new() -> Self {
        Self {
            current_schema: "public".to_string(),
        }
    }

    pub fn get_schemata(&self) -> Vec<SchemaRow> {
        vec![SchemaRow {
            schema_name: self.current_schema.clone(),
            schema_owner: "root".to_string(),
        }]
    }

    pub fn get_tables(&self, tables: &[(&str, &str)]) -> Vec<TableRow> {
        tables
            .iter()
            .map(|(name, table_type)| TableRow {
                table_schema: self.current_schema.clone(),
                table_name: name.to_string(),
                table_type: table_type.to_string(),
                is_insertable_into: "YES".to_string(),
            })
            .collect()
    }

    pub fn get_columns(&self, table_name: &str, columns: &[(&str, &str)]) -> Vec<ColumnRow> {
        columns
            .iter()
            .enumerate()
            .map(|(i, (name, data_type))| ColumnRow {
                table_schema: self.current_schema.clone(),
                table_name: table_name.to_string(),
                column_name: name.to_string(),
                ordinal_position: (i + 1) as i32,
                column_default: None,
                is_nullable: "YES".to_string(),
                data_type: data_type.to_string(),
                character_maximum_length: None,
                numeric_precision: None,
                numeric_scale: None,
            })
            .collect()
    }
}

impl Default for InformationSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schemata() {
        let schema = InformationSchema::new();
        let rows = schema.get_schemata();
        assert!(!rows.is_empty());
        assert_eq!(rows[0].schema_name, "public");
    }

    #[test]
    fn test_tables() {
        let schema = InformationSchema::new();
        let tables = vec![("users", "BASE TABLE"), ("v_user", "VIEW")];
        let rows = schema.get_tables(&tables);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_columns() {
        let schema = InformationSchema::new();
        let columns = vec![("id", "INTEGER"), ("name", "TEXT")];
        let rows = schema.get_columns("users", &columns);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].ordinal_position, 1);
    }
}
