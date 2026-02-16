//! SQL Value types
//!
//! Core data types for SQLRustGo database system.
//!
//! ## Type Mapping
//!
//! | SQL Type | Rust Type | Notes |
//! |----------|-----------|-------|
//! | NULL     | Null      | Missing value |
//! | BOOLEAN  | bool      | TRUE/FALSE |
//! | INTEGER  | i64       | 64-bit signed |
//! | FLOAT    | f64       | 64-bit float |
//! | TEXT     | String    | UTF-8 string |
//! | BLOB     | Vec<u8>   | Binary data |

use serde::{Deserialize, Serialize};
use std::fmt;

/// SQL Value enum representing all supported SQL data types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// NULL value
    Null,
    /// Boolean (TRUE/FALSE)
    Boolean(bool),
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit floating point
    Float(f64),
    /// Text string
    Text(String),
    /// Binary large object
    Blob(Vec<u8>),
}

impl Value {
    /// Get integer value if this is an Integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert Value to String representation
    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "NULL".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => s.clone(),
            Value::Blob(b) => format!("X'{}'", hex::encode(b)),
        }
    }

    /// Get the SQL type name
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "NULL",
            Value::Boolean(_) => "BOOLEAN",
            Value::Integer(_) => "INTEGER",
            Value::Float(_) => "FLOAT",
            Value::Text(_) => "TEXT",
            Value::Blob(_) => "BLOB",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::Null.to_string(), "NULL");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Text("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(Value::Null.type_name(), "NULL");
        assert_eq!(Value::Boolean(true).type_name(), "BOOLEAN");
        assert_eq!(Value::Integer(1).type_name(), "INTEGER");
        assert_eq!(Value::Float(1.0).type_name(), "FLOAT");
        assert_eq!(Value::Text("test".to_string()).type_name(), "TEXT");
        assert_eq!(Value::Blob(vec![0x01, 0x02]).type_name(), "BLOB");
    }

    #[test]
    fn test_value_boolean_false() {
        assert_eq!(Value::Boolean(false).to_string(), "false");
    }

    #[test]
    fn test_value_integer_negative() {
        assert_eq!(Value::Integer(-100).to_string(), "-100");
    }

    #[test]
    fn test_value_blob() {
        let blob = Value::Blob(vec![0x01, 0x02, 0x03]);
        assert_eq!(blob.type_name(), "BLOB");
    }

    #[test]
    fn test_value_as_integer() {
        assert_eq!(Value::Integer(42).as_integer(), Some(42));
        assert_eq!(Value::Null.as_integer(), None);
        assert_eq!(Value::Text("test".to_string()).as_integer(), None);
    }

    #[test]
    fn test_value_as_integer_negative() {
        assert_eq!(Value::Integer(-100).as_integer(), Some(-100));
    }

    #[test]
    fn test_value_blob_to_string() {
        let blob = Value::Blob(vec![0x0a, 0x0b, 0x0c]);
        let s = blob.to_string();
        assert!(s.starts_with("X'"));
        assert!(s.contains("0a0b0c"));
    }

    #[test]
    fn test_value_display_trait() {
        use std::fmt::Write;
        let mut s = String::new();
        write!(&mut s, "{}", Value::Integer(42)).unwrap();
        assert_eq!(s, "42");
    }

    #[test]
    fn test_value_float_precision() {
        assert_eq!(Value::Float(3.14159).to_string(), "3.14159");
    }

    #[test]
    fn test_value_text_special_chars() {
        let text = Value::Text("hello world".to_string());
        assert_eq!(text.to_string(), "hello world");
    }
}
