//! SQLRustGo Tools
//!
//! Collection of utility tools for SQLRustGo database.

mod backup;
mod catalog_check;
mod ha;
mod mysqldump;
mod physical_backup;
mod upgrade;

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "sqlrustgo-tools", about = "SQLRustGo database tools")]
enum Command {
    /// Backup and restore database (logical backup)
    Backup(backup::BackupCommand),
    /// Physical backup based on storage snapshots
    PhysicalBackup(physical_backup::PhysicalBackupCommand),
    /// Validate catalog invariants
    CatalogCheck(catalog_check::Opt),
    /// High Availability cluster management
    HA(ha::HACommand),
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
        Command::Backup(backup_cmd) => match backup_cmd {
            backup::BackupCommand::Backup {
                dir,
                format,
                data_dir,
            } => backup::create_full_backup(&dir, &format, &data_dir),
            backup::BackupCommand::Incremental {
                parent,
                dir,
                format,
                data_dir,
            } => backup::create_incremental_backup(&parent, &dir, &format, &data_dir),
            backup::BackupCommand::List { dir } => backup::list_backups(&dir),
            backup::BackupCommand::Verify { dir } => backup::verify_backup(&dir),
            backup::BackupCommand::Restore { dir, target, clean } => {
                backup::restore_backup(&dir, &target, clean)
            }
        },
        Command::PhysicalBackup(physical_cmd) => match physical_cmd {
            physical_backup::PhysicalBackupCommand::Backup {
                dir,
                data_dir,
                wal_dir,
                compress,
                parent,
            } => physical_backup::create_physical_backup(
                &dir,
                &data_dir,
                &wal_dir,
                compress,
                parent.as_deref(),
            ),
            physical_backup::PhysicalBackupCommand::List { dir } => {
                physical_backup::list_physical_backups(&dir)
            }
            physical_backup::PhysicalBackupCommand::Verify { dir } => {
                physical_backup::verify_physical_backup(&dir)
            }
            physical_backup::PhysicalBackupCommand::Restore {
                dir,
                target,
                wal_target,
            } => physical_backup::restore_physical_backup(&dir, &target, wal_target.as_deref()),
        },
        Command::CatalogCheck(opt) => catalog_check::run_with_opt(opt),
        Command::HA(_ha_cmd) => ha::run(),
        Command::Upgrade(upgrade_cmd) => upgrade::run_with_opt(upgrade_cmd),
        Command::Import(import_cmd) => mysqldump::run_import(import_cmd),
    }
}
