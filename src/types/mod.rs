//! Core Type System Module
//!
//! This module defines the fundamental types and errors for SQLRustGo.
//! All SQL data types and error handling are centralized here.

pub mod error;
pub mod value;

pub use error::{SqlError, SqlResult};
pub use value::Value;

/// Convert a SQL literal string to Value
/// Supports: NULL, TRUE, FALSE, numbers, strings
pub fn parse_sql_literal(s: &str) -> Value {
    let s = s.trim();

    match s.to_uppercase().as_str() {
        "NULL" => Value::Null,
        "TRUE" => Value::Boolean(true),
        "FALSE" => Value::Boolean(false),
        _ if s.starts_with('\'') && s.ends_with('\'') => Value::Text(s[1..s.len() - 1].to_string()),
        _ if s.parse::<i64>().is_ok() => Value::Integer(s.parse().unwrap()),
        _ if s.parse::<f64>().is_ok() => Value::Float(s.parse().unwrap()),
        _ => Value::Text(s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sql_literal() {
        assert_eq!(parse_sql_literal("NULL"), Value::Null);
        assert_eq!(parse_sql_literal("TRUE"), Value::Boolean(true));
        assert_eq!(parse_sql_literal("FALSE"), Value::Boolean(false));
        assert_eq!(parse_sql_literal("42"), Value::Integer(42));
        assert_eq!(parse_sql_literal("3.14"), Value::Float(3.14));
        assert_eq!(
            parse_sql_literal("'hello'"),
            Value::Text("hello".to_string())
        );
    }

    #[test]
    fn test_parse_sql_literal_case_insensitive() {
        // Test case insensitivity
        assert_eq!(parse_sql_literal("null"), Value::Null);
        assert_eq!(parse_sql_literal("true"), Value::Boolean(true));
        assert_eq!(parse_sql_literal("false"), Value::Boolean(false));
    }

    #[test]
    fn test_parse_sql_literal_whitespace() {
        // Test whitespace handling
        assert_eq!(parse_sql_literal("  NULL  "), Value::Null);
        assert_eq!(parse_sql_literal("  42  "), Value::Integer(42));
    }

    #[test]
    fn test_parse_sql_literal_negative() {
        // Test negative numbers
        assert_eq!(parse_sql_literal("-10"), Value::Integer(-10));
        assert_eq!(parse_sql_literal("-3.14"), Value::Float(-3.14));
    }

    #[test]
    fn test_parse_sql_literal_default_text() {
        // Test default to text for unknown values
        let result = parse_sql_literal("unknown");
        assert_eq!(result, Value::Text("unknown".to_string()));
    }
}
