//! Dataset Generator - benches/dataset_generator.rs
//!
//! Reusable test data generation utilities for SQLRustGo benchmarks.

use sqlrustgo_storage::{ColumnDefinition, TableInfo};
use sqlrustgo_types::Value;

/// Generate sequential integer rows
pub fn generate_integer_rows(count: usize) -> Vec<Vec<Value>> {
    (0..count)
        .map(|i| vec![Value::Integer(i as i64)])
        .collect()
}

/// Generate rows with multiple columns (id, name, value)
pub fn generate_multi_column_rows(count: usize) -> Vec<Vec<Value>> {
    (0..count)
        .map(|i| {
            vec![
                Value::Integer(i as i64),
                Value::Text(format!("user_{}", i)),
                Value::Integer((i * 10) as i64),
            ]
        })
        .collect()
}

/// Generate rows with random-ish distribution for testing
pub fn generate_ordered_rows(count: usize) -> Vec<Vec<Value>> {
    (0..count)
        .map(|i| {
            let value = if i % 2 == 0 { i } else { count - i };
            vec![
                Value::Integer(i as i64),
                Value::Integer(value as i64),
            ]
        })
        .collect()
}

/// Create a simple table info with single INTEGER column
pub fn simple_table_info(name: &str) -> TableInfo {
    TableInfo {
        name: name.to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
        }],
    }
}

/// Create a multi-column table info
pub fn multi_column_table_info(name: &str) -> TableInfo {
    TableInfo {
        name: name.to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
            },
            ColumnDefinition {
                name: "value".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
            },
        ],
    }
}

/// Create an orders table for aggregate testing
pub fn orders_table_info() -> TableInfo {
    TableInfo {
        name: "orders".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "amount".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
        ],
    }
}

/// Generate orders data for aggregate benchmarks
pub fn generate_orders(count: usize) -> Vec<Vec<Value>> {
    (0..count)
        .map(|i| vec![Value::Integer(i as i64), Value::Integer((i * 10) as i64)])
        .collect()
}

/// Benchmark dataset sizes
pub const BENCH_SIZES: [usize; 3] = [1_000, 10_000, 100_000];

/// Small dataset for quick tests
pub const SMALL_SIZES: [usize; 3] = [100, 1000, 10_000];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_integer_rows() {
        let rows = generate_integer_rows(100);
        assert_eq!(rows.len(), 100);
        assert_eq!(rows[0][0], Value::Integer(0));
        assert_eq!(rows[99][0], Value::Integer(99));
    }

    #[test]
    fn test_generate_multi_column_rows() {
        let rows = generate_multi_column_rows(10);
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].len(), 3);
    }

    #[test]
    fn test_simple_table_info() {
        let info = simple_table_info("test");
        assert_eq!(info.name, "test");
        assert_eq!(info.columns.len(), 1);
    }

    #[test]
    fn test_orders_table_info() {
        let info = orders_table_info();
        assert_eq!(info.name, "orders");
        assert_eq!(info.columns.len(), 2);
    }

    #[test]
    fn test_generate_orders() {
        let rows = generate_orders(50);
        assert_eq!(rows.len(), 50);
        assert_eq!(rows[0][1], Value::Integer(0));
        assert_eq!(rows[10][1], Value::Integer(100));
    }
}
