//! Transaction Module
//!
//! # What (是什么)
//! 事务管理模块，提供 ACID 特性保证
//!
//! # Why (为什么)
//! 数据库的核心价值之一是提供事务支持，确保数据一致性
//! 即使发生系统崩溃，也不会导致数据损坏
//!
//! # How (如何实现)
//! - WAL (Write-Ahead Log)：先写日志再写数据
//! - 事务状态机：Active -> Committed / Aborted
//! - Commit：持久化日志
//! - Rollback：逆向操作

pub mod manager;
pub mod wal;

pub use manager::{Transaction, TransactionManager, TxState};
pub use wal::{WalRecord, WriteAheadLog};
