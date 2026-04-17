// SQLRustGo Common Module - Common types and errors

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
}
