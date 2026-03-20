//! Database adapters for benchmark

pub mod postgres;
pub mod sqlite;
pub mod sqlrustgo;

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Database trait for benchmark
#[async_trait]
pub trait Database: Send + Sync {
    /// Read a record by key
    async fn read(&self, key: usize) -> Result<()>;

    /// Update a record by key
    async fn update(&self, key: usize) -> Result<()>;

    /// Insert a new record
    async fn insert(&self, key: usize) -> Result<()>;

    /// Scan records in range [start, end)
    async fn scan(&self, start: usize, end: usize) -> Result<()>;
}

/// Create a database adapter
pub async fn create_db(name: &str, config: &DbConfig) -> Result<Arc<dyn Database>> {
    match name.to_lowercase().as_str() {
        "sqlrustgo" => Ok(Arc::new(
            sqlrustgo::SqlRustGoDB::new(&config.sqlrustgo_addr).await?,
        )),
        "postgres" | "postgresql" => {
            Ok(Arc::new(postgres::PostgresDB::new(&config.pg_conn).await?))
        }
        "sqlite" => Ok(Arc::new(
            sqlite::SqliteDB::new(&config.sqlite_path, config.scale).await?,
        )),
        _ => panic!("Unknown database: {}", name),
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub sqlrustgo_addr: String,
    pub pg_conn: String,
    pub sqlite_path: String,
    pub scale: usize,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            sqlrustgo_addr: "127.0.0.1:4000".to_string(),
            pg_conn: "host=localhost user=postgres password=postgres dbname=bench".to_string(),
            sqlite_path: "bench.db".to_string(),
            scale: 10000,
        }
    }
}

impl From<&crate::cli::BenchArgs> for DbConfig {
    fn from(args: &crate::cli::BenchArgs) -> Self {
        Self {
            sqlrustgo_addr: args.sqlrustgo_addr.clone(),
            pg_conn: args.get_pg_conn(),
            sqlite_path: args.get_sqlite_path(),
            scale: args.scale,
        }
    }
}
