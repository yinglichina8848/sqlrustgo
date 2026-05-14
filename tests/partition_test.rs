// Partition Tests (BP2-7)
//! Tests for RANGE Partition pruning functionality
//!
//! BP2 Gate: cargo test --test partition_test

use sqlrustgo_distributed::partition::{PartitionKey, PartitionPruner, PartitionValue};

/// Test range partition with yearly boundaries
#[test]
fn test_range_partition_yearly() {
    // Simulates: PARTITION BY RANGE (YEAR(batch_date))
    // Partitions: p2024 (less than 2025), p2025 (less than 2026), p_future (MAXVALUE)
    let partition_key = PartitionKey::new_range("batch_date", vec![2025, 2026]);

    // Value from 2024 should go to partition 0
    let value_2024 = PartitionValue::from_i64(2024);
    assert_eq!(partition_key.partition(&value_2024), Some(0));

    // Value from 2025 should go to partition 1
    let value_2025 = PartitionValue::from_i64(2025);
    assert_eq!(partition_key.partition(&value_2025), Some(1));

    // Value from 2026 should go to partition 2 (MAXVALUE equivalent)
    let value_2026 = PartitionValue::from_i64(2026);
    assert_eq!(partition_key.partition(&value_2026), Some(2));
}

/// Test partition pruning for equality predicate
#[test]
fn test_partition_pruner_equality() {
    let partition_key = PartitionKey::new_range("batch_date", vec![2025, 2026]);
    let pruner = PartitionPruner::new(partition_key);

    // Query: WHERE batch_date = '2025-06-01' -> YEAR = 2025
    let value = PartitionValue::from_i64(2025);
    let partitions = pruner.prune_for_equality(&value);
    assert_eq!(partitions, vec![1], "Should only scan partition p2025");
}

/// Test partition pruning for range predicate
#[test]
fn test_partition_pruner_range() {
    let partition_key = PartitionKey::new_range("batch_date", vec![2025, 2026]);
    let pruner = PartitionPruner::new(partition_key);

    // prune_for_range is conservative: it includes partitions whose ranges
    // overlap with the query range. For range [2025, 2025], partitions
    // 0 and 1 both overlap because partition 0 ends at 2025 (inclusive).
    let partitions = pruner.prune_for_range(2025, 2025);
    assert_eq!(
        partitions,
        vec![0, 1],
        "Overlapping partitions 0 and 1 for year 2025"
    );

    // Query: WHERE batch_date BETWEEN '2024-01-01' AND '2025-06-01'
    // -> YEAR between 2024 and 2025
    let partitions = pruner.prune_for_range(2024, 2025);
    assert_eq!(
        partitions,
        vec![0, 1],
        "Should scan partitions p2024 and p2025"
    );

    // prune_for_range(2025, i64::MAX) returns [0, 1, 2] because partition 0
    // has range (-inf, 2025) and 2025 overlaps with its boundary
    let partitions = pruner.prune_for_range(2025, i64::MAX);
    assert_eq!(
        partitions,
        vec![0, 1, 2],
        "Conservative: all partitions overlap with [2025, +inf)"
    );
}

/// Test partition pruning returns all partitions for unknown value
#[test]
fn test_partition_pruner_unknown_value() {
    let partition_key = PartitionKey::new_range("batch_date", vec![2025, 2026]);
    let pruner = PartitionPruner::new(partition_key);

    // Text value can't be evaluated for range partition
    let text_value = PartitionValue::from_text("not_a_number");
    let partitions = pruner.prune_for_value(&text_value);
    assert_eq!(
        partitions,
        vec![0, 1, 2],
        "Should scan all partitions when value type is incompatible"
    );
}

/// Test hash partition basic functionality
#[test]
fn test_hash_partition_basic() {
    let partition_key = PartitionKey::new_hash("user_id", 4);
    let pruner = PartitionPruner::new(partition_key);

    // Same value should always return same partition
    let value = PartitionValue::from_i64(42);
    let partitions = pruner.prune_for_equality(&value);
    assert_eq!(partitions.len(), 1);
    assert!(partitions[0] < 4);

    // Same value again should be consistent
    let partitions2 = pruner.prune_for_equality(&value);
    assert_eq!(partitions, partitions2);
}

/// Test list partition functionality
#[test]
fn test_list_partition() {
    use sqlrustgo_distributed::partition::ListPartition;

    let partitions = vec![
        ListPartition {
            id: 0,
            values: vec![1, 2, 3],
        },
        ListPartition {
            id: 1,
            values: vec![4, 5, 6],
        },
        ListPartition {
            id: 2,
            values: vec![7, 8, 9],
        },
    ];
    let partition_key = PartitionKey::new_list("region_id", partitions);
    let pruner = PartitionPruner::new(partition_key);

    let value = PartitionValue::from_i64(5);
    let result = pruner.prune_for_equality(&value);
    assert_eq!(result, vec![1], "Value 5 should be in partition 1");

    let value2 = PartitionValue::from_i64(9);
    let result2 = pruner.prune_for_equality(&value2);
    assert_eq!(result2, vec![2], "Value 9 should be in partition 2");
}

/// Test key partition functionality
#[test]
fn test_key_partition() {
    let partition_key = PartitionKey::new_key(vec!["dept_id".to_string()], 3);
    let pruner = PartitionPruner::new(partition_key);

    let value = PartitionValue::from_i64(100);
    let partitions = pruner.prune_for_equality(&value);
    assert_eq!(partitions.len(), 1);
    assert!(partitions[0] < 3);
}

/// Test total partitions count
#[test]
fn test_total_partitions() {
    // Range partition: 2 boundaries = 3 partitions
    let range_key = PartitionKey::new_range("year", vec![2020, 2025]);
    assert_eq!(range_key.total_partitions(), 3);

    // Hash partition: 4 shards = 4 partitions
    let hash_key = PartitionKey::new_hash("id", 4);
    assert_eq!(hash_key.total_partitions(), 4);

    // List partition: 3 list entries = 3 partitions
    use sqlrustgo_distributed::partition::ListPartition;
    let list_key = PartitionKey::new_list(
        "region",
        vec![
            ListPartition {
                id: 0,
                values: vec![1],
            },
            ListPartition {
                id: 1,
                values: vec![2],
            },
            ListPartition {
                id: 2,
                values: vec![3],
            },
        ],
    );
    assert_eq!(list_key.total_partitions(), 3);
}
