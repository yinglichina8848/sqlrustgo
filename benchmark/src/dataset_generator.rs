//! Dataset Generator for SQLRustGo Benchmarks
//!
//! Provides utilities for generating test data at various scales.

use rand::rngs::ThreadRng;
use rand::Rng;
use std::time::Instant;

/// Data scale levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataScale {
    /// 1,000 rows
    Tiny,
    /// 10,000 rows
    Small,
    /// 100,000 rows
    Medium,
    /// 1,000,000 rows
    Large,
}

impl DataScale {
    /// Get the number of rows for this scale
    pub fn rows(&self) -> usize {
        match self {
            DataScale::Tiny => 1_000,
            DataScale::Small => 10_000,
            DataScale::Medium => 100_000,
            DataScale::Large => 1_000_000,
        }
    }

    /// Get scale name
    pub fn name(&self) -> &'static str {
        match self {
            DataScale::Tiny => "tiny",
            DataScale::Small => "small",
            DataScale::Medium => "medium",
            DataScale::Large => "large",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tiny" | "1k" | "1000" => Some(DataScale::Tiny),
            "small" | "10k" | "10000" => Some(DataScale::Small),
            "medium" | "100k" | "100000" => Some(DataScale::Medium),
            "large" | "1m" | "1000000" => Some(DataScale::Large),
            _ => None,
        }
    }
}

/// Test row for benchmarking
#[derive(Debug, Clone)]
pub struct TestRow {
    pub id: u64,
    pub name: String,
    pub value: i64,
}

impl TestRow {
    /// Generate a new test row
    pub fn new(id: u64) -> Self {
        Self {
            id,
            name: format!("user_{}", id),
            value: (id % 1000) as i64,
        }
    }

    /// Generate a batch of test rows
    pub fn batch(start_id: u64, count: usize) -> Vec<Self> {
        (0..count).map(|i| Self::new(start_id + i as u64)).collect()
    }
}

/// Dataset generator for SQLRustGo benchmarks
pub struct DatasetGenerator {
    rng: ThreadRng,
}

impl DatasetGenerator {
    /// Create a new dataset generator
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    /// Generate test rows
    pub fn generate_rows(&mut self, count: usize) -> Vec<TestRow> {
        TestRow::batch(0, count)
    }

    /// Generate test rows with custom ID offset
    pub fn generate_rows_with_offset(&mut self, start_id: u64, count: usize) -> Vec<TestRow> {
        TestRow::batch(start_id, count)
    }

    /// Generate random test rows
    pub fn generate_random_rows(&mut self, count: usize) -> Vec<TestRow> {
        (0..count)
            .map(|i| {
                let value: i64 = self.rng.gen_range(0..10000);
                TestRow {
                    id: i as u64,
                    name: format!("name_{}", self.rng.gen::<u32>()),
                    value,
                }
            })
            .collect()
    }

    /// Generate SQL INSERT statements
    pub fn generate_insert_sql(&mut self, table: &str, count: usize) -> Vec<String> {
        self.generate_rows(count)
            .into_iter()
            .map(|row| {
                format!(
                    "INSERT INTO {} VALUES ({}, '{}', {})",
                    table, row.id, row.name, row.value
                )
            })
            .collect()
    }
}

impl Default for DatasetGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate benchmark data for a given scale
pub fn generate_benchmark_data(scale: DataScale) -> Vec<TestRow> {
    let mut generator = DatasetGenerator::new();
    generator.generate_rows(scale.rows())
}

/// Time the generation of data
pub fn time_generation(scale: DataScale) -> (Vec<TestRow>, std::time::Duration) {
    let start = Instant::now();
    let mut generator = DatasetGenerator::new();
    let data = generator.generate_rows(scale.rows());
    let duration = start.elapsed();
    (data, duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_scale_rows() {
        assert_eq!(DataScale::Tiny.rows(), 1_000);
        assert_eq!(DataScale::Small.rows(), 10_000);
        assert_eq!(DataScale::Medium.rows(), 100_000);
        assert_eq!(DataScale::Large.rows(), 1_000_000);
    }

    #[test]
    fn test_data_scale_from_str() {
        assert_eq!(DataScale::from_str("tiny"), Some(DataScale::Tiny));
        assert_eq!(DataScale::from_str("1k"), Some(DataScale::Tiny));
        assert_eq!(DataScale::from_str("1000"), Some(DataScale::Tiny));
        assert_eq!(DataScale::from_str("small"), Some(DataScale::Small));
        assert_eq!(DataScale::from_str("10k"), Some(DataScale::Small));
        assert_eq!(DataScale::from_str("medium"), Some(DataScale::Medium));
        assert_eq!(DataScale::from_str("100k"), Some(DataScale::Medium));
        assert_eq!(DataScale::from_str("large"), Some(DataScale::Large));
        assert_eq!(DataScale::from_str("1m"), Some(DataScale::Large));
        assert_eq!(DataScale::from_str("invalid"), None);
    }

    #[test]
    fn test_test_row_new() {
        let row = TestRow::new(42);
        assert_eq!(row.id, 42);
        assert_eq!(row.name, "user_42");
        assert_eq!(row.value, 42);
    }

    #[test]
    fn test_test_row_batch() {
        let rows = TestRow::batch(100, 5);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].id, 100);
        assert_eq!(rows[4].id, 104);
    }

    #[test]
    fn test_dataset_generator_new() {
        let mut generator = DatasetGenerator::new();
        let rows = generator.generate_rows(100);
        assert_eq!(rows.len(), 100);
    }

    #[test]
    fn test_dataset_generator_with_offset() {
        let mut generator = DatasetGenerator::new();
        let rows = generator.generate_rows_with_offset(1000, 50);
        assert_eq!(rows.len(), 50);
        assert_eq!(rows[0].id, 1000);
    }

    #[test]
    fn test_generate_insert_sql() {
        let mut generator = DatasetGenerator::new();
        let sql = generator.generate_insert_sql("test_table", 3);
        assert_eq!(sql.len(), 3);
        assert!(sql[0].starts_with("INSERT INTO test_table VALUES"));
    }

    #[test]
    fn test_generate_benchmark_data() {
        let data = generate_benchmark_data(DataScale::Tiny);
        assert_eq!(data.len(), 1_000);
    }

    #[test]
    fn test_time_generation() {
        let (data, duration) = time_generation(DataScale::Tiny);
        assert_eq!(data.len(), 1_000);
        assert!(duration.as_millis() < 100);
    }
}
