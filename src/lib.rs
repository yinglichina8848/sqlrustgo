//! SQLRustGo Database System Library
//! 
//! A Rust implementation of a SQL-92 compliant database system.

pub mod types;
pub use types::{Value, SqlError, SqlResult, parse_sql_literal};

/// Initialize the database system
pub fn init() {
    println!("SQLRustGo Database System initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }

    #[test]
    fn test_types_module() {
        let v = types::Value::Integer(42);
        assert_eq!(v.to_string(), "42");
    }
}
