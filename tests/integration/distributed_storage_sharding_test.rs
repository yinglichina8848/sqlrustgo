//! Distributed Storage Sharding Integration Tests
//!
//! Tests for distributed graph and vector storage with multi-node sharding:
//! - ShardGraph: label-based partitioning for graph data
//! - ShardedVectorIndex: hash-based partitioning for vector data
//! - Cross-shard queries and routing
//! - Multi-node cluster simulation
//!
//! For real distributed tests with 3 nodes, run:
//!   cargo test --test distributed_storage_sharding_test -- --nocapture

use sqlrustgo_distributed::{
    partition::{PartitionKey, PartitionStrategy, PartitionValue},
    shard_manager::{NodeId, ShardId, ShardInfo, ShardManager, ShardStatus},
    shard_router::{RoutedPlan, RouterError, ShardRouter},
};
use sqlrustgo_graph::{
    model::{Edge, Node, NodeId as GraphNodeId, PropertyMap},
    sharded_graph::{
        CrossShardTraversal, GraphShardId, LabelBasedGraphPartitioner, MultiShardGraphStore,
    },
    store::GraphStore,
    store::InMemoryGraphStore,
};
use sqlrustgo_vector::{
    sharded_index::{HashPartitioner, ShardedVectorIndex, VectorShardId},
    traits::{IndexEntry, VectorIndex},
    DistanceMetric, FlatIndex,
};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Test Utilities
// ============================================================================

fn generate_vectors(count: usize, dimension: usize) -> Vec<(u64, Vec<f32>)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..count as u64)
        .map(|i| {
            let vector: Vec<f32> = (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect();
            (i, vector)
        })
        .collect()
}

fn setup_3_node_cluster() -> (ShardManager, Vec<NodeId>) {
    let nodes = vec![1, 2, 3];
    let mut manager = ShardManager::new();

    manager.initialize_table_shards("users", 4, &nodes);

    (manager, nodes)
}

// ============================================================================
// Part 1: ShardManager and Partition Tests
// ============================================================================

#[test]
fn test_shard_manager_3_node_setup() {
    let (manager, nodes) = setup_3_node_cluster();

    assert_eq!(manager.num_nodes(), 3);
    assert_eq!(manager.num_shards(), 4);

    for node_id in &nodes {
        let shards = manager.get_shards_by_node(*node_id);
        assert!(!shards.is_empty(), "Node {} should have shards", node_id);
    }
}

#[test]
fn test_partition_key_hash_distribution() {
    let partition_key = PartitionKey::new_hash("id", 4);

    let mut distribution = HashMap::new();
    for i in 0..1000 {
        let shard = partition_key
            .partition(&PartitionValue::Integer(i))
            .unwrap();
        *distribution.entry(shard).or_insert(0) += 1;
    }

    // Should be roughly uniformly distributed
    let total: usize = distribution.values().sum();
    let avg = total / 4;

    for (shard, count) in distribution {
        let deviation = (count as f64 - avg as f64).abs() / avg as f64;
        assert!(
            deviation < 0.2,
            "Shard {} deviation {}% too high (avg={}, actual={})",
            shard,
            deviation * 100.0,
            avg,
            count
        );
    }
}

#[test]
fn test_partition_key_range_distribution() {
    let partition_key = PartitionKey::new_range("age", vec![18, 30, 65]);

    let mut bucket_counts = vec![0usize; 4];

    for i in 0..100 {
        let age = i as i64;
        let shard = partition_key
            .partition(&PartitionValue::Integer(age))
            .unwrap();
        bucket_counts[shard as usize] += 1;
    }

    // Age 0-17 -> bucket 0, 18-29 -> bucket 1, 30-64 -> bucket 2, 65+ -> bucket 3
    assert_eq!(bucket_counts[0], 18); // ages 0-17
    assert_eq!(bucket_counts[1], 12); // ages 18-29
    assert_eq!(bucket_counts[2], 35); // ages 30-64
    assert_eq!(bucket_counts[3], 35); // ages 65-99
}

