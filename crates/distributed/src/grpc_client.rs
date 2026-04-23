//! gRPC client for distributed shard communication
//!
//! Provides client for cross-node vector and graph operations.

#![allow(dead_code)]

use crate::error::DistributedError;
use crate::proto::distributed::{
    shard_service_client::ShardServiceClient, CreateEdgeRequest, CreateEdgeResponse,
    CreateNodeRequest, CreateNodeResponse, DeleteNodeRequest, DeleteNodeResponse,
    DeleteVectorRequest, DeleteVectorResponse, GetEdgesRequest, GetEdgesResponse, GetNodeRequest,
    GetNodeResponse, InsertVectorRequest, InsertVectorResponse, PropertyMap, SearchResult,
    SearchVectorsRequest, SearchVectorsResponse, VectorRecord,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::{Channel, Endpoint};

pub type NodeId = u64;
pub type ShardId = u64;

#[derive(Clone)]
pub struct ShardClient {
    node_id: NodeId,
    channel: Channel,
    client: Arc<RwLock<ShardServiceClient<Channel>>>,
}

impl ShardClient {
    pub async fn connect(addr: &str, node_id: NodeId) -> Result<Self, DistributedError> {
        let channel = Endpoint::from_shared(addr.to_string())
            .map_err(|e| DistributedError::Connection(e.to_string()))?
            .connect()
            .await
            .map_err(|e| DistributedError::Connection(e.to_string()))?;

        let client = ShardServiceClient::new(channel.clone());

        Ok(Self {
            node_id,
            channel,
            client: Arc::new(RwLock::new(client)),
        })
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub async fn insert_vector(
        &self,
        shard_id: ShardId,
        id: u64,
        vector: &[f32],
    ) -> Result<(), DistributedError> {
        let mut client = self.client.write().await;

        let request = InsertVectorRequest {
            shard_id,
            record: Some(VectorRecord {
                id,
                vector: vector.to_vec(),
                metadata: HashMap::new(),
            }),
        };

        let response = client
            .insert_vector(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let InsertVectorResponse { success, error } = response.into_inner();
        if success {
            Ok(())
        } else {
            Err(DistributedError::Operation(error))
        }
    }

    pub async fn search_vectors(
        &self,
        shard_id: ShardId,
        query: &[f32],
        top_k: u32,
    ) -> Result<Vec<SearchResult>, DistributedError> {
        let mut client = self.client.write().await;

        let request = SearchVectorsRequest {
            shard_id,
            query: query.to_vec(),
            top_k,
        };

        let response = client
            .search_vectors(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let SearchVectorsResponse { results, error } = response.into_inner();
        if error.is_empty() {
            Ok(results)
        } else {
            Err(DistributedError::Operation(error))
        }
    }

    pub async fn delete_vector(
        &self,
        shard_id: ShardId,
        vector_id: u64,
    ) -> Result<(), DistributedError> {
        let mut client = self.client.write().await;

        let request = DeleteVectorRequest {
            shard_id,
            vector_id,
        };

        let response = client
            .delete_vector(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let DeleteVectorResponse { success, error } = response.into_inner();
        if success {
            Ok(())
        } else {
            Err(DistributedError::Operation(error))
        }
    }

    pub async fn create_node(
        &self,
        shard_id: ShardId,
        label: &str,
        properties: HashMap<String, String>,
    ) -> Result<u64, DistributedError> {
        let mut client = self.client.write().await;

        let request = CreateNodeRequest {
            shard_id,
            label: label.to_string(),
            properties: Some(PropertyMap { properties }),
        };

        let response = client
            .create_node(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let CreateNodeResponse {
            node_id,
            success,
            error,
        } = response.into_inner();
        if success {
            Ok(node_id)
        } else {
            Err(DistributedError::Operation(error))
        }
    }

    pub async fn get_node(
        &self,
        node_id: u64,
    ) -> Result<Option<(String, HashMap<String, String>)>, DistributedError> {
        let mut client = self.client.write().await;

        let request = GetNodeRequest { node_id };

        let response = client
            .get_node(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let GetNodeResponse {
            node_id: _,
            label,
            properties,
            found,
        } = response.into_inner();

        if found {
            let props = properties.map(|p| p.properties).unwrap_or_default();
            Ok(Some((label, props)))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_node(&self, node_id: u64) -> Result<(), DistributedError> {
        let mut client = self.client.write().await;

        let request = DeleteNodeRequest { node_id };

        let response = client
            .delete_node(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let DeleteNodeResponse { success, error } = response.into_inner();
        if success {
            Ok(())
        } else {
            Err(DistributedError::Operation(error))
        }
    }

    pub async fn create_edge(
        &self,
        from_node: u64,
        to_node: u64,
        label: &str,
        properties: HashMap<String, String>,
    ) -> Result<u64, DistributedError> {
        let mut client = self.client.write().await;

        let request = CreateEdgeRequest {
            from_node,
            to_node,
            label: label.to_string(),
            properties: Some(PropertyMap { properties }),
        };

        let response = client
            .create_edge(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let CreateEdgeResponse {
            edge_id,
            success,
            error,
        } = response.into_inner();
        if success {
            Ok(edge_id)
        } else {
            Err(DistributedError::Operation(error))
        }
    }

    pub async fn get_edges(
        &self,
        node_id: u64,
    ) -> Result<Vec<(u64, u64, u64, String)>, DistributedError> {
        let mut client = self.client.write().await;

        let request = GetEdgesRequest { node_id };

        let response = client
            .get_edges(request)
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        let GetEdgesResponse { edges, error } = response.into_inner();
        if error.is_empty() {
            Ok(edges
                .into_iter()
                .map(|e| (e.edge_id, e.from, e.to, e.label))
                .collect())
        } else {
            Err(DistributedError::Operation(error))
        }
    }

    pub async fn health_check(&self) -> Result<bool, DistributedError> {
        let mut client = self.client.write().await;

        let response = client
            .health_check(crate::proto::distributed::Empty {})
            .await
            .map_err(|e| DistributedError::Rpc(e.to_string()))?;

        Ok(response.into_inner().healthy)
    }
}

#[derive(Clone)]
pub struct ClientPool {
    clients: Arc<RwLock<HashMap<NodeId, ShardClient>>>,
}

impl ClientPool {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_client(&self, node_id: NodeId, addr: &str) -> Result<(), DistributedError> {
        let client = ShardClient::connect(addr, node_id).await?;
        let mut clients = self.clients.write().await;
        clients.insert(node_id, client);
        Ok(())
    }

    pub async fn get_client(&self, node_id: NodeId) -> Option<ShardClient> {
        let clients = self.clients.read().await;
        clients.get(&node_id).cloned()
    }

    pub async fn remove_client(&self, node_id: NodeId) -> Option<ShardClient> {
        let mut clients = self.clients.write().await;
        clients.remove(&node_id)
    }

    pub async fn health_check_all(&self) -> HashMap<NodeId, bool> {
        let clients = self.clients.read().await;
        let mut results = HashMap::new();

        for (&node_id, client) in clients.iter() {
            let healthy = client.health_check().await.unwrap_or(false);
            results.insert(node_id, healthy);
        }

        results
    }
}

impl Default for ClientPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_pool_new() {
        let pool = ClientPool::new();
        assert!(pool.clients.try_read().is_ok());
    }

    #[test]
    fn test_client_pool_default() {
        let pool = ClientPool::default();
        assert!(pool.clients.try_read().is_ok());
    }

    #[tokio::test]
    async fn test_client_pool_get_client_empty() {
        let pool = ClientPool::new();
        let client = pool.get_client(1).await;
        assert!(client.is_none());
    }

    #[tokio::test]
    async fn test_client_pool_remove_client_empty() {
        let pool = ClientPool::new();
        let removed = pool.remove_client(1).await;
        assert!(removed.is_none());
    }

    #[tokio::test]
    async fn test_client_pool_health_check_empty() {
        let pool = ClientPool::new();
        let health = pool.health_check_all().await;
        assert!(health.is_empty());
    }

    #[tokio::test]
    async fn test_client_pool_get_client_after_remove() {
        let pool = ClientPool::new();
        let removed = pool.remove_client(1).await;
        assert!(removed.is_none());
        let client = pool.get_client(1).await;
        assert!(client.is_none());
    }

    #[tokio::test]
    async fn test_client_pool_multiple_clients() {
        let pool = ClientPool::new();
        let result1 = pool.remove_client(1).await;
        let result2 = pool.remove_client(2).await;
        let result3 = pool.remove_client(3).await;
        assert!(result1.is_none());
        assert!(result2.is_none());
        assert!(result3.is_none());
    }

    #[test]
    fn test_vector_record_creation() {
        let record = VectorRecord {
            id: 123,
            vector: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        assert_eq!(record.id, 123);
        assert_eq!(record.vector.len(), 3);
        assert!(record.metadata.is_empty());
    }

    #[test]
    fn test_property_map_creation() {
        let mut props = HashMap::new();
        props.insert("name".to_string(), "test".to_string());
        let property_map = PropertyMap { properties: props.clone() };
        assert_eq!(property_map.properties.get("name"), Some(&"test".to_string()));
    }

    #[test]
    fn test_insert_vector_request() {
        let request = InsertVectorRequest {
            shard_id: 1,
            record: Some(VectorRecord {
                id: 42,
                vector: vec![0.1, 0.2],
                metadata: HashMap::new(),
            }),
        };
        assert_eq!(request.shard_id, 1);
        assert!(request.record.is_some());
        assert_eq!(request.record.as_ref().unwrap().id, 42);
    }

    #[test]
    fn test_search_vectors_request() {
        let request = SearchVectorsRequest {
            shard_id: 5,
            query: vec![1.0, 2.0, 3.0],
            top_k: 10,
        };
        assert_eq!(request.shard_id, 5);
        assert_eq!(request.query.len(), 3);
        assert_eq!(request.top_k, 10);
    }

    #[test]
    fn test_delete_vector_request() {
        let request = DeleteVectorRequest {
            shard_id: 3,
            vector_id: 100,
        };
        assert_eq!(request.shard_id, 3);
        assert_eq!(request.vector_id, 100);
    }

    #[test]
    fn test_create_node_request() {
        let request = CreateNodeRequest {
            shard_id: 2,
            label: "User".to_string(),
            properties: Some(PropertyMap {
                properties: HashMap::new(),
            }),
        };
        assert_eq!(request.shard_id, 2);
        assert_eq!(request.label, "User");
        assert!(request.properties.is_some());
    }

    #[test]
    fn test_get_node_request() {
        let request = GetNodeRequest { node_id: 55 };
        assert_eq!(request.node_id, 55);
    }

    #[test]
    fn test_delete_node_request() {
        let request = DeleteNodeRequest { node_id: 77 };
        assert_eq!(request.node_id, 77);
    }

    #[test]
    fn test_create_edge_request() {
        let request = CreateEdgeRequest {
            from_node: 10,
            to_node: 20,
            label: "KNOWS".to_string(),
            properties: Some(PropertyMap {
                properties: HashMap::new(),
            }),
        };
        assert_eq!(request.from_node, 10);
        assert_eq!(request.to_node, 20);
        assert_eq!(request.label, "KNOWS");
    }

    #[test]
    fn test_get_edges_request() {
        let request = GetEdgesRequest { node_id: 33 };
        assert_eq!(request.node_id, 33);
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            id: 1,
            score: 0.95,
            record: None,
        };
        assert_eq!(result.id, 1);
        assert_eq!(result.score, 0.95);
    }

    #[test]
    fn test_search_vectors_response_empty() {
        let response = SearchVectorsResponse {
            results: vec![],
            error: String::new(),
        };
        assert!(response.results.is_empty());
        assert!(response.error.is_empty());
    }

    #[test]
    fn test_search_vectors_response_with_error() {
        let response = SearchVectorsResponse {
            results: vec![],
            error: "Server error".to_string(),
        };
        assert!(response.results.is_empty());
        assert_eq!(response.error, "Server error");
    }

    #[test]
    fn test_insert_vector_response_success() {
        let response = InsertVectorResponse {
            success: true,
            error: String::new(),
        };
        assert!(response.success);
        assert!(response.error.is_empty());
    }

    #[test]
    fn test_insert_vector_response_failure() {
        let response = InsertVectorResponse {
            success: false,
            error: "Shard not found".to_string(),
        };
        assert!(!response.success);
        assert_eq!(response.error, "Shard not found");
    }

    #[test]
    fn test_delete_vector_response_success() {
        let response = DeleteVectorResponse {
            success: true,
            error: String::new(),
        };
        assert!(response.success);
    }

    #[test]
    fn test_create_node_response_success() {
        let response = CreateNodeResponse {
            node_id: 123,
            success: true,
            error: String::new(),
        };
        assert!(response.success);
        assert_eq!(response.node_id, 123);
    }

    #[test]
    fn test_create_node_response_failure() {
        let response = CreateNodeResponse {
            node_id: 0,
            success: false,
            error: "Invalid label".to_string(),
        };
        assert!(!response.success);
        assert_eq!(response.error, "Invalid label");
    }

    #[test]
    fn test_get_node_response_found() {
        let response = GetNodeResponse {
            node_id: 42,
            label: "Person".to_string(),
            properties: Some(PropertyMap {
                properties: HashMap::new(),
            }),
            found: true,
        };
        assert!(response.found);
        assert_eq!(response.label, "Person");
    }

    #[test]
    fn test_get_node_response_not_found() {
        let response = GetNodeResponse {
            node_id: 0,
            label: String::new(),
            properties: None,
            found: false,
        };
        assert!(!response.found);
    }

    #[test]
    fn test_delete_node_response_success() {
        let response = DeleteNodeResponse {
            success: true,
            error: String::new(),
        };
        assert!(response.success);
    }

    #[test]
    fn test_create_edge_response_success() {
        let response = CreateEdgeResponse {
            edge_id: 999,
            success: true,
            error: String::new(),
        };
        assert!(response.success);
        assert_eq!(response.edge_id, 999);
    }

    #[test]
    fn test_create_edge_response_failure() {
        let response = CreateEdgeResponse {
            edge_id: 0,
            success: false,
            error: "Node not found".to_string(),
        };
        assert!(!response.success);
    }

    #[test]
    fn test_get_edges_response_empty() {
        let response = GetEdgesResponse {
            edges: vec![],
            error: String::new(),
        };
        assert!(response.edges.is_empty());
        assert!(response.error.is_empty());
    }
}
