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
