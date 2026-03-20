//! Command-line interface for benchmark runner

use clap::Parser;

/// Benchmark configuration arguments
#[derive(Parser, Debug)]
#[command(author, version, about = "SQLRustGo Benchmark Runner")]
pub struct BenchArgs {
    /// Database to benchmark: sqlrustgo, postgres, sqlite
    #[arg(long, default_value = "sqlrustgo")]
    pub db: String,

    /// Workload type: oltp, tpch
    #[arg(long, default_value = "oltp")]
    pub workload: String,

    /// Number of concurrent threads
    #[arg(long, short, default_value_t = 10)]
    pub threads: usize,

    /// Test duration in seconds
    #[arg(long, short, default_value_t = 60)]
    pub duration: u64,

    /// Data scale (number of rows)
    #[arg(long, short, default_value_t = 10000)]
    pub scale: usize,

    /// Enable query cache (for comparison)
    #[arg(long, default_value_t = false)]
    pub enable_cache: bool,

    /// Output format: json, text
    #[arg(long, default_value = "json")]
    pub output: String,

    /// PostgreSQL connection string (when using postgres)
    #[arg(long)]
    pub pg_conn: Option<String>,

    /// SQLite database path (when using sqlite)
    #[arg(long)]
    pub sqlite_path: Option<String>,

    /// SQLRustGo TCP server address
    #[arg(long, default_value = "127.0.0.1:4000")]
    pub sqlrustgo_addr: String,
}

impl BenchArgs {
    /// Get PostgreSQL connection string or use default
    pub fn get_pg_conn(&self) -> String {
        self.pg_conn.clone().unwrap_or_else(|| {
            "host=localhost user=postgres password=postgres dbname=bench".to_string()
        })
    }

    /// Get SQLite database path or use default
    pub fn get_sqlite_path(&self) -> String {
        self.sqlite_path
            .clone()
            .unwrap_or_else(|| "bench.db".to_string())
    }
}
