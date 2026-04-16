//! SQLRustGo Distributed Module
//!
//! Provides distributed database capabilities:
//! - Sharding with hash and range partitioning
//! - Raft-based leader election and log replication
//! - Two-Phase Commit for distributed transactions
//! - Distributed locking
//! - gRPC-based cross-node communication

pub mod cross_shard_query;
pub mod distributed_lock;
pub mod error;
pub mod grpc_client;
pub mod grpc_server;
pub mod partition;
pub mod proto;
pub mod raft;
pub mod shard_manager;
pub mod shard_router;
pub mod two_phase_commit;

pub use cross_shard_query::{CrossShardQueryExecutor, QueryRouter};
pub use distributed_lock::{DistributedLockManager, LockEntry};
pub use error::DistributedError;
pub use grpc_client::{ClientPool, ShardClient};
pub use grpc_server::{GraphStorage, ShardServer, ShardServerConfig, VectorStorage, start_server};
pub use partition::{PartitionKey, PartitionStrategy};
pub use raft::{RaftMessage, RaftNode, RaftState};
pub use shard_manager::{NodeId, ShardId, ShardInfo, ShardManager, ShardStatus};
pub use shard_router::{RoutedPlan, RoutedQuery, RouterError, ShardRouter};
pub use two_phase_commit::{
    DistributedTransaction, Participant, TransactionState, TwoPhaseCommit, Vote,
};
