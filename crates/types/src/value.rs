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

impl Value {
    /// Get integer value if this is an Integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert to boolean for predicate evaluation
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Null => false,
            _ => false,
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
    pub fn estimate_memory_size(&self) -> usize {
        match self {
            Value::Null => 0,
            Value::Boolean(_) => 1,
            Value::Integer(_) => 8,
            Value::Float(_) => 8,
            Value::Text(s) => s.capacity(),
            Value::Blob(b) => b.capacity(),
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
    fn test_value_partial_eq() {
        assert_eq!(Value::Null, Value::Null);
        assert_eq!(Value::Boolean(true), Value::Boolean(true));
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_eq!(Value::Float(3.14), Value::Float(3.14));
        assert_eq!(
            Value::Text("hello".to_string()),
            Value::Text("hello".to_string())
        );
        assert_eq!(Value::Blob(vec![1, 2, 3]), Value::Blob(vec![1, 2, 3]));
    }

    #[test]
    fn test_value_partial_eq_not_equal() {
        assert_ne!(Value::Null, Value::Integer(1));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));
        assert_ne!(Value::Integer(1), Value::Integer(2));
        assert_ne!(Value::Float(1.0), Value::Float(2.0));
        assert_ne!(Value::Text("a".to_string()), Value::Text("b".to_string()));
        assert_ne!(Value::Blob(vec![1]), Value::Blob(vec![2]));
    }

    #[test]
    fn test_value_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let v1 = Value::Integer(42);
        let v2 = Value::Integer(42);
        assert_eq!(v1, v2);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        v1.hash(&mut h1);
        v2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_value_hash_null() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let v = Value::Null;
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        assert!(h.finish() >= 0);
    }

    #[test]
    fn test_value_hash_boolean() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let v = Value::Boolean(true);
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        assert!(h.finish() >= 0);
    }

    #[test]
    fn test_value_hash_text() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let v = Value::Text("test".to_string());
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        assert!(h.finish() >= 0);
    }

    #[test]
    fn test_value_hash_blob() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let v = Value::Blob(vec![1, 2, 3]);
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        assert!(h.finish() >= 0);
    }

    #[test]
    fn test_value_hash_float_nan() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let v = Value::Float(f64::NAN);
        let mut h = DefaultHasher::new();
        v.hash(&mut h);
        assert!(h.finish() >= 0);
    }

    #[test]
    fn test_value_to_index_key_integer() {
        assert_eq!(Value::Integer(42).to_index_key(), Some(42));
    }

    #[test]
    fn test_value_to_index_key_text() {
        let key = Value::Text("test".to_string()).to_index_key();
        assert!(key.is_some());
    }

    #[test]
    fn test_value_to_index_key_null() {
        assert_eq!(Value::Null.to_index_key(), None);
    }

    #[test]
    fn test_value_to_index_key_float() {
        assert_eq!(Value::Float(3.14).to_index_key(), None);
    }

    #[test]
    fn test_value_to_index_key_blob() {
        assert_eq!(Value::Blob(vec![1, 2]).to_index_key(), None);
    }

    #[test]
    fn test_value_clone() {
        let v1 = Value::Text("hello".to_string());
        let v2 = v1.clone();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_value_debug() {
        let v = Value::Integer(42);
        let debug = format!("{:?}", v);
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_value_blob_hex_encoding() {
        let blob = Value::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let s = blob.to_sql_string();
        assert!(s.contains("deadbeef"));
    }

    #[test]
    fn test_value_float_infinity() {
        let pos_inf = Value::Float(f64::INFINITY);
        let neg_inf = Value::Float(f64::NEG_INFINITY);
        assert_eq!(pos_inf.to_sql_string(), "inf");
        assert_eq!(neg_inf.to_sql_string(), "-inf");
    }
}
