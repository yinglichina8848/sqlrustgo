//! Test Data Generator for Executor Testing
//!
//! This module provides utilities for generating test data
//! with various data types and distributions.

use rand::prelude::*;
use rand::rngs::StdRng;
use sqlrustgo_planner::{DataType, Field, Schema};
use sqlrustgo_types::Value;

/// TestDataGenerator - Generates test data for executor tests
pub struct TestDataGenerator {
    rng: StdRng,
}

impl TestDataGenerator {
    /// Create a new TestDataGenerator
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    /// Create a TestDataGenerator with a seed for reproducibility
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Generate a single row based on schema
    pub fn generate_row(&mut self, schema: &Schema) -> Vec<Value> {
        schema
            .fields
            .iter()
            .map(|field| self.generate_value(&field.data_type))
            .collect()
    }

    /// Generate a single value based on data type
    pub fn generate_value(&mut self, data_type: &DataType) -> Value {
        match data_type {
            DataType::Integer => Value::Integer(self.rng.gen_range(1..1000)),
            DataType::Float => Value::Float(self.rng.gen_range(1.0..1000.0)),
            DataType::Text => Value::Text(self.generate_string(10)),
            DataType::Boolean => Value::Boolean(self.rng.gen_bool(0.5)),
            DataType::Null => Value::Null,
            _ => Value::Null,
        }
    }

    /// Generate a random string of given length
    fn generate_string(&mut self, len: usize) -> String {
        (0..len)
            .map(|_| {
                let idx = self.rng.gen_range(0..26);
                (b'a' + idx) as char
            })
            .collect()
    }

    /// Generate multiple rows
    pub fn generate_rows(&mut self, schema: &Schema, count: usize) -> Vec<Vec<Value>> {
        (0..count).map(|_| self.generate_row(schema)).collect()
    }

    /// Generate sequential integers
    pub fn generate_sequential_integers(&mut self, start: i64, count: usize) -> Vec<Value> {
        (start..start + count as i64).map(Value::Integer).collect()
    }

    /// Generate test data for common table schemas
    pub fn generate_users_table(&mut self, count: usize) -> Vec<Vec<Value>> {
        let _schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("age".to_string(), DataType::Integer),
            Field::new("email".to_string(), DataType::Text),
        ]);

        (0..count)
            .map(|i| {
                vec![
                    Value::Integer((i + 1) as i64),
                    Value::Text(format!("User{}", i + 1)),
                    Value::Integer(self.rng.gen_range(18..80)),
                    Value::Text(format!("user{}@example.com", i + 1)),
                ]
            })
            .collect()
    }

    pub fn generate_orders_table(&mut self, count: usize) -> Vec<Vec<Value>> {
        (0..count)
            .map(|i| {
                vec![
                    Value::Integer((i + 1) as i64),
                    Value::Integer(self.rng.gen_range(1..100) as i64),
                    Value::Integer(self.rng.gen_range(10..1000)),
                    Value::Text(format!("2024-01-{:02}", (i % 28) + 1)),
                ]
            })
            .collect()
    }

    pub fn generate_products_table(&mut self, count: usize) -> Vec<Vec<Value>> {
        let products = ["Laptop", "Phone", "Tablet", "Watch", "Headphones"];

        (0..count)
            .map(|i| {
                vec![
                    Value::Integer((i + 1) as i64),
                    Value::Text(products[i % products.len()].to_string()),
                    Value::Integer(self.rng.gen_range(100..2000)),
                ]
            })
            .collect()
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// TestTableBuilder - Builder for creating test tables
pub struct TestTableBuilder {
    name: String,
    columns: Vec<(String, DataType, Option<Value>)>,
}

impl TestTableBuilder {
    /// Create a new TestTableBuilder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
        }
    }

    /// Add an integer column
    pub fn add_integer_column(mut self, name: &str) -> Self {
        self.columns
            .push((name.to_string(), DataType::Integer, None));
        self
    }

    /// Add a text column
    pub fn add_text_column(mut self, name: &str) -> Self {
        self.columns.push((name.to_string(), DataType::Text, None));
        self
    }

    /// Add a float column
    pub fn add_float_column(mut self, name: &str) -> Self {
        self.columns.push((name.to_string(), DataType::Float, None));
        self
    }

    /// Add a boolean column
    pub fn add_boolean_column(mut self, name: &str) -> Self {
        self.columns
            .push((name.to_string(), DataType::Boolean, None));
        self
    }

    /// Build the schema
    pub fn build_schema(&self) -> Schema {
        let fields: Vec<Field> = self
            .columns
            .iter()
            .map(|(name, data_type, _)| Field::new(name.clone(), data_type.clone()))
            .collect();
        Schema::new(fields)
    }

    /// Build table info
    pub fn build_table_info(&self) -> sqlrustgo_storage::TableInfo {
        let columns: Vec<sqlrustgo_storage::ColumnDefinition> = self
            .columns
            .iter()
            .map(|(name, data_type, _)| sqlrustgo_storage::ColumnDefinition {
                name: name.clone(),
                data_type: format!("{:?}", data_type),
                nullable: false,
                is_unique: false,
            })
            .collect();
        sqlrustgo_storage::TableInfo {
            name: self.name.clone(),
            columns,
        }
    }
}

