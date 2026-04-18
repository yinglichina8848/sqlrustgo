//! SQLRustGo Tools Library
//!
//! This library provides utility functionality for SQLRustGo including:
//! - Mysqldump import tool
//! - Upgrade tools
//! - HA cluster management
//! - Backup and restore

pub mod backup_restore;
pub mod mysqldump;
pub mod upgrade;

pub use backup_restore::{BackupManager, BackupMetadata, BackupStatus, BackupType, ExportOptions, RestoreResult};
pub use mysqldump::{
    ColumnDef, DumpImporter, ForeignKeyRef, ImportMode, ImportStats, SqlStatement,
};
