//! Test Data Generation Utilities

use sqlrustgo_types::Value;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_new() {
        let generator = TestDataGenerator::new(42);
        assert_eq!(generator.seed, 42);
    }

    #[test]
    fn test_generate_ints() {
        let mut generator = TestDataGenerator::new(5);
        let ints = generator.generate_ints(3, 0, 10);
        assert_eq!(ints.len(), 3);
    }

    #[test]
    fn test_generate_strings() {
        let mut generator = TestDataGenerator::new(1);
        let strings = generator.generate_strings(2, "user_");
        assert_eq!(strings.len(), 2);
    }

    #[test]
    fn test_generate_floats() {
        let mut generator = TestDataGenerator::new(100);
        let floats = generator.generate_floats(2);
        assert_eq!(floats.len(), 2);
    }

    #[test]
    fn test_table_builder_new() {
        let builder = TestTableBuilder::new("users");
        let (name, cols, rows) = builder.build();
        assert_eq!(name, "users");
        assert!(cols.is_empty());
        assert!(rows.is_empty());
    }

    #[test]
    fn test_table_builder_add_column() {
        let builder = TestTableBuilder::new("users")
            .add_column("id", "INTEGER")
            .add_column("name", "TEXT");
        let (_, cols, _) = builder.build();
        assert_eq!(cols.len(), 2);
    }

    #[test]
    fn test_table_builder_add_row() {
        let builder = TestTableBuilder::new("users")
            .add_column("id", "INTEGER")
            .add_row(vec![Value::Integer(1)])
            .add_row(vec![Value::Integer(2)]);
        let (_, _, rows) = builder.build();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_users_table() {
        let builder = schemas::users_table();
        let (name, cols, _) = builder.build();
        assert_eq!(name, "users");
        assert_eq!(cols.len(), 3);
    }

    #[test]
    fn test_orders_table() {
        let builder = schemas::orders_table();
        let (name, cols, _) = builder.build();
        assert_eq!(name, "orders");
    }

    #[test]
    fn test_products_table() {
        let builder = schemas::products_table();
        let (name, _, _) = builder.build();
        assert_eq!(name, "products");
    }
}
