// SQLRustGo Types Module
// Core data types for SQLRustGo

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
        assert_eq!(parse_sql_literal("null"), Value::Null);
        assert_eq!(parse_sql_literal("true"), Value::Boolean(true));
        assert_eq!(parse_sql_literal("false"), Value::Boolean(false));
    }

    #[test]
    fn test_parse_sql_literal_negative() {
        assert_eq!(parse_sql_literal("-42"), Value::Integer(-42));
        assert_eq!(parse_sql_literal("-3.14"), Value::Float(-3.14));
    }

    #[test]
    fn test_parse_sql_literal_whitespace() {
        assert_eq!(parse_sql_literal("  NULL  "), Value::Null);
        assert_eq!(parse_sql_literal("  42  "), Value::Integer(42));
    }

    #[test]
    fn test_parse_sql_literal_string_with_quotes() {
        assert_eq!(
            parse_sql_literal("'hello world'"),
            Value::Text("hello world".to_string())
        );
        assert_eq!(
            parse_sql_literal("'test''s'"),
            Value::Text("test''s".to_string())
        );
    }

    #[test]
    fn test_parse_sql_literal_unquoted_text() {
        assert_eq!(parse_sql_literal("abc"), Value::Text("abc".to_string()));
        assert_eq!(
            parse_sql_literal("unknown"),
            Value::Text("unknown".to_string())
        );
    }

    #[test]
    fn test_parse_sql_literal_float_edge_cases() {
        assert_eq!(parse_sql_literal("0.0"), Value::Float(0.0));
        assert_eq!(parse_sql_literal("-0.001"), Value::Float(-0.001));
        assert_eq!(parse_sql_literal("1e10"), Value::Float(10000000000.0));
    }

    #[test]
    fn test_parse_sql_literal_large_integer() {
        assert_eq!(
            parse_sql_literal("9223372036854775807"),
            Value::Integer(9223372036854775807i64)
        );
    }

    #[test]
    fn test_parse_sql_literal_zero() {
        assert_eq!(parse_sql_literal("0"), Value::Integer(0));
    }
}
