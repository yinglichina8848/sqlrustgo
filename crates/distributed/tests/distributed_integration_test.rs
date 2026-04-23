use sqlrustgo_distributed::{
    consensus::ShardReplicaManager,
    failover_manager::{FailoverConfig, FailoverManager},
    partition::{PartitionKey, PartitionPruner, PartitionValue},
    replication::{GtidInterval, GtidManager, GtidSet, SemiSyncManager, SemiSyncState},
    shard_manager::{PartitionRule, ShardInfo, ShardManager},
    shard_router::ShardRouter,
    two_phase_commit::{DistributedTransaction, Participant, TwoPhaseCommit},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_gtid_manager_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = GtidManager::new(1);

    manager.add_gtid(1, 100);
    manager.add_gtid(1, 200);
    manager.add_gtid(1, 300);

    assert!(manager.contains(1, 100));
    assert!(manager.contains(1, 200));
    assert!(!manager.contains(1, 150));

    let executed = manager.get_executed_set();
    assert!(!executed.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_gtid_set_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut set1 = GtidSet::new();
    set1.add_interval(GtidInterval::new(1, 1, 100));
    set1.add_interval(GtidInterval::new(1, 200, 300));

    assert!(set1.contains(1, 50));
    assert!(set1.contains(1, 250));
    assert!(!set1.contains(1, 150));

    Ok(())
}

#[tokio::test]
async fn test_semisync_state() -> Result<(), Box<dyn std::error::Error>> {
    let state = SemiSyncState::default();
    assert_eq!(state, SemiSyncState::Off);

    let state2 = SemiSyncState::WaitServer;
    assert_eq!(state2, SemiSyncState::WaitServer);

    Ok(())
}

#[tokio::test]
async fn test_semisync_manager() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = SemiSyncManager::new();

    manager.add_replica(2);
    manager.add_replica(3);

    assert_eq!(manager.get_replica_count(), 2);

    manager.record_ack(2);
    manager.remove_replica(2);
    assert_eq!(manager.get_replica_count(), 1);

    Ok(())
}

#[tokio::test]
async fn test_hash_partition() -> Result<(), Box<dyn std::error::Error>> {
    let partition_key = PartitionKey::new_hash("user_id", 4);

    let results: Vec<u64> = (0..1000)
        .map(|i| {
            let value = PartitionValue::from_i64(i);
            partition_key.partition(&value).unwrap_or(0)
        })
        .collect();

    let mut counts = vec![0u64; 4];
    for r in &results {
        counts[*r as usize] += 1;
    }

    let avg = 250;
    for c in &counts {
        assert!((*c as i64 - avg as i64).abs() < 100);
    }

    let p1 = {
        let value = PartitionValue::from_text("user_123");
        partition_key.partition(&value).unwrap_or(0)
    };
    let p2 = {
        let value = PartitionValue::from_text("user_123");
        partition_key.partition(&value).unwrap_or(0)
    };
    assert_eq!(p1, p2);

    Ok(())
}

#[tokio::test]
async fn test_partition_pruner() -> Result<(), Box<dyn std::error::Error>> {
    let partition_key = PartitionKey::new_hash("name", 4);
    let pruner = PartitionPruner::new(partition_key);

    let value = PartitionValue::from_text("test");
    let result = pruner.prune_for_equality(&value);
    assert!(!result.is_empty());

    let range_result = pruner.prune_for_range(0, 100);
    assert!(!range_result.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_shard_manager_register() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = ShardManager::new();

    let info = ShardInfo::new(0, 1);
    manager.create_shard(info);

    let shard = manager.get_shard(0);
    assert!(shard.is_some());

    Ok(())
}

#[tokio::test]
async fn test_shard_manager_multi_replica() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = ShardManager::new();

    let mut info = ShardInfo::new(0, 1);
    info.add_replica(2);
    info.add_replica(3);
    manager.create_shard(info);

    let all_shards = manager.get_shards();
    assert_eq!(all_shards.len(), 1);

    let active = manager.get_active_shards();
    assert!(!active.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_failover_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
    let shard_manager = Arc::new(RwLock::new(ShardManager::new()));
    let replica_manager = Arc::new(RwLock::new(ShardReplicaManager::new(1)));

    let config = FailoverConfig {
        election_timeout: Duration::from_millis(5000),
        heartbeat_interval: Duration::from_millis(500),
        max_replication_lag_ms: 1000,
    };

    let _manager =
        FailoverManager::with_config(1, shard_manager.clone(), replica_manager.clone(), config);

    Ok(())
}

#[tokio::test]
async fn test_cluster_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let shard_manager = Arc::new(RwLock::new(ShardManager::new()));
    let replica_manager = Arc::new(RwLock::new(ShardReplicaManager::new(1)));

    let info = ShardInfo::new(0, 1);
    shard_manager.write().await.create_shard(info);

    replica_manager.write().await.register_shard(0, vec![1]);

    let manager = FailoverManager::new(1, shard_manager, replica_manager);

    let health = manager.get_cluster_health().await;
    assert_eq!(health.total_nodes, 1);
    assert_eq!(health.total_shards, 1);

    Ok(())
}

