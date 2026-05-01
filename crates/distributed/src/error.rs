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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distributed_error_connection() {
        let err = DistributedError::Connection("timeout".to_string());
        assert!(err.to_string().contains("Connection error"));
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_distributed_error_rpc() {
        let err = DistributedError::Rpc("service unavailable".to_string());
        assert!(err.to_string().contains("RPC error"));
    }

    #[test]
    fn test_distributed_error_operation() {
        let err = DistributedError::Operation("failed".to_string());
        assert!(err.to_string().contains("Operation failed"));
    }

    #[test]
    fn test_distributed_error_shard_not_found() {
        let err = DistributedError::ShardNotFound(42);
        assert!(err.to_string().contains("Shard not found"));
        assert!(err.to_string().contains("42"));
    }

    #[test]
    fn test_distributed_error_node_not_found() {
        let err = DistributedError::NodeNotFound(10);
        assert!(err.to_string().contains("Node not found"));
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_distributed_error_invalid_partition_key() {
        let err = DistributedError::InvalidPartitionKey("missing column".to_string());
        assert!(err.to_string().contains("Invalid partition key"));
    }

    #[test]
    fn test_distributed_error_no_partition_rule() {
        let err = DistributedError::NoPartitionRule("users".to_string());
        assert!(err.to_string().contains("No partition rule for table"));
    }

    #[test]
    fn test_distributed_error_transaction() {
        let err = DistributedError::Transaction("commit failed".to_string());
        assert!(err.to_string().contains("Transaction error"));
    }

    #[test]
    fn test_distributed_error_lock() {
        let err = DistributedError::Lock("deadlock".to_string());
        assert!(err.to_string().contains("Lock error"));
    }

    #[test]
    fn test_distributed_error_consensus() {
        let err = DistributedError::Consensus("raft leader election failed".to_string());
        assert!(err.to_string().contains("Consensus error"));
    }

    #[test]
    fn test_distributed_error_debug() {
        let err = DistributedError::ShardNotFound(5);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("ShardNotFound"));
    }
}
