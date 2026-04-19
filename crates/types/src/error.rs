//! SQL Error types
//! Error handling for SQLRustGo database system

/// SQL Error enum representing all possible error types
#[derive(thiserror::Error, Debug)]
pub enum SqlError {
    /// Syntax error during parsing
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Execution error during query processing
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Type mismatch error
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Null value error (operation on NULL)
    #[error("Null value error: {0}")]
    NullValueError(String),

    /// Constraint violation
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Table not found
    #[error("Table not found: {0}")]
    TableNotFound(String),

    /// Column not found
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Duplicate key error
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Network protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    TimeoutError(String),

    /// Overflow error (numeric overflow)
    #[error("Overflow: {0}")]
    OverflowError(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthError(String),
}

/// Result type alias for SQL operations
pub type SqlResult<T> = Result<T, SqlError>;

impl From<String> for SqlError {
    fn from(s: String) -> Self {
        SqlError::ExecutionError(s)
    }
}

impl From<&str> for SqlError {
    fn from(s: &str) -> Self {
        SqlError::ExecutionError(s.to_string())
    }
}

impl From<std::io::Error> for SqlError {
    fn from(e: std::io::Error) -> Self {
        SqlError::IoError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = SqlError::ParseError("invalid syntax".to_string());
        assert!(err.to_string().contains("Parse error"));

        let err = SqlError::TableNotFound("users".to_string());
        assert!(err.to_string().contains("Table not found"));
    }

    #[test]
    fn test_all_error_types() {
        // Test all error variants
        let errors = vec![
            SqlError::ParseError("test".to_string()),
            SqlError::ExecutionError("test".to_string()),
            SqlError::TypeMismatch("test".to_string()),
            SqlError::DivisionByZero,
            SqlError::NullValueError("test".to_string()),
            SqlError::ConstraintViolation("test".to_string()),
            SqlError::TableNotFound("test".to_string()),
            SqlError::ColumnNotFound("test".to_string()),
            SqlError::DuplicateKey("test".to_string()),
            SqlError::IoError("test".to_string()),
            SqlError::ProtocolError("test".to_string()),
            SqlError::TimeoutError("test".to_string()),
            SqlError::OverflowError("test".to_string()),
            SqlError::AuthError("test".to_string()),
        ];

        for err in errors {
            let msg = err.to_string();
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn test_error_from_string() {
        // Test From<String>
        let err: SqlError = String::from("test error").into();
        assert!(matches!(err, SqlError::ExecutionError(_)));

        // Test From<&str>
        let err: SqlError = "test error".into();
        assert!(matches!(err, SqlError::ExecutionError(_)));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: SqlError = io_err.into();
        assert!(matches!(err, SqlError::IoError(_)));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_sql_result_alias() {
        let ok_result: SqlResult<i32> = Ok(42);
        assert_eq!(ok_result.ok(), Some(42));

        let err_result: SqlResult<i32> = Err(SqlError::TableNotFound("test".to_string()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_error_debug() {
        let err = SqlError::ParseError("debug test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ParseError"));
    }

    #[test]
    fn test_error_display_parse() {
        let err = SqlError::ParseError("syntax error".to_string());
        let display = format!("{}", err);
        assert!(display.contains("syntax error"));
    }

    #[test]
    fn test_error_display_execution() {
        let err = SqlError::ExecutionError("query failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("query failed"));
    }

    #[test]
    fn test_error_display_type_mismatch() {
        let err = SqlError::TypeMismatch("expected int".to_string());
        let display = format!("{}", err);
        assert!(display.contains("expected int"));
    }

    #[test]
    fn test_error_display_division_by_zero() {
        let err = SqlError::DivisionByZero;
        let display = format!("{}", err);
        assert!(display.contains("Division by zero"));
    }

    #[test]
    fn test_error_display_null_value() {
        let err = SqlError::NullValueError("cannot be null".to_string());
        let display = format!("{}", err);
        assert!(display.contains("cannot be null"));
    }

    #[test]
    fn test_error_display_constraint() {
        let err = SqlError::ConstraintViolation("unique key".to_string());
        let display = format!("{}", err);
        assert!(display.contains("unique key"));
    }

    #[test]
    fn test_error_display_table_not_found() {
        let err = SqlError::TableNotFound("users".to_string());
        let display = format!("{}", err);
        assert!(display.contains("users"));
    }

    #[test]
    fn test_error_display_column_not_found() {
        let err = SqlError::ColumnNotFound("id".to_string());
        let display = format!("{}", err);
        assert!(display.contains("id"));
    }

    #[test]
    fn test_error_display_duplicate_key() {
        let err = SqlError::DuplicateKey("email".to_string());
        let display = format!("{}", err);
        assert!(display.contains("email"));
    }

    #[test]
    fn test_error_display_io() {
        let err = SqlError::IoError("file not found".to_string());
        let display = format!("{}", err);
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_error_display_protocol() {
        let err = SqlError::ProtocolError("invalid packet".to_string());
        let display = format!("{}", err);
        assert!(display.contains("invalid packet"));
    }

    #[test]
    fn test_error_display_timeout() {
        let err = SqlError::TimeoutError("connection timeout".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Timeout"));
    }

    #[test]
    fn test_error_display_overflow() {
        let err = SqlError::OverflowError("i32 overflow".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Overflow"));
    }

    #[test]
    fn test_error_display_auth() {
        let err = SqlError::AuthError("invalid credentials".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Authentication"));
    }
}
