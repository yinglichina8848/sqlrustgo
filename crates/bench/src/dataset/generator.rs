//! Dataset generator for benchmark tests

use crate::db::Database;
use anyhow::Result;
use std::sync::Arc;

use super::sbtest_schema::SBTEST_SCHEMA;

/// Dataset generator for creating sysbench-compatible test data
pub struct DatasetGenerator {
    db: Arc<dyn Database>,
    tables: usize,
    rows_per_table: usize,
}

impl DatasetGenerator {
    /// Create a new DatasetGenerator
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            tables: 1,
            rows_per_table: 1_000_000,
        }
    }

    /// Set number of tables to generate
    pub fn tables(mut self, n: usize) -> Self {
        self.tables = n;
        self
    }

    /// Set number of rows per table
    pub fn rows(mut self, n: usize) -> Self {
        self.rows_per_table = n;
        self
    }

    /// Generate the dataset (stub implementation)
    ///
    /// TODO: Implement actual data generation when sqlrustgo crate provides
    /// a proper Database implementation with execute method
    pub async fn generate(&self) -> Result<()> {
        // Stub: Create tables and insert data
        for i in 0..self.tables {
            let table_name = format!(
                "sbtest{}",
                if i == 0 {
                    String::new()
                } else {
                    i.to_string()
                }
            );

            // CREATE TABLE
            let create_sql = format!("CREATE TABLE {} ({})", table_name, SBTEST_SCHEMA);
            self.db.execute(&create_sql).await?;

            // Bulk insert with chunk size of 10,000
            let chunk_size = 10_000;
            for offset in (0..self.rows_per_table).step_by(chunk_size) {
                let values: Vec<String> = (offset..(offset + chunk_size))
                    .map(|id| {
                        format!(
                            "({}, {}, 'c{}', 'pad{}')",
                            id,
                            id % 1_000_000,
                            id,
                            id
                        )
                    })
                    .collect();

                let insert_sql = format!(
                    "INSERT INTO {} VALUES {}",
                    table_name,
                    values.join(",")
                );
                self.db.execute(&insert_sql).await?;
            }
        }
        Ok(())
    }
}
