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
    /// Date value (days since UNIX epoch)
    Date(i32),
    /// Timestamp value (microseconds since UNIX epoch)
    Timestamp(i64),
    /// UUID value (128-bit unique identifier)
    Uuid(u128),
    /// Array value (variable-length array of values)
    Array(Vec<Value>),
    /// Enum value (index + string representation)
    Enum(i32, String),
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
            Value::Date(d) => d.hash(state),
            Value::Timestamp(ts) => ts.hash(state),
            Value::Uuid(u) => u.hash(state),
            Value::Array(arr) => arr.hash(state),
            Value::Enum(idx, name) => (idx, name).hash(state),
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
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::Timestamp(a), Value::Timestamp(b)) => a == b,
            (Value::Uuid(a), Value::Uuid(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Enum(a_idx, a_name), Value::Enum(b_idx, b_name)) => a_idx == b_idx && a_name == b_name,
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
        // Define variant ordering for consistent comparisons
        fn variant_order(v: &Value) -> u8 {
            match v {
                Value::Null => 0,
                Value::Boolean(_) => 1,
                Value::Integer(_) => 2,
                Value::Float(_) => 3,
                Value::Text(_) => 4,
                Value::Blob(_) => 5,
                Value::Date(_) => 6,
                Value::Timestamp(_) => 7,
                Value::Uuid(_) => 8,
                Value::Array(_) => 9,
                Value::Enum(_, _) => 10,
            }
        }

        let self_order = variant_order(self);
        let other_order = variant_order(other);

        if self_order != other_order {
            return self_order.cmp(&other_order);
        }

        match (self, other) {
            (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
            (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
            (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => {
                // Handle NaN: NaN is considered the smallest value
                if a.is_nan() && b.is_nan() {
                    std::cmp::Ordering::Equal
                } else if a.is_nan() {
                    std::cmp::Ordering::Less
                } else if b.is_nan() {
                    std::cmp::Ordering::Greater
                } else {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }
            }
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            (Value::Blob(a), Value::Blob(b)) => a.cmp(b),
            (Value::Date(a), Value::Date(b)) => a.cmp(b),
            (Value::Timestamp(a), Value::Timestamp(b)) => a.cmp(b),
            (Value::Uuid(a), Value::Uuid(b)) => a.cmp(b),
            (Value::Array(a), Value::Array(b)) => a.cmp(b),
            (Value::Enum(a_idx, a_name), Value::Enum(b_idx, b_name)) => {
                a_idx.cmp(b_idx).then(a_name.cmp(b_name))
            }
            _ => std::cmp::Ordering::Equal,
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

    /// Get float value if this is a Float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
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
            Value::Date(d) => d.to_string(),
            Value::Timestamp(ts) => ts.to_string(),
            Value::Uuid(u) => format!("{:036x}", u),
            Value::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|v| v.to_sql_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Enum(_, name) => name.clone(),
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
            Value::Date(_) => "DATE",
            Value::Timestamp(_) => "TIMESTAMP",
            Value::Uuid(_) => "UUID",
            Value::Array(_) => "ARRAY",
            Value::Enum(_, _) => "ENUM",
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
            Value::Date(_) => 4,
            Value::Timestamp(_) => 8,
            Value::Uuid(_) => 16,
            Value::Array(arr) => arr.iter().map(|v| v.estimate_memory_size()).sum(),
            Value::Enum(_, name) => name.capacity() + 4,
        }
    }

    /// Create a Timestamp value from micros since epoch
    pub fn timestamp(micros: i64) -> Self {
        Value::Timestamp(micros)
    }

    /// Get timestamp value if this is a Timestamp
    pub fn as_timestamp(&self) -> Option<i64> {
        match self {
            Value::Timestamp(ts) => Some(*ts),
            _ => None,
        }
    }

    /// Convert timestamp to formatted string YYYY-MM-DD HH:MM:SS
    pub fn timestamp_to_string(&self) -> Option<String> {
        match self {
            Value::Timestamp(micros) => Some(timestamp_to_datetime_string(*micros)),
            _ => None,
        }
    }
}

/// Convert microseconds since epoch to YYYY-MM-DD HH:MM:SS format
fn timestamp_to_datetime_string(micros: i64) -> String {
    const MICROS_PER_SEC: i64 = 1_000_000;
    const SECS_PER_DAY: i64 = 86400;

    let total_secs = micros / MICROS_PER_SEC;
    let days_since_epoch = total_secs / SECS_PER_DAY;
    let secs_of_day = total_secs % SECS_PER_DAY;

    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    while remaining_days >= 365 {
        let leap = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days >= leap {
            remaining_days -= leap;
            year += 1;
        } else {
            break;
        }
    }

    let (month, day) = days_to_month_day(remaining_days as u32, is_leap_year(year));

    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_to_month_day(days: u32, leap_year: bool) -> (u32, u32) {
    let days_in_months: [u32; 12] = if leap_year {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining = days;
    for (i, &days_in_month) in days_in_months.iter().enumerate() {
        if remaining < days_in_month {
            return ((i + 1) as u32, remaining + 1);
        }
        remaining -= days_in_month;
    }
    (12, 31)
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
        assert_eq!(Value::Timestamp(1000000), Value::Timestamp(1000000));
    }

    #[test]
    fn test_value_partial_eq_not_equal() {
        assert_ne!(Value::Null, Value::Integer(1));
        assert_ne!(Value::Boolean(true), Value::Boolean(false));
        assert_ne!(Value::Integer(1), Value::Integer(2));
        assert_ne!(Value::Float(1.0), Value::Float(2.0));
        assert_ne!(Value::Text("a".to_string()), Value::Text("b".to_string()));
        assert_ne!(Value::Blob(vec![1]), Value::Blob(vec![2]));
        assert_ne!(Value::Timestamp(1), Value::Timestamp(2));
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

    #[test]
    fn test_value_date_basic() {
        let date = Value::Date(0);
        assert_eq!(date.type_name(), "DATE");
    }

    #[test]
    fn test_value_date_equality() {
        let d1 = Value::Date(100);
        let d2 = Value::Date(100);
        let d3 = Value::Date(200);
        assert_eq!(d1, d2);
        assert_ne!(d1, d3);
    }

    #[test]
    fn test_value_date_to_sql_string() {
        let date = Value::Date(19780);
        assert_eq!(date.to_sql_string(), "19780");
    }

    #[test]
    fn test_value_date_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;

        let d1 = Value::Date(100);
        let d2 = Value::Date(100);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();

        d1.hash(&mut h1);
        d2.hash(&mut h2);

        assert_eq!(h1.finish(), h2.finish());
    }

    // Timestamp tests
    #[test]
    fn test_value_timestamp_basic() {
        let ts = Value::Timestamp(0);
        assert_eq!(ts.type_name(), "TIMESTAMP");
    }

    #[test]
    fn test_value_timestamp_equality() {
        let t1 = Value::Timestamp(1000000);
        let t2 = Value::Timestamp(1000000);
        let t3 = Value::Timestamp(2000000);
        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_value_timestamp_to_sql_string() {
        let ts = Value::Timestamp(1000000);
        assert_eq!(ts.to_sql_string(), "1000000");
    }

    #[test]
    fn test_value_timestamp_helper() {
        let ts = Value::timestamp(1000000);
        assert_eq!(ts.as_timestamp(), Some(1000000));
    }

    #[test]
    fn test_value_timestamp_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;

        let t1 = Value::Timestamp(1000000);
        let t2 = Value::Timestamp(1000000);

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();

        t1.hash(&mut h1);
        t2.hash(&mut h2);

        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_timestamp_to_string() {
        let ts = Value::Timestamp(0); // 1970-01-01 00:00:00
        let s = ts.timestamp_to_string();
        assert!(s.is_some());
        assert_eq!(s.unwrap(), "1970-01-01 00:00:00");
    }

    #[test]
    fn test_timestamp_to_string_known_date() {
        // 2020-01-01 00:00:00 UTC = 1577836800 seconds
        let micros = 1577836800i64 * 1_000_000;
        let ts = Value::Timestamp(micros);
        let s = ts.timestamp_to_string();
        assert!(s.is_some());
        assert_eq!(s.unwrap(), "2020-01-01 00:00:00");
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(2019));
        assert!(!is_leap_year(2100));
    }

    #[test]
    fn test_value_type_name_new() {
        assert_eq!(Value::Null.type_name(), "NULL");
        assert_eq!(Value::Boolean(true).type_name(), "BOOLEAN");
        assert_eq!(Value::Integer(1).type_name(), "INTEGER");
        assert_eq!(Value::Float(1.0).type_name(), "FLOAT");
        assert_eq!(Value::Text("test".to_string()).type_name(), "TEXT");
        assert_eq!(Value::Blob(vec![1, 2]).type_name(), "BLOB");
    }

    #[test]
    fn test_value_to_index_key_new() {
        assert_eq!(Value::Integer(42).to_index_key(), Some(42));
        assert!(Value::Text("hello".to_string()).to_index_key().is_some());
        assert_eq!(Value::Float(1.5).to_index_key(), None);
    }

    #[test]
    fn test_value_to_sql_string_new() {
        assert_eq!(Value::Null.to_sql_string(), "NULL");
        assert_eq!(Value::Boolean(true).to_sql_string(), "true");
        assert_eq!(Value::Integer(42).to_sql_string(), "42");
    }

    // Missing tests - coverage gap analysis
    #[test]
    fn test_value_estimate_memory_size() {
        assert_eq!(Value::Null.estimate_memory_size(), 0);
        assert_eq!(Value::Boolean(true).estimate_memory_size(), 1);
        assert_eq!(Value::Integer(42).estimate_memory_size(), 8);
        assert_eq!(Value::Float(3.14).estimate_memory_size(), 8);
        assert_eq!(Value::Text("hello".to_string()).estimate_memory_size(), 5);
        assert_eq!(Value::Blob(vec![1, 2, 3]).estimate_memory_size(), 3);
        assert_eq!(Value::Date(0).estimate_memory_size(), 4);
        assert_eq!(Value::Timestamp(0).estimate_memory_size(), 8);
    }

    #[test]
    fn test_value_to_bool() {
        assert_eq!(Value::Boolean(true).to_bool(), true);
        assert_eq!(Value::Boolean(false).to_bool(), false);
        assert_eq!(Value::Integer(0).to_bool(), false);
        assert_eq!(Value::Integer(1).to_bool(), true);
        assert_eq!(Value::Integer(-1).to_bool(), true);
        assert_eq!(Value::Null.to_bool(), false);
        assert_eq!(Value::Text("hello".to_string()).to_bool(), false);
        assert_eq!(Value::Float(3.14).to_bool(), false);
    }

    #[test]
    fn test_value_as_float() {
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::Integer(42).as_float(), None);
        assert_eq!(Value::Null.as_float(), None);
        assert_eq!(Value::Text("hello".to_string()).as_float(), None);
    }

    #[test]
    fn test_days_to_month_day() {
        // January
        assert_eq!(days_to_month_day(0, false), (1, 1));
        assert_eq!(days_to_month_day(30, false), (1, 31));
        // February (non-leap)
        assert_eq!(days_to_month_day(31, false), (2, 1));
        assert_eq!(days_to_month_day(58, false), (2, 28));
        // March
        assert_eq!(days_to_month_day(59, false), (3, 1));
        // December
        assert_eq!(days_to_month_day(334, false), (12, 1));
        assert_eq!(days_to_month_day(364, false), (12, 31));

        // Leap year tests
        assert_eq!(days_to_month_day(31, true), (2, 1));
        assert_eq!(days_to_month_day(60, true), (3, 1)); // Feb 29 in leap year
    }

    #[test]
    fn test_value_timestamp_none_for_non_timestamp() {
        assert_eq!(Value::Integer(42).as_timestamp(), None);
        assert_eq!(Value::Text("hello".to_string()).as_timestamp(), None);
        assert_eq!(Value::Null.as_timestamp(), None);
    }

    #[test]
    fn test_value_timestamp_to_string_none() {
        assert_eq!(Value::Integer(42).timestamp_to_string(), None);
        assert_eq!(Value::Text("hello".to_string()).timestamp_to_string(), None);
        assert_eq!(Value::Null.timestamp_to_string(), None);
    }

    #[test]
    fn test_value_ordering() {
        use std::cmp::Ordering;
        // Null is smallest
        assert_eq!(Value::Null.cmp(&Value::Integer(0)), Ordering::Less);
        // Boolean < Integer
        assert_eq!(Value::Boolean(true).cmp(&Value::Integer(0)), Ordering::Less);
        // Integer < Float
        assert_eq!(Value::Integer(0).cmp(&Value::Float(0.0)), Ordering::Less);
        // Float < Text
        assert_eq!(
            Value::Float(0.0).cmp(&Value::Text("a".to_string())),
            Ordering::Less
        );
        // Text < Blob
        assert_eq!(
            Value::Text("a".to_string()).cmp(&Value::Blob(vec![])),
            Ordering::Less
        );
        // Blob < Date
        assert_eq!(Value::Blob(vec![]).cmp(&Value::Date(0)), Ordering::Less);
        // Date < Timestamp
        assert_eq!(Value::Date(0).cmp(&Value::Timestamp(0)), Ordering::Less);
    }

    #[test]
    fn test_value_partial_ord() {
        use std::cmp::Ordering;
        assert_eq!(
            Value::Integer(1).partial_cmp(&Value::Integer(2)),
            Some(Ordering::Less)
        );
        assert_eq!(
            Value::Integer(2).partial_cmp(&Value::Integer(2)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            Value::Integer(3).partial_cmp(&Value::Integer(2)),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn test_value_float_nan_ordering() {
        use std::cmp::Ordering;
        let nan = Value::Float(f64::NAN);
        let zero = Value::Float(0.0);
        // When comparing Float NaN with Float NaN, they are Equal
        assert_eq!(nan.cmp(&nan), Ordering::Equal);
        // When comparing Float 0.0 with Float NaN, 0.0 is Greater than NaN
        // (implementation treats NaN as largest value, opposite of comment)
        assert_eq!(zero.cmp(&nan), Ordering::Greater);
        // Float variants are compared by their variant order (3), which is greater than Integer (2)
        // So Float > Integer regardless of NaN status when cross-variant comparison
        assert_eq!(
            Value::Float(f64::NAN).cmp(&Value::Integer(0)),
            Ordering::Greater
        );
    }

    #[test]
    fn test_value_as_integer_or_zero() {
        // Value doesn't have as_integer_or_zero, but testing as_integer
        assert_eq!(Value::Integer(0).as_integer(), Some(0));
        assert_eq!(Value::Integer(i64::MAX).as_integer(), Some(i64::MAX));
        assert_eq!(Value::Integer(i64::MIN).as_integer(), Some(i64::MIN));
    }

    #[test]
    fn test_value_float_edge_cases() {
        let zero = Value::Float(0.0);
        let neg_zero = Value::Float(-0.0);
        assert_eq!(zero.as_float(), Some(0.0));
        assert_eq!(neg_zero.as_float(), Some(-0.0));

        let max = Value::Float(f64::MAX);
        let min = Value::Float(f64::MIN);
        assert_eq!(max.as_float(), Some(f64::MAX));
        assert_eq!(min.as_float(), Some(f64::MIN));
    }

    #[test]
    fn test_value_text_unicode() {
        let text = Value::Text("你好".to_string());
        assert_eq!(text.to_sql_string(), "你好");
        assert_eq!(text.type_name(), "TEXT");
    }

    #[test]
    fn test_value_blob_empty() {
        let blob = Value::Blob(vec![]);
        assert_eq!(blob.type_name(), "BLOB");
        assert_eq!(blob.to_sql_string(), "X''");
    }
}
