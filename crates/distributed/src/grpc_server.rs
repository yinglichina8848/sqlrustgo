use crate::error::DistributedError;
use crate::proto::distributed::{
    shard_service_server::{ShardService, ShardServiceServer},
    CreateEdgeRequest, CreateEdgeResponse, CreateNodeRequest, CreateNodeResponse,
    DeleteNodeRequest, DeleteNodeResponse, DeleteVectorRequest, DeleteVectorResponse, EdgeInfo,
    GetEdgesRequest, GetEdgesResponse, GetNodeRequest, GetNodeResponse, HealthCheckResponse,
    InsertVectorRequest, InsertVectorResponse, PropertyMap, SearchResult, SearchVectorsRequest,
    SearchVectorsResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tonic::{Request, Response, Status};

pub type NodeId = u64;
pub type ShardId = u64;

pub trait VectorStorage: Send + Sync {
    fn insert(
        &mut self,
        shard_id: ShardId,
        id: u64,
        vector: Vec<f32>,
    ) -> Result<(), DistributedError>;
    fn search(
        &self,
        shard_id: ShardId,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, DistributedError>;
    fn delete(&mut self, shard_id: ShardId, id: u64) -> Result<(), DistributedError>;
}

pub trait GraphStorage: Send + Sync {
    fn create_node(
        &mut self,
        shard_id: ShardId,
        label: &str,
        props: HashMap<String, String>,
    ) -> Result<u64, DistributedError>;
    fn get_node(&self, node_id: u64) -> Option<(String, HashMap<String, String>)>;
    fn delete_node(&mut self, node_id: u64) -> Result<(), DistributedError>;
    fn create_edge(
        &mut self,
        from: u64,
        to: u64,
        label: &str,
        props: HashMap<String, String>,
    ) -> Result<u64, DistributedError>;
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockVectorStorage {
        vectors: HashMap<u64, Vec<f32>>,
    }

    impl MockVectorStorage {
        fn new() -> Self {
            Self {
                vectors: HashMap::new(),
            }
        }
    }

    impl VectorStorage for MockVectorStorage {
        fn insert(
            &mut self,
            _shard_id: ShardId,
            id: u64,
            vector: Vec<f32>,
        ) -> Result<(), DistributedError> {
            self.vectors.insert(id, vector);
            Ok(())
        }

        fn search(
            &self,
            _shard_id: ShardId,
            query: &[f32],
            top_k: usize,
        ) -> Result<Vec<SearchResult>, DistributedError> {
            let mut results: Vec<(u64, f32)> = self
                .vectors
                .iter()
                .map(|(id, v)| {
                    let dot = v.iter().zip(query.iter()).map(|(a, b)| a * b).sum::<f32>();
                    let norm_v = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let norm_q = query.iter().map(|x| x * x).sum::<f32>().sqrt();
                    let similarity = if norm_v > 0.0 && norm_q > 0.0 {
                        dot / (norm_v * norm_q)
                    } else {
                        0.0
                    };
                    (*id, similarity)
                })
                .collect();

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            results.truncate(top_k);

            Ok(results
                .into_iter()
                .map(|(id, score)| SearchResult {
                    id,
                    score,
                    record: None,
                })
                .collect())
        }

        fn delete(&mut self, _shard_id: ShardId, id: u64) -> Result<(), DistributedError> {
            self.vectors.remove(&id);
            Ok(())
        }
    }

    struct MockGraphStorage {
        nodes: HashMap<u64, (String, HashMap<String, String>)>,
        edges: HashMap<u64, Vec<(u64, u64, String)>>,
        next_node_id: u64,
    }

    impl MockGraphStorage {
        fn new() -> Self {
            Self {
                nodes: HashMap::new(),
                edges: HashMap::new(),
                next_node_id: 1,
            }
        }
    }

    impl GraphStorage for MockGraphStorage {
        fn create_node(
            &mut self,
            _shard_id: ShardId,
            label: &str,
            props: HashMap<String, String>,
        ) -> Result<u64, DistributedError> {
            let node_id = self.next_node_id;
            self.next_node_id += 1;
            self.nodes.insert(node_id, (label.to_string(), props));
            self.edges.insert(node_id, Vec::new());
            Ok(node_id)
        }

        fn get_node(&self, node_id: u64) -> Option<(String, HashMap<String, String>)> {
            self.nodes.get(&node_id).cloned()
        }

        fn delete_node(&mut self, node_id: u64) -> Result<(), DistributedError> {
            self.nodes.remove(&node_id);
            self.edges.remove(&node_id);
            Ok(())
        }

        fn create_edge(
            &mut self,
            from: u64,
            to: u64,
            label: &str,
            _props: HashMap<String, String>,
        ) -> Result<u64, DistributedError> {
            let edge_id = self.next_node_id;
            self.next_node_id += 1;
            if let Some(edges) = self.edges.get_mut(&from) {
                edges.push((edge_id, to, label.to_string()));
            } else {
                self.edges.insert(from, vec![(edge_id, to, label.to_string())]);
            }
            Ok(edge_id)
        }

        fn get_edges(&self, node_id: u64) -> Vec<(u64, u64, u64, String)> {
            self.edges
                .get(&node_id)
                .map(|edges| {
                    edges
                        .iter()
                        .map(|(edge_id, to, label)| (0, *edge_id, *to, label.clone()))
                        .collect()
                })
                .unwrap_or_default()
        }
    }

    #[test]
    fn test_shard_server_config() {
        let config = ShardServerConfig {
            node_id: 1,
            listen_addr: "127.0.0.1:50051".to_string(),
        };
        assert_eq!(config.node_id, 1);
        assert_eq!(config.listen_addr, "127.0.0.1:50051");
    }

    #[test]
    fn test_shard_server_new() {
        let vector_storage = MockVectorStorage::new();
        let graph_storage = MockGraphStorage::new();
        let server = ShardServer::new(vector_storage, graph_storage, 1);
        assert_eq!(server.node_id, 1);
    }

    #[test]
    fn test_mock_vector_storage_insert() {
        let mut storage = MockVectorStorage::new();
        let result = storage.insert(0, 1, vec![0.1, 0.2, 0.3]);
        assert!(result.is_ok());
        assert!(storage.vectors.contains_key(&1));
    }

    #[test]
    fn test_mock_vector_storage_search() {
        let mut storage = MockVectorStorage::new();
        storage.insert(0, 1, vec![1.0, 0.0, 0.0]).unwrap();
        storage.insert(0, 2, vec![0.0, 1.0, 0.0]).unwrap();

        let results = storage.search(0, &[1.0, 0.0, 0.0], 10);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_mock_vector_storage_delete() {
        let mut storage = MockVectorStorage::new();
        storage.insert(0, 1, vec![0.1, 0.2, 0.3]).unwrap();
        assert!(storage.vectors.contains_key(&1));

        let result = storage.delete(0, 1);
        assert!(result.is_ok());
        assert!(!storage.vectors.contains_key(&1));
    }

    #[test]
    fn test_mock_graph_storage_create_node() {
        let mut storage = MockGraphStorage::new();
        let result = storage.create_node(0, "User", HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let result2 = storage.create_node(0, "Product", HashMap::new());
        assert_eq!(result2.unwrap(), 2);
    }

    #[test]
    fn test_mock_graph_storage_get_node() {
        let mut storage = MockGraphStorage::new();
        let mut props = HashMap::new();
        props.insert("name".to_string(), "test".to_string());

        storage.create_node(0, "User", props).unwrap();
        let result = storage.get_node(1);
        assert!(result.is_some());
        let (label, retrieved_props) = result.unwrap();
        assert_eq!(label, "User");
        assert_eq!(retrieved_props.get("name").unwrap(), "test");
    }

    #[test]
    fn test_mock_graph_storage_get_node_not_found() {
        let storage = MockGraphStorage::new();
        let result = storage.get_node(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_mock_graph_storage_delete_node() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        assert!(storage.nodes.contains_key(&1));

        let result = storage.delete_node(1);
        assert!(result.is_ok());
        assert!(!storage.nodes.contains_key(&1));
    }

    #[test]
    fn test_mock_graph_storage_create_edge() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        storage.create_node(0, "Product", HashMap::new()).unwrap();

        let result = storage.create_edge(1, 2, "OWNS", HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_mock_graph_storage_get_edges() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        storage.create_node(0, "Product", HashMap::new()).unwrap();
        storage.create_edge(1, 2, "OWNS", HashMap::new()).unwrap();

        let edges = storage.get_edges(1);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].2, 2);
        assert_eq!(edges[0].3, "OWNS");
    }

    #[test]
    fn test_mock_graph_storage_get_edges_empty() {
        let storage = MockGraphStorage::new();
        let edges = storage.get_edges(999);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_vector_storage_trait_object() {
        let mut storage: Box<dyn VectorStorage> = Box::new(MockVectorStorage::new());
        storage.insert(0, 1, vec![1.0, 2.0]).unwrap();
        assert!(storage.search(0, &[1.0, 2.0], 10).is_ok());
        storage.delete(0, 1).unwrap();
    }

    #[test]
    fn test_graph_storage_trait_object() {
        let mut storage: Box<dyn GraphStorage> = Box::new(MockGraphStorage::new());
        storage.create_node(0, "User", HashMap::new()).unwrap();
        assert!(storage.get_node(1).is_some());
        storage.delete_node(1).unwrap();
        assert!(storage.get_node(1).is_none());
    }

    #[test]
    fn test_mock_vector_storage_empty_search() {
        let storage = MockVectorStorage::new();
        let results = storage.search(0, &[1.0, 0.0], 10);
        assert!(results.is_ok());
        assert!(results.unwrap().is_empty());
    }

    #[test]
    fn test_mock_vector_storage_multiple_shards() {
        let mut storage = MockVectorStorage::new();
        storage.insert(0, 1, vec![1.0, 0.0]).unwrap();
        storage.insert(1, 2, vec![0.0, 1.0]).unwrap();
        storage.insert(0, 3, vec![0.5, 0.5]).unwrap();
        assert_eq!(storage.vectors.len(), 3);
    }

    #[test]
    fn test_mock_vector_storage_search_with_different_query() {
        let mut storage = MockVectorStorage::new();
        storage.insert(0, 1, vec![1.0, 0.0, 0.0]).unwrap();
        storage.insert(0, 2, vec![0.0, 1.0, 0.0]).unwrap();
        storage.insert(0, 3, vec![0.0, 0.0, 1.0]).unwrap();
        let results = storage.search(0, &[0.0, 1.0, 0.0], 2);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_mock_graph_storage_delete_nonexistent_node() {
        let mut storage = MockGraphStorage::new();
        let result = storage.delete_node(999);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_graph_storage_create_edge_nonexistent_from_node() {
        let mut storage = MockGraphStorage::new();
        let result = storage.create_edge(999, 1, "KNOWS", HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_graph_storage_create_edge_nonexistent_to_node() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        let result = storage.create_edge(1, 999, "KNOWS", HashMap::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_graph_storage_get_edges_after_delete() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        storage.create_node(0, "Product", HashMap::new()).unwrap();
        storage.create_edge(1, 2, "OWNS", HashMap::new()).unwrap();
        storage.delete_node(1).unwrap();
        let edges = storage.get_edges(1);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_mock_graph_storage_node_count() {
        let mut storage = MockGraphStorage::new();
        assert_eq!(storage.nodes.len(), 0);
        storage.create_node(0, "User", HashMap::new()).unwrap();
        assert_eq!(storage.nodes.len(), 1);
        storage.create_node(0, "Product", HashMap::new()).unwrap();
        assert_eq!(storage.nodes.len(), 2);
    }

    #[test]
    fn test_mock_graph_storage_edge_count() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        storage.create_node(0, "Product", HashMap::new()).unwrap();
        assert_eq!(storage.edges.len(), 2);
        storage.create_edge(1, 2, "OWNS", HashMap::new()).unwrap();
        assert_eq!(storage.edges.len(), 2);
    }

    #[test]
    fn test_search_result_record_field() {
        let result = SearchResult {
            id: 42,
            score: 0.95,
            record: None,
        };
        assert_eq!(result.id, 42);
        assert_eq!(result.score, 0.95);
    }

    #[test]
    fn test_edge_info_fields() {
        let edge = EdgeInfo {
            edge_id: 1,
            from: 10,
            to: 20,
            label: "KNOWS".to_string(),
        };
        assert_eq!(edge.edge_id, 1);
        assert_eq!(edge.from, 10);
        assert_eq!(edge.to, 20);
    }

    #[test]
    fn test_health_check_response() {
        let response = HealthCheckResponse {
            healthy: true,
            node_id: 1,
            uptime_ms: 1000,
        };
        assert!(response.healthy);
        assert_eq!(response.node_id, 1);
        assert_eq!(response.uptime_ms, 1000);
    }

    #[test]
    fn test_health_check_response_unhealthy() {
        let response = HealthCheckResponse {
            healthy: false,
            node_id: 2,
            uptime_ms: 5000,
        };
        assert!(!response.healthy);
    }

    #[test]
    fn test_mock_vector_storage_search_empty_shard() {
        let mut storage = MockVectorStorage::new();
        storage.insert(0, 1, vec![1.0, 0.0]).unwrap();
        let results = storage.search(999, &[1.0, 0.0], 10);
        assert!(results.is_ok());
    }

    #[test]
    fn test_mock_vector_storage_insert_multiple() {
        let mut storage = MockVectorStorage::new();
        for i in 0..5 {
            let result = storage.insert(0, i, vec![i as f32, 0.0]);
            assert!(result.is_ok());
        }
        assert_eq!(storage.vectors.len(), 5);
    }

    #[test]
    fn test_mock_graph_storage_multiple_nodes() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        storage.create_node(0, "Product", HashMap::new()).unwrap();
        storage.create_node(0, "Order", HashMap::new()).unwrap();
        assert_eq!(storage.nodes.len(), 3);
    }

    #[test]
    fn test_mock_graph_storage_get_node_after_update() {
        let mut storage = MockGraphStorage::new();
        storage.create_node(0, "User", HashMap::new()).unwrap();
        let mut props = HashMap::new();
        props.insert("name".to_string(), "John".to_string());
        storage.create_node(0, "User", props).unwrap();
        assert_eq!(storage.nodes.len(), 2);
    }

    #[test]
    fn test_search_result_with_record() {
        use crate::proto::distributed::VectorRecord;
        let result = SearchResult {
            id: 10,
            score: 0.85,
            record: Some(VectorRecord {
                id: 10,
                vector: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            }),
        };
        assert_eq!(result.id, 10);
        assert_eq!(result.score, 0.85);
        assert!(result.record.is_some());
    }

    #[test]
    fn test_insert_vector_request_with_metadata() {
        use crate::proto::distributed::VectorRecord;
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());
        let request = InsertVectorRequest {
            shard_id: 5,
            record: Some(VectorRecord {
                id: 42,
                vector: vec![0.1, 0.2],
                metadata,
            }),
        };
        assert_eq!(request.shard_id, 5);
        assert!(request.record.is_some());
        let record = request.record.unwrap();
        assert_eq!(record.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_delete_vector_response_with_error() {
        let response = DeleteVectorResponse {
            success: false,
            error: "Shard unavailable".to_string(),
        };
        assert!(!response.success);
        assert_eq!(response.error, "Shard unavailable");
    }

    #[test]
    fn test_create_edge_response_with_error() {
        let response = CreateEdgeResponse {
            edge_id: 0,
            success: false,
            error: "Target node not found".to_string(),
        };
        assert!(!response.success);
        assert!(response.error.contains("not found"));
    }

    #[test]
    fn test_get_node_response_with_properties() {
        let mut props = HashMap::new();
        props.insert("age".to_string(), "30".to_string());
        let response = GetNodeResponse {
            node_id: 5,
            label: "Person".to_string(),
            properties: Some(PropertyMap { properties: props }),
            found: true,
        };
        assert!(response.found);
        assert_eq!(response.label, "Person");
        assert_eq!(response.properties.unwrap().properties.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_search_vectors_request_fields() {
        let request = SearchVectorsRequest {
            shard_id: 1,
            query: vec![0.1, 0.2, 0.3, 0.4],
            top_k: 50,
        };
        assert_eq!(request.shard_id, 1);
        assert_eq!(request.query.len(), 4);
        assert_eq!(request.top_k, 50);
    }
}
