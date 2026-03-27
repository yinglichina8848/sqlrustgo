//! Catalog error types

use thiserror::Error;

/// Catalog-specific errors
#[derive(Error, Debug)]
pub enum CatalogError {
    /// Schema not found
    #[error("Schema '{0}' not found")]
    SchemaNotFound(String),

    /// Table not found
    #[error("Table '{table}' not found in schema '{schema}'")]
    TableNotFound { schema: String, table: String },

    /// Column not found
    #[error("Column '{column}' not found in table '{schema}.{table}'")]
    ColumnNotFound {
        schema: String,
        table: String,
        column: String,
    },

    /// Duplicate schema name
    #[error("Schema '{0}' already exists")]
    DuplicateSchema(String),

    /// Duplicate table name
    #[error("Table '{table}' already exists in schema '{schema}'")]
    DuplicateTable { schema: String, table: String },

    /// Duplicate column name
    #[error("Column '{column}' already exists in table '{schema}.{table}'")]
    DuplicateColumn {
        schema: String,
        table: String,
        column: String,
    },

    /// Invalid primary key
    #[error("Invalid primary key: {0}")]
    InvalidPrimaryKey(String),

    /// Foreign key reference failed
    #[error(
        "Foreign key reference to '{referenced}' in table '{schema}.{table}' failed: {reason}"
    )]
    ForeignKeyViolation {
        schema: String,
        table: String,
        column: String,
        referenced: String,
        reason: String,
    },

    /// Cyclic foreign key dependency detected
    #[error("Cyclic foreign key dependency detected involving table '{0}'")]
    CyclicDependency(String),

    /// Invariant violation
    #[error("Catalog invariant violation: {0}")]
    InvariantViolation(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for catalog operations
pub type CatalogResult<T> = Result<T, CatalogError>;
