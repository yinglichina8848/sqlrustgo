use clap::Parser;
use serde::{Deserialize, Serialize};

/// Benchmark configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Scale factor for TPC-H
    pub scale: Option<f64>,
    /// Number of iterations
    pub iterations: Option<u32>,
    /// Number of threads for OLTP
    pub threads: Option<u32>,
    /// Duration in seconds
    pub duration: Option<u64>,
    /// Workload type (read/write/mixed)
    pub workload: Option<String>,
    /// PostgreSQL connection string
    pub pg_conn: Option<String>,
    /// Output path
    pub output: Option<String>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            scale: Some(1.0),
            iterations: Some(3),
            threads: Some(1),
            duration: Some(60),
            workload: Some("read".to_string()),
            pg_conn: None,
            output: None,
        }
    }
}

#[derive(Parser, Debug)]
pub struct TpchArgs {
    /// Load configuration from file
    #[arg(long)]
    pub config: Option<String>,
    #[arg(long, default_value = "1")]
    pub scale: f64,
    #[arg(long, default_value = "3")]
    pub iterations: u32,
    #[arg(long, default_value = "Q1,Q3,Q6,Q10")]
    pub queries: String,
    #[arg(long)]
    pub output: Option<String>,
    /// PostgreSQL connection string for comparison
    #[arg(long)]
    pub pg_conn: Option<String>,
}

#[derive(Parser, Debug)]
pub struct OltpArgs {
    /// Load configuration from file
    #[arg(long)]
    pub config: Option<String>,
    #[arg(long, default_value = "1")]
    pub threads: u32,
    #[arg(long, default_value = "60")]
    pub duration: u64,
    #[arg(long, default_value = "read")]
    pub workload: String,
    #[arg(long)]
    pub output: Option<String>,
    /// PostgreSQL connection string for comparison
    #[arg(long)]
    pub pg_conn: Option<String>,
}

#[derive(Parser, Debug)]
pub struct CustomArgs {
    /// Load configuration from file
    #[arg(long)]
    pub config: Option<String>,
    #[arg(long)]
    pub file: String,
    #[arg(long, default_value = "1")]
    pub iterations: u32,
    #[arg(long, default_value = "1")]
    pub parallel: u32,
    #[arg(long)]
    pub output: Option<String>,
    /// PostgreSQL connection string for comparison
    #[arg(long)]
    pub pg_conn: Option<String>,
}