#[tokio::test]
async fn test_shard_router_with_partition() -> Result<(), Box<dyn std::error::Error>> {
    let mut shard_manager = ShardManager::new();

    let mut info = ShardInfo::new(0, 1);
    info.add_replica(2);
    shard_manager.create_shard(info);

    shard_manager.add_partition_rule(PartitionRule::new("users", PartitionKey::new_hash("id", 4)));

    let router = ShardRouter::new(shard_manager, 1);

    let value = PartitionValue::from_i64(100);
    let shard_id = router
        .get_shard_manager()
        .get_partition_key("users")
        .and_then(|pk| pk.partition(&value));

    assert!(shard_id.is_some());

    Ok(())
}

#[tokio::test]
async fn test_two_phase_commit_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut tpc = TwoPhaseCommit::new(1, true);

    let participant1 = Participant::new(2, 0);
    let participant2 = Participant::new(3, 0);

    let tx_id = tpc.begin_transaction(vec![participant1, participant2]);

    let tx = tpc.get_transaction(tx_id);
    assert!(tx.is_some());

    Ok(())
}

#[tokio::test]
async fn test_distributed_transaction_voting() -> Result<(), Box<dyn std::error::Error>> {
    let tx = DistributedTransaction::new(1, 1, vec![]);

    assert!(tx.all_voted_yes());
    assert!(!tx.any_voted_no());

    let mut participant1 = Participant::new(2, 0);
    let mut participant2 = Participant::new(3, 0);
    participant1.vote_yes();
    participant2.vote_yes();

    let tx2 = DistributedTransaction::new(1, 1, vec![participant1, participant2]);

    assert!(tx2.all_voted_yes());
    assert!(!tx2.any_voted_no());

    Ok(())
}

#[tokio::test]
async fn test_replica_manager_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = ShardReplicaManager::new(1);

    manager.register_shard(0, vec![1, 2, 3]);

    manager.become_leader(0)?;
    assert!(manager.is_leader(0));
    assert!(manager.get_primary(0).is_some());

    Ok(())
}

#[tokio::test]
async fn test_partition_lookup_performance() -> Result<(), Box<dyn std::error::Error>> {
    let partition_key = PartitionKey::new_hash("id", 16);

    for i in 0..100 {
        let value = PartitionValue::from_i64(i);
        partition_key.partition(&value);
    }

    let start = std::time::Instant::now();
    for i in 0..10000 {
        let value = PartitionValue::from_i64(i);
        partition_key.partition(&value);
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 100,
        "Partition lookup too slow: {:?}",
        elapsed
    );

    Ok(())
}

#[tokio::test]
async fn test_multi_shard_transaction() -> Result<(), Box<dyn std::error::Error>> {
    let mut tpc = TwoPhaseCommit::new(1, true);

    let p1 = Participant::new(2, 0);
    let p2 = Participant::new(3, 1);
    let p3 = Participant::new(4, 2);

    let tx_id = tpc.begin_transaction(vec![p1, p2, p3]);

    let prepare_result = tpc.prepare(tx_id);
    assert!(prepare_result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_partition_key_different_types() -> Result<(), Box<dyn std::error::Error>> {
    let hash_key = PartitionKey::new_hash("user_id", 4);

    let value_i64 = PartitionValue::from_i64(42);
    let shard1 = hash_key.partition(&value_i64).unwrap();

    let value_text = PartitionValue::from_text("hello");
    let shard2 = hash_key.partition(&value_text);

    assert!(shard1 < 4);
    assert!(shard2.is_some());

    Ok(())
}
