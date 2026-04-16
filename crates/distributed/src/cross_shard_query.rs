use crate::error::DistributedError;
use crate::grpc_client::{ClientPool, NodeId, ShardClient, ShardId};
use crate::proto::distributed::SearchResult as ProtoSearchResult;
use futures::future::join_all;
use std::collections::HashMap;

pub struct CrossShardQueryExecutor {
    client_pool: ClientPool,
}

impl CrossShardQueryExecutor {
    pub fn new(client_pool: ClientPool) -> Self {
        Self { client_pool }
    }

    pub async fn execute_vector_search_all_shards(
        &self,
        shard_to_node: &HashMap<ShardId, NodeId>,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<ProtoSearchResult>, DistributedError> {
        let mut futures = Vec::new();

        for (shard_id, node_id) in shard_to_node.iter() {
            if let Some(client) = self.client_pool.get_client(*node_id).await {
                let shard_id = *shard_id;
                let query = query.to_vec();
                futures.push(async move {
                    client.search_vectors(shard_id, &query, top_k as u32).await
                });
            }
        }

        let results = join_all(futures).await;
        let mut all_results = Vec::new();

        for result in results {
            match result {
                Ok(shard_results) => all_results.extend(shard_results),
                Err(e) => tracing::warn!("Shard search failed: {}", e),
            }
        }

        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(top_k);
        all_results.dedup_by(|a, b| a.id == b.id);

        Ok(all_results)
    }

    pub async fn scatter_gather_vector_search(
        &self,
        nodes: &[(NodeId, ShardId)],
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<ProtoSearchResult>, DistributedError> {
        let mut futures = Vec::new();

        for (node_id, shard_id) in nodes {
            if let Some(client) = self.client_pool.get_client(*node_id).await {
                let shard_id = *shard_id;
                let query = query.to_vec();
                futures.push(async move {
                    client.search_vectors(shard_id, &query, top_k as u32).await
                });
            }
        }

        let results = join_all(futures).await;
        let mut merged = Vec::new();

        for result in results {
            if let Ok(shard_results) = result {
                merged.extend(shard_results);
            }
        }

        merged.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        merged.truncate(top_k);

        Ok(merged)
    }

    pub async fn broadcast_vector_insert(
        &self,
        nodes: &[(NodeId, ShardId)],
        id: u64,
        vector: &[f32],
    ) -> HashMap<NodeId, Result<(), DistributedError>> {
        let mut futures = Vec::new();

        for (node_id, shard_id) in nodes {
            if let Some(client) = self.client_pool.get_client(*node_id).await {
                let shard_id = *shard_id;
                let vector = vector.to_vec();
                futures.push(async move {
                    let client = client;
                    let result = client.insert_vector(shard_id, id, &vector).await;
                    (*node_id, result)
                });
            }
        }

        let results = join_all(futures).await;
        results.into_iter().collect()
    }

    pub async fn health_check_nodes(&self) -> HashMap<NodeId, bool> {
        self.client_pool.health_check_all().await
    }
}

pub struct QueryRouter {
    shard_manager: crate::ShardManager,
    local_node_id: NodeId,
}

impl QueryRouter {
    pub fn new(shard_manager: crate::ShardManager, local_node_id: NodeId) -> Self {
        Self {
            shard_manager,
            local_node_id,
        }
    }

    pub fn get_shard_nodes(&self, shard_id: ShardId) -> Option<NodeId> {
        self.shard_manager
            .get_shard(shard_id)
            .and_then(|info| info.primary_node())
    }

    pub fn get_all_shard_nodes(&self) -> Vec<(NodeId, ShardId)> {
        self.shard_manager
            .get_active_shards()
            .iter()
            .filter_map(|shard| {
                shard.primary_node().map(|node_id| (node_id, shard.shard_id))
            })
            .collect()
    }

    pub fn is_local_shard(&self, shard_id: ShardId) -> bool {
        self.get_shard_nodes(shard_id)
            .map(|node_id| node_id == self.local_node_id)
            .unwrap_or(false)
    }
}
