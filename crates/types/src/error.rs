//! SQL Error types
//! Error handling for SQLRustGo database system

/// MySQL-style SQLSTATE error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SqlState {
    /// 42000: Syntax error or access rule violation
    SyntaxError,
    /// 42S02: Base table or view not found
    NoSuchTable,
    /// 42S22: Column not found
    NoSuchColumn,
    /// 23000: Integrity constraint violation
    IntegrityConstraintViolation,
    /// 22012: Division by zero
    DivisionByZero,
    /// 22003: Numeric value out of range
    NumericValueOutOfRange,
    /// 22005: Error in assignment
    DataException,
    /// 01000: General warning
    Warning,
    /// 00000: Successful completion
    Success,
    /// Unknown error code
    #[default]
    Unknown,
}

impl SqlState {
    pub fn code(&self) -> &'static str {
        match self {
            SqlState::SyntaxError => "42000",
            SqlState::NoSuchTable => "42S02",
            SqlState::NoSuchColumn => "42S22",
            SqlState::IntegrityConstraintViolation => "23000",
            SqlState::DivisionByZero => "22012",
            SqlState::NumericValueOutOfRange => "22003",
            SqlState::DataException => "22005",
            SqlState::Warning => "01000",
            SqlState::Success => "00000",
            SqlState::Unknown => "HY000",
        }
    }
}

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
    #[error("Table '{table}' doesn't exist")]
    TableNotFound { table: String },

    /// Column not found
    #[error("Unknown column '{column}' in '{location}'")]
    ColumnNotFound { column: String, location: String },

    /// Duplicate key error
    #[error("Duplicate entry '{value}' for key '{key}'")]
    DuplicateKey { value: String, key: String },

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Network protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

impl SqlError {
    /// Get MySQL-style SQLSTATE code
    pub fn sqlstate(&self) -> SqlState {
        match self {
            SqlError::ParseError(_) => SqlState::SyntaxError,
            SqlError::ExecutionError(_) => SqlState::Unknown,
            SqlError::TypeMismatch(_) => SqlState::DataException,
            SqlError::DivisionByZero => SqlState::DivisionByZero,
            SqlError::NullValueError(_) => SqlState::DataException,
            SqlError::ConstraintViolation(_) => SqlState::IntegrityConstraintViolation,
            SqlError::TableNotFound { .. } => SqlState::NoSuchTable,
            SqlError::ColumnNotFound { .. } => SqlState::NoSuchColumn,
            SqlError::DuplicateKey { .. } => SqlState::IntegrityConstraintViolation,
            SqlError::IoError(_) => SqlState::Unknown,
            SqlError::ProtocolError(_) => SqlState::Unknown,
        }
    }

    /// Get MySQL-style error number (ERROR 1054, etc.)
    pub fn error_number(&self) -> u16 {
        match self {
            SqlError::ParseError(_) => 1064,
            SqlError::ExecutionError(_) => 1105,
            SqlError::TypeMismatch(_) => 1110,
            SqlError::DivisionByZero => 1365,
            SqlError::NullValueError(_) => 1048,
            SqlError::ConstraintViolation(_) => 1216,
            SqlError::TableNotFound { .. } => 1146,
            SqlError::ColumnNotFound { .. } => 1054,
            SqlError::DuplicateKey { .. } => 1062,
            SqlError::IoError(_) => 1025,
            SqlError::ProtocolError(_) => 1100,
        }
    }

    /// Format as MySQL-style error message
    /// Example: ERROR 1054 (42S22): Unknown column 'xxx' in 'field list'
    pub fn to_mysql_format(&self) -> String {
        let error_num = self.error_number();
        let sqlstate = self.sqlstate();
        let message = self.to_string();

        format!("ERROR {} ({}): {}", error_num, sqlstate.code(), message)
    }
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

