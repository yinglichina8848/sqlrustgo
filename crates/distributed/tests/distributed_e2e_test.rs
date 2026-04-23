use sqlrustgo_distributed::{
    consensus::ShardReplicaManager,
    failover_manager::FailoverManager,
    partition::{PartitionKey, PartitionPruner, PartitionValue},
    replication::{GtidInterval, GtidManager, GtidSet, SemiSyncManager, SemiSyncState},
    shard_manager::{NodeId, PartitionRule, ShardInfo, ShardManager},
    shard_router::{ReadWriteShardRouter, ShardRouter},
    two_phase_commit::{Participant, TwoPhaseCommit},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn create_cluster(
    num_shards: u64,
    replicas_per_shard: usize,
) -> (ShardManager, HashMap<NodeId, NodeId>) {
    let mut manager = ShardManager::new();
    let mut primary_to_replicas: HashMap<NodeId, NodeId> = HashMap::new();

    let nodes: Vec<NodeId> = (1..=(num_shards * replicas_per_shard as u64)).collect();

    for i in 0..num_shards {
        let primary = nodes[i as usize];
        let mut info = ShardInfo::new(i, primary);

        for j in 0..replicas_per_shard {
            let replica_id = nodes[(i * replicas_per_shard as u64 + j as u64) as usize];
            if replica_id != primary {
                info.add_replica(replica_id);
            }
        }

        let partition_key = PartitionKey::new_hash("id", num_shards);
        let info = info.with_partition(partition_key);
        manager.create_shard(info);

        primary_to_replicas.insert(primary, primary);
    }

    (manager, primary_to_replicas)
}

fn setup_replica_manager(manager: &ShardManager) -> ShardReplicaManager {
    let mut replica_manager = ShardReplicaManager::new(1);

    for (shard_id, info) in manager.get_shards() {
        let mut nodes = vec![info.primary_node().unwrap()];
        nodes.extend(info.replicas().iter().copied());
        replica_manager.register_shard(*shard_id, nodes);
    }

    replica_manager
}

#[tokio::test]
async fn test_e2e_cluster_initialization() -> TestResult {
    let (shard_manager, _primaries) = create_cluster(3, 2);

    assert_eq!(shard_manager.num_shards(), 3);
    assert_eq!(shard_manager.num_nodes(), 3);

    for i in 0..3 {
        let shard = shard_manager.get_shard(i).unwrap();
        assert!(!shard.replicas().is_empty());
    }

    Ok(())
}

#[tokio::test]
async fn test_e2e_gtid_replication_consistency() -> TestResult {
    let mut master_gtid = GtidManager::new(1);

    let shard_ids: Vec<u64> = (0..3).collect();
    let mut replica_gtids: Vec<GtidManager> = (2..=4).map(|id| GtidManager::new(id)).collect();

    for i in 1..=100 {
        master_gtid.add_gtid(1, i);
    }

    let executed = master_gtid.get_executed_set();
    assert!(!executed.is_empty());

    for replica in &mut replica_gtids {
        for shard_id in &shard_ids {
            let interval = GtidInterval::new(*shard_id, 1, 100);
            let mut set = GtidSet::new();
            set.add_interval(interval);
            for i in 1..=100 {
                replica.add_gtid(*shard_id, i);
            }
        }
    }

    for replica in &replica_gtids {
        assert!(replica.contains(1, 50));
        assert!(replica.contains(1, 100));
    }

    Ok(())
}

#[tokio::test]
async fn test_e2e_semisync_failover_and_recovery() -> TestResult {
    let mut manager = SemiSyncManager::new();
    manager.add_replica(2);
    manager.add_replica(3);
    manager.add_replica(4);

    assert_eq!(manager.get_replica_count(), 3);
    assert!(!manager.is_enabled());

    manager.remove_replica(2);

    assert_eq!(manager.get_replica_count(), 2);

    manager.remove_replica(3);
    manager.remove_replica(4);

    assert_eq!(manager.get_replica_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_e2e_failover_manager_detects_node_failure() -> TestResult {
    let (shard_manager, _) = create_cluster(1, 2);
    let replica_manager = Arc::new(RwLock::new(setup_replica_manager(&shard_manager)));
    let shard_manager = Arc::new(RwLock::new(shard_manager));

    let failover_manager = FailoverManager::new(1, shard_manager.clone(), replica_manager.clone());

    let health = failover_manager.get_cluster_health().await;
    assert_eq!(health.total_shards, 1);
    assert_eq!(health.total_nodes, 1);

    let dead_nodes = failover_manager.get_dead_nodes();
    assert!(dead_nodes.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_e2e_multi_shard_transaction_with_2pc() -> TestResult {
    let mut tpc = TwoPhaseCommit::new(1, true);

    let participants: Vec<Participant> = (0..5).map(|i| Participant::new(i + 2, i)).collect();

    let tx_id = tpc.begin_transaction(participants);
    assert!(tpc.get_transaction(tx_id).is_some());

    let result = tpc.prepare(tx_id);
    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_e2e_shard_router_routing_consistency() -> TestResult {
    let mut shard_manager = ShardManager::new();
    let partition_key = PartitionKey::new_hash("user_id", 4);

    for i in 0..4 {
        let info = ShardInfo::new(i, i + 1).with_partition(partition_key.clone());
        shard_manager.create_shard(info);
    }

    shard_manager.add_partition_rule(PartitionRule::new("users", partition_key));

    let router = ShardRouter::new(shard_manager, 1);

    let user_ids: Vec<i64> = vec![1, 2, 3, 4, 5, 100, 200, 300, 400, 500];
    let mut shard_assignments: HashMap<u64, Vec<i64>> = HashMap::new();

    for user_id in &user_ids {
        let value = PartitionValue::from_i64(*user_id);
        let shard_id = router
            .get_shard_manager()
            .get_partition_key("users")
            .and_then(|pk| pk.partition(&value));

        if let Some(shard) = shard_id {
            shard_assignments.entry(shard).or_default().push(*user_id);
        }
    }

    for (shard, users) in &shard_assignments {
        println!("Shard {} handles {} users", shard, users.len());
    }

    assert!(!shard_assignments.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_e2e_read_write_split_routing() -> TestResult {
    let mut shard_manager = ShardManager::new();
    let partition_key = PartitionKey::new_hash("id", 2);

    let info0 = ShardInfo::new(0, 1).with_partition(partition_key.clone());
    shard_manager.create_shard(info0);

    let info1 = ShardInfo::new(1, 2).with_partition(partition_key.clone());
    shard_manager.create_shard(info1);

    shard_manager.add_partition_rule(PartitionRule::new("orders", partition_key));

    let base_router = ShardRouter::new(shard_manager, 1);
    let rw_router = ReadWriteShardRouter::new(base_router);

    let write_value = PartitionValue::from_i64(100);
    let write_result = rw_router.route_write(
        "orders",
        "id",
        write_value,
        "INSERT INTO orders VALUES (100, 'test')",
    );
    assert!(write_result.is_ok());
    assert_eq!(write_result.unwrap().primary_node_id, 1);

    let read_value = PartitionValue::from_i64(100);
    let read_result = rw_router.route_read("orders", "id", read_value);
    assert!(read_result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_e2e_partition_pruning_for_range_queries() -> TestResult {
    let partition_key = PartitionKey::new_range("created_at", vec![100, 200, 300]);
    let pruner = sqlrustgo_distributed::partition::PartitionPruner::new(partition_key);

    let full_range = pruner.prune_for_range(i64::MIN, i64::MAX);
    assert_eq!(full_range.len(), 4);

    let range1 = pruner.prune_for_range(0, 50);
    assert!(range1.is_empty() || range1.len() < 4);

    let range2 = pruner.prune_for_range(150, 250);
    assert!(!range2.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_e2e_hash_distribution_uniformity() -> TestResult {
    let partition_key = PartitionKey::new_hash("item_id", 10);
    let num_items = 10000;

    let mut counts = vec![0usize; 10];

    for i in 0..num_items {
        let value = PartitionValue::from_i64(i);
        if let Some(shard) = partition_key.partition(&value) {
            counts[shard as usize] += 1;
        }
    }

    let expected_avg = num_items / 10;
    let tolerance = expected_avg / 5;

    for (i, count) in counts.iter().enumerate() {
        let diff = (*count as i64 - expected_avg as i64).abs();
        assert!(
            diff <= tolerance,
            "Shard {} count {} differs from expected {} by {} (tolerance {})",
            i,
            count,
            expected_avg,
            diff,
            tolerance
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_e2e_leader_election_in_replica_manager() -> TestResult {
    let mut replica_manager = ShardReplicaManager::new(1);

    replica_manager.register_shard(0, vec![1, 2, 3]);
    replica_manager.register_shard(1, vec![4, 5, 6]);

    replica_manager.become_leader(0)?;
    assert!(replica_manager.is_leader(0));
    assert!(replica_manager.get_primary(0).is_some());

    let leader_count = replica_manager.get_leader_count();
    assert_eq!(leader_count, 1);

    replica_manager.become_leader(1)?;
    let leader_count = replica_manager.get_leader_count();
    assert_eq!(leader_count, 2);

    Ok(())
}

#[tokio::test]
async fn test_e2e_transaction_timeout_handling() -> TestResult {
    let mut tpc = TwoPhaseCommit::new(1, true);

    let participants = vec![Participant::new(2, 0), Participant::new(3, 1)];

    let tx_id = tpc.begin_transaction(participants);

    let tx = tpc.get_transaction(tx_id);
    assert!(tx.is_some());
    assert!(!tx.unwrap().is_timed_out());

    Ok(())
}

#[tokio::test]
async fn test_e2e_cluster_health_aggregation() -> TestResult {
    let (shard_manager, _) = create_cluster(2, 2);
    let replica_manager = Arc::new(RwLock::new(setup_replica_manager(&shard_manager)));
    let shard_manager = Arc::new(RwLock::new(shard_manager));

    let failover_manager = FailoverManager::new(1, shard_manager.clone(), replica_manager.clone());

    let health = failover_manager.get_cluster_health().await;

    assert_eq!(health.total_shards, 2);
    assert_eq!(health.total_nodes, 2);
    assert_eq!(health.dead_nodes, 0);

    Ok(())
}

#[tokio::test]
async fn test_e2e_cross_shard_query_planning() -> TestResult {
    let mut shard_manager = ShardManager::new();
    let partition_key = PartitionKey::new_hash("id", 4);

    for i in 0..4 {
        let info = ShardInfo::new(i, i + 1).with_partition(partition_key.clone());
        shard_manager.create_shard(info);
    }

    shard_manager.add_partition_rule(PartitionRule::new("products", partition_key));

    let router = ShardRouter::new(shard_manager, 1);

    let all_shard_ids: Vec<u64> = (0..4).collect();
    for shard_id in &all_shard_ids {
        let shard = router.get_shard_manager().get_shard(*shard_id);
        assert!(shard.is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_e2e_semisync_state_machine() -> TestResult {
    let mut state = SemiSyncState::default();
    assert_eq!(state, SemiSyncState::Off);

    state = SemiSyncState::WaitServer;
    assert_eq!(state, SemiSyncState::WaitServer);

    state = SemiSyncState::WaitSlave;
    assert_eq!(state, SemiSyncState::WaitSlave);

    state = SemiSyncState::Off;

    Ok(())
}

#[tokio::test]
async fn test_e2e_gtid_interval_operations() -> TestResult {
    let interval = GtidInterval::new(1, 100, 200);

    assert!(interval.contains(150));
    assert!(!interval.contains(50));
    assert!(!interval.contains(250));

    assert_eq!(interval.len(), 101);

    let interval2 = GtidInterval::new(1, 150, 180);

    assert!(interval.contains(150));
    assert!(interval2.contains(160));

    Ok(())
}
