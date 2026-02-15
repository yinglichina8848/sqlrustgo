//! SQL Value types
//! Core data types for SQLRustGo database system

use serde::{Deserialize, Serialize};
use std::fmt;

/// SQL Value enum representing all supported SQL data types
///
/// # What (是什么)
/// Value 是 SQL 数据的运行时表示，支持 NULL、布尔、整数、浮点、文本和二进制类型
/// 这些是 SQL-92 标准中最常用的数据类型，涵盖了绝大多数业务场景的需求
///
/// # Why (为什么)
/// SQL 标准定义了多种数据类型，数据库需要在内部统一表示这些类型：
/// - 为 SQL 语句的执行结果提供标准化的数据结构
/// - 支持类型检查和类型转换
/// - 便于数据的序列化和持久化存储
/// - 实现跨平台的数据交换（通过 serde）
///
/// # How (如何实现)
/// - 使用 Rust 枚举表示 SQL 类型，每种变体对应一种 SQL 类型
/// - 实现 Display trait 用于 SQL 输出格式（与 to_string() 保持一致）
/// - 使用 serde 进行序列化/反序列化（支持持久化和网络传输）
/// - 通过 type_name() 方法提供运行时类型信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// NULL value - represents missing or unknown data
    /// Example: INSERT INTO t VALUES (NULL)
    Null,

    /// Boolean - TRUE/FALSE values for logical operations
    /// Example: WHERE active = true
    Boolean(bool),

    /// 64-bit signed integer - whole numbers in range [-2^63, 2^63-1]
    /// Example: age = 25, quantity = 1000
    Integer(i64),

    /// 64-bit floating point - decimal numbers with double precision
    /// Example: price = 19.99, ratio = 0.5
    Float(f64),

    /// Text string - variable-length Unicode text
    /// Example: name = 'John', email = 'user@example.com'
    Text(String),

    /// Binary large object - arbitrary binary data
    /// Example: image_data, file_content
    Blob(Vec<u8>),
}

impl Value {
    /// Convert Value to String representation for SQL output
    ///
    /// # What
    /// 将 Value 转换为标准的 SQL 字符串格式，用于：
    /// - SQL 查询结果的显示输出
    /// - SQL 语句的拼接和生成
    ///
    /// # Why
    /// SQL 有其独特的字符串表示规范，需要特殊处理：
    /// - NULL 显示为 "NULL"（非空字符串）
    /// - Blob 显示为十六进制格式 X'...'
    /// - 文本直接输出，不加引号（与 PostgreSQL 的 \x 格式类似）
    ///
    /// # How
    /// - 使用 match 遍历所有 Value 变体
    /// - 对于 Blob，使用 hex::encode 转换为十六进制字符串
    pub fn to_string(&self) -> String {
        match self {
            Value::Null => "NULL".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => s.clone(),
            Value::Blob(b) => format!("X'{}'", hex::encode(b)),
        }
    }

    /// Get the SQL type name as a string literal
    ///
    /// # What
    /// 返回 Value 对应的 SQL 类型名称
    ///
    /// # Why
    /// 在运行时需要知道 Value 的具体类型：
    /// - 用于错误消息的生成（如类型不匹配错误）
    /// - 用于元数据查询和 SHOW 命令
    /// - 用于类型推断和类型转换
    ///
    /// # How
    /// - 返回静态字符串字面量，避免运行时内存分配
    /// - 使用 match 遍历所有变体
    /// - 对于多态类型（如 Integer、Boolean），统一返回类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "NULL",
            Value::Boolean(_) => "BOOLEAN",
            Value::Integer(_) => "INTEGER",
            Value::Float(_) => "FLOAT",
            Value::Text(_) => "TEXT",
            Value::Blob(_) => "BLOB",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::Null.to_string(), "NULL");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Text("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn test_value_type_name() {
        assert_eq!(Value::Null.type_name(), "NULL");
        assert_eq!(Value::Boolean(true).type_name(), "BOOLEAN");
        assert_eq!(Value::Integer(1).type_name(), "INTEGER");
        assert_eq!(Value::Float(1.0).type_name(), "FLOAT");
        assert_eq!(Value::Text("test".to_string()).type_name(), "TEXT");
        assert_eq!(Value::Blob(vec![0x01, 0x02]).type_name(), "BLOB");
    }
}
