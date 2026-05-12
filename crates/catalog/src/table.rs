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
    /// Whether this table uses a hidden rowid for tables without explicit PK
    /// When true, rows are assigned a unique auto-incrementing rowid
    pub has_hidden_rowid: bool,
    /// Next auto-incrementing rowid value for tables with hidden rowid
    /// Starts at 1 and increments after each INSERT
    pub next_rowid: u64,
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
            has_hidden_rowid: false,
            next_rowid: 1,
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
        // Explicit PK means no hidden rowid needed
        self.has_hidden_rowid = false;
        Ok(self)
    }

    /// Enable hidden rowid for this table (used when no explicit primary key is defined)
    /// Returns error if table already has a primary key
    pub fn enable_hidden_rowid(mut self) -> CatalogResult<Self> {
        if self.primary_key.is_some() {
            return Err(CatalogError::InvalidOperation(
                "Cannot enable hidden rowid on table with explicit primary key".to_string(),
            ));
        }
        self.has_hidden_rowid = true;
        Ok(self)
    }

    /// Get the next auto-incrementing rowid and increment the counter
    /// Returns None if hidden rowid is not enabled
    pub fn get_next_rowid(&mut self) -> Option<u64> {
        if !self.has_hidden_rowid {
            return None;
        }
        let rowid = self.next_rowid;
        self.next_rowid += 1;
        Some(rowid)
    }

    /// Peek at the next rowid without incrementing
    /// Returns None if hidden rowid is not enabled
    pub fn peek_next_rowid(&self) -> Option<u64> {
        if !self.has_hidden_rowid {
            return None;
        }
        Some(self.next_rowid)
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

    #[test]
    fn test_validate_self_referencing_foreign_key() {
        let fk = ForeignKeyRef {
            referenced_schema: "public".to_string(),
            referenced_table: "employees".to_string(),
            referenced_columns: vec!["id".to_string()],
            columns: vec!["manager_id".to_string()],
            on_delete: Some(ForeignKeyAction::SetNull),
            on_update: Some(ForeignKeyAction::Cascade),
        };
        let table = Table::new(
            "employees",
            vec![
                ColumnDefinition::new("id", DataType::Integer),
                ColumnDefinition::new("name", DataType::Text),
                ColumnDefinition::new("manager_id", DataType::Integer),
            ],
        )
        .add_foreign_key(fk)
        .validate("public");
        assert!(table.is_ok());
    }

    #[test]
    fn test_validate_duplicate_column_in_new() {
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
    fn test_add_column_duplicate_detection() {
        let result = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        )
        .add_column(ColumnDefinition::new("id", DataType::Text));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_composite_primary_key() {
        let table = Table::new(
            "order_items",
            vec![
                ColumnDefinition::new("order_id", DataType::Integer),
                ColumnDefinition::new("item_id", DataType::Integer),
                ColumnDefinition::new("quantity", DataType::Integer),
            ],
        )
        .primary_key(vec!["order_id".to_string(), "item_id".to_string()])
        .unwrap()
        .validate("public");
        assert!(table.is_ok());
    }

    #[test]
    fn test_validate_composite_foreign_key() {
        let fk = ForeignKeyRef {
            referenced_schema: "public".to_string(),
            referenced_table: "orders".to_string(),
            referenced_columns: vec!["order_id".to_string(), "product_id".to_string()],
            columns: vec!["order_id".to_string(), "product_id".to_string()],
            on_delete: Some(ForeignKeyAction::Cascade),
            on_update: Some(ForeignKeyAction::Cascade),
        };
        let table = Table::new(
            "order_items",
            vec![
                ColumnDefinition::new("order_id", DataType::Integer),
                ColumnDefinition::new("product_id", DataType::Integer),
                ColumnDefinition::new("quantity", DataType::Integer),
            ],
        )
        .add_foreign_key(fk)
        .validate("public");
        assert!(table.is_ok());
    }

    #[test]
    fn test_get_column_mut() {
        let mut table = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        );
        table.columns[0].nullable = false;
        assert!(!table.columns[0].nullable);
    }

    #[test]
    fn test_table_with_indices() {
        let table = Table::new(
            "users",
            vec![
                ColumnDefinition::new("id", DataType::Integer),
                ColumnDefinition::new("email", DataType::Text),
            ],
        )
        .add_index(IndexInfo::new(
            "idx_email".to_string(),
            "users".to_string(),
            vec!["email".to_string()],
        ))
        .primary_key(vec!["id".to_string()])
        .unwrap();
        assert_eq!(table.indices.len(), 2);
    }

    // Hidden rowid tests

    #[test]
    fn test_hidden_rowid_disabled_by_default() {
        let table = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        );
        assert!(!table.has_hidden_rowid);
    }

    #[test]
    fn test_hidden_rowid_enabled_for_table_without_pk() {
        let table = Table::new(
            "logs",
            vec![ColumnDefinition::new("message", DataType::Text)],
        )
        .enable_hidden_rowid()
        .unwrap();
        assert!(table.has_hidden_rowid);
        assert!(table.primary_key.is_none());
    }

    #[test]
    fn test_hidden_rowid_rejected_for_table_with_pk() {
        let result = Table::new(
            "users",
            vec![
                ColumnDefinition::new("id", DataType::Integer),
                ColumnDefinition::new("name", DataType::Text),
            ],
        )
        .primary_key(vec!["id".to_string()])
        .unwrap()
        .enable_hidden_rowid();

        assert!(matches!(result, Err(CatalogError::InvalidOperation(_))));
    }

    #[test]
    fn test_explicit_pk_disables_hidden_rowid() {
        let table = Table::new(
            "users",
            vec![
                ColumnDefinition::new("id", DataType::Integer),
                ColumnDefinition::new("email", DataType::Text),
            ],
        )
        .primary_key(vec!["id".to_string()])
        .unwrap();

        // Explicit PK means no hidden rowid needed
        assert!(!table.has_hidden_rowid);
        assert!(table.primary_key.is_some());
    }

    #[test]
    fn test_hidden_rowid_survives_serialization() {
        let table = Table::new("logs", vec![ColumnDefinition::new("msg", DataType::Text)])
            .enable_hidden_rowid()
            .unwrap();

        // Round-trip through JSON serialization
        let json = serde_json::to_string(&table).unwrap();
        let deserialized: Table = serde_json::from_str(&json).unwrap();

        assert!(deserialized.has_hidden_rowid);
        assert!(deserialized.primary_key.is_none());
        assert_eq!(deserialized.name, "logs");
    }

    #[test]
    fn test_get_next_rowid() {
        let mut table = Table::new("logs", vec![ColumnDefinition::new("msg", DataType::Text)])
            .enable_hidden_rowid()
            .unwrap();

        assert_eq!(table.peek_next_rowid(), Some(1));
        assert_eq!(table.get_next_rowid(), Some(1));
        assert_eq!(table.peek_next_rowid(), Some(2));
        assert_eq!(table.get_next_rowid(), Some(2));
        assert_eq!(table.get_next_rowid(), Some(3));
        assert_eq!(table.peek_next_rowid(), Some(4));
    }

    #[test]
    fn test_get_next_rowid_requires_hidden_rowid() {
        let mut table = Table::new(
            "users",
            vec![ColumnDefinition::new("id", DataType::Integer)],
        );

        assert_eq!(table.get_next_rowid(), None);
        assert_eq!(table.peek_next_rowid(), None);
    }

    #[test]
    fn test_next_rowid_survives_serialization() {
        let table = Table::new("logs", vec![ColumnDefinition::new("msg", DataType::Text)])
            .enable_hidden_rowid()
            .unwrap();

        let json = serde_json::to_string(&table).unwrap();
        let deserialized: Table = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.next_rowid, 1);
    }
}
