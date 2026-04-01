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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_not_found() {
        let err = CatalogError::SchemaNotFound("test_schema".to_string());
        assert!(err.to_string().contains("test_schema"));
    }

    #[test]
    fn test_table_not_found() {
        let err = CatalogError::TableNotFound {
            schema: "public".to_string(),
            table: "users".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("public"));
        assert!(msg.contains("users"));
    }

    #[test]
    fn test_column_not_found() {
        let err = CatalogError::ColumnNotFound {
            schema: "public".to_string(),
            table: "users".to_string(),
            column: "email".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("email"));
    }

    #[test]
    fn test_duplicate_schema() {
        let err = CatalogError::DuplicateSchema("test_schema".to_string());
        assert!(err.to_string().contains("test_schema"));
    }

    #[test]
    fn test_duplicate_table() {
        let err = CatalogError::DuplicateTable {
            schema: "public".to_string(),
            table: "users".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("public"));
        assert!(msg.contains("users"));
    }

    #[test]
    fn test_duplicate_column() {
        let err = CatalogError::DuplicateColumn {
            schema: "public".to_string(),
            table: "users".to_string(),
            column: "id".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("id"));
    }

    #[test]
    fn test_invalid_primary_key() {
        let err = CatalogError::InvalidPrimaryKey("NULL in primary key".to_string());
        assert!(err.to_string().contains("NULL"));
    }

    #[test]
    fn test_foreign_key_violation() {
        let err = CatalogError::ForeignKeyViolation {
            schema: "public".to_string(),
            table: "orders".to_string(),
            column: "user_id".to_string(),
            referenced: "users".to_string(),
            reason: "no matching primary key".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("orders"));
        assert!(msg.contains("users"));
    }

    #[test]
    fn test_cyclic_dependency() {
        let err = CatalogError::CyclicDependency("table_a".to_string());
        assert!(err.to_string().contains("table_a"));
    }

    #[test]
    fn test_invariant_violation() {
        let err = CatalogError::InvariantViolation("schema corrupted".to_string());
        assert!(err.to_string().contains("schema"));
    }

    #[test]
    fn test_serialization_error() {
        let err = CatalogError::SerializationError("invalid format".to_string());
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn test_catalog_result_ok() {
        let result: CatalogResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_catalog_result_err() {
        let result: CatalogResult<i32> = Err(CatalogError::SchemaNotFound("test".to_string()));
        assert!(result.is_err());
    }
}
