//! SQL Value types
//! Core data types for SQLRustGo database system

use std::fmt;

/// SQL Value enum representing all supported SQL data types
#[derive(Debug, Clone, PartialEq)]
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
}
