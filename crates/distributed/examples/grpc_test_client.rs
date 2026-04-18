//! Simple gRPC test client for distributed storage

use sqlrustgo_distributed::{
    grpc_client::{ClientPool, ShardClient},
    ShardManager, ShardReplicaManager,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===========================================");
    println!("  Distributed Storage gRPC Test Client");
    println!("===========================================");

    let mut pool = ClientPool::new();
    pool.add_client(1, "http://127.0.0.1:50051").await?;
    pool.add_client(2, "http://127.0.0.1:50052").await?;
    pool.add_client(3, "http://127.0.0.1:50053").await?;

    println!("\n=== Test 1: Health Check All ===");
    let results = pool.health_check_all().await;
    for (node_id, healthy) in &results {
        println!("Node {}: healthy={}", node_id, healthy);
    }

    println!("\n=== Test 2: Health Check Individual ===");
    if let Some(client) = pool.get_client(1).await {
        match client.health_check().await {
            Ok(healthy) => println!("Node 1: healthy={}", healthy),
            Err(e) => println!("Node 1: FAILED - {}", e),
        }
    }

    println!("\n=== Test 3: Insert Vectors to Node 1 ===");
    let test_vectors = vec![
        (1, vec![0.1, 0.2, 0.3, 0.4]),
        (2, vec![0.2, 0.3, 0.4, 0.5]),
        (3, vec![0.3, 0.4, 0.5, 0.6]),
    ];

    if let Some(client) = pool.get_client(1).await {
        for (id, vector) in &test_vectors {
            match client.insert_vector(0, *id, vector).await {
                Ok(_) => println!("Inserted vector {} to node 1", id),
                Err(e) => println!("Failed to insert vector {}: {}", id, e),
            }
        }
    }

    println!("\n=== Test 4: Search Vectors ===");
    let query = vec![0.15, 0.25, 0.35, 0.45];
    if let Some(client) = pool.get_client(1).await {
        match client.search_vectors(0, &query, 3).await {
            Ok(results) => {
                println!("Search results from node 1:");
                for r in &results {
                    println!("  id={}, score={}", r.id, r.score);
                }
            }
            Err(e) => println!("Search failed: {}", e),
        }
    }

    println!("\n=== Test 5: Raft Consensus ===");
    let mut replica_manager = ShardReplicaManager::new(1);
    replica_manager.register_shard(0, vec![1, 2, 3]);

    println!("Initial state: is_leader={}", replica_manager.is_leader(0));

    replica_manager.become_leader(0).unwrap();
    println!(
        "After become_leader: is_leader={}",
        replica_manager.is_leader(0)
    );
    println!("Primary: {:?}", replica_manager.get_primary(0));

    println!("\n=== Test 6: Shard Routing ===");
    let mut shard_manager = ShardManager::new();
    shard_manager.initialize_table_shards("users", 4, &[1, 2, 3]);

    let key = sqlrustgo_distributed::partition::PartitionValue::Integer(7);
    let shard_id = shard_manager
        .get_partition_key("users")
        .unwrap()
        .partition(&key)
        .unwrap();
    println!("Key 7 partitions to shard {}", shard_id);

    let shard_info = shard_manager.get_shard(shard_id).unwrap();
    println!(
        "Shard {} primary: {:?}",
        shard_id,
        shard_info.primary_node()
    );
    println!("Shard {} replicas: {:?}", shard_id, shard_info.replicas());

    println!("\n===========================================");
    println!("  All tests completed!");
    println!("===========================================");

    Ok(())
}