#[test]
fn test_shard_status_transitions() {
    let mut manager = ShardManager::new();
    manager.create_shard(ShardInfo::new(0, 1));

    // Initial state
    assert_eq!(manager.get_shard(0).unwrap().status, ShardStatus::Active);

    // Migrating
    manager.set_shard_status(0, ShardStatus::Migrating);
    assert_eq!(manager.get_shard(0).unwrap().status, ShardStatus::Migrating);

    // Readonly
    manager.set_shard_status(0, ShardStatus::Readonly);
    assert_eq!(manager.get_shard(0).unwrap().status, ShardStatus::Readonly);

    // Offline
    manager.set_shard_status(0, ShardStatus::Offline);
    assert_eq!(manager.get_shard(0).unwrap().status, ShardStatus::Offline);
}

#[test]
fn test_get_active_shards() {
    let mut manager = ShardManager::new();
    manager.create_shard(ShardInfo::new(0, 1));
    manager.create_shard(ShardInfo::new(1, 2));
    manager.create_shard(ShardInfo::new(2, 3));

    manager.set_shard_status(1, ShardStatus::Offline);

    let active = manager.get_active_shards();
    assert_eq!(active.len(), 2);
    assert!(active.iter().all(|s| s.status == ShardStatus::Active));
}

// ============================================================================
// Part 2: ShardRouter Tests
// ============================================================================

#[test]
fn test_router_point_query_single_shard() {
    let (manager, _nodes) = setup_3_node_cluster();
    let router = ShardRouter::new(manager, 1);

    // User ID 5 should route to a specific shard
    let result = router.route_point_query("users", "id", PartitionValue::Integer(5));
    assert!(result.is_ok());

    let query = result.unwrap();
    assert!(query.node_id >= 1 && query.node_id <= 3);
}

#[test]
fn test_router_range_query() {
    let (manager, _nodes) = setup_3_node_cluster();
    let router = ShardRouter::new(manager, 1);

    let result = router.route_range_query("users", "id", 0, 10);
    assert!(result.is_ok());

    let plan = result.unwrap();
    // Range query may touch multiple shards
    assert!(!plan.involved_shards.is_empty());
}

#[test]
fn test_router_to_all_shards() {
    let (manager, _nodes) = setup_3_node_cluster();
    let router = ShardRouter::new(manager, 1);

    let result = router.route_to_all_shards("SELECT * FROM users", "users");
    assert!(result.is_ok());

    let plan = result.unwrap();
    assert!(plan.is_distributed);
    assert_eq!(plan.queries.len(), 4); // 4 shards for users table
}

#[test]
fn test_router_local_query() {
    let (manager, nodes) = setup_3_node_cluster();
    let router = ShardRouter::new(manager, nodes[0]);

    let result = router.route_local("SELECT 1");
    assert!(result.is_ok());

    let plan = result.unwrap();
    assert!(!plan.is_distributed);
    assert_eq!(plan.queries.len(), 1);
}

#[test]
fn test_router_no_partition_rule() {
    let (manager, _nodes) = setup_3_node_cluster();
    let router = ShardRouter::new(manager, 1);

    let result = router.route_point_query("unknown_table", "id", PartitionValue::Integer(5));
    assert!(matches!(result, Err(RouterError::NoPartitionRule(_))));
}

// ============================================================================
// Part 3: ShardGraph Tests (Label-Based Partitioning)
// ============================================================================

#[test]
fn test_shard_graph_label_partitioning() {
    let mut store = MultiShardGraphStore::new();

    // Setup label-based sharding
    store.register_label_sharding("User", GraphShardId(0));
    store.register_label_sharding("Product", GraphShardId(1));
    store.register_label_sharding("Order", GraphShardId(2));
    store.set_default_shard(GraphShardId(3));

    // Create nodes with different labels
    let user1 = store.create_node("User", PropertyMap::new());
    let user2 = store.create_node("User", PropertyMap::new());
    let product1 = store.create_node("Product", PropertyMap::new());
    let order1 = store.create_node("Order", PropertyMap::new());
    let unknown = store.create_node("Unknown", PropertyMap::new());

    // Verify label-based routing
    assert_eq!(store.get_shard_for_node(user1), Some(GraphShardId(0)));
    assert_eq!(store.get_shard_for_node(user2), Some(GraphShardId(0)));
    assert_eq!(store.get_shard_for_node(product1), Some(GraphShardId(1)));
    assert_eq!(store.get_shard_for_node(order1), Some(GraphShardId(2)));
    assert_eq!(store.get_shard_for_node(unknown), Some(GraphShardId(3))); // default

    // Verify node counts
    assert_eq!(store.nodes_by_label("User").len(), 2);
    assert_eq!(store.nodes_by_label("Product").len(), 1);
    assert_eq!(store.nodes_by_label("Order").len(), 1);
    assert_eq!(store.node_count(), 5);
}

