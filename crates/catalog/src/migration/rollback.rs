//! Rollback Module
//!
//! Provides rollback support for DDL operations.

use super::migration_log::{MigrationEntry, MigrationLog, MigrationType};

#[derive(Debug, Clone)]
pub struct RollbackResult {
    pub success: bool,
    pub message: String,
    pub rollback_state: Option<String>,
}

pub struct DdlRollback;

impl DdlRollback {
    pub fn new() -> Self {
        Self
    }

    pub fn rollback_entry(&self, log: &mut MigrationLog, entry_id: u64) -> RollbackResult {
        let entry = match log.get_entry_mut(entry_id) {
            Some(e) => e,
            None => {
                return RollbackResult {
                    success: false,
                    message: format!("Migration entry {} not found", entry_id),
                    rollback_state: None,
                };
            }
        };

        if entry.status == super::MigrationStatus::RolledBack {
            return RollbackResult {
                success: false,
                message: format!("Migration {} already rolled back", entry_id),
                rollback_state: None,
            };
        }

        let rollback_state = match entry.migration_type {
            MigrationType::AddColumn => entry.before_state.clone(),
            MigrationType::DropColumn => entry.after_state.clone(),
            MigrationType::RenameColumn => entry.before_state.clone(),
            MigrationType::RenameTable => entry.before_state.clone(),
            MigrationType::AddIndex => entry.before_state.clone(),
            MigrationType::DropIndex => entry.after_state.clone(),
        };

        entry.rollback();

        RollbackResult {
            success: true,
            message: format!(
                "Migration {} ({}) rolled back successfully",
                entry_id,
                entry.migration_type.as_str()
            ),
            rollback_state,
        }
    }

    pub fn can_rollback(entry: &MigrationEntry) -> bool {
        use super::MigrationStatus;
        matches!(
            entry.status,
            MigrationStatus::Pending | MigrationStatus::InProgress | MigrationStatus::Completed
        ) && entry.migration_type == MigrationType::AddColumn
    }

    pub fn get_rollback_state(entry: &MigrationEntry) -> Option<String> {
        if !Self::can_rollback(entry) {
            return None;
        }
        entry.before_state.clone()
    }
}

impl Default for DdlRollback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::migration_log::MigrationEntry;
    use super::*;

    #[test]
    fn test_rollback_add_column() {
        let mut log = MigrationLog::new();
        log.create_entry(
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            Some("{\"columns\":[]}".to_string()),
            Some("{\"columns\":[{\"name\":\"email\"}]}".to_string()),
            100,
        );

        let rollback = DdlRollback::new();
        let result = rollback.rollback_entry(&mut log, 1);

        assert!(result.success);
        assert!(result.rollback_state.is_some());
    }

    #[test]
    fn test_rollback_not_found() {
        let mut log = MigrationLog::new();
        let rollback = DdlRollback::new();
        let result = rollback.rollback_entry(&mut log, 999);

        assert!(!result.success);
        assert!(result.message.contains("not found"));
    }

    #[test]
    fn test_can_rollback() {
        let entry = MigrationEntry::new(
            1,
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            None,
            None,
            100,
        );

        assert!(DdlRollback::can_rollback(&entry));
    }

    #[test]
    fn test_get_rollback_state() {
        let entry = MigrationEntry::new(
            1,
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            Some("before".to_string()),
            Some("after".to_string()),
            100,
        );

        let state = DdlRollback::get_rollback_state(&entry);
        assert_eq!(state, Some("before".to_string()));
    }
}
