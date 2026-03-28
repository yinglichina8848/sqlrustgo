//! SQLRustGo Tools
//!
//! Collection of utility tools for SQLRustGo database.

mod backup;
mod catalog_check;

use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqlrustgo-tools", about = "SQLRustGo database tools")]
enum Command {
    /// Backup and restore database
    Backup(backup::BackupCommand),
    /// Validate catalog invariants
    CatalogCheck(catalog_check::Opt),
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cmd = Command::from_args();

    match cmd {
        Command::Backup(backup_cmd) => {
            // Convert backup command to its internal format and run
            match backup_cmd {
                backup::BackupCommand::Backup { dir, format, data_dir } => {
                    backup::create_full_backup(&dir, &format, &data_dir)
                }
                backup::BackupCommand::Incremental { parent, dir, format, data_dir } => {
                    backup::create_incremental_backup(&parent, &dir, &format, &data_dir)
                }
                backup::BackupCommand::List { dir } => {
                    backup::list_backups(&dir)
                }
                backup::BackupCommand::Verify { dir } => {
                    backup::verify_backup(&dir)
                }
                backup::BackupCommand::Restore { dir, target, clean } => {
                    backup::restore_backup(&dir, &target, clean)
                }
            }
        }
        Command::CatalogCheck(opt) => {
            catalog_check::run_with_opt(opt)
        }
    }
}
