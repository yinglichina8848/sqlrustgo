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
}

/// Result type alias for SQL operations
pub type SqlResult<T> = Result<T, SqlError>;

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
}
