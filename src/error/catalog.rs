//! Catalog Error Module

use thiserror::Error;

/// Catalog error types
#[derive(Error, Debug)]
pub enum CatalogError {
    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Database not found: {0}")]
    DatabaseNotFound(String),

    #[error("Duplicate name: {0}")]
    DuplicateName(String),

    #[error("Schema error: {0}")]
    SchemaError(String),
}

impl CatalogError {
    pub fn new_table_not_found(name: &str) -> Self {
        CatalogError::TableNotFound(name.to_string())
    }

    pub fn new_column_not_found(name: &str) -> Self {
        CatalogError::ColumnNotFound(name.to_string())
    }
}
