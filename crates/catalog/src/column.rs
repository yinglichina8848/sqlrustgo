//! Column definition for catalog tables

use crate::data_type::DataType;
use crate::error::CatalogError;
use crate::error::CatalogResult;
use serde::{Deserialize, Serialize};
use sqlrustgo_types::Value;

/// Column definition describing a single column in a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: DataType,
    /// Whether the column allows NULL values
    pub nullable: bool,
    /// Whether the column has a uniqueness constraint
    pub is_unique: bool,
    /// Default value expression (parsed as Value)
    pub default_value: Option<Value>,
    /// Primary key position (Some(index) if part of primary key)
    pub primary_key_position: Option<usize>,
}

impl ColumnDefinition {
    /// Create a new column definition
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
            is_unique: false,
            default_value: None,
            primary_key_position: None,
        }
    }

    /// Create a NOT NULL column
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Create a UNIQUE column
    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    /// Set as primary key with given position
    pub fn primary_key(mut self, position: usize) -> Self {
        self.primary_key_position = Some(position);
        self.nullable = false;
        self.is_unique = true;
        self
    }

    /// Set a default value
    pub fn default_value(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Check if this column is part of a primary key
    pub fn is_primary_key(&self) -> bool {
        self.primary_key_position.is_some()
    }

    /// Validate the column definition
    pub fn validate(&self, schema_name: &str, table_name: &str) -> CatalogResult<()> {
        if self.name.is_empty() {
            return Err(CatalogError::InvariantViolation(
                "Column name cannot be empty".to_string(),
            ));
        }

        // Primary key columns cannot be nullable
        if self.is_primary_key() && self.nullable {
            return Err(CatalogError::InvalidPrimaryKey(format!(
                "Primary key column '{}' in '{}.{}' cannot be nullable",
                self.name, schema_name, table_name
            )));
        }

        // If default value is set, it should match the data type
        if let Some(ref default) = self.default_value {
            if !Self::value_matches_type(default, &self.data_type) {
                return Err(CatalogError::InvariantViolation(format!(
                    "Default value type mismatch for column '{}' in '{}.{}': expected {:?}",
                    self.name, schema_name, table_name, self.data_type
                )));
            }
        }

        Ok(())
    }

    /// Check if a value matches the column's data type
    fn value_matches_type(value: &Value, data_type: &DataType) -> bool {
        match (value, data_type) {
            (Value::Null, _) => true, // NULL is compatible with any type
            (Value::Boolean(_), DataType::Boolean) => true,
            (Value::Integer(_), DataType::Integer) => true,
            (Value::Float(_), DataType::Float) => true,
            (Value::Text(_), DataType::Text) => true,
            (Value::Blob(_), DataType::Blob) => true,
            (Value::Date(_), DataType::Date) => true,
            (Value::Timestamp(_), DataType::Timestamp) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_creation() {
        let col = ColumnDefinition::new("id", DataType::Integer);
        assert_eq!(col.name, "id");
        assert_eq!(col.data_type, DataType::Integer);
        assert!(col.nullable);
        assert!(!col.is_primary_key());
    }

    #[test]
    fn test_column_not_null() {
        let col = ColumnDefinition::new("id", DataType::Integer).not_null();
        assert!(!col.nullable);
    }

    #[test]
    fn test_column_primary_key() {
        let col = ColumnDefinition::new("id", DataType::Integer).primary_key(0);
        assert!(col.is_primary_key());
        assert!(!col.nullable);
        assert!(col.is_unique);
        assert_eq!(col.primary_key_position, Some(0));
    }

    #[test]
    fn test_validate_empty_name() {
        let col = ColumnDefinition::new("", DataType::Integer);
        let result = col.validate("public", "users");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_nullable_pk() {
        let col = ColumnDefinition::new("id", DataType::Integer)
            .primary_key(0)
            .not_null(); // Explicitly not nullable is OK
        assert!(col.validate("public", "users").is_ok());
    }

    #[test]
    fn test_value_matches_type() {
        assert!(ColumnDefinition::new("id", DataType::Integer)
            .default_value(Value::Integer(42))
            .validate("public", "users")
            .is_ok());

        // NULL is always valid
        assert!(ColumnDefinition::new("id", DataType::Integer)
            .default_value(Value::Null)
            .validate("public", "users")
            .is_ok());
    }
}
