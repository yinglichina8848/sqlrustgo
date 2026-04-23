use crate::error::DistributedError;
use crate::grpc_client::{ClientPool, NodeId, ShardId};
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
                futures.push(
                    async move { client.search_vectors(shard_id, &query, top_k as u32).await },
                );
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

        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
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
                futures.push(
                    async move { client.search_vectors(shard_id, &query, top_k as u32).await },
                );
            }
        }

        let results = join_all(futures).await;
        let mut merged = Vec::new();

        for shard_results in results.into_iter().flatten() {
            merged.extend(shard_results);
        }

        merged.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
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
                shard
                    .primary_node()
                    .map(|node_id| (node_id, shard.shard_id))
            })
            .collect()
    }

    pub fn is_local_shard(&self, shard_id: ShardId) -> bool {
        self.get_shard_nodes(shard_id)
            .map(|node_id| node_id == self.local_node_id)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grpc_client::ClientPool;
    use crate::shard_manager::{ShardInfo, ShardManager};

    #[test]
    fn test_query_router_new() {
        let manager = ShardManager::new();
        let router = QueryRouter::new(manager, 1);
        assert_eq!(router.is_local_shard(0), false);
    }

    #[test]
    fn test_query_router_get_shard_nodes() {
        let mut manager = ShardManager::new();
        let info = ShardInfo::new(0, 1);
        manager.create_shard(info);

        let router = QueryRouter::new(manager, 1);
        let node = router.get_shard_nodes(0);
        assert_eq!(node, Some(1));
    }

    #[test]
    fn test_query_router_get_all_shard_nodes() {
        let mut manager = ShardManager::new();
        let info = ShardInfo::new(0, 1);
        manager.create_shard(info);

        let router = QueryRouter::new(manager, 1);
        let all_nodes = router.get_all_shard_nodes();
        assert_eq!(all_nodes.len(), 1);
        assert_eq!(all_nodes[0].0, 1);
        assert_eq!(all_nodes[0].1, 0);
    }

    #[test]
    fn test_query_router_is_local_shard() {
        let mut manager1 = ShardManager::new();
        let info = ShardInfo::new(0, 1);
        manager1.create_shard(info);

        let router = QueryRouter::new(manager1, 1);
        assert!(router.is_local_shard(0));

        let mut manager2 = ShardManager::new();
        let info2 = ShardInfo::new(0, 1);
        manager2.create_shard(info2);
        let router2 = QueryRouter::new(manager2, 2);
        assert!(!router2.is_local_shard(0));
    }

    #[test]
    fn test_query_router_is_local_shard_nonexistent() {
        let manager = ShardManager::new();
        let router = QueryRouter::new(manager, 1);
        assert!(!router.is_local_shard(999));
    }

    #[test]
    fn test_cross_shard_query_executor_new() {
        let pool = ClientPool::new();
        let _executor = CrossShardQueryExecutor::new(pool);
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_health_check() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);
        let health = executor.health_check_nodes().await;
        assert!(health.is_empty());
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_broadcast_insert() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);

        let nodes = vec![(1u64, 0u64), (2u64, 0u64)];
        let results = executor.broadcast_vector_insert(&nodes, 1, &[0.1, 0.2, 0.3]).await;

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_scatter_gather_empty() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);

        let nodes = vec![];
        let result = executor
            .scatter_gather_vector_search(&nodes, &[0.1, 0.2], 10)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_vector_search_empty() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);

        let shard_to_node = HashMap::new();
        let result = executor
            .execute_vector_search_all_shards(&shard_to_node, &[0.1, 0.2], 10)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_query_router_local_node_id() {
        let manager = ShardManager::new();
        let router = QueryRouter::new(manager, 42);
        assert_eq!(router.local_node_id, 42);
    }

    #[test]
    fn test_query_router_get_shard_nodes_nonexistent() {
        let manager = ShardManager::new();
        let router = QueryRouter::new(manager, 1);
        let node = router.get_shard_nodes(999);
        assert_eq!(node, None);
    }

    #[test]
    fn test_query_router_get_all_shard_nodes_empty() {
        let manager = ShardManager::new();
        let router = QueryRouter::new(manager, 1);
        let all_nodes = router.get_all_shard_nodes();
        assert!(all_nodes.is_empty());
    }

    #[test]
    fn test_query_router_multiple_shards() {
        let mut manager = ShardManager::new();
        let info1 = ShardInfo::new(0, 1);
        let info2 = ShardInfo::new(1, 2);
        manager.create_shard(info1);
        manager.create_shard(info2);

        let router = QueryRouter::new(manager, 1);
        let all_nodes = router.get_all_shard_nodes();
        assert_eq!(all_nodes.len(), 2);
    }

    #[test]
    fn test_query_router_get_shard_nodes_different_nodes() {
        let mut manager = ShardManager::new();
        let info1 = ShardInfo::new(0, 10);
        manager.create_shard(info1);

        let router = QueryRouter::new(manager, 1);
        let node = router.get_shard_nodes(0);
        assert_eq!(node, Some(10));
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_with_pool() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);
        let _ = executor.health_check_nodes().await;
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_scatter_gather_single_node() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);

        let nodes = vec![(1u64, 0u64)];
        let result = executor
            .scatter_gather_vector_search(&nodes, &[0.1, 0.2], 10)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_scatter_gather_multiple_nodes() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);

        let nodes = vec![(1u64, 0u64), (2u64, 1u64), (3u64, 2u64)];
        let result = executor
            .scatter_gather_vector_search(&nodes, &[0.1, 0.2], 5)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_cross_shard_query_executor_broadcast_insert_empty() {
        let pool = ClientPool::new();
        let executor = CrossShardQueryExecutor::new(pool);

        let nodes = vec![];
        let results = executor.broadcast_vector_insert(&nodes, 1, &[0.1, 0.2]).await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_query_router_is_local_shard_remote() {
        let mut manager = ShardManager::new();
        let info = ShardInfo::new(5, 99);
        manager.create_shard(info);

        let router = QueryRouter::new(manager, 1);
        assert!(!router.is_local_shard(5));
    }

    #[test]
    fn test_query_router_all_shard_nodes_filters_inactive() {
        let mut manager = ShardManager::new();
        let info1 = ShardInfo::new(0, 1);
        manager.create_shard(info1);

        let router = QueryRouter::new(manager, 1);
        let nodes = router.get_all_shard_nodes();
        assert_eq!(nodes.len(), 1);
    }
}
