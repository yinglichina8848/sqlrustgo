//! Migration Log Module
//!
//! Provides DDL journal for safe schema changes with rollback support.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationType {
    AddColumn,
    DropColumn,
    RenameColumn,
    RenameTable,
    AddIndex,
    DropIndex,
}

impl MigrationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MigrationType::AddColumn => "ADD_COLUMN",
            MigrationType::DropColumn => "DROP_COLUMN",
            MigrationType::RenameColumn => "RENAME_COLUMN",
            MigrationType::RenameTable => "RENAME_TABLE",
            MigrationType::AddIndex => "ADD_INDEX",
            MigrationType::DropIndex => "DROP_INDEX",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationEntry {
    pub id: u64,
    pub migration_type: MigrationType,
    pub table_name: String,
    pub target_name: String,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub status: MigrationStatus,
    pub timestamp: u64,
    pub tx_id: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MigrationStatus {
    Pending,
    InProgress,
    Completed,
    RolledBack,
    Failed,
}

impl MigrationEntry {
    pub fn new(
        id: u64,
        migration_type: MigrationType,
        table_name: String,
        target_name: String,
        before_state: Option<String>,
        after_state: Option<String>,
        tx_id: u64,
    ) -> Self {
        Self {
            id,
            migration_type,
            table_name,
            target_name,
            before_state,
            after_state,
            status: MigrationStatus::Pending,
            timestamp: now_timestamp(),
            tx_id,
        }
    }

    pub fn start(&mut self) {
        self.status = MigrationStatus::InProgress;
    }

    pub fn complete(&mut self) {
        self.status = MigrationStatus::Completed;
    }

    pub fn rollback(&mut self) {
        self.status = MigrationStatus::RolledBack;
    }

    pub fn fail(&mut self) {
        self.status = MigrationStatus::Failed;
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationLog {
    entries: Vec<MigrationEntry>,
    next_id: u64,
}

impl MigrationLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_entry(&mut self, entry: MigrationEntry) -> &MigrationEntry {
        self.next_id = entry.id + 1;
        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    pub fn create_entry(
        &mut self,
        migration_type: MigrationType,
        table_name: String,
        target_name: String,
        before_state: Option<String>,
        after_state: Option<String>,
        tx_id: u64,
    ) -> u64 {
        let entry = MigrationEntry::new(
            self.next_id,
            migration_type,
            table_name,
            target_name,
            before_state,
            after_state,
            tx_id,
        );
        let id = entry.id;
        self.add_entry(entry);
        id
    }

    pub fn find_pending(&self) -> Vec<&MigrationEntry> {
        self.entries
            .iter()
            .filter(|e| {
                e.status == MigrationStatus::Pending || e.status == MigrationStatus::InProgress
            })
            .collect()
    }

    pub fn find_failed(&self) -> Vec<&MigrationEntry> {
        self.entries
            .iter()
            .filter(|e| e.status == MigrationStatus::Failed)
            .collect()
    }

    pub fn get_entry(&self, id: u64) -> Option<&MigrationEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn get_entry_mut(&mut self, id: u64) -> Option<&mut MigrationEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    pub fn entries(&self) -> &[MigrationEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_log_create_entry() {
        let mut log = MigrationLog::new();
        let id = log.create_entry(
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            None,
            Some("{\"name\":\"email\",\"type\":\"VARCHAR\"}".to_string()),
            100,
        );

        let entry = log.get_entry(id).unwrap();
        assert_eq!(entry.id, 1);
        assert_eq!(entry.migration_type, MigrationType::AddColumn);
        assert_eq!(entry.status, MigrationStatus::Pending);
    }

    #[test]
    fn test_migration_log_find_pending() {
        let mut log = MigrationLog::new();
        log.create_entry(
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            None,
            None,
            100,
        );
        log.create_entry(
            MigrationType::DropColumn,
            "users".to_string(),
            "phone".to_string(),
            None,
            None,
            101,
        );

        let pending = log.find_pending();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_migration_entry_status_transitions() {
        let mut entry = MigrationEntry::new(
            1,
            MigrationType::AddColumn,
            "users".to_string(),
            "email".to_string(),
            None,
            None,
            100,
        );

        assert_eq!(entry.status, MigrationStatus::Pending);
        entry.start();
        assert_eq!(entry.status, MigrationStatus::InProgress);
        entry.complete();
        assert_eq!(entry.status, MigrationStatus::Completed);
    }

    #[test]
    fn test_migration_type_str() {
        assert_eq!(MigrationType::AddColumn.as_str(), "ADD_COLUMN");
        assert_eq!(MigrationType::DropColumn.as_str(), "DROP_COLUMN");
        assert_eq!(MigrationType::RenameColumn.as_str(), "RENAME_COLUMN");
    }
}
