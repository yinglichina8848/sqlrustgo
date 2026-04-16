use thiserror::Error;

#[derive(Error, Debug)]
pub enum DistributedError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Operation failed: {0}")]
    Operation(String),

    #[error("Shard not found: {0}")]
    ShardNotFound(u64),

    #[error("Node not found: {0}")]
    NodeNotFound(u64),

    #[error("Invalid partition key: {0}")]
    InvalidPartitionKey(String),

    #[error("No partition rule for table: {0}")]
    NoPartitionRule(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Consensus error: {0}")]
    Consensus(String),
}