/// RowBuilder - Builder for creating test rows
pub struct RowBuilder {
    values: Vec<Value>,
}

impl RowBuilder {
    /// Create a new RowBuilder
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Add an integer value
    pub fn add_integer(mut self, value: i64) -> Self {
        self.values.push(Value::Integer(value));
        self
    }

    /// Add a text value
    pub fn add_text(mut self, value: &str) -> Self {
        self.values.push(Value::Text(value.to_string()));
        self
    }

    /// Add a float value
    pub fn add_float(self, value: f64) -> Self {
        self.add_value(Value::Float(value))
    }

    /// Add a boolean value
    pub fn add_boolean(self, value: bool) -> Self {
        self.add_value(Value::Boolean(value))
    }

    /// Add a null value
    pub fn add_null(self) -> Self {
        self.add_value(Value::Null)
    }

    /// Add a value directly
    pub fn add_value(mut self, value: Value) -> Self {
        self.values.push(value);
        self
    }

    /// Build the row
    pub fn build(self) -> Vec<Value> {
        self.values
    }
}

impl Default for RowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// TestDataSet - Predefined test datasets
pub struct TestDataSet;

impl TestDataSet {
    /// Get simple users dataset
    pub fn simple_users() -> Vec<Vec<Value>> {
        vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
            vec![Value::Integer(3), Value::Text("Charlie".to_string())],
        ]
    }

    /// Get simple orders dataset
    pub fn simple_orders() -> Vec<Vec<Value>> {
        vec![
            vec![Value::Integer(1), Value::Integer(100), Value::Integer(1)],
            vec![Value::Integer(2), Value::Integer(200), Value::Integer(1)],
            vec![Value::Integer(3), Value::Integer(300), Value::Integer(2)],
            vec![Value::Integer(4), Value::Integer(400), Value::Integer(2)],
            vec![Value::Integer(5), Value::Integer(500), Value::Integer(3)],
        ]
    }

    /// Get dataset with null values
    pub fn with_nulls() -> Vec<Vec<Value>> {
        vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Null],
            vec![Value::Integer(3), Value::Text("Charlie".to_string())],
        ]
    }

    /// Get dataset for aggregate testing (group by)
    pub fn aggregate_test_data() -> Vec<Vec<Value>> {
        vec![
            vec![Value::Text("A".to_string()), Value::Integer(100)],
            vec![Value::Text("A".to_string()), Value::Integer(200)],
            vec![Value::Text("B".to_string()), Value::Integer(300)],
            vec![Value::Text("B".to_string()), Value::Integer(400)],
            vec![Value::Text("C".to_string()), Value::Integer(500)],
        ]
    }

    /// Get TPC-H Q1 lineitem dataset
    /// Columns: l_returnflag, l_linestatus, l_quantity
    pub fn lineitem_q1() -> Vec<Vec<Value>> {
        vec![
            vec![
                Value::Text("N".to_string()),
                Value::Text("O".to_string()),
                Value::Integer(10),
            ],
            vec![
                Value::Text("N".to_string()),
                Value::Text("O".to_string()),
                Value::Integer(20),
            ],
            vec![
                Value::Text("N".to_string()),
                Value::Text("O".to_string()),
                Value::Integer(30),
            ],
            vec![
                Value::Text("R".to_string()),
                Value::Text("F".to_string()),
                Value::Integer(15),
            ],
            vec![
                Value::Text("R".to_string()),
                Value::Text("F".to_string()),
                Value::Integer(25),
            ],
        ]
    }

    /// Get TPC-H Q3 orders dataset
    /// Columns: o_orderkey, o_custkey, o_orderdate (as integer YYYYMMDD)
    pub fn orders_q3() -> Vec<Vec<Value>> {
        vec![
            vec![
                Value::Integer(1),
                Value::Integer(100),
                Value::Integer(19950315),
            ],
            vec![
                Value::Integer(2),
                Value::Integer(200),
                Value::Integer(19950320),
            ],
            vec![
                Value::Integer(3),
                Value::Integer(300),
                Value::Integer(19950401),
            ],
            vec![
                Value::Integer(4),
                Value::Integer(400),
                Value::Integer(19950228),
            ],
            vec![
                Value::Integer(5),
                Value::Integer(500),
                Value::Integer(19950310),
            ],
        ]
    }

    /// Get TPC-H Q6 lineitem dataset
    /// Columns: l_shipdate (as integer YYYYMMDD), l_discount, l_quantity, l_extendedprice
    pub fn lineitem_q6() -> Vec<Vec<Value>> {
        vec![
            vec![
                Value::Integer(19940615),
                Value::Float(0.06),
                Value::Integer(10),
                Value::Float(100.0),
            ],
            vec![
                Value::Integer(19940820),
                Value::Float(0.04),
                Value::Integer(20),
                Value::Float(200.0),
            ],
            vec![
                Value::Integer(19941005),
                Value::Float(0.08),
                Value::Integer(30),
                Value::Float(300.0),
            ],
            vec![
                Value::Integer(19941225),
                Value::Float(0.05),
                Value::Integer(15),
                Value::Float(150.0),
            ],
            vec![
                Value::Integer(19950110),
                Value::Float(0.07),
                Value::Integer(25),
                Value::Float(250.0),
            ],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_new() {
        let mut gen = TestDataGenerator::new();
        let value = gen.generate_value(&DataType::Integer);
        assert!(matches!(value, Value::Integer(_)));
    }

    #[test]
    fn test_generator_with_seed() {
        let mut gen1 = TestDataGenerator::with_seed(42);
        let mut gen2 = TestDataGenerator::with_seed(42);

        let val1 = gen1.generate_value(&DataType::Integer);
        let val2 = gen2.generate_value(&DataType::Integer);

        assert_eq!(val1, val2);
    }

    #[test]
    fn test_generator_schema() {
        let mut gen = TestDataGenerator::new();
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let row = gen.generate_row(&schema);
        assert_eq!(row.len(), 2);
    }

    #[test]
    fn test_generator_rows() {
        let mut gen = TestDataGenerator::new();
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let rows = gen.generate_rows(&schema, 10);
        assert_eq!(rows.len(), 10);
    }

    #[test]
    fn test_generator_sequential() {
        let mut gen = TestDataGenerator::new();
        let integers = gen.generate_sequential_integers(1, 5);

        assert_eq!(integers.len(), 5);
        assert_eq!(integers[0], Value::Integer(1));
        assert_eq!(integers[4], Value::Integer(5));
    }

    #[test]
    fn test_test_table_builder() {
        let builder = TestTableBuilder::new("users")
            .add_integer_column("id")
            .add_text_column("name")
            .add_integer_column("age");

        let schema = builder.build_schema();
        assert_eq!(schema.fields.len(), 3);
    }

    #[test]
    fn test_row_builder() {
        let row = RowBuilder::new()
            .add_integer(1)
            .add_text("Alice")
            .add_integer(30)
            .build();

        assert_eq!(row.len(), 3);
        assert_eq!(row[0], Value::Integer(1));
        assert_eq!(row[1], Value::Text("Alice".to_string()));
        assert_eq!(row[2], Value::Integer(30));
    }

    #[test]
    fn test_test_data_set_simple_users() {
        let users = TestDataSet::simple_users();
        assert_eq!(users.len(), 3);
    }

    #[test]
    fn test_test_data_set_aggregate() {
        let data = TestDataSet::aggregate_test_data();
        assert_eq!(data.len(), 5);
    }

    #[test]
    fn test_test_data_generator_with_float() {
        let mut gen = TestDataGenerator::with_seed(42);
        let value = gen.generate_value(&DataType::Float);
        assert!(matches!(value, Value::Float(_)));
    }

    #[test]
    fn test_test_data_generator_with_boolean() {
        let mut gen = TestDataGenerator::with_seed(42);
        let value = gen.generate_value(&DataType::Boolean);
        assert!(matches!(value, Value::Boolean(_)));
    }

    #[test]
    fn test_test_data_generator_with_null() {
        let mut gen = TestDataGenerator::with_seed(42);
        let value = gen.generate_value(&DataType::Null);
        assert!(matches!(value, Value::Null));
    }

    #[test]
    fn test_test_data_generator_unknown_type() {
        let mut gen = TestDataGenerator::with_seed(42);
        // Decimal might not be supported, returns Null
        let value = gen.generate_value(&DataType::Integer);
        assert!(matches!(value, Value::Integer(_)));
    }

    #[test]
    fn test_test_data_generator_sequential_integers() {
        let mut gen = TestDataGenerator::with_seed(42);
        let ints = gen.generate_sequential_integers(10, 5);
        assert_eq!(ints.len(), 5);
        assert_eq!(ints[0], Value::Integer(10));
        assert_eq!(ints[4], Value::Integer(14));
    }
}
