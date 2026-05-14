//! DDL Recovery Module
//!
//! Provides recovery for interrupted ALTER TABLE operations.

use super::migration_log::{MigrationEntry, MigrationLog, MigrationStatus, MigrationType};

#[derive(Debug, Clone)]
pub struct DdlRecoveryResult {
    pub recovered_entries: u64,
    pub failed_recoveries: u64,
    pub errors: Vec<String>,
}

pub struct DdlRecovery;

impl DdlRecovery {
    pub fn new() -> Self {
        Self
    }

    pub fn recover_interrupted(&self, log: &mut MigrationLog) -> DdlRecoveryResult {
        let mut result = DdlRecoveryResult {
            recovered_entries: 0,
            failed_recoveries: 0,
            errors: Vec::new(),
        };

        let pending = log.find_pending();
        for entry in pending {
            let err = format!(
                "Interrupted migration {} of type {} on table {}",
                entry.id,
                entry.migration_type.as_str(),
                entry.table_name
            );
            result.errors.push(err);
            result.failed_recoveries += 1;
        }

        let failed = log.find_failed();
        result.recovered_entries = failed.len() as u64;

        DdlRecoveryResult {
            recovered_entries: result.recovered_entries,
            failed_recoveries: result.failed_recoveries,
            errors: result.errors,
        }
    }

    pub fn can_recover(entry: &MigrationEntry) -> bool {
        matches!(
            entry.status,
            MigrationStatus::Pending | MigrationStatus::InProgress
        ) && entry.migration_type == MigrationType::AddColumn
    }

    pub fn recover_add_column(entry: &MigrationEntry) -> Option<String> {
        if !Self::can_recover(entry) {
            return None;
        }
        entry.before_state.clone()
    }
}

impl Default for DdlRecovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::migration_log::MigrationEntry;
    use super::*;

    #[test]
    fn test_ddl_recovery_recover_interrupted() {
        let mut log = MigrationLog::new();
        log.create_entry(
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            None,
            Some("{}".to_string()),
            100,
        );

        let recovery = DdlRecovery::new();
        let result = recovery.recover_interrupted(&mut log);

        assert_eq!(result.failed_recoveries, 1);
    }

    #[test]
    fn test_can_recover() {
        let entry = MigrationEntry::new(
            1,
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            None,
            None,
            100,
        );

        assert!(DdlRecovery::can_recover(&entry));
    }

    #[test]
    fn test_recover_add_column() {
        let entry = MigrationEntry::new(
            1,
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            Some("old_state".to_string()),
            Some("new_state".to_string()),
            100,
        );

        let recovered = DdlRecovery::recover_add_column(&entry);
        assert_eq!(recovered, Some("old_state".to_string()));
    }
}