#[test]
fn test_shard_graph_same_shard_edge_creation() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));

    let user1 = store.create_node("User", PropertyMap::new());
    let user2 = store.create_node("User", PropertyMap::new());

    // Same shard - edge creation should succeed
    let edge_id = store.create_edge(user1, user2, "KNOWS", PropertyMap::new());
    assert!(edge_id.is_ok());
    assert_eq!(store.edge_count(), 1);
}

#[test]
fn test_shard_graph_cross_shard_edge_rejection() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));
    store.register_label_sharding("Product", GraphShardId(1));

    let user1 = store.create_node("User", PropertyMap::new());
    let product1 = store.create_node("Product", PropertyMap::new());

    // Different shards - edge creation should fail
    let result = store.create_edge(user1, product1, "OWNED_BY", PropertyMap::new());
    assert!(result.is_err());
}

#[test]
fn test_shard_graph_bfs_traversal() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));

    let node1 = store.create_node("User", PropertyMap::new());
    store.set_default_shard(GraphShardId(0));

    let node2 = store.create_node("User", PropertyMap::new());

    store
        .create_edge(node1, node2, "KNOWS", PropertyMap::new())
        .unwrap();

    let mut visited = Vec::new();
    store.bfs(node1, |node_id| {
        visited.push(node_id);
        false
    });

    assert!(visited.contains(&node1), "Start node should be visited");
}

#[test]
fn test_shard_graph_delete_node() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));

    let user = store.create_node("User", PropertyMap::new());
    assert_eq!(store.node_count(), 1);

    let result = store.delete_node(user);
    assert!(result.is_ok());
    assert_eq!(store.node_count(), 0);
    assert!(store.get_node(user).is_none());
}

#[test]
fn test_shard_graph_multi_shard_counts() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));
    store.register_label_sharding("Product", GraphShardId(1));

    for _ in 0..10 {
        store.create_node("User", PropertyMap::new());
    }
    for _ in 0..5 {
        store.create_node("Product", PropertyMap::new());
    }

    assert_eq!(store.node_count(), 15);
    assert_eq!(store.total_node_count(), 15);
    assert_eq!(store.get_shard_ids().len(), 2);
}

// ============================================================================
// Part 4: ShardedVectorIndex Tests (Hash-Based Partitioning)
// ============================================================================

#[test]
fn test_sharded_vector_hash_partitioning() {
    let partitioner = HashPartitioner::new(3);

    // ID % 3 should determine shard
    assert_eq!(partitioner.get_shard_for_id(0), VectorShardId(0));
    assert_eq!(partitioner.get_shard_for_id(1), VectorShardId(1));
    assert_eq!(partitioner.get_shard_for_id(2), VectorShardId(2));
    assert_eq!(partitioner.get_shard_for_id(3), VectorShardId(0));
    assert_eq!(partitioner.get_shard_for_id(4), VectorShardId(1));
    assert_eq!(partitioner.get_shard_for_id(5), VectorShardId(2));
}

#[test]
fn test_sharded_vector_insert_and_search() {
    let mut index = ShardedVectorIndex::new(3, DistanceMetric::Cosine);

    // Insert vectors with known IDs
    index.insert(0, &[1.0, 0.0, 0.0]).unwrap();
    index.insert(1, &[0.0, 1.0, 0.0]).unwrap();
    index.insert(2, &[0.0, 0.0, 1.0]).unwrap();
    index.insert(3, &[0.707, 0.707, 0.0]).unwrap();
    index.insert(4, &[0.0, 0.707, 0.707]).unwrap();
    index.insert(5, &[0.707, 0.0, 0.707]).unwrap();

    assert_eq!(index.len(), 6);
    assert_eq!(index.dimension(), 3);

    // Search for similar vectors
    let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].id, 0); // Should find exact match first
}

