//! Test Data Generation Utilities
//!
//! Provides utilities for generating test data for executor tests

use crate::types::Value;

/// Test data generator
pub struct TestDataGenerator {
    seed: u64,
}

impl TestDataGenerator {
    pub fn new(seed: u64) -> Self { Self { seed } }
    
    pub fn generate_ints(&mut self, count: usize, min: i64, max: i64) -> Vec<Value> {
        (0..count).map(|_| Value::Integer((self.seed as i64) % (max - min) + min)).collect()
    }
    
    pub fn generate_strings(&mut self, count: usize, prefix: &str) -> Vec<Value> {
        (0..count).map(|i| Value::Text(format!("{}{}", prefix, i))).collect()
    }
    
    pub fn generate_floats(&mut self, count: usize) -> Vec<Value> {
        (0..count).map(|_| Value::Float(self.seed as f64 / 100.0)).collect()
    }
}

/// Builder for test tables
pub struct TestTableBuilder {
    name: String,
    columns: Vec<(String, String)>,
    rows: Vec<Vec<Value>>,
}

impl TestTableBuilder {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), columns: vec![], rows: vec![] }
    }
    
    pub fn add_column(mut self, name: &str, data_type: &str) -> Self {
        self.columns.push((name.to_string(), data_type.to_string()));
        self
    }
    
    pub fn add_row(mut self, row: Vec<Value>) -> Self {
        self.rows.push(row);
        self
    }
    
    pub fn build(self) -> (String, Vec<(String, String)>, Vec<Vec<Value>>) {
        (self.name, self.columns, self.rows)
    }
}

/// Common test schemas
pub mod schemas {
    use super::*;
    
    pub fn users_table() -> TestTableBuilder {
        TestTableBuilder::new("users")
            .add_column("id", "INTEGER")
            .add_column("name", "TEXT")
            .add_column("email", "TEXT")
    }
    
    pub fn orders_table() -> TestTableBuilder {
        TestTableBuilder::new("orders")
            .add_column("id", "INTEGER")
            .add_column("user_id", "INTEGER")
            .add_column("amount", "FLOAT")
    }
    
    pub fn products_table() -> TestTableBuilder {
        TestTableBuilder::new("products")
            .add_column("id", "INTEGER")
            .add_column("name", "TEXT")
            .add_column("price", "FLOAT")
    }
}
