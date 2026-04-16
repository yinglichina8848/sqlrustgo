//! SQLRustGo Distributed Module
//!
//! Provides distributed database capabilities:
//! - Sharding with hash and range partitioning
//! - Raft-based leader election and log replication
//! - Two-Phase Commit for distributed transactions
//! - Distributed locking

pub mod distributed_lock;
pub mod partition;
pub mod raft;
pub mod shard_manager;
pub mod shard_router;
pub mod two_phase_commit;

pub use distributed_lock::{DistributedLockManager, LockEntry};
pub use partition::{PartitionKey, PartitionStrategy};
pub use raft::{RaftMessage, RaftNode, RaftState};
pub use shard_manager::{ShardInfo, ShardManager, ShardStatus};
pub use shard_router::{RoutedPlan, ShardRouter};
pub use two_phase_commit::{
    DistributedTransaction, Participant, TransactionState, TwoPhaseCommit, Vote,
};
