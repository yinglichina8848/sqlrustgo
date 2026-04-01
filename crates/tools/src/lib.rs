//! SQLRustGo Tools Library
//!
//! This library provides utility functionality for SQLRustGo including:
//! - Backup and restore tools
//! - Mysqldump import tool
//! - Upgrade tools
//! - HA cluster management

pub mod backup;
pub mod mysqldump;
pub mod upgrade;

pub use mysqldump::{
    ColumnDef, DumpImporter, ForeignKeyRef, ImportMode, ImportStats, SqlStatement,
};

pub use backup::{
    BackupManifest, BackupType, ChangeOperation, ChangeRecord, ChangeSet, IncrementalBackupContext,
    TableBackupInfo,
};