        let err = SqlError::TableNotFound {
            table: "users".to_string(),
        };
        assert!(err.to_string().contains("users"));
    }

    #[test]
    fn test_all_error_types() {
        let errors = vec![
            SqlError::ParseError("test".to_string()),
            SqlError::ExecutionError("test".to_string()),
            SqlError::TypeMismatch("test".to_string()),
            SqlError::DivisionByZero,
            SqlError::NullValueError("test".to_string()),
            SqlError::ConstraintViolation("test".to_string()),
            SqlError::TableNotFound {
                table: "test".to_string(),
            },
            SqlError::ColumnNotFound {
                column: "test".to_string(),
                location: "where".to_string(),
            },
            SqlError::DuplicateKey {
                value: "test".to_string(),
                key: "id".to_string(),
            },
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
        let err: SqlError = String::from("test error").into();
        assert!(matches!(err, SqlError::ExecutionError(_)));

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

        let err_result: SqlResult<i32> = Err(SqlError::TableNotFound {
            table: "test".to_string(),
        });
        assert!(err_result.is_err());
    }

    #[test]
    fn test_error_equality() {
        let err1 = SqlError::TableNotFound {
            table: "t1".to_string(),
        };
        let err2 = SqlError::TableNotFound {
            table: "t1".to_string(),
        };
        // Cannot use assert_eq because SqlError doesn't implement PartialEq
        assert!(err1.to_string() == err2.to_string());
        assert!(
            err1.to_string()
                != SqlError::TableNotFound {
                    table: "t2".to_string()
                }
                .to_string()
        );
    }

    #[test]
    fn test_sqlstate_codes() {
        let err = SqlError::TableNotFound {
            table: "users".to_string(),
        };
        assert_eq!(err.sqlstate(), SqlState::NoSuchTable);
        assert_eq!(err.sqlstate().code(), "42S02");

        let err = SqlError::ColumnNotFound {
            column: "id".to_string(),
            location: "select".to_string(),
        };
        assert_eq!(err.sqlstate(), SqlState::NoSuchColumn);
        assert_eq!(err.sqlstate().code(), "42S22");

        let err = SqlError::DuplicateKey {
            value: "1".to_string(),
            key: "id".to_string(),
        };
        assert_eq!(err.sqlstate(), SqlState::IntegrityConstraintViolation);
        assert_eq!(err.sqlstate().code(), "23000");

        let err = SqlError::DivisionByZero;
        assert_eq!(err.sqlstate(), SqlState::DivisionByZero);
        assert_eq!(err.sqlstate().code(), "22012");
    }

    #[test]
    fn test_error_numbers() {
        let err = SqlError::TableNotFound {
            table: "users".to_string(),
        };
        assert_eq!(err.error_number(), 1146);

        let err = SqlError::ColumnNotFound {
            column: "id".to_string(),
            location: "select".to_string(),
        };
        assert_eq!(err.error_number(), 1054);

        let err = SqlError::DuplicateKey {
            value: "1".to_string(),
            key: "id".to_string(),
        };
        assert_eq!(err.error_number(), 1062);

        let err = SqlError::ParseError("syntax error".to_string());
        assert_eq!(err.error_number(), 1064);
    }

    #[test]
    fn test_mysql_format() {
        let err = SqlError::ColumnNotFound {
            column: "xxx".to_string(),
            location: "field list".to_string(),
        };
        let mysql_err = err.to_mysql_format();
        assert!(mysql_err.contains("ERROR 1054"));
        assert!(mysql_err.contains("42S22"));
        assert!(mysql_err.contains("xxx"));

        let err = SqlError::TableNotFound {
            table: "users".to_string(),
        };
        let mysql_err = err.to_mysql_format();
        assert!(mysql_err.contains("ERROR 1146"));
        assert!(mysql_err.contains("42S02"));
        assert!(mysql_err.contains("users"));

        let err = SqlError::DuplicateKey {
            value: "1".to_string(),
            key: "id".to_string(),
        };
        let mysql_err = err.to_mysql_format();
        assert!(mysql_err.contains("ERROR 1062"));
        assert!(mysql_err.contains("23000"));
        assert!(mysql_err.contains("1"));
        assert!(mysql_err.contains("id"));
    }

    #[test]
    fn test_sqlstate_default() {
        let state: SqlState = Default::default();
        assert_eq!(state, SqlState::Unknown);
    }

    #[test]
    fn test_sqlstate_all_codes() {
        assert_eq!(SqlState::SyntaxError.code(), "42000");
        assert_eq!(SqlState::NoSuchTable.code(), "42S02");
        assert_eq!(SqlState::NoSuchColumn.code(), "42S22");
        assert_eq!(SqlState::IntegrityConstraintViolation.code(), "23000");
        assert_eq!(SqlState::DivisionByZero.code(), "22012");
        assert_eq!(SqlState::NumericValueOutOfRange.code(), "22003");
        assert_eq!(SqlState::DataException.code(), "22005");
        assert_eq!(SqlState::Warning.code(), "01000");
        assert_eq!(SqlState::Success.code(), "00000");
        assert_eq!(SqlState::Unknown.code(), "HY000");
    }

    #[test]
    fn test_error_number() {
        let err = SqlError::ParseError("test".to_string());
        assert_eq!(err.error_number(), 1064);

        let err = SqlError::TableNotFound {
            table: "users".to_string(),
        };
        assert_eq!(err.error_number(), 1146);

        let err = SqlError::ColumnNotFound {
            column: "id".to_string(),
            location: "where".to_string(),
        };
        assert_eq!(err.error_number(), 1054);
    }

    #[test]
    fn test_all_error_sqlstate() {
        assert_eq!(
            SqlError::ParseError("x".to_string()).sqlstate(),
            SqlState::SyntaxError
        );
        assert_eq!(
            SqlError::TableNotFound {
                table: "t".to_string()
            }
            .sqlstate(),
            SqlState::NoSuchTable
        );
    }

    #[test]
    fn test_all_error_types_sqlstate() {
        assert_eq!(
            SqlError::ExecutionError("test".to_string()).sqlstate(),
            SqlState::Unknown
        );
        assert_eq!(
            SqlError::TypeMismatch("test".to_string()).sqlstate(),
            SqlState::DataException
        );
        assert_eq!(
            SqlError::NullValueError("test".to_string()).sqlstate(),
            SqlState::DataException
        );
        assert_eq!(
            SqlError::ConstraintViolation("test".to_string()).sqlstate(),
            SqlState::IntegrityConstraintViolation
        );
        assert_eq!(
            SqlError::IoError("test".to_string()).sqlstate(),
            SqlState::Unknown
        );
        assert_eq!(
            SqlError::ProtocolError("test".to_string()).sqlstate(),
            SqlState::Unknown
        );
    }

    #[test]
    fn test_all_error_types_error_number() {
        assert_eq!(
            SqlError::ExecutionError("test".to_string()).error_number(),
            1105
        );
        assert_eq!(
            SqlError::TypeMismatch("test".to_string()).error_number(),
            1110
        );
        assert_eq!(SqlError::DivisionByZero.error_number(), 1365);
        assert_eq!(
            SqlError::NullValueError("test".to_string()).error_number(),
            1048
        );
        assert_eq!(
            SqlError::ConstraintViolation("test".to_string()).error_number(),
            1216
        );
        assert_eq!(SqlError::IoError("test".to_string()).error_number(), 1025);
        assert_eq!(
            SqlError::ProtocolError("test".to_string()).error_number(),
            1100
        );
    }

    #[test]
    fn test_sqlstate_variants() {
        use std::mem::size_of;
        assert_eq!(size_of::<SqlState>(), 1); // Should be small enum
    }

    #[test]
    fn test_sqlstate_clone() {
        let state = SqlState::SyntaxError;
        let cloned = state;
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_sqlstate_debug() {
        let state = SqlState::NoSuchTable;
        let debug = format!("{:?}", state);
        assert!(debug.contains("NoSuchTable"));
    }

    #[test]
    fn test_sql_result_ok() {
        let result: SqlResult<i32> = Ok(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_sql_result_err() {
        let result: SqlResult<i32> = Err(SqlError::ParseError("test".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_mysql_format_all_errors() {
        for err in [
            SqlError::ParseError("test".to_string()),
            SqlError::ExecutionError("test".to_string()),
            SqlError::TypeMismatch("test".to_string()),
            SqlError::DivisionByZero,
            SqlError::NullValueError("test".to_string()),
            SqlError::ConstraintViolation("test".to_string()),
            SqlError::TableNotFound {
                table: "test".to_string(),
            },
            SqlError::ColumnNotFound {
                column: "col".to_string(),
                location: "loc".to_string(),
            },
            SqlError::DuplicateKey {
                value: "v".to_string(),
                key: "k".to_string(),
            },
            SqlError::IoError("test".to_string()),
            SqlError::ProtocolError("test".to_string()),
        ] {
            let mysql_err = err.to_mysql_format();
            assert!(mysql_err.starts_with("ERROR "), "Error: {}", mysql_err);
            assert!(mysql_err.contains("("), "Error: {}", mysql_err);
            assert!(mysql_err.contains("):"), "Error: {}", mysql_err);
        }
    }
}
