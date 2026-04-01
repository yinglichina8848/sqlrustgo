// SQLRustGo Common Module - Common types and errors

pub mod connection_pool;
pub mod logging;
pub mod metrics;
pub mod metrics_aggregator;
pub mod network_metrics;

#[derive(Debug)]
pub struct SqlError {
    message: String,
}

impl std::fmt::Display for SqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqlError: {}", self.message)
    }
}

impl std::error::Error for SqlError {}

impl SqlError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

pub type SqlResult<T> = Result<T, SqlError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_sql_error_new() {
        let err = SqlError::new("test error");
        assert_eq!(err.message, "test error");
    }

    #[test]
    fn test_sql_error_display() {
        let err = SqlError::new("test error");
        assert_eq!(err.to_string(), "SqlError: test error");
    }

    #[test]
    fn test_sql_error_debug() {
        let err = SqlError::new("test error");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("SqlError"));
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_sql_error_source() {
        let err = SqlError::new("test error");
        let source = err.source();
        assert!(source.is_none());
    }

    #[test]
    fn test_sql_result_ok() {
        let result: SqlResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_sql_result_err() {
        let result: SqlResult<i32> = Err(SqlError::new("error"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "error");
    }
}
