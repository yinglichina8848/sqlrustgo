use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Active,
    Committed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryEntry {
    pub tx_id: u64,
    pub tx_uuid: String,
    pub start_time: u64,
    pub commit_time: Option<u64>,
    pub abort_time: Option<u64>,
    pub isolation_level: String,
    pub status: TransactionStatus,
}

impl TransactionHistoryEntry {
    pub fn new(tx_id: u64, isolation: String) -> Self {
        Self {
            tx_id,
            tx_uuid: uuid::Uuid::new_v4().to_string(),
            start_time: current_timestamp(),
            commit_time: None,
            abort_time: None,
            isolation_level: isolation,
            status: TransactionStatus::Active,
        }
    }

    pub fn commit(&mut self) {
        self.commit_time = Some(current_timestamp());
        self.status = TransactionStatus::Committed;
    }

    pub fn abort(&mut self) {
        self.abort_time = Some(current_timestamp());
        self.status = TransactionStatus::Aborted;
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct TransactionHistory {
    store: VecDeque<TransactionHistoryEntry>,
    max_memory: usize,
}

impl TransactionHistory {
    pub fn new(max_memory: usize) -> Self {
        Self {
            store: VecDeque::with_capacity(max_memory),
            max_memory,
        }
    }

    pub fn append(&mut self, entry: TransactionHistoryEntry) {
        if self.store.len() >= self.max_memory {
            self.store.pop_front();
        }
        self.store.push_back(entry);
    }

    pub fn query(&self, limit: Option<usize>) -> Vec<&TransactionHistoryEntry> {
        let limit = limit.unwrap_or(self.store.len());
        self.store.iter().rev().take(limit).collect()
    }

    pub fn find_by_tx_id(&self, tx_id: u64) -> Option<&TransactionHistoryEntry> {
        self.store.iter().find(|e| e.tx_id == tx_id)
    }

    pub fn update_status(&mut self, tx_id: u64, status: TransactionStatus) {
        if let Some(entry) = self.store.iter_mut().find(|e| e.tx_id == tx_id) {
            match status {
                TransactionStatus::Committed => entry.commit(),
                TransactionStatus::Aborted => entry.abort(),
                TransactionStatus::Active => {}
            }
        }
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_history_append_and_query() {
        let mut history = TransactionHistory::new(100);
        let entry = TransactionHistoryEntry::new(1, "SI".to_string());
        history.append(entry);
        assert_eq!(history.len(), 1);

        let results = history.query(Some(10));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tx_id, 1);
    }

    #[test]
    fn test_transaction_history_commit() {
        let mut history = TransactionHistory::new(100);
        let entry = TransactionHistoryEntry::new(1, "SI".to_string());
        history.append(entry);
        history.update_status(1, TransactionStatus::Committed);

        let result = history.find_by_tx_id(1).unwrap();
        assert_eq!(result.status, TransactionStatus::Committed);
        assert!(result.commit_time.is_some());
    }

    #[test]
    fn test_transaction_history_eviction() {
        let mut history = TransactionHistory::new(3);
        for i in 1..=4 {
            let entry = TransactionHistoryEntry::new(i, "SI".to_string());
            history.append(entry);
        }
        assert_eq!(history.len(), 3);
        assert!(history.find_by_tx_id(1).is_none());
        assert!(history.find_by_tx_id(4).is_some());
    }
}
