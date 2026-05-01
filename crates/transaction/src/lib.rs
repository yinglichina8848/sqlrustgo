//! SQLRustGo Transaction Module
//!
//! Provides MVCC-based transaction management with Serializable Snapshot Isolation (SSI).
//!
//! # Core Components
//!
//! - `TransactionManager`: Manages transaction lifecycle with SSI conflict detection
//! - `mvcc`: Multi-Version Concurrency Control implementation
//! - `ssi`: Serializable Snapshot Isolation conflict detection
//! - `lock_manager`: Distributed lock management
//!
//! # Transaction Isolation
//!
//! Supports Snapshot Isolation and Serializable isolation levels.

pub mod deadlock;
pub mod dtc;
pub mod gid;
pub mod lock;
pub mod lock_manager;
pub mod mvcc;
pub mod savepoint;
pub mod ssi;
pub mod transaction_manager;
pub mod version_chain;

pub use gid::NodeId;
pub use lock_manager::DistributedLockManager;
pub use mvcc::{Snapshot, Transaction, TxId, INVALID_TX_ID};
pub use ssi::{SerializationGraph, SireadLock, SsiDetector, SsiDetectorSync, SsiError};
pub use transaction_manager::{
    ActiveTransaction, IsolationLevel, TransactionManager, TransactionState,
};
