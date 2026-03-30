//! Table definition for catalog schemas

use crate::column::ColumnDefinition;
use crate::error::CatalogError;
use crate::error::CatalogResult;
use crate::index::IndexInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Foreign key reference definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRef {
    /// Referenced schema name
    pub referenced_schema: String,
    /// Referenced table name
    pub referenced_table: String,
    /// Referenced column names
    pub referenced_columns: Vec<String>,
    /// Local column names (the foreign key columns)
    pub columns: Vec<String>,
    /// ON DELETE action
    pub on_delete: Option<ForeignKeyAction>,
    /// ON UPDATE action
    pub on_update: Option<ForeignKeyAction>,
}

/// Foreign key referential action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForeignKeyAction {
    /// CASCADE: Delete/Update parent and children
    Cascade,
    /// SET NULL: Set child columns to NULL
    SetNull,
    /// RESTRICT: Reject operation
    Restrict,
    /// NO ACTION: Same as RESTRICT but check at end of statement
    NoAction,
}

/// Table definition in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Column definitions (in order)
    pub columns: Vec<ColumnDefinition>,
    /// Primary key column names (if any)
    pub primary_key: Option<Vec<String>>,
    /// Table indices (including primary key index)
    pub indices: Vec<IndexInfo>,
    /// Foreign key constraints
    pub foreign_keys: Vec<ForeignKeyRef>,
    /// Current row count (estimated or actual)
    pub row_count: u64,
}

impl Table {
    /// Create a new table with columns
    pub fn new(name: impl Into<String>, columns: Vec<ColumnDefinition>) -> Self {
        Self {
            name: name.into(),
            columns,
            primary_key: None,
            indices: Vec::new(),
            foreign_keys: Vec::new(),
            row_count: 0,
        }
    }

    /// Add a column to the table
    pub fn add_column(mut self, column: ColumnDefinition) -> CatalogResult<Self> {
        // Check for duplicate column names
        if self.columns.iter().any(|c| c.name == column.name) {
            return Err(CatalogError::DuplicateColumn {
                schema: "unknown".to_string(),
                table: self.name.clone(),
                column: column.name.clone(),
            });
        }

        self.columns.push(column);
        Ok(self)
    }

    /// Set the primary key
    pub fn primary_key(mut self, column_names: Vec<String>) -> CatalogResult<Self> {
        // Verify all columns exist
        let col_set: HashSet<&str> = self.columns.iter().map(|c| c.name.as_str()).collect();
        for col in &column_names {
            if !col_set.contains(col.as_str()) {
                return Err(CatalogError::ColumnNotFound {
                    schema: "unknown".to_string(),
                    table: self.name.clone(),
                    column: col.clone(),
                });
            }
        }

        // Set primary key position for each column
        for (i, col_name) in column_names.iter().enumerate() {
            if let Some(col) = self.columns.iter_mut().find(|c| &c.name == col_name) {
                col.primary_key_position = Some(i);
                col.nullable = false;
                col.is_unique = true;
            }
        }

        // Add primary key index
        self.indices.push(
            IndexInfo::new(
                format!("pk_{}", self.name),
                self.name.clone(),
                column_names.clone(),
            )
            .primary_key(),
        );

        self.primary_key = Some(column_names);
        Ok(self)
    }

    /// Add a foreign key constraint
    pub fn add_foreign_key(mut self, fk: ForeignKeyRef) -> Self {
        self.foreign_keys.push(fk);
        self
    }

    /// Add an index
    pub fn add_index(mut self, index: IndexInfo) -> Self {
        self.indices.push(index);
        self
    }

