//! Test Data Generator
//!
//! Provides utilities for generating test data for the executor tests.

use crate::executor::{TableData, TableInfo};
use crate::parser::ColumnDefinition;
use crate::types::Value;
use rand::rngs::StdRng;
use rand::prelude::*;

/// Generator for creating test data
pub struct TestDataGenerator {
    rng: StdRng,
}

impl TestDataGenerator {
    /// Create a new generator with a seed for reproducibility
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Create a new generator with random seed
    pub fn random() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    /// Generate random integer
    pub fn random_integer(&mut self, min: i64, max: i64) -> i64 {
        self.rng.gen_range(min..=max)
    }

    /// Generate random float
    pub fn random_float(&mut self, min: f64, max: f64) -> f64 {
        self.rng.gen_range(min..=max)
    }

    /// Generate random text string
    pub fn random_text(&mut self, length: usize) -> String {
        let chars: Vec<char> = (0..length)
            .map(|_| {
                let idx = self.rng.gen_range(0..26);
                (b'a' + idx) as char
            })
            .collect();
        chars.into_iter().collect()
    }

    /// Generate random boolean
    pub fn random_bool(&mut self) -> bool {
        self.rng.gen_range(0..2) == 1
    }

    /// Generate a random Value based on type string
    pub fn random_value(&mut self, data_type: &str) -> Value {
        match data_type.to_uppercase().as_str() {
            "INTEGER" => Value::Integer(self.random_integer(1, 10000)),
            "FLOAT" | "DOUBLE" | "REAL" => Value::Float(self.random_float(0.0, 10000.0)),
            "TEXT" | "VARCHAR" | "CHAR" => Value::Text(self.random_text(10)),
            "BOOLEAN" | "BOOL" => Value::Boolean(self.random_bool()),
            _ => Value::Null,
        }
    }

    /// Generate a row based on column definitions
    pub fn generate_row(&mut self, columns: &[ColumnDefinition]) -> Vec<Value> {
        columns
            .iter()
            .map(|col| {
                if col.nullable && self.rng.gen_bool(0.2) {
                    Value::Null
                } else {
                    self.random_value(&col.data_type)
                }
            })
            .collect()
    }

    /// Generate multiple rows
    pub fn generate_rows(&mut self, columns: &[ColumnDefinition], count: usize) -> Vec<Vec<Value>> {
        (0..count).map(|_| self.generate_row(columns)).collect()
    }

    /// Generate a complete table with data
    pub fn generate_table(
        &mut self,
        name: &str,
        columns: Vec<ColumnDefinition>,
        row_count: usize,
    ) -> TableData {
        let rows = self.generate_rows(&columns, row_count);
        TableData {
            info: TableInfo {
                name: name.to_string(),
                columns,
            },
            rows,
        }
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new(42) // Default seed for reproducibility
    }
}

/// Predefined table schemas for testing
pub mod schemas {
    use super::*;

    /// Users table schema
    pub fn users_schema() -> Vec<ColumnDefinition> {
        vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "email".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
            },
            ColumnDefinition {
                name: "age".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
            },
            ColumnDefinition {
                name: "active".to_string(),
                data_type: "BOOLEAN".to_string(),
                nullable: false,
            },
        ]
    }

    /// Products table schema
    pub fn products_schema() -> Vec<ColumnDefinition> {
        vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "price".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "stock".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
            },
            ColumnDefinition {
                name: "category".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
            },
        ]
    }

    /// Orders table schema
    pub fn orders_schema() -> Vec<ColumnDefinition> {
        vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "user_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "product_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "quantity".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "total".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "status".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
            },
        ]
    }

    /// Simple key-value table schema
    pub fn key_value_schema() -> Vec<ColumnDefinition> {
        vec![
            ColumnDefinition {
                name: "key".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "value".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
            },
        ]
    }
}

/// Helper to create test tables quickly
pub struct TestTableBuilder {
    name: String,
    columns: Vec<ColumnDefinition>,
    rows: Vec<Vec<Value>>,
}

impl TestTableBuilder {
    /// Create a new table builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Add a column
    pub fn column(mut self, name: impl Into<String>, data_type: impl Into<String>, nullable: bool) -> Self {
        self.columns.push(ColumnDefinition {
            name: name.into(),
            data_type: data_type.into(),
            nullable,
        });
        self
    }

    /// Add an integer column
    pub fn integer_column(mut self, name: impl Into<String>) -> Self {
        self.columns.push(ColumnDefinition {
            name: name.into(),
            data_type: "INTEGER".to_string(),
            nullable: false,
        });
        self
    }

    /// Add a text column
    pub fn text_column(mut self, name: impl Into<String>, nullable: bool) -> Self {
        self.columns.push(ColumnDefinition {
            name: name.into(),
            data_type: "TEXT".to_string(),
            nullable,
        });
        self
    }

    /// Add a row
    pub fn row(mut self, values: Vec<Value>) -> Self {
        self.rows.push(values);
        self
    }

    /// Add multiple rows at once
    pub fn rows(mut self, values: Vec<Vec<Value>>) -> Self {
        self.rows.extend(values);
        self
    }

    /// Build the table data
    pub fn build(self) -> TableData {
        TableData {
            info: TableInfo {
                name: self.name,
                columns: self.columns,
            },
            rows: self.rows,
        }
    }

    /// Build and insert into storage
    pub fn build_and_insert(self, storage: &mut crate::executor::test_framework::mock_storage::MockStorage) {
        let table = self.build();
        storage.insert_table(table.info.name.clone(), table).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_seed() {
        let mut gen1 = TestDataGenerator::new(42);
        let mut gen2 = TestDataGenerator::new(42);
        
        // Same seed should produce same results
        for _ in 0..100 {
            assert_eq!(
                gen1.random_integer(1, 100),
                gen2.random_integer(1, 100)
            );
        }
    }

    #[test]
    fn test_generate_row() {
        let mut generator = TestDataGenerator::new(42);
        let columns = schemas::users_schema();
        let row = generator.generate_row(&columns);
        
        assert_eq!(row.len(), columns.len());
    }

    #[test]
    fn test_test_table_builder() {
        let table = TestTableBuilder::new("test")
            .column("id", "INTEGER", false)
            .column("name", "TEXT", false)
            .row(vec![Value::Integer(1), Value::Text("Alice".to_string())])
            .row(vec![Value::Integer(2), Value::Text("Bob".to_string())])
            .build();
        
        assert_eq!(table.info.name, "test");
        assert_eq!(table.info.columns.len(), 2);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_generate_table() {
        let mut generator = TestDataGenerator::new(42);
        let table = generator.generate_table("users", schemas::users_schema(), 10);
        
        assert_eq!(table.info.name, "users");
        assert_eq!(table.rows.len(), 10);
    }
}