#[test]
fn test_sharded_vector_delete() {
    let mut index = ShardedVectorIndex::new(2, DistanceMetric::Euclidean);

    index.insert(0, &[1.0, 0.0]).unwrap();
    index.insert(1, &[0.0, 1.0]).unwrap();
    index.insert(2, &[0.5, 0.5]).unwrap();

    assert_eq!(index.len(), 3);

    // Delete one vector
    index.delete(1).unwrap();
    assert_eq!(index.len(), 2);

    // Search should not return deleted vector
    let results = index.search(&[0.0, 1.0], 1).unwrap();
    assert!(results.iter().all(|r| r.id != 1));
}

#[test]
fn test_sharded_vector_build_index() {
    let mut index = ShardedVectorIndex::new(2, DistanceMetric::Cosine);

    // Insert many vectors
    for i in 0..100 {
        let vector = vec![rand::random::<f32>(), rand::random::<f32>()];
        index.insert(i, &vector).unwrap();
    }

    assert_eq!(index.len(), 100);
    index.build_index().unwrap();
    assert_eq!(index.len(), 100); // Should still be 100 after build
}

#[test]
fn test_sharded_vector_shard_stats() {
    let mut index = ShardedVectorIndex::new(3, DistanceMetric::Cosine);

    // Insert 10 vectors - should distribute across shards
    for i in 0..10 {
        index.insert(i, &[rand::random(), rand::random()]).unwrap();
    }

    let stats = index.all_shard_stats();
    assert_eq!(stats.len(), 3);

    let total_count: usize = stats.iter().map(|s| s.vector_count).sum();
    assert_eq!(total_count, 10);

    // Each shard should have dimension 2
    for stat in stats {
        assert_eq!(stat.dimension, 2);
    }
}

#[test]
fn test_sharded_vector_distribution_uniformity() {
    let mut index = ShardedVectorIndex::new(4, DistanceMetric::Cosine);

    // Insert 1000 vectors
    for i in 0..1000 {
        index.insert(i, &[rand::random(), rand::random()]).unwrap();
    }

    let stats = index.all_shard_stats();
    let counts: Vec<usize> = stats.iter().map(|s| s.vector_count).collect();

    // Should be roughly uniformly distributed (each ~250)
    let total: usize = counts.iter().sum();
    let avg = total / 4;

    for count in &counts {
        let deviation = ((*count as f64) - (avg as f64)).abs() / (avg as f64);
        assert!(
            deviation < 0.3,
            "Shard count {} deviates {}% from average {}",
            count,
            deviation * 100.0,
            avg
        );
    }
}

// ============================================================================
// Part 5: Cross-Shard Query Tests
// ============================================================================

#[test]
fn test_cross_shard_graph_traversal() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));
    store.set_default_shard(GraphShardId(0)); // All on same shard for traversal

    let node1 = store.create_node("User", PropertyMap::new());
    let node2 = store.create_node("User", PropertyMap::new());
    let node3 = store.create_node("User", PropertyMap::new());

    store
        .create_edge(node1, node2, "KNOWS", PropertyMap::new())
        .unwrap();
    store
        .create_edge(node2, node3, "KNOWS", PropertyMap::new())
        .unwrap();

    let traversal = CrossShardTraversal::new(store);
    let result = traversal.distributed_bfs(node1, 2);

    assert!(result.contains(&node1));
    assert!(result.contains(&node2));
    assert!(result.contains(&node3));
}

#[test]
fn test_cross_shard_vector_search_all_shards() {
    let mut index = ShardedVectorIndex::new(3, DistanceMetric::Cosine);

    // Insert vectors across all shards
    for i in 0..30 {
        let angle = (i as f32) * 0.1;
        let vector = &[angle.cos(), angle.sin()];
        index.insert(i, vector).unwrap();
    }

    index.build_index().unwrap();

    // Search should aggregate results from all shards
    let results = index.search(&[0.0, 1.0], 10).unwrap();
    assert!(!results.is_empty());
    assert!(results.len() <= 10);
}

#[test]
fn test_multi_table_routing() {
    let (manager, _nodes) = setup_3_node_cluster();
    let router = ShardRouter::new(manager, 1);

    let plan = router
        .route_to_all_shards("SELECT * FROM users", "users")
        .unwrap();
    assert_eq!(plan.queries.len(), 4);
    assert!(plan.is_distributed);
}

