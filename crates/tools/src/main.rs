//! SQLRustGo Tools
//!
//! Collection of utility tools for SQLRustGo database.

mod mysqldump;
mod upgrade;

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqlrustgo-tools", about = "SQLRustGo database tools")]
enum Command {
    /// Upgrade database version
    Upgrade(upgrade::UpgradeCommand),
    /// Import mysqldump format SQL files
    Import(mysqldump::ImportCommand),
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cmd = Command::from_args();

    match cmd {
        Command::Upgrade(upgrade_cmd) => upgrade::run_with_opt(upgrade_cmd),
        Command::Import(import_cmd) => mysqldump::run_import(import_cmd),
    }
}
