//! SQL Error types
//! Error handling for SQLRustGo database system
//!
//! # What (是什么)
//! 本模块定义了 SQLRustGo 数据库的错误类型系统，包括 SqlError 枚举和 SqlResult 类型。
//!
//! # Why (为什么)
//! 统一的错误处理是数据库系统的核心需求。通过定义标准化的错误类型，
//! 可以让调用者清楚地知道发生了什么错误，并采取相应的恢复措施。
//!
//! # How (如何实现)
//! - SqlError: 使用枚举变体表示不同类型的错误（解析错误、执行错误、存储错误等）
//! - SqlResult: 是 Result 类型的别名，简化错误传播
//! - SqlError 实现 Display trait 以便人类可读的错误信息

/// SQL Error enum representing all possible error types
///
/// # 常见错误类型说明
///
/// - **ParseError**: SQL 语法错误，例如拼写错误、缺少关键字等
/// - **ExecutionError**: 查询执行过程中的错误，例如除零运算、无效操作等
/// - **TypeMismatch**: 数据类型不匹配，例如对字符串进行数值运算
/// - **TableNotFound**: 引用的表不存在
/// - **ColumnNotFound**: 引用的列不存在
#[derive(thiserror::Error, Debug)]
pub enum SqlError {
    /// Syntax error during parsing
    /// 发生场景：用户输入的 SQL 语句不符合 SQL 语法规范
    /// 示例：SELECT * FORM users (FORM 是拼写错误，应该是 FROM)
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Execution error during query processing
    /// 发生场景：SQL 语法正确，但在执行时出现错误
    /// 示例：UPDATE non_existent_table SET col = 1
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Type mismatch error
    /// 发生场景：操作符两边的数据类型不兼容
    /// 示例：SELECT 'string' + 1 (字符串不能与数字相加)
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    /// Division by zero
    /// 发生场景：进行除法运算时除数为零
    /// 示例：SELECT 10 / 0
    #[error("Division by zero")]
    DivisionByZero,

    /// Null value error (operation on NULL)
    /// 发生场景：对 NULL 值进行了不支持的操作
    /// 示例：在 NOT NULL 约束的列中插入 NULL
    #[error("Null value error: {0}")]
    NullValueError(String),

    /// Constraint violation
    /// 发生场景：违反了表定义的约束条件
    /// 示例：外键引用不存在的主键、CHECK 约束失败等
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Table not found
    /// 发生场景：SQL 语句引用了不存在的表
    /// 示例：SELECT * FROM non_existent_table
    #[error("Table not found: {0}")]
    TableNotFound(String),

    /// Column not found
    /// 发生场景：SQL 语句引用了表中不存在的列
    /// 示例：SELECT non_existent_column FROM users
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Duplicate key error
    /// 发生场景：插入或更新数据时，违反了唯一键约束
    /// 示例：INSERT INTO users (id) VALUES (1) -- 假设 id 已经是 1
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),

    /// I/O error
    /// 发生场景：磁盘读写、文件操作等 I/O 失败
    /// 示例：数据库文件损坏、磁盘空间不足等
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
    use std::io;

    #[test]
    fn test_error_messages() {
        let err = SqlError::ParseError("invalid syntax".to_string());
        assert!(err.to_string().contains("Parse error"));

        let err = SqlError::TableNotFound("users".to_string());
        assert!(err.to_string().contains("Table not found"));
    }

    #[test]
    fn test_execution_error() {
        let err = SqlError::ExecutionError("invalid operation".to_string());
        assert!(err.to_string().contains("Execution error"));
        assert!(err.to_string().contains("invalid operation"));
    }

    #[test]
    fn test_type_mismatch() {
        let err = SqlError::TypeMismatch("cannot add string and number".to_string());
        assert!(err.to_string().contains("Type mismatch"));
        assert!(err.to_string().contains("cannot add string and number"));
    }

    #[test]
    fn test_division_by_zero() {
        let err = SqlError::DivisionByZero;
        assert!(err.to_string().contains("Division by zero"));
    }

    #[test]
    fn test_null_value_error() {
        let err = SqlError::NullValueError("column cannot be null".to_string());
        assert!(err.to_string().contains("Null value error"));
        assert!(err.to_string().contains("column cannot be null"));
    }

    #[test]
    fn test_constraint_violation() {
        let err = SqlError::ConstraintViolation("unique constraint failed".to_string());
        assert!(err.to_string().contains("Constraint violation"));
        assert!(err.to_string().contains("unique constraint failed"));
    }

    #[test]
    fn test_column_not_found() {
        let err = SqlError::ColumnNotFound("age".to_string());
        assert!(err.to_string().contains("Column not found"));
        assert!(err.to_string().contains("age"));
    }

    #[test]
    fn test_duplicate_key() {
        let err = SqlError::DuplicateKey("id=1".to_string());
        assert!(err.to_string().contains("Duplicate key"));
        assert!(err.to_string().contains("id=1"));
    }

    #[test]
    fn test_io_error() {
        let err = SqlError::IoError("file not found".to_string());
        assert!(err.to_string().contains("I/O error"));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_from_string() {
        let err: SqlError = String::from("test error").into();
        assert!(matches!(err, SqlError::ExecutionError(_)));
    }

    #[test]
    fn test_from_str() {
        let err: SqlError = "test error".into();
        assert!(matches!(err, SqlError::ExecutionError(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: SqlError = io_err.into();
        assert!(matches!(err, SqlError::IoError(_)));
    }

    #[test]
    fn test_error_debug() {
        let err = SqlError::ParseError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ParseError"));
    }

    #[test]
    fn test_sql_result_alias() {
        fn return_result() -> SqlResult<i32> {
            Ok(42)
        }
        fn return_error() -> SqlResult<i32> {
            Err(SqlError::TableNotFound("test".to_string()))
        }

        assert_eq!(return_result().unwrap(), 42);
        assert!(return_error().is_err());
    }
}
