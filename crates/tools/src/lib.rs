//! SQLRustGo Tools Library
//!
//! This library provides utility functionality for SQLRustGo including:
//! - Mysqldump import tool
//! - Upgrade tools
//! - HA cluster management
//! - Backup and restore

#![allow(
    clippy::map_clone,
    clippy::unwrap_or_default,
    unused_imports,
    unused_assignments,
    renamed_and_removed_lints
)]

pub mod backup_restore;
pub mod mysqldump;
pub mod upgrade;

pub use backup_restore::{
    BackupManager, BackupMetadata, BackupStatus, BackupType, ExportOptions, RestoreResult,
};
pub use mysqldump::{
    ColumnDef, DumpImporter, ForeignKeyRef, ImportMode, ImportStats, SqlStatement,
};
