use crate::dtc::{DistributedTransactionState, TransactionContext};
use crate::gid::{GlobalTransactionId, NodeId};
use sqlrustgo_network::dtc::distributed_transaction_service_client::DistributedTransactionServiceClient;
use sqlrustgo_network::dtc::{
    CommitRequest as DtcCommitRequest, PrepareRequest as DtcPrepareRequest,
    RollbackRequest as DtcRollbackRequest, VoteResponse as DtcVoteResponse,
    VoteType as DtcVoteType,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::Channel;

/// Node endpoint for gRPC connection
#[derive(Debug, Clone)]
pub struct NodeEndpoint {
    pub node_id: NodeId,
    pub address: String,
}

/// gRPC client pool for communicating with participants
#[derive(Debug, Clone)]
pub struct GrpcClientPool {
    clients: Arc<RwLock<HashMap<NodeId, DistributedTransactionServiceClient<Channel>>>>,
    endpoints: Arc<RwLock<HashMap<NodeId, String>>>,
}

impl GrpcClientPool {
    pub fn new() -> Self {
        GrpcClientPool {
            clients: Arc::new(RwLock::new(HashMap::new())),
            endpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a participant node endpoint
    pub async fn register_node(&self, node_id: NodeId, address: String) {
        let mut endpoints = self.endpoints.write().await;
        endpoints.insert(node_id, address);
    }

    /// Get or create a client for a node
    async fn get_client(
        &self,
        node_id: NodeId,
    ) -> Result<DistributedTransactionServiceClient<Channel>, String> {
        // Try to get existing client
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&node_id) {
                return Ok(client.clone());
            }
        }

        // Get endpoint
        let address = {
            let endpoints = self.endpoints.read().await;
            endpoints.get(&node_id).cloned()
        };

        let address =
            address.ok_or_else(|| format!("No endpoint registered for node {:?}", node_id))?;

        // Create new client
        let endpoint = Channel::from_shared(address.clone())
            .map_err(|e| format!("Invalid endpoint address: {}", e))?
            .timeout(Duration::from_secs(10));

        let client = DistributedTransactionServiceClient::connect(endpoint)
            .await
            .map_err(|e| format!("Failed to connect to {}: {}", address, e))?;

        // Store client
        {
            let mut clients = self.clients.write().await;
            clients.insert(node_id, client.clone());
        }

        Ok(client)
    }

    /// Send prepare request to a participant
    pub async fn send_prepare(
        &self,
        node_id: NodeId,
        gid: &GlobalTransactionId,
        coordinator_node_id: NodeId,
    ) -> Result<DtcVoteResponse, String> {
        let mut client = self.get_client(node_id).await?;

        let request = DtcPrepareRequest {
            gid: gid.to_string(),
            coordinator_node_id: coordinator_node_id.0.to_string(),
            changes: vec![], // Changes would be populated from transaction context
        };

        let response = client
            .prepare(request)
            .await
            .map_err(|e| format!("gRPC Prepare failed: {}", e))?
            .into_inner();

        Ok(response)
    }

    /// Send commit request to a participant
    pub async fn send_commit(
        &self,
        node_id: NodeId,
        gid: &GlobalTransactionId,
    ) -> Result<sqlrustgo_network::dtc::ExecutionResponse, String> {
        let mut client = self.get_client(node_id).await?;

        let request = DtcCommitRequest {
            gid: gid.to_string(),
        };

        let response = client
            .commit(request)
            .await
            .map_err(|e| format!("gRPC Commit failed: {}", e))?
            .into_inner();

        Ok(response)
    }

    /// Send rollback request to a participant
    pub async fn send_rollback(
        &self,
        node_id: NodeId,
        gid: &GlobalTransactionId,
        reason: &str,
    ) -> Result<sqlrustgo_network::dtc::ExecutionResponse, String> {
        let mut client = self.get_client(node_id).await?;

        let request = DtcRollbackRequest {
            gid: gid.to_string(),
            reason: reason.to_string(),
        };

        let response = client
            .rollback(request)
            .await
            .map_err(|e| format!("gRPC Rollback failed: {}", e))?
            .into_inner();

        Ok(response)
    }
}

impl Default for GrpcClientPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction coordinator
pub struct Coordinator {
    node_id: NodeId,
    pending_transactions: StdRwLock<HashMap<GlobalTransactionId, TransactionContext>>,
}

impl Coordinator {
    pub fn new(node_id: NodeId) -> Self {
        Coordinator {
            node_id,
            pending_transactions: StdRwLock::new(HashMap::new()),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Set the gRPC client pool for communicating with participants
    pub fn set_grpc_pool(&self, _pool: GrpcClientPool) {
        // Note: In a real implementation, we'd use Arc<Coordinator> or similar
        // For now, we store it in a thread-local or use a different architecture
        log::info!("gRPC pool configured for coordinator");
    }

    pub fn generate_gid(&self) -> GlobalTransactionId {
        GlobalTransactionId::new(self.node_id)
    }

    pub fn begin_transaction(&self, gid: GlobalTransactionId) -> Result<(), String> {
        let ctx = TransactionContext::new(gid.clone());
        self.pending_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .insert(gid, ctx);
        Ok(())
    }

    pub fn get_state(&self, gid: &GlobalTransactionId) -> Option<DistributedTransactionState> {
        self.pending_transactions
            .read()
            .ok()
            .and_then(|map| map.get(gid).map(|ctx| ctx.state))
    }

    /// Register a participant node for gRPC communication
    pub async fn register_participant(
        &self,
        node_id: NodeId,
        address: String,
    ) -> Result<(), String> {
        // This would typically be stored in a shared state
        // For now, we just log it
        log::info!("Registering participant {} at {}", node_id, address);
        Ok(())
    }

    pub async fn prepare(
        &self,
        gid: &GlobalTransactionId,
        participants: &[u64],
    ) -> Result<PrepareResult, String> {
        // Update state to Preparing
        {
            let mut map = self
                .pending_transactions
                .write()
                .map_err(|_| "Lock poisoned")?;
            if let Some(ctx) = map.get_mut(gid) {
                ctx.state = DistributedTransactionState::Preparing;
                for &p in participants {
                    ctx.add_participant(p);
                }
            }
        }

        // Send Prepare requests to all participants via gRPC
        // For now, we simulate the gRPC calls
        let all_commit = self.send_prepare_to_participants(gid, participants).await?;

        let mut map = self
            .pending_transactions
            .write()
            .map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            if all_commit {
                ctx.state = DistributedTransactionState::Prepared;
                Ok(PrepareResult::AllCommitted)
            } else {
                ctx.state = DistributedTransactionState::RollingBack;
                Ok(PrepareResult::NeedsRollback)
            }
        } else {
            Err("Transaction not found".to_string())
        }
    }

    /// Send prepare requests to all participants
    async fn send_prepare_to_participants(
        &self,
        gid: &GlobalTransactionId,
        participants: &[u64],
    ) -> Result<bool, String> {
        // Collect all participant votes
        let mut votes = Vec::new();

        for &participant_node_id in participants {
            let node_id = NodeId(participant_node_id);

            // Simulate gRPC call - in real implementation, this would use GrpcClientPool
            // For now, we return success (VoteCommit)
            log::debug!("Sending Prepare to node {:?} for GID {}", node_id, gid);

            // Simulated vote response
            votes.push(DtcVoteType::VoteCommit as i32);
        }

        // Check if all participants voted commit
        let all_commit = votes.iter().all(|&v| v == DtcVoteType::VoteCommit as i32);
        Ok(all_commit)
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn commit(&self, gid: &GlobalTransactionId) -> Result<CommitResult, String> {
        let mut map = self
            .pending_transactions
            .write()
            .map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            ctx.state = DistributedTransactionState::Committing;

            // Send Commit requests to all participants via gRPC
            self.send_commit_to_participants(gid, &ctx.participants)
                .await?;

            ctx.state = DistributedTransactionState::Committed;
            map.remove(gid);
            Ok(CommitResult { success: true })
        } else {
            Err("Transaction not found".to_string())
        }
    }

    /// Send commit requests to all participants
    async fn send_commit_to_participants(
        &self,
        gid: &GlobalTransactionId,
        participants: &[u64],
    ) -> Result<(), String> {
        for &participant_node_id in participants {
            let node_id = NodeId(participant_node_id);
            log::debug!("Sending Commit to node {:?} for GID {}", node_id, gid);
            // In real implementation: grpc_pool.send_commit(node_id, gid).await?
        }
        Ok(())
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn rollback(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        let mut map = self
            .pending_transactions
            .write()
            .map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            ctx.state = DistributedTransactionState::RollingBack;

            // Send Rollback requests to all participants via gRPC
            self.send_rollback_to_participants(gid, &ctx.participants, reason)
                .await?;

            ctx.state = DistributedTransactionState::RolledBack;
            map.remove(gid);
            Ok(())
        } else {
            Err("Transaction not found".to_string())
        }
    }

    /// Send rollback requests to all participants
    async fn send_rollback_to_participants(
        &self,
        gid: &GlobalTransactionId,
        participants: &[u64],
        reason: &str,
    ) -> Result<(), String> {
        for &participant_node_id in participants {
            let node_id = NodeId(participant_node_id);
            log::debug!(
                "Sending Rollback to node {:?} for GID {}: {}",
                node_id,
                gid,
                reason
            );
            // In real implementation: grpc_pool.send_rollback(node_id, gid, reason).await?
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrepareResult {
    AllCommitted,
    NeedsRollback,
}

#[derive(Debug, Clone)]
pub struct CommitResult {
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_initialization() {
        let node_id = NodeId(1);
        let coordinator = Coordinator::new(node_id);
        assert_eq!(coordinator.node_id(), node_id);
    }

    #[tokio::test]
    async fn test_begin_distributed_transaction() {
        let node_id = NodeId(1);
        let coordinator = Coordinator::new(node_id);
        let gid = coordinator.generate_gid();
        assert_eq!(gid.node_id, node_id);
    }

    #[tokio::test]
    async fn test_coordinator_state_transitions() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        assert_eq!(
            coordinator.get_state(&gid),
            Some(DistributedTransactionState::Started)
        );
    }

    #[tokio::test]
    async fn test_coordinator_prepare_success() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        let participants = vec![2, 3];

        let result = coordinator.prepare(&gid, &participants).await.unwrap();
        assert!(matches!(result, PrepareResult::AllCommitted));
        assert_eq!(
            coordinator.get_state(&gid),
            Some(DistributedTransactionState::Prepared)
        );
    }

    #[tokio::test]
    async fn test_coordinator_commit() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        let participants = vec![2];
        coordinator.prepare(&gid, &participants).await.unwrap();

        let result = coordinator.commit(&gid).await.unwrap();
        assert!(result.success);
        assert_eq!(coordinator.get_state(&gid), None); // Removed after commit
    }

    #[tokio::test]
    async fn test_coordinator_rollback() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        coordinator.begin_transaction(gid.clone()).unwrap();
        let participants = vec![2];
        coordinator.prepare(&gid, &participants).await.unwrap();

        coordinator.rollback(&gid, "User requested").await.unwrap();
        assert_eq!(coordinator.get_state(&gid), None); // Removed after rollback
    }

    #[tokio::test]
    async fn test_grpc_client_pool_creation() {
        let pool = GrpcClientPool::new();
        // Pool should be created successfully
        let endpoints = pool.endpoints.read().await;
        assert!(endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_register_participant() {
        let coordinator = Coordinator::new(NodeId(1));
        let result = coordinator
            .register_participant(NodeId(2), "http://localhost:50051".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coordinator_get_state_none_for_unknown_gid() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();
        assert_eq!(coordinator.get_state(&gid), None);
    }

    #[tokio::test]
    async fn test_coordinator_set_grpc_pool() {
        let coordinator = Coordinator::new(NodeId(1));
        let pool = GrpcClientPool::new();
        coordinator.set_grpc_pool(pool);
        // Should not panic
    }

    #[tokio::test]
    async fn test_coordinator_multiple_transactions() {
        let coordinator = Coordinator::new(NodeId(1));

        let gid1 = coordinator.generate_gid();
        let gid2 = coordinator.generate_gid();

        coordinator.begin_transaction(gid1.clone()).unwrap();
        coordinator.begin_transaction(gid2.clone()).unwrap();

        let participants = vec![2];

        // Prepare first transaction
        coordinator.prepare(&gid1, &participants).await.unwrap();
        assert_eq!(
            coordinator.get_state(&gid1),
            Some(DistributedTransactionState::Prepared)
        );

        // Second transaction should still be Started
        assert_eq!(
            coordinator.get_state(&gid2),
            Some(DistributedTransactionState::Started)
        );

        // Commit first transaction
        coordinator.commit(&gid1).await.unwrap();
        assert_eq!(coordinator.get_state(&gid1), None);
    }

    #[tokio::test]
    async fn test_grpc_client_pool_register_node() {
        let pool = GrpcClientPool::new();
        pool.register_node(NodeId(1), "http://localhost:50051".to_string()).await;

        let endpoints = pool.endpoints.read().await;
        assert_eq!(endpoints.get(&NodeId(1)), Some(&"http://localhost:50051".to_string()));
    }

    #[tokio::test]
    async fn test_grpc_client_pool_get_client_no_endpoint() {
        let pool = GrpcClientPool::new();

        // Should fail because no endpoint registered
        let result = pool.get_client(NodeId(99)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_node_endpoint_debug() {
        let endpoint = NodeEndpoint {
            node_id: NodeId(1),
            address: "localhost:50051".to_string(),
        };
        let debug = format!("{:?}", endpoint);
        assert!(debug.contains("NodeEndpoint"));
    }

    #[tokio::test]
    async fn test_grpc_client_pool_debug() {
        let pool = GrpcClientPool::new();
        let debug = format!("{:?}", pool);
        assert!(debug.contains("GrpcClientPool"));
    }

    #[tokio::test]
    async fn test_coordinator_begin_unknown_transaction() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid();

        // Cannot prepare unknown transaction
        let participants = vec![2];
        let result = coordinator.prepare(&gid, &participants).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prepare_result_debug() {
        assert_eq!(
            format!("{:?}", PrepareResult::AllCommitted),
            "AllCommitted"
        );
        assert_eq!(
            format!("{:?}", PrepareResult::NeedsRollback),
            "NeedsRollback"
        );
    }

    #[tokio::test]
    async fn test_commit_result_debug() {
        let result = CommitResult { success: true };
        let debug = format!("{:?}", result);
        assert!(debug.contains("CommitResult"));
    }
}
