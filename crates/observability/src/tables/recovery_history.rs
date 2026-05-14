use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStatus {
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryHistoryEntry {
    pub recovery_id: u64,
    pub crash_timestamp: u64,
    pub recovery_timestamp: u64,
    pub lsn_recovered: u64,
    pub transactions_replayed: u64,
    pub status: RecoveryStatus,
    pub error_message: Option<String>,
}

impl RecoveryHistoryEntry {
    pub fn new(
        recovery_id: u64,
        crash_timestamp: u64,
        lsn_recovered: u64,
        transactions_replayed: u64,
        status: RecoveryStatus,
    ) -> Self {
        Self {
            recovery_id,
            crash_timestamp,
            recovery_timestamp: current_timestamp(),
            lsn_recovered,
            transactions_replayed,
            status,
            error_message: None,
        }
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct RecoveryHistory {
    entries: VecDeque<RecoveryHistoryEntry>,
    file_path: PathBuf,
    max_entries: usize,
}

impl RecoveryHistory {
    pub fn new(data_dir: PathBuf, max_entries: usize) -> Self {
        let file_path = data_dir.join("recovery_history.bin");
        let mut history = Self {
            entries: VecDeque::with_capacity(max_entries),
            file_path,
            max_entries,
        };
        let _ = history.load();
        history
    }

    pub fn append(&mut self, entry: RecoveryHistoryEntry) -> std::io::Result<()> {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry.clone());
        self.flush_entry(&entry)
    }

    fn flush_entry(&self, entry: &RecoveryHistoryEntry) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        let mut writer = BufWriter::new(file);
        let bytes = bincode::serialize(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        writer.write_all(&bytes.len().to_le_bytes())?;
        writer.write_all(&bytes)?;
        writer.flush()?;
        Ok(())
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        let file = File::open(&self.file_path)?;
        let mut reader = BufReader::new(file);
        self.entries.clear();

        loop {
            let mut len_bytes = [0u8; 8];
            match reader.read_exact(&mut len_bytes) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let len = u64::from_le_bytes(len_bytes) as usize;
            let mut entry_bytes = vec![0u8; len];
            reader.read_exact(&mut entry_bytes)?;
            if let Ok(entry) = bincode::deserialize(&entry_bytes) {
                self.entries.push_back(entry);
            }
        }

        Ok(())
    }

    pub fn query(&self, limit: Option<usize>) -> Vec<&RecoveryHistoryEntry> {
        let limit = limit.unwrap_or(self.entries.len());
        self.entries.iter().rev().take(limit).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_history_append_and_query() {
        let dir = std::env::temp_dir();
        let mut history = RecoveryHistory::new(dir.clone(), 100);

        let entry = RecoveryHistoryEntry::new(1, 1000, 5000, 10, RecoveryStatus::Success);
        history.append(entry).unwrap();

        assert_eq!(history.len(), 1);
        let results = history.query(Some(10));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].recovery_id, 1);
    }

    #[test]
    fn test_recovery_history_with_error() {
        let dir = std::env::temp_dir();
        let mut history = RecoveryHistory::new(dir.clone(), 100);

        let entry = RecoveryHistoryEntry::new(1, 1000, 5000, 10, RecoveryStatus::Failed)
            .with_error("Corruption detected".to_string());
        history.append(entry).unwrap();

        let results = history.query(None);
        assert!(results[0].error_message.is_some());
    }
}
