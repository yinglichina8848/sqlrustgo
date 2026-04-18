//! SQLRustGo Distributed Module
//!
//! Provides distributed database capabilities:
//! - Sharding with hash and range partitioning
//! - Raft-based leader election and log replication
//! - Two-Phase Commit for distributed transactions
//! - Distributed locking
//! - gRPC-based cross-node communication
//! - Consensus-based replica management
//! - Failover and health checking
//! - Replica synchronization

pub mod consensus;
pub mod cross_shard_query;
pub mod distributed_lock;
pub mod error;
pub mod failover_manager;
pub mod grpc_client;
pub mod grpc_server;
pub mod partition;
pub mod proto;
pub mod raft;
pub mod replica_sync;
pub mod replication;
pub mod shard_manager;
pub mod shard_router;
pub mod two_phase_commit;

pub use consensus::{Operation, ShardReplicaManager};
pub use cross_shard_query::{CrossShardQueryExecutor, QueryRouter};
pub use distributed_lock::{DistributedLockManager, LockEntry};
pub use error::DistributedError;
pub use failover_manager::{ClusterHealth, FailoverConfig, FailoverManager};
pub use grpc_client::{ClientPool, ShardClient};
pub use grpc_server::{start_server, GraphStorage, ShardServer, ShardServerConfig, VectorStorage};
pub use partition::{PartitionKey, PartitionStrategy};
pub use raft::{RaftMessage, RaftNode, RaftState};
pub use replica_sync::{ReplicaSynchronizer, SyncConfig, SyncProgress, SyncResult, LSN};
pub use replication::{
    BinlogEvent, BinlogManager, BinlogStatus, MasterStatus, ReplicationConfig, ReplicationRole,
    ReplicationState, SlaveStatus,
};
pub use shard_manager::{NodeId, ShardId, ShardInfo, ShardManager, ShardStatus};
pub use shard_router::{RoutedPlan, RoutedQuery, RouterError, ShardRouter};
pub use two_phase_commit::{
    DistributedTransaction, Participant, TransactionState, TwoPhaseCommit, Vote,
};