// ============================================================================
// Part 6: Large Scale Tests
// ============================================================================

#[test]
fn test_large_scale_graph_sharding() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));
    store.register_label_sharding("Product", GraphShardId(1));
    store.register_label_sharding("Order", GraphShardId(2));
    store.register_label_sharding("Review", GraphShardId(3));

    // Create 1000 users, 500 products, 2000 orders, 100 reviews
    for i in 0..1000 {
        store.create_node("User", PropertyMap::new());
    }
    for i in 0..500 {
        store.create_node("Product", PropertyMap::new());
    }
    for i in 0..2000 {
        store.create_node("Order", PropertyMap::new());
    }
    for i in 0..100 {
        store.create_node("Review", PropertyMap::new());
    }

    assert_eq!(store.node_count(), 3600);
    assert_eq!(store.nodes_by_label("User").len(), 1000);
    assert_eq!(store.nodes_by_label("Product").len(), 500);
    assert_eq!(store.nodes_by_label("Order").len(), 2000);
    assert_eq!(store.nodes_by_label("Review").len(), 100);
}

#[test]
fn test_large_scale_vector_sharding() {
    let mut index = ShardedVectorIndex::new(4, DistanceMetric::Cosine);

    // Insert 10000 vectors
    for i in 0..10000 {
        let vector = vec![rand::random::<f32>(), rand::random::<f32>()];
        index.insert(i, &vector).unwrap();
    }

    assert_eq!(index.len(), 10000);

    // Verify distribution across shards
    let stats = index.all_shard_stats();
    let total: usize = stats.iter().map(|s| s.vector_count).sum();
    assert_eq!(total, 10000);

    // Each shard should have roughly 2500 vectors
    for stat in stats {
        let deviation = ((stat.vector_count as f64) - 2500.0).abs() / 2500.0;
        assert!(
            deviation < 0.15,
            "Shard {:?} has {} vectors, deviation too high",
            stat.shard_id,
            stat.vector_count
        );
    }
}

#[test]
fn test_concurrent_vector_operations() {
    let mut index = ShardedVectorIndex::new(4, DistanceMetric::Euclidean);

    // Insert in batches
    for batch in 0..10 {
        let mut batch_vectors = Vec::new();
        for i in 0..100 {
            let id = batch * 100 + i;
            let vector = vec![rand::random(), rand::random()];
            batch_vectors.push((id, vector));
        }

        for (id, vector) in batch_vectors {
            index.insert(id, &vector).unwrap();
        }
    }

    assert_eq!(index.len(), 1000);

    // Search should work correctly
    let results = index.search(&[0.5, 0.5], 10).unwrap();
    assert!(!results.is_empty());
}

// ============================================================================
// Part 7: Error Handling Tests
// ============================================================================

#[test]
fn test_vector_shard_not_found_error() {
    let index = ShardedVectorIndex::new(2, DistanceMetric::Cosine);

    // Try to get stats for non-existent shard
    let stats = index.get_shard_stats(VectorShardId(99));
    assert!(stats.is_none());
}

#[test]
fn test_graph_shard_not_found() {
    let store = MultiShardGraphStore::new();

    // No shards registered, should not find any
    let node_id = GraphNodeId(999);
    let shard = store.get_shard_for_node(node_id);
    assert!(shard.is_none());
}

#[test]
fn test_empty_sharded_vector_search() {
    let index = ShardedVectorIndex::new(2, DistanceMetric::Cosine);

    // Search on empty index should return error
    let result = index.search(&[0.5, 0.5], 5);
    assert!(result.is_err());
}

#[test]
fn test_graph_after_delete_all_nodes() {
    let mut store = MultiShardGraphStore::new();
    store.register_label_sharding("User", GraphShardId(0));

    let user1 = store.create_node("User", PropertyMap::new());
    let user2 = store.create_node("User", PropertyMap::new());

    assert_eq!(store.node_count(), 2);

    store.delete_node(user1).unwrap();
    store.delete_node(user2).unwrap();

    assert_eq!(store.node_count(), 0);
    assert_eq!(store.nodes_by_label("User").len(), 0);
}
