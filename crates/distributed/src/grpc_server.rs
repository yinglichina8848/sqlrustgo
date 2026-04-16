use crate::error::DistributedError;
use crate::proto::distributed::{
    shard_service_server::{ShardService, ShardServiceServer},
    CreateEdgeRequest, CreateEdgeResponse, CreateNodeRequest, CreateNodeResponse,
    DeleteNodeRequest, DeleteNodeResponse, DeleteVectorRequest, DeleteVectorResponse,
    EdgeInfo, GetEdgesRequest, GetEdgesResponse, GetNodeRequest, GetNodeResponse,
    HealthCheckResponse, InsertVectorRequest, InsertVectorResponse, PropertyMap, SearchResult,
    SearchVectorsRequest, SearchVectorsResponse, VectorRecord,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tonic::{Request, Response, Status};

pub type NodeId = u64;
pub type ShardId = u64;

pub trait VectorStorage: Send + Sync {
    fn insert(&mut self, shard_id: ShardId, id: u64, vector: Vec<f32>) -> Result<(), DistributedError>;
    fn search(&self, shard_id: ShardId, query: &[f32], top_k: usize) -> Result<Vec<SearchResult>, DistributedError>;
    fn delete(&mut self, shard_id: ShardId, id: u64) -> Result<(), DistributedError>;
}

pub trait GraphStorage: Send + Sync {
    fn create_node(&mut self, shard_id: ShardId, label: &str, props: HashMap<String, String>) -> Result<u64, DistributedError>;
    fn get_node(&self, node_id: u64) -> Option<(String, HashMap<String, String>)>;
    fn delete_node(&mut self, node_id: u64) -> Result<(), DistributedError>;
    fn create_edge(&mut self, from: u64, to: u64, label: &str, props: HashMap<String, String>) -> Result<u64, DistributedError>;
    fn get_edges(&self, node_id: u64) -> Vec<(u64, u64, u64, String)>;
}

pub struct ShardServerConfig {
    pub node_id: NodeId,
    pub listen_addr: String,
}

pub struct ShardServer<V: VectorStorage, G: GraphStorage> {
    node_id: NodeId,
    start_time: Instant,
    vector_storage: Arc<RwLock<V>>,
    graph_storage: Arc<RwLock<G>>,
}

impl<V: VectorStorage, G: GraphStorage> ShardServer<V, G> {
    pub fn new(vector_storage: V, graph_storage: G, node_id: NodeId) -> Self {
        Self {
            node_id,
            start_time: Instant::now(),
            vector_storage: Arc::new(RwLock::new(vector_storage)),
            graph_storage: Arc::new(RwLock::new(graph_storage)),
        }
    }

    pub fn into_service(self) -> ShardServiceServer<Self> {
        ShardServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl<V: VectorStorage + 'static, G: GraphStorage + 'static> ShardService for ShardServer<V, G> {
    async fn insert_vector(
        &self,
        request: Request<InsertVectorRequest>,
    ) -> Result<Response<InsertVectorResponse>, Status> {
        let req = request.into_inner();

        let mut storage = self.vector_storage.write().await;
        let result = storage.insert(
            req.shard_id,
            req.record.as_ref().map(|r| r.id).unwrap_or(0),
            req.record.map(|r| r.vector).unwrap_or_default(),
        );

        match result {
            Ok(()) => Ok(Response::new(InsertVectorResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(InsertVectorResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn search_vectors(
        &self,
        request: Request<SearchVectorsRequest>,
    ) -> Result<Response<SearchVectorsResponse>, Status> {
        let req = request.into_inner();

        let storage = self.vector_storage.read().await;
        let result = storage.search(req.shard_id, &req.query, req.top_k as usize);

        match result {
            Ok(results) => Ok(Response::new(SearchVectorsResponse {
                results,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(SearchVectorsResponse {
                results: vec![],
                error: e.to_string(),
            })),
        }
    }

    async fn delete_vector(
        &self,
        request: Request<DeleteVectorRequest>,
    ) -> Result<Response<DeleteVectorResponse>, Status> {
        let req = request.into_inner();

        let mut storage = self.vector_storage.write().await;
        let result = storage.delete(req.shard_id, req.vector_id);

        match result {
            Ok(()) => Ok(Response::new(DeleteVectorResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(DeleteVectorResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn create_node(
        &self,
        request: Request<CreateNodeRequest>,
    ) -> Result<Response<CreateNodeResponse>, Status> {
        let req = request.into_inner();

        let props = req.properties.map(|p| p.properties).unwrap_or_default();

        let mut storage = self.graph_storage.write().await;
        let result = storage.create_node(req.shard_id, &req.label, props);

        match result {
            Ok(node_id) => Ok(Response::new(CreateNodeResponse {
                node_id,
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(CreateNodeResponse {
                node_id: 0,
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn get_node(
        &self,
        request: Request<GetNodeRequest>,
    ) -> Result<Response<GetNodeResponse>, Status> {
        let req = request.into_inner();

        let storage = self.graph_storage.read().await;
        let result = storage.get_node(req.node_id);

        if let Some((label, props)) = result {
            Ok(Response::new(GetNodeResponse {
                node_id: req.node_id,
                label,
                properties: Some(PropertyMap { properties: props }),
                found: true,
            }))
        } else {
            Ok(Response::new(GetNodeResponse {
                node_id: req.node_id,
                label: String::new(),
                properties: None,
                found: false,
            }))
        }
    }

    async fn delete_node(
        &self,
        request: Request<DeleteNodeRequest>,
    ) -> Result<Response<DeleteNodeResponse>, Status> {
        let req = request.into_inner();

        let mut storage = self.graph_storage.write().await;
        let result = storage.delete_node(req.node_id);

        match result {
            Ok(()) => Ok(Response::new(DeleteNodeResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(DeleteNodeResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn create_edge(
        &self,
        request: Request<CreateEdgeRequest>,
    ) -> Result<Response<CreateEdgeResponse>, Status> {
        let req = request.into_inner();

        let props = req.properties.map(|p| p.properties).unwrap_or_default();

        let mut storage = self.graph_storage.write().await;
        let result = storage.create_edge(req.from_node, req.to_node, &req.label, props);

        match result {
            Ok(edge_id) => Ok(Response::new(CreateEdgeResponse {
                edge_id,
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(CreateEdgeResponse {
                edge_id: 0,
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn get_edges(
        &self,
        request: Request<GetEdgesRequest>,
    ) -> Result<Response<GetEdgesResponse>, Status> {
        let req = request.into_inner();

        let storage = self.graph_storage.read().await;
        let edges = storage.get_edges(req.node_id);

        let edge_infos = edges
            .into_iter()
            .map(|(edge_id, from, to, label)| EdgeInfo {
                edge_id,
                from,
                to,
                label,
            })
            .collect();

        Ok(Response::new(GetEdgesResponse {
            edges: edge_infos,
            error: String::new(),
        }))
    }

    async fn health_check(
        &self,
        _request: Request<crate::proto::distributed::Empty>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            healthy: true,
            node_id: self.node_id,
            uptime_ms: self.start_time.elapsed().as_millis() as u64,
        }))
    }
}

pub async fn start_server<V: VectorStorage + 'static, G: GraphStorage + 'static>(
    config: ShardServerConfig,
    vector_storage: V,
    graph_storage: G,
) -> Result<(), DistributedError> {
    let server = ShardServer::new(vector_storage, graph_storage, config.node_id);

    let addr = config.listen_addr.parse().map_err(|e| {
        DistributedError::Connection(format!("Invalid address {}: {}", config.listen_addr, e))
    })?;

    tonic::transport::Server::builder()
        .add_service(server.into_service())
        .serve(addr)
        .await
        .map_err(|e| DistributedError::Connection(e.to_string()))?;

    Ok(())
}
