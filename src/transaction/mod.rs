//! Transaction Management Module
//!
//! Provides ACID transaction support with Write-Ahead Logging (WAL).

pub mod manager;
pub mod wal;

pub use manager::{Transaction, TransactionManager, TxState};
pub use wal::{WalRecord, WriteAheadLog};
