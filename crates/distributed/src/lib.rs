//! SQLRustGo Distributed Module
//!
//! Provides distributed database capabilities:
//! - Sharding with hash and range partitioning
//! - Raft-based leader election and log replication
//! - Two-Phase Commit for distributed transactions
//! - Distributed locking

pub mod partition;
pub mod shard_manager;
pub mod shard_router;
pub mod raft;
pub mod two_phase_commit;
pub mod distributed_lock;

pub use partition::{PartitionKey, PartitionStrategy};
pub use shard_manager::{ShardInfo, ShardManager, ShardStatus};
pub use shard_router::{RoutedPlan, ShardRouter};
pub use raft::{RaftMessage, RaftNode, RaftState};
pub use two_phase_commit::{DistributedTransaction, Participant, TwoPhaseCommit, TransactionState, Vote};
pub use distributed_lock::{DistributedLockManager, LockEntry};
