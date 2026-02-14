//! Transaction Management Module
//! 
//! Provides ACID transaction support with Write-Ahead Logging (WAL).

pub mod wal;
pub mod manager;

pub use wal::{WriteAheadLog, WalRecord};
pub use manager::{TransactionManager, Transaction, TxState};
