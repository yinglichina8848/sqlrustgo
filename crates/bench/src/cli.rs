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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_args_defaults() {
        let args = BenchArgs::parse_from(["bench", "--db", "sqlrustgo"]);
        assert_eq!(args.db, "sqlrustgo");
        assert_eq!(args.workload, "oltp");
        assert_eq!(args.threads, 10);
        assert_eq!(args.duration, 60);
        assert_eq!(args.scale, 10000);
        assert!(!args.enable_cache);
        assert_eq!(args.output, "json");
    }

    #[test]
    fn test_bench_args_custom() {
        let args = BenchArgs::parse_from([
            "bench",
            "--db",
            "postgres",
            "--workload",
            "tpch",
            "--threads",
            "4",
            "--duration",
            "30",
            "--scale",
            "5000",
            "--enable-cache",
            "--output",
            "text",
        ]);
        assert_eq!(args.db, "postgres");
        assert_eq!(args.workload, "tpch");
        assert_eq!(args.threads, 4);
        assert_eq!(args.duration, 30);
        assert_eq!(args.scale, 5000);
        assert!(args.enable_cache);
        assert_eq!(args.output, "text");
    }

    #[test]
    fn test_bench_args_pg_conn() {
        let args = BenchArgs::parse_from(["bench", "--pg-conn", "host=localhost user=test"]);
        assert_eq!(args.get_pg_conn(), "host=localhost user=test");
    }

    #[test]
    fn test_bench_args_pg_conn_default() {
        let args = BenchArgs::parse_from(["bench"]);
        let conn = args.get_pg_conn();
        assert!(conn.contains("host=localhost"));
        assert!(conn.contains("user=postgres"));
    }

    #[test]
    fn test_bench_args_sqlite_path() {
        let args = BenchArgs::parse_from(["bench", "--sqlite-path", "/tmp/test.db"]);
        assert_eq!(args.get_sqlite_path(), "/tmp/test.db");
    }

    #[test]
    fn test_bench_args_sqlite_path_default() {
        let args = BenchArgs::parse_from(["bench"]);
        assert_eq!(args.get_sqlite_path(), "bench.db");
    }

    #[test]
    fn test_bench_args_sqlrustgo_addr() {
        let args = BenchArgs::parse_from(["bench", "--sqlrustgo-addr", "192.168.1.1:4000"]);
        assert_eq!(args.sqlrustgo_addr, "192.168.1.1:4000");
    }
}
