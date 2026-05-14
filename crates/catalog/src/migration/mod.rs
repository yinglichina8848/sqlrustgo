//! Migration Module - Safe DDL with Rollback Support
//!
//! Provides DDL journal, recovery, and rollback for ALTER TABLE operations.

pub mod ddl_recovery;
pub mod migration_log;
pub mod rollback;

pub use ddl_recovery::{DdlRecovery, DdlRecoveryResult};
pub use migration_log::{MigrationEntry, MigrationLog, MigrationStatus, MigrationType};
pub use rollback::{DdlRollback, RollbackResult};
