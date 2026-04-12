use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoRecord {
    Insert { key: Vec<u8> },
    Delete { key: Vec<u8>, old_value: Vec<u8> },
    Update { key: Vec<u8>, old_value: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct Savepoint {
    pub name: String,
    pub undo_log_index: usize,
}

impl Savepoint {
    pub fn new(name: String, undo_log_index: usize) -> Self {
        Self {
            name,
            undo_log_index,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SavepointError {
    NotFound,
    InvalidOperation,
}

impl std::fmt::Display for SavepointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SavepointError::NotFound => write!(f, "savepoint not found"),
            SavepointError::InvalidOperation => write!(f, "invalid savepoint operation"),
        }
    }
}

impl std::error::Error for SavepointError {}

pub struct SavepointManager {
    savepoints: Vec<Savepoint>,
    undo_log: Vec<UndoRecord>,
}

impl SavepointManager {
    pub fn new() -> Self {
        Self {
            savepoints: Vec::new(),
            undo_log: Vec::new(),
        }
    }

    pub fn savepoint(&mut self, name: String) -> Result<(), SavepointError> {
        if let Some(idx) = self.savepoints.iter().rposition(|s| s.name == name) {
            self.savepoints[idx].undo_log_index = self.undo_log.len();
        } else {
            self.savepoints
                .push(Savepoint::new(name, self.undo_log.len()));
        }
        Ok(())
    }

    pub fn rollback_to(&mut self, name: &str) -> Result<(), SavepointError> {
        let idx = self
            .savepoints
            .iter()
            .rposition(|s| s.name == name)
            .ok_or(SavepointError::NotFound)?;

        let sp = &self.savepoints[idx];

        while self.undo_log.len() > sp.undo_log_index {
            self.undo_log.pop();
        }

        self.savepoints.truncate(idx + 1);

        Ok(())
    }

    pub fn release_savepoint(&mut self, name: &str) -> Result<(), SavepointError> {
        self.savepoints.retain(|s| s.name != name);
        Ok(())
    }

    pub fn add_undo(&mut self, record: UndoRecord) {
        self.undo_log.push(record);
    }

    pub fn get_savepoint_count(&self) -> usize {
        self.savepoints.len()
    }
}

impl Default for SavepointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_record_insert() {
        let record = UndoRecord::Insert { key: vec![1, 2, 3] };
        assert!(matches!(record, UndoRecord::Insert { .. }));
    }

    #[test]
    fn test_savepoint_new() {
        let sp = Savepoint::new("test".to_string(), 10);
        assert_eq!(sp.name, "test");
        assert_eq!(sp.undo_log_index, 10);
    }

    #[test]
    fn test_savepoint_create() {
        let mut manager = SavepointManager::new();
        manager.savepoint("sp1".to_string()).unwrap();
        assert_eq!(manager.savepoints.len(), 1);
    }

    #[test]
    fn test_nested_savepoints() {
        let mut manager = SavepointManager::new();
        manager.savepoint("sp1".to_string()).unwrap();
        manager.savepoint("sp2".to_string()).unwrap();
        assert_eq!(manager.savepoints.len(), 2);

        manager.rollback_to("sp1").unwrap();
        assert_eq!(manager.savepoints.len(), 1);
    }

    #[test]
    fn test_savepoint_not_found() {
        let mut manager = SavepointManager::new();
        let result = manager.rollback_to("nonexistent");
        assert!(matches!(result, Err(SavepointError::NotFound)));
    }

    #[test]
    fn test_savepoint_error_display() {
        let err_not_found = SavepointError::NotFound;
        let err_invalid = SavepointError::InvalidOperation;
        assert_eq!(err_not_found.to_string(), "savepoint not found");
        assert_eq!(err_invalid.to_string(), "invalid savepoint operation");
    }

    #[test]
    fn test_add_undo() {
        let mut manager = SavepointManager::new();
        manager.add_undo(UndoRecord::Insert { key: vec![1, 2, 3] });
        assert_eq!(manager.undo_log.len(), 1);
    }

    #[test]
    fn test_add_undo_update() {
        let mut manager = SavepointManager::new();
        manager.add_undo(UndoRecord::Update {
            key: vec![1],
            old_value: vec![2],
        });
        manager.add_undo(UndoRecord::Delete {
            key: vec![3],
            old_value: vec![4],
        });
        assert_eq!(manager.undo_log.len(), 2);
    }

    #[test]
    fn test_get_savepoint_count() {
        let mut manager = SavepointManager::new();
        assert_eq!(manager.get_savepoint_count(), 0);

        manager.savepoint("sp1".to_string()).unwrap();
        assert_eq!(manager.get_savepoint_count(), 1);

        manager.savepoint("sp2".to_string()).unwrap();
        assert_eq!(manager.get_savepoint_count(), 2);
    }

    #[test]
    fn test_release_savepoint() {
        let mut manager = SavepointManager::new();
        manager.savepoint("sp1".to_string()).unwrap();
        manager.savepoint("sp2".to_string()).unwrap();

        manager.release_savepoint("sp1").unwrap();
        assert_eq!(manager.get_savepoint_count(), 1);

        manager.release_savepoint("sp2").unwrap();
        assert_eq!(manager.get_savepoint_count(), 0);
    }

    #[test]
    fn test_release_nonexistent_savepoint() {
        let mut manager = SavepointManager::new();
        let result = manager.release_savepoint("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rollback_preserves_earlier_savepoints() {
        let mut manager = SavepointManager::new();
        manager.savepoint("sp1".to_string()).unwrap();
        manager.savepoint("sp2".to_string()).unwrap();

        manager.rollback_to("sp1").unwrap();

        assert!(manager.savepoints.iter().any(|s| s.name == "sp1"));
    }

    #[test]
    fn test_savepoint_override() {
        let mut manager = SavepointManager::new();
        manager.add_undo(UndoRecord::Insert { key: vec![1] });
        manager.savepoint("sp1".to_string()).unwrap();

        manager.add_undo(UndoRecord::Insert { key: vec![2] });
        manager.add_undo(UndoRecord::Insert { key: vec![3] });

        let idx = manager
            .savepoints
            .iter()
            .rposition(|s| s.name == "sp1")
            .unwrap();
        assert_eq!(manager.savepoints[idx].undo_log_index, 1);

        manager.savepoint("sp1".to_string()).unwrap();

        let idx2 = manager
            .savepoints
            .iter()
            .rposition(|s| s.name == "sp1")
            .unwrap();
        assert_eq!(manager.savepoints[idx2].undo_log_index, 3);
    }
}
