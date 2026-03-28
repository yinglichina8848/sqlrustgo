use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

/// 节点 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        NodeId(id)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

/// 全局唯一事务 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalTransactionId {
    pub node_id: NodeId,
    pub txn_id: u64,
    pub timestamp: u64,
}

static TXN_COUNTER: AtomicU64 = AtomicU64::new(0);

impl GlobalTransactionId {

    pub fn new(node_id: NodeId) -> Self {
        let txn_id = TXN_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        GlobalTransactionId {
            node_id,
            txn_id,
            timestamp,
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        // 格式: "node_id:txn_id:timestamp"
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err("Invalid GID format".to_string());
        }

        let node_id = NodeId(parts[0].parse().map_err(|_| "Invalid node_id")?);
        let txn_id = parts[1].parse().map_err(|_| "Invalid txn_id")?;
        let timestamp = parts[2].parse().map_err(|_| "Invalid timestamp")?;

        Ok(GlobalTransactionId {
            node_id,
            txn_id,
            timestamp,
        })
    }
}

impl fmt::Display for GlobalTransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.node_id.0, self.txn_id, self.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gid_generation() {
        let node_id = NodeId(1);
        let gid1 = GlobalTransactionId::new(node_id);
        let gid2 = GlobalTransactionId::new(node_id);

        // GID 应该单调递增
        assert!(gid1.txn_id < gid2.txn_id);
    }

    #[test]
    fn test_gid_equality() {
        let node_id = NodeId(1);
        let gid1 = GlobalTransactionId::new(node_id);
        let gid2 = gid1.clone();

        assert_eq!(gid1, gid2);
    }

    #[test]
    fn test_gid_display() {
        let node_id = NodeId(1);
        let gid = GlobalTransactionId::new(node_id);
        let display = format!("{}", gid);

        assert!(display.contains("1")); // 包含 node_id
        assert!(display.contains(&gid.txn_id.to_string())); // 包含 txn_id
    }
}
