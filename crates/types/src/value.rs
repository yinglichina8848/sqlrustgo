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
//! | BLOB     | `Vec<u8>` | Binary data |

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

/// SQL Value enum representing all supported SQL data types
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => 0.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => {
                if f.is_nan() {
                    0.hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Value::Text(s) => s.hash(state),
            Value::Blob(b) => b.hash(state),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Blob(a), Value::Blob(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Null, _) => Ordering::Greater,
            (_, Value::Null) => Ordering::Less,
            (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            (Value::Blob(a), Value::Blob(b)) => a.cmp(b),
            _ => Ordering::Equal,
        }
    }
}

impl Value {
    /// Get integer value if this is an Integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert Value to SQL string representation
    pub fn to_sql_string(&self) -> String {
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

    /// Convert value to index key (i64)
    /// Used for B+Tree index key extraction
    pub fn to_index_key(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Text(s) => {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                s.hash(&mut hasher);
                Some(hasher.finish() as i64)
            }
            _ => None,
        }
    }

    /// Estimate memory size in bytes
    /// Used for query cache memory accounting
    pub fn estimate_memory_size(&self) -> usize {
        match self {
            Value::Null => 0,
            Value::Boolean(_) => 1,
            Value::Integer(_) => 8,
            Value::Float(_) => 8,
            Value::Text(s) => s.len(),
            Value::Blob(b) => b.len(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_sql_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_value_to_string() {
        assert_eq!(Value::Null.to_sql_string(), "NULL");
        assert_eq!(Value::Boolean(true).to_sql_string(), "true");
        assert_eq!(Value::Integer(42).to_sql_string(), "42");
        assert_eq!(Value::Float(3.14).to_sql_string(), "3.14");
        assert_eq!(Value::Text("hello".to_string()).to_sql_string(), "hello");
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
        assert_eq!(Value::Boolean(false).to_sql_string(), "false");
    }

    #[test]
    fn test_value_integer_negative() {
        assert_eq!(Value::Integer(-100).to_sql_string(), "-100");
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
        let s = blob.to_sql_string();
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
    #[allow(clippy::approx_constant)]
    fn test_value_float_precision() {
        assert_eq!(Value::Float(3.14159).to_sql_string(), "3.14159");
    }

    #[test]
    fn test_value_text_special_chars() {
        let text = Value::Text("hello world".to_string());
        assert_eq!(text.to_sql_string(), "hello world");
    }

    #[test]
    fn test_value_to_index_key_integer() {
        assert_eq!(Value::Integer(42).to_index_key(), Some(42));
        assert_eq!(Value::Integer(-100).to_index_key(), Some(-100));
        assert_eq!(Value::Integer(0).to_index_key(), Some(0));
    }

    #[test]
    fn test_value_to_index_key_text() {
        let text_hash = Value::Text("hello".to_string()).to_index_key();
        assert!(text_hash.is_some());
        let text_hash2 = Value::Text("hello".to_string()).to_index_key();
        assert_eq!(text_hash, text_hash2);
    }

    #[test]
    fn test_value_to_index_key_invalid() {
        assert_eq!(Value::Null.to_index_key(), None);
        assert_eq!(Value::Float(3.14).to_index_key(), None);
        assert_eq!(Value::Blob(vec![0x01]).to_index_key(), None);
    }

    #[test]
    fn test_value_sql_string_null() {
        assert_eq!(Value::Null.to_sql_string(), "NULL");
    }

    #[test]
    fn test_value_sql_string_boolean() {
        assert_eq!(Value::Boolean(true).to_sql_string(), "true");
        assert_eq!(Value::Boolean(false).to_sql_string(), "false");
    }

    #[test]
    fn test_value_sql_string_integer() {
        assert_eq!(Value::Integer(0).to_sql_string(), "0");
        assert_eq!(Value::Integer(999).to_sql_string(), "999");
    }

    #[test]
    fn test_value_sql_string_float() {
        assert_eq!(Value::Float(0.0).to_sql_string(), "0");
        assert_eq!(Value::Float(1.5).to_sql_string(), "1.5");
    }

    #[test]
    fn test_value_sql_string_text() {
        assert_eq!(Value::Text("".to_string()).to_sql_string(), "");
        assert_eq!(Value::Text("abc".to_string()).to_sql_string(), "abc");
    }

    #[test]
    fn test_value_sql_string_blob() {
        let blob = Value::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let s = blob.to_sql_string();
        assert!(s.starts_with("X'"));
        assert!(s.ends_with("'"));
    }

    #[test]
    fn test_value_clone() {
        let v1 = Value::Integer(42);
        let v2 = v1.clone();
        assert_eq!(v1, v2);
        let v3 = Value::Text("test".to_string());
        let v4 = v3.clone();
        assert_eq!(v3, v4);
    }

    #[test]
    fn test_value_eq() {
        assert_eq!(Value::Null, Value::Null);
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_eq!(Value::Float(3.14), Value::Float(3.14));
        assert_eq!(
            Value::Text("test".to_string()),
            Value::Text("test".to_string())
        );
        assert_eq!(Value::Blob(vec![0x01]), Value::Blob(vec![0x01]));
    }

    #[test]
    fn test_value_ne() {
        assert_ne!(Value::Null, Value::Integer(0));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));
        assert_ne!(Value::Integer(1), Value::Integer(2));
        assert_ne!(Value::Text("a".to_string()), Value::Text("b".to_string()));
        assert_ne!(Value::Blob(vec![0x01]), Value::Blob(vec![0x02]));
    }
}
