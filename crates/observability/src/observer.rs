use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservableEvent {
    BeginTransaction {
        tx_id: u64,
        isolation: String,
    },
    CommitTransaction {
        tx_id: u64,
        timestamp: u64,
    },
    AbortTransaction {
        tx_id: u64,
        timestamp: u64,
    },
    LockWait {
        waiter: u64,
        holder: u64,
        key: String,
        mode: String,
    },
    LockAcquire {
        tx_id: u64,
        key: String,
        mode: String,
    },
    LockRelease {
        tx_id: u64,
        key: String,
    },
    WalWrite {
        bytes: u64,
        lsn: u64,
    },
    WalFlush {
        lsn: u64,
    },
    RecoveryStart {
        crash_timestamp: u64,
    },
    RecoveryComplete {
        transactions_replayed: u64,
        status: String,
    },
}

pub trait Observable {
    fn record(&self, event: ObservableEvent);
}

pub struct InMemoryStore<T> {
    buffer: VecDeque<T>,
    max_size: usize,
}

impl<T> InMemoryStore<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn append(&mut self, item: T) {
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_store_append() {
        let mut store = InMemoryStore::new(3);
        store.append(1);
        store.append(2);
        store.append(3);
        assert_eq!(store.len(), 3);
        let items: Vec<_> = store.iter().copied().collect();
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn test_in_memory_store_eviction() {
        let mut store = InMemoryStore::new(3);
        store.append(1);
        store.append(2);
        store.append(3);
        store.append(4);
        assert_eq!(store.len(), 3);
        let items: Vec<_> = store.iter().copied().collect();
        assert_eq!(items, vec![2, 3, 4]);
    }
}
