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
        // Test that SqlResult works as expected
        let ok_result: SqlResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: SqlResult<i32> = Err(SqlError::TableNotFound("test".to_string()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_error_debug() {
        let err = SqlError::ParseError("debug test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ParseError"));
    }
}
