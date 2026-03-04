//! SQL Error Domain Module
//!
//! # What (是什么)
//! 错误域模块，提供清晰的错误分类和层次结构
//!
//! # Why (为什么)
//! 分离不同模块的错误，便于错误处理和分布式错误传播
//!
//! # How (如何实现)
//! - SQLError 顶层枚举包含各子模块错误
//! - 各子模块定义专属错误类型
//! - 支持错误链和上下文信息

pub mod catalog;
pub mod execution;
pub mod optimizer;
pub mod parser;
pub mod storage;

pub use catalog::CatalogError;
pub use execution::ExecutionError;
pub use optimizer::OptimizerError;
pub use parser::ParserError;
pub use storage::StorageError;

/// SQL Error enum representing all error domains
///
/// # What
/// SQLError 是顶层错误枚举，包含所有子模块的错误
///
/// # Why
/// 统一的错误入口，便于调用者统一处理和错误传播
///
/// # How
/// - 使用嵌套枚举表示错误域
/// - 每个变体包含对应子模块的错误类型
/// - 实现 Display trait 提供人类可读的错误信息
#[derive(thiserror::Error, Debug)]
pub enum SQLError {
    /// Parser errors
    #[error("Parser error: {0}")]
    Parser(#[from] ParserError),

    /// Optimizer errors
    #[error("Optimizer error: {0}")]
    Optimizer(#[from] OptimizerError),

    /// Execution errors
    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),

    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Catalog errors
    #[error("Catalog error: {0}")]
    Catalog(#[from] CatalogError),

    /// Legacy error - maps to existing SqlError
    #[error("{0}")]
    Legacy(String),
}

impl SQLError {
    /// Convert from SqlError (legacy)
    pub fn from_legacy(error: &str) -> Self {
        SQLError::Legacy(error.to_string())
    }
}

/// Result type alias for SQL operations
pub type SQLResult<T> = Result<T, SQLError>;

/// Convert from legacy SqlError
impl From<crate::types::SqlError> for SQLError {
    fn from(err: crate::types::SqlError) -> Self {
        SQLError::Legacy(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_error_from_parser() {
        let err: SQLError = ParserError::new("syntax error").into();
        assert!(err.to_string().contains("Parser error"));
    }

    #[test]
    fn test_sql_error_from_execution() {
        let err: SQLError = ExecutionError::new("execution failed").into();
        assert!(err.to_string().contains("Execution error"));
    }

    #[test]
    fn test_sql_error_from_storage() {
        let err: SQLError = StorageError::new("I/O error").into();
        assert!(err.to_string().contains("Storage error"));
    }

    #[test]
    fn test_sql_error_from_legacy() {
        let err = SQLError::from_legacy("legacy error");
        assert!(err.to_string().contains("legacy error"));
    }
}
