use criterion::{black_box, criterion_group, criterion_main, BenchmarkTime, Criterion};
use sqlrustgo_distributed::{
    consensus::ShardReplicaManager,
    failover_manager::{ClusterHealth, FailoverConfig, FailoverManager},
    replica_sync::{ReplicaSynchronizer, SyncConfig},
    shard_manager::{ShardId, ShardInfo, ShardManager},
    shard_router::ShardRouter,
    ClientPool, CrossShardQueryExecutor,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.benchmark_group("consensus")
        .bench_function("shard_replica_manager_register", |b| {
            b.to_async(&rt).iter(|| {
                let mut manager = ShardReplicaManager::new(1);
                manager.register_shard(black_box(ShardId(0)), vec![1, 2, 3]);
            });
        })
        .bench_function("become_leader", |b| {
            b.to_async(&rt).iter(|| {
                let mut manager = ShardReplicaManager::new(1);
                manager.register_shard(ShardId(0), vec![1, 2, 3]);
                manager.become_leader(black_box(ShardId(0))).unwrap();
            });
        });

    c.benchmark_group("failover")
        .bench_function("failover_manager_creation", |b| {
            let shard_manager = Arc::new(tokio::sync::RwLock::new(ShardManager::new()));
            let replica_manager = Arc::new(tokio::sync::RwLock::new(ShardReplicaManager::new(1)));
            let config = FailoverConfig::default();

            b.to_async(&rt).iter(|| {
                FailoverManager::with_config(
                    black_box(1),
                    shard_manager.clone(),
                    replica_manager.clone(),
                    config.clone(),
                )
            });
        })
        .bench_function("cluster_health_check", |b| {
            let rt = Runtime::new().unwrap();
            let shard_manager = Arc::new(tokio::sync::RwLock::new(ShardManager::new()));
            let replica_manager = Arc::new(tokio::sync::RwLock::new(ShardReplicaManager::new(1)));
            let mut manager = FailoverManager::new(1, shard_manager, replica_manager);

            b.to_async(&rt)
                .iter(|| async { manager.get_cluster_health().await });
        });

    c.benchmark_group("replica_sync")
        .bench_function("sync_progress_tracking", |b| {
            let router = Arc::new(tokio::sync::RwLock::new(ShardRouter::new(
                ShardManager::new(),
                1,
            )));
            let mut sync = ReplicaSynchronizer::new(router);

            b.iter(|| {
                sync.update_lsn(black_box(1), 100);
                sync.get_lsn(black_box(1))
            });
        });

    c.benchmark_group("sharding")
        .bench_function("label_based_partitioning", |b| {
            use sqlrustgo_graph::model::PropertyMap;
            use sqlrustgo_graph::sharded_graph::GraphShardId;
            use sqlrustgo_graph::sharded_graph::MultiShardGraphStore;

            let mut store = MultiShardGraphStore::new();
            store.register_label_sharding("User", GraphShardId(0));
            store.register_label_sharding("Product", GraphShardId(1));
            store.register_label_sharding("Order", GraphShardId(2));

            b.iter(|| {
                for i in 0..100 {
                    black_box(store.create_node("User", PropertyMap::new()));
                }
            });
        })
        .bench_function("hash_based_partitioning", |b| {
            use sqlrustgo_vector::sharded_index::ShardedVectorIndex;
            use sqlrustgo_vector::DistanceMetric;

            let mut index = ShardedVectorIndex::new(4, DistanceMetric::Cosine);

            b.iter(|| {
                for i in 0..100 {
                    let vector = vec![rand::random(), rand::random()];
                    black_box(index.insert(black_box(i as u64), &vector));
                }
            });
        })
        .bench_function("cross_shard_vector_search", |b| {
            use sqlrustgo_vector::sharded_index::ShardedVectorIndex;
            use sqlrustgo_vector::DistanceMetric;

            let mut index = ShardedVectorIndex::new(4, DistanceMetric::Cosine);
            for i in 0..1000 {
                let vector = vec![rand::random(), rand::random()];
                index.insert(i, &vector).unwrap();
            }

            b.iter(|| black_box(index.search(&[0.5, 0.5], 10)));
        });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(std::time::Duration::from_secs(1));
    targets = criterion_benchmark
}
criterion_main!(benches);
