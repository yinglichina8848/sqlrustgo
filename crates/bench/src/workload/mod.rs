//! Workload generators for benchmark tests

pub mod oltp_point_select;
pub mod oltp_read_only;
pub mod oltp_read_write;

use async_trait::async_trait;
use rand::rngs::SmallRng;
use std::sync::Arc;

use crate::db::Database;

/// Workload trait - defines a benchmark workload
#[async_trait]
pub trait Workload: Send + Sync {
    /// Execute one iteration of the workload
    async fn execute(&self, db: &dyn Database) -> anyhow::Result<()>;

    /// Get workload name
    fn name(&self) -> &str;

    /// Generate a single SQL statement
    fn generate_sql(&self, rng: &mut SmallRng) -> String;

    /// Generate a complete transaction with SQL statements
    fn generate_transaction(&self, rng: &mut SmallRng) -> Vec<String>;

    /// Number of statements per transaction
    fn statements_per_tx(&self) -> usize;

    /// Whether this is a read-only workload
    fn is_read_only(&self) -> bool;

    /// Get the table names used by this workload
    fn table_names(&self) -> Vec<String>;
}

/// Create a workload by name
pub fn create_workload(name: &str, _scale: usize) -> Arc<dyn Workload> {
    match name.to_lowercase().as_str() {
        "oltp_point_select" => Arc::new(oltp_point_select::OltpPointSelect::new()),
        "oltp_read_only" => Arc::new(oltp_read_only::OltpReadOnly::new()),
        "oltp_read_write" => Arc::new(oltp_read_write::OltpReadWrite::new()),
        _ => panic!("Unknown workload: {}", name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_workload_sql_generation() {
        let workloads = vec![
            create_workload("oltp_point_select", 1),
            create_workload("oltp_read_only", 1),
            create_workload("oltp_read_write", 1),
        ];

        for workload in workloads {
            let mut rng = SmallRng::seed_from_u64(42);

            // Test single SQL generation
            let sql = workload.generate_sql(&mut rng);
            assert!(
                !sql.is_empty(),
                "SQL should not be empty for {}",
                workload.name()
            );

            // Test transaction generation
            let tx = workload.generate_transaction(&mut rng);
            assert!(
                !tx.is_empty(),
                "Transaction should have statements for {}",
                workload.name()
            );
            assert_eq!(
                tx.len(),
                workload.statements_per_tx(),
                "Statement count should match"
            );

            // Test table names
            let tables = workload.table_names();
            assert!(
                !tables.is_empty(),
                "Should have table names for {}",
                workload.name()
            );
        }
    }

    #[test]
    fn test_oltp_point_select() {
        let workload = create_workload("oltp_point_select", 1);
        let mut rng = SmallRng::seed_from_u64(42);

        assert_eq!(workload.name(), "oltp_point_select");
        assert!(workload.is_read_only(), "Point select should be read-only");

        // Test SQL contains point select
        let sql = workload.generate_sql(&mut rng);
        assert!(sql.contains("SELECT"), "Should contain SELECT");
        assert!(sql.contains("WHERE id"), "Should contain WHERE id");

        // Test table names
        let tables = workload.table_names();
        assert!(
            tables.iter().any(|t| t.starts_with("sbtest")),
            "Should use sbtest table"
        );
    }

    #[test]
    fn test_oltp_read_only() {
        let workload = create_workload("oltp_read_only", 1);
        let mut rng = SmallRng::seed_from_u64(42);

        assert_eq!(workload.name(), "oltp_read_only");
        assert!(workload.is_read_only(), "Read-only should be read-only");

        // Test transaction contains only read operations
        let tx = workload.generate_transaction(&mut rng);
        for sql in &tx {
            let upper = sql.to_uppercase();
            assert!(
                upper.contains("SELECT")
                    && !upper.contains("UPDATE")
                    && !upper.contains("INSERT")
                    && !upper.contains("DELETE"),
                "Read-only transaction should only contain SELECT: {}",
                sql
            );
        }
    }

    #[test]
    fn test_oltp_read_write() {
        let workload = create_workload("oltp_read_write", 1);
        let mut rng = SmallRng::seed_from_u64(42);

        assert_eq!(workload.name(), "oltp_read_write");
        assert!(
            !workload.is_read_only(),
            "Read-write should not be read-only"
        );

        // Test transaction contains both read and write operations
        let tx = workload.generate_transaction(&mut rng);
        let has_read = tx.iter().any(|sql| sql.to_uppercase().contains("SELECT"));
        let has_write = tx.iter().any(|sql| {
            let upper = sql.to_uppercase();
            upper.contains("UPDATE") || upper.contains("INSERT") || upper.contains("DELETE")
        });
        assert!(has_read, "Read-write should contain SELECT");
        assert!(has_write, "Read-write should contain write operations");
    }
}
