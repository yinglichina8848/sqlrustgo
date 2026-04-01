//! SQL data types for catalog definitions
//!
//! This module defines the data types that can be used in column definitions.
//! These are the logical types used in the catalog, separate from the
//! physical storage types in the Value enum.

use serde::{Deserialize, Serialize};
use std::fmt;

/// SQL data types supported by the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DataType {
    /// NULL type
    Null,
    /// Boolean type (TRUE/FALSE)
    Boolean,
    /// 64-bit signed integer
    Integer,
    /// 64-bit floating point
    Float,
    /// Variable-length text
    #[default]
    Text,
    /// Binary large object
    Blob,
    /// Date type (days since epoch)
    Date,
    /// Timestamp type (microseconds since epoch)
    Timestamp,
    /// UUID type (128-bit unique identifier)
    Uuid,
    /// Array type (variable-length array of elements)
    Array,
    /// Enum type (enumeration with allowed values)
    Enum,
}

impl DataType {
    /// Get the SQL name for this data type
    pub fn sql_name(&self) -> &'static str {
        match self {
            DataType::Null => "NULL",
            DataType::Boolean => "BOOLEAN",
            DataType::Integer => "INTEGER",
            DataType::Float => "FLOAT",
            DataType::Text => "TEXT",
            DataType::Blob => "BLOB",
            DataType::Date => "DATE",
            DataType::Timestamp => "TIMESTAMP",
            DataType::Uuid => "UUID",
            DataType::Array => "ARRAY",
            DataType::Enum => "ENUM",
        }
    }

    /// Parse a SQL type name into a DataType
    /// Note: ARRAY<T> and ENUM(...) are handled specially in the parser
    pub fn parse_sql_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "NULL" => Some(DataType::Null),
            "BOOLEAN" | "BOOL" => Some(DataType::Boolean),
            "INTEGER" | "INT" | "INT64" | "BIGINT" => Some(DataType::Integer),
            "FLOAT" | "DOUBLE" | "REAL" => Some(DataType::Float),
            "TEXT" | "VARCHAR" | "CHAR" | "STRING" => Some(DataType::Text),
            "BLOB" | "BINARY" | "VARBINARY" => Some(DataType::Blob),
            "DATE" => Some(DataType::Date),
            "TIMESTAMP" | "DATETIME" => Some(DataType::Timestamp),
            "UUID" => Some(DataType::Uuid),
            "ARRAY" => Some(DataType::Array),
            "ENUM" => Some(DataType::Enum),
            _ => None,
        }
    }

    /// Check if this type can be used in a primary key
    pub fn is_valid_for_primary_key(&self) -> bool {
        matches!(
            self,
            DataType::Integer | DataType::Text | DataType::Boolean | DataType::Date | DataType::Uuid
        )
    }

    /// Check if this type supports ordering comparisons
    pub fn is_orderable(&self) -> bool {
        !matches!(self, DataType::Blob | DataType::Array | DataType::Enum)
    }

    /// Check if this type supports equality comparisons
    pub fn is_equatable(&self) -> bool {
        true
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sql_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_sql_name() {
        assert_eq!(DataType::Integer.sql_name(), "INTEGER");
        assert_eq!(DataType::Text.sql_name(), "TEXT");
        assert_eq!(DataType::Boolean.sql_name(), "BOOLEAN");
    }

    #[test]
    fn test_parse_sql_name() {
        assert_eq!(DataType::parse_sql_name("INTEGER"), Some(DataType::Integer));
        assert_eq!(DataType::parse_sql_name("INT"), Some(DataType::Integer));
        assert_eq!(DataType::parse_sql_name("VARCHAR"), Some(DataType::Text));
        assert_eq!(DataType::parse_sql_name("BLOB"), Some(DataType::Blob));
        assert_eq!(DataType::parse_sql_name("UNKNOWN"), None);
    }

    #[test]
    fn test_valid_for_primary_key() {
        assert!(DataType::Integer.is_valid_for_primary_key());
        assert!(DataType::Text.is_valid_for_primary_key());
        assert!(!DataType::Blob.is_valid_for_primary_key());
        assert!(!DataType::Float.is_valid_for_primary_key());
    }

    #[test]
    fn test_is_orderable() {
        assert!(DataType::Integer.is_orderable());
        assert!(DataType::Text.is_orderable());
        assert!(!DataType::Blob.is_orderable());
    }

    #[test]
    fn test_default() {
        assert_eq!(DataType::default(), DataType::Text);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DataType::Integer), "INTEGER");
        assert_eq!(format!("{}", DataType::Text), "TEXT");
    }
}