    /// Get column by name
    pub fn get_column(&self, name: &str) -> Option<&ColumnDefinition> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Get column names as a set
    pub fn column_names(&self) -> HashSet<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    /// Validate table invariants
    pub fn validate(&self, schema_name: &str) -> CatalogResult<()> {
        // Check for duplicate column names
        let mut seen = HashSet::new();
        for col in &self.columns {
            if !seen.insert(&col.name) {
                return Err(CatalogError::DuplicateColumn {
                    schema: schema_name.to_string(),
                    table: self.name.clone(),
                    column: col.name.clone(),
                });
            }
        }

        // Validate each column
        for col in &self.columns {
            col.validate(schema_name, &self.name)?;
        }

        // Validate primary key columns exist
        if let Some(ref pk_cols) = self.primary_key {
            for col_name in pk_cols {
                if !self.column_names().contains(col_name.as_str()) {
                    return Err(CatalogError::ColumnNotFound {
                        schema: schema_name.to_string(),
                        table: self.name.clone(),
                        column: col_name.clone(),
                    });
                }
            }

            // Primary key columns must be NOT NULL
            for col_name in pk_cols {
                if let Some(col) = self.get_column(col_name) {
                    if col.nullable {
                        return Err(CatalogError::InvalidPrimaryKey(format!(
                            "Primary key column '{}' in '{}.{}' cannot be nullable",
                            col_name, schema_name, self.name
                        )));
                    }
                }
            }
        }

        // Validate foreign key references
        for fk in &self.foreign_keys {
            if fk.columns.len() != fk.referenced_columns.len() {
                return Err(CatalogError::ForeignKeyViolation {
                    schema: schema_name.to_string(),
                    table: self.name.clone(),
                    column: fk.columns.join(", "),
                    referenced: format!("{}.{}", fk.referenced_schema, fk.referenced_table),
                    reason: "Column count mismatch".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Set row count estimate
    pub fn set_row_count(mut self, count: u64) -> Self {
        self.row_count = count;
        self
    }

    /// Get primary key columns
    pub fn get_primary_key_columns(&self) -> Option<Vec<&ColumnDefinition>> {
        self.primary_key.as_ref().map(|pk_cols| {
            pk_cols
                .iter()
                .filter_map(|name| self.columns.iter().find(|c| &c.name == name))
                .collect()
        })
    }
}

/// Arc-wrapped table for shared ownership
pub type TableRef = Arc<Table>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataType;

    fn create_test_table() -> Table {
        Table::new(
            "users",
            vec![
                ColumnDefinition::new("id", DataType::Integer).primary_key(0),
                ColumnDefinition::new("email", DataType::Text).not_null(),
                ColumnDefinition::new("name", DataType::Text),
            ],
        )
        .primary_key(vec!["id".to_string()])
        .unwrap()
    }

    #[test]
    fn test_table_creation() {
        let table = create_test_table();
        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 3);
    }

    #[test]
    fn test_get_column() {
        let table = create_test_table();
        assert!(table.get_column("id").is_some());
        assert!(table.get_column("nonexistent").is_none());
    }

    #[test]
    fn test_column_names() {
        let table = create_test_table();
        let names = table.column_names();
        assert!(names.contains("id"));
        assert!(names.contains("email"));
        assert!(names.contains("name"));
    }

    #[test]
    fn test_validate_duplicate_column() {
        let result = Table::new(
            "users",
            vec![
                ColumnDefinition::new("id", DataType::Integer),
                ColumnDefinition::new("id", DataType::Text),
            ],
        )
        .validate("public");

        assert!(matches!(result, Err(CatalogError::DuplicateColumn { .. })));
    }

    #[test]
    fn test_validate_pk_nullable() {
        let result = Table::new(
            "users",
            vec![
                ColumnDefinition::new("id", DataType::Integer)
                    .primary_key(0)
                    .not_null(), // Explicitly set NOT NULL
            ],
        )
        .primary_key(vec!["id".to_string()])
        .unwrap()
        .validate("public");

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_column() {
        let table = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        )
        .add_column(ColumnDefinition::new("name", DataType::Text))
        .unwrap();
        assert_eq!(table.columns.len(), 2);
        assert!(table.get_column("name").is_some());
    }

    #[test]
    fn test_add_column_duplicate_error() {
        let result = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        )
        .add_column(ColumnDefinition::new("id", DataType::Text));
        assert!(matches!(result, Err(CatalogError::DuplicateColumn { .. })));
    }

    #[test]
    fn test_add_foreign_key() {
        let fk = ForeignKeyRef {
            referenced_schema: "public".to_string(),
            referenced_table: "parent".to_string(),
            referenced_columns: vec!["id".to_string()],
            columns: vec!["parent_id".to_string()],
            on_delete: Some(ForeignKeyAction::Cascade),
            on_update: Some(ForeignKeyAction::Cascade),
        };
        let table = Table::new(
            "child",
            vec![ColumnDefinition::new("parent_id", DataType::Integer)],
        )
        .add_foreign_key(fk);
        assert_eq!(table.foreign_keys.len(), 1);
    }

    #[test]
    fn test_add_index() {
        let index = IndexInfo::new("idx_name", "users", vec!["name".to_string()]);
        let table = Table::new("users", vec![ColumnDefinition::new("name", DataType::Text)])
            .add_index(index);
        assert_eq!(table.indices.len(), 1);
    }

    #[test]
    fn test_set_row_count() {
        let table = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        )
        .set_row_count(100);
        assert_eq!(table.row_count, 100);
    }

    #[test]
    fn test_get_primary_key_columns() {
        let table = create_test_table();
        let pk_cols = table.get_primary_key_columns();
        assert!(pk_cols.is_some());
        let cols = pk_cols.unwrap();
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].name, "id");
    }

    #[test]
    fn test_get_primary_key_columns_none() {
        let table = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        );
        assert!(table.get_primary_key_columns().is_none());
    }

    #[test]
    fn test_validate_foreign_key_column_count_mismatch() {
        let fk = ForeignKeyRef {
            referenced_schema: "public".to_string(),
            referenced_table: "parent".to_string(),
            referenced_columns: vec!["id".to_string(), "other".to_string()],
            columns: vec!["parent_id".to_string()], // Mismatch: 1 vs 2
            on_delete: None,
            on_update: None,
        };
        let table = Table::new(
            "child",
            vec![ColumnDefinition::new("parent_id", DataType::Integer)],
        )
        .add_foreign_key(fk);
        let result = table.validate("public");
        assert!(matches!(
            result,
            Err(CatalogError::ForeignKeyViolation { .. })
        ));
    }

    #[test]
    fn test_primary_key_column_not_found() {
        let result = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        )
        .primary_key(vec!["nonexistent".to_string()]);
        assert!(matches!(result, Err(CatalogError::ColumnNotFound { .. })));
    }
}
