// SQLRustGo Common Module - Common types and errors

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub size: usize,
    pub timeout_ms: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            size: 4, // default pool size
            timeout_ms: 5000,
        }
    }
}

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
        let err = SqlError::new("display test");
        let display = format!("{}", err);
        assert!(display.contains("display test"));
    }

    #[test]
    fn test_sql_error_debug() {
        let err = SqlError::new("debug test");
        let debug = format!("{:?}", err);
        assert!(debug.contains("debug test"));
    }

    #[test]
    fn test_sql_error_from_string() {
        let err = SqlError::new(String::from("from string"));
        assert_eq!(err.message, "from string");
    }

    #[test]
    fn test_sql_error_from_str() {
        let err = SqlError::new("from &str");
        assert_eq!(err.message, "from &str");
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
        assert!(result.unwrap_err().message.contains("error"));
    }

    #[test]
    fn test_sql_result_into() {
        let err: SqlError = SqlError::new("test");
        let result: SqlResult<i32> = Err(err);
        assert!(result.is_err());
    }

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert!(config.size > 0);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig {
            size: 10,
            timeout_ms: 10000,
        };
        assert_eq!(config.size, 10);
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_pool_config_clone() {
        let config = PoolConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.size, config.size);
        assert_eq!(cloned.timeout_ms, config.timeout_ms);
    }

    #[test]
    fn test_pool_config_debug() {
        let config = PoolConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("PoolConfig"));
    }

    #[test]
    fn test_sql_error_source() {
        let err = SqlError::new("source test");
        let source = err.source();
        assert!(source.is_none());
    }

    #[test]
    fn test_sql_error_clone() {
        let err = SqlError::new("clone test");
        let cloned = err.clone();
        assert_eq!(cloned.message, err.message);
    }

    #[test]
    fn test_pool_config_modified() {
        let mut config = PoolConfig::default();
        config.size = 20;
        config.timeout_ms = 20000;
        assert_eq!(config.size, 20);
        assert_eq!(config.timeout_ms, 20000);
    }
}
