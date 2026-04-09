//! SQLRustGo MySQL Protocol Server Binary
//!
//! Starts a MySQL Wire Protocol server that accepts connections
//! from standard MySQL clients (mysql CLI, DBeaver, etc.)
//!
//! Usage:
//!   sqlrustgo-mysql-server --host 127.0.0.1 --port 3306

use clap::Parser;
use sqlrustgo_mysql_server::run_server;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "sqlrustgo-mysql-server")]
#[command(about = "SQLRustGo MySQL Wire Protocol Server")]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value = "3306")]
    port: u16,

    #[arg(long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&args.log_level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    tracing::info!("SQLRustGo MySQL Server v2.4.0");
    tracing::info!("MySQL protocol server for SQLRustGo");
    tracing::info!("Accepts standard MySQL client connections");

    run_server(&args.host, args.port)?;

    Ok(())
}
