//! Distributed Storage Test Server
//!
//! A simple server for testing distributed storage across multiple nodes.
//!
//! Usage:
//!   cargo run --example distributed_test_server -- --node-id 1 --listen-addr 127.0.0.1:50051 --peers 127.0.0.1:50052,127.0.0.1:50053

use sqlrustgo_distributed::{
    error::DistributedError,
    grpc_server::{start_server, GraphStorage, ShardServerConfig, VectorStorage},
    proto::distributed::SearchResult,
};
use std::collections::HashMap;

// Simple in-memory vector storage for testing
#[derive(Clone)]
struct TestVectorStorage {
    vectors: HashMap<u64, Vec<f32>>,
}

impl TestVectorStorage {
    fn new() -> Self {
        Self {
            vectors: HashMap::new(),
        }
    }
}

impl VectorStorage for TestVectorStorage {
    fn insert(
        &mut self,
        _shard_id: u64,
        id: u64,
        vector: Vec<f32>,
    ) -> Result<(), DistributedError> {
        self.vectors.insert(id, vector);
        Ok(())
    }

    fn search(
        &self,
        _shard_id: u64,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<SearchResult>, DistributedError> {
        let mut results: Vec<(u64, f32, Vec<f32>)> = self
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
                (*id, similarity, v.clone())
            })
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(results
            .into_iter()
            .take(top_k)
            .map(|(id, score, vector)| SearchResult {
                id,
                score,
                record: Some(sqlrustgo_distributed::proto::distributed::VectorRecord {
                    id,
                    vector,
                    metadata: std::collections::HashMap::new(),
                }),
            })
            .collect())
    }

    fn delete(&mut self, _shard_id: u64, id: u64) -> Result<(), DistributedError> {
        self.vectors.remove(&id);
        Ok(())
    }
}

// Simple in-memory graph storage for testing
#[derive(Clone)]
struct TestGraphStorage {
    nodes: HashMap<u64, (String, HashMap<String, String>)>,
    edges: HashMap<u64, Vec<(u64, u64, String)>>,
    next_node_id: u64,
}

impl TestGraphStorage {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            next_node_id: 1,
        }
    }
}

impl GraphStorage for TestGraphStorage {
    fn create_node(
        &mut self,
        _shard_id: u64,
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
            self.edges
                .insert(from, vec![(edge_id, to, label.to_string())]);
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut node_id: u64 = 1;
    let mut listen_addr = "127.0.0.1:50051".to_string();
    let mut peers: Vec<String> = Vec::new();

    for i in 1..args.len() {
        match args[i].as_str() {
            "--node-id" => {
                node_id = args[i + 1].parse().unwrap_or(1);
            }
            "--listen-addr" => {
                listen_addr = args[i + 1].clone();
            }
            "--peers" => {
                peers = args[i + 1].split(',').map(|s| s.to_string()).collect();
            }
            _ => {}
        }
        if args[i] == "--help" || args[i] == "-h" {
            println!("Distributed Storage Test Server");
            println!(
                "Usage: {} --node-id N --listen-addr ADDR --peers PEERS",
                args[0]
            );
            println!("  --node-id N       Node ID (default: 1)");
            println!("  --listen-addr     Address to listen on (default: 127.0.0.1:50051)");
            println!("  --peers           Comma-separated list of peer addresses");
            return Ok(());
        }
    }

    println!("===========================================");
    println!("  Distributed Storage Test Server");
    println!("===========================================");
    println!("  Node ID: {}", node_id);
    println!("  Listen:  {}", listen_addr);
    println!("  Peers:   {:?}", peers);
    println!("===========================================");

    // Create storage backends
    let vector_storage = TestVectorStorage::new();
    let graph_storage = TestGraphStorage::new();

    // Create server config
    let config = ShardServerConfig {
        node_id,
        listen_addr: listen_addr.clone(),
    };

    println!(
        "\n[Node {}] Starting gRPC server on {}...",
        node_id, listen_addr
    );

    // Start the server
    start_server(config, vector_storage, graph_storage)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
