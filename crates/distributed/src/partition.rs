//! Partition strategies for sharding
//!
//! Supports both hash and range partitioning.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStrategy {
    Hash {
        num_shards: u64,
    },
    Range {
        boundaries: Vec<i64>,
    },
    Key {
        columns: Vec<String>,
        num_partitions: u64,
    },
    List {
        partitions: Vec<ListPartition>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPartition {
    pub id: u64,
    pub values: Vec<i64>,
}

impl Default for PartitionStrategy {
    fn default() -> Self {
        PartitionStrategy::Hash { num_shards: 4 }
    }
}

impl PartitionStrategy {
    pub fn new_key(columns: Vec<String>, num_partitions: u64) -> Self {
        PartitionStrategy::Key {
            columns,
            num_partitions,
        }
    }

    pub fn new_list(partitions: Vec<ListPartition>) -> Self {
        PartitionStrategy::List { partitions }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionKey {
    pub column: String,
    pub strategy: PartitionStrategy,
}

impl PartitionKey {
    pub fn new_hash(column: &str, num_shards: u64) -> Self {
        Self {
            column: column.to_string(),
            strategy: PartitionStrategy::Hash { num_shards },
        }
    }

    pub fn new_range(column: &str, boundaries: Vec<i64>) -> Self {
        Self {
            column: column.to_string(),
            strategy: PartitionStrategy::Range { boundaries },
        }
    }

    pub fn new_key(columns: Vec<String>, num_partitions: u64) -> Self {
        Self {
            column: columns.first().cloned().unwrap_or_default(),
            strategy: PartitionStrategy::Key {
                columns,
                num_partitions,
            },
        }
    }

    pub fn new_list(column: &str, partitions: Vec<ListPartition>) -> Self {
        Self {
            column: column.to_string(),
            strategy: PartitionStrategy::List { partitions },
        }
    }

    pub fn partition(&self, value: &PartitionValue) -> Option<u64> {
        match &self.strategy {
            PartitionStrategy::Hash { num_shards } => Some(hash_partition(value, *num_shards)),
            PartitionStrategy::Range { boundaries } => range_partition(value, boundaries),
            PartitionStrategy::Key {
                columns,
                num_partitions,
            } => Some(key_partition(value, columns, *num_partitions)),
            PartitionStrategy::List { partitions } => list_partition(value, partitions),
        }
    }

    pub fn total_partitions(&self) -> u64 {
        match &self.strategy {
            PartitionStrategy::Hash { num_shards } => *num_shards,
            PartitionStrategy::Range { boundaries } => boundaries.len() as u64 + 1,
            PartitionStrategy::Key { num_partitions, .. } => *num_partitions,
            PartitionStrategy::List { partitions } => partitions.len() as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartitionValue {
    Integer(i64),
    Text(u64),
}

impl PartitionValue {
    pub fn from_i64(n: i64) -> Self {
        PartitionValue::Integer(n)
    }

    pub fn from_text(s: &str) -> Self {
        PartitionValue::Text(calculate_hash(s))
    }

    pub fn abs_value(&self) -> i64 {
        match self {
            PartitionValue::Integer(n) => n.abs(),
            PartitionValue::Text(h) => *h as i64,
        }
    }
}

pub fn hash_partition(value: &PartitionValue, num_shards: u64) -> u64 {
    match value {
        PartitionValue::Integer(n) => (*n).unsigned_abs() % num_shards,
        PartitionValue::Text(h) => h % num_shards,
    }
}

pub fn range_partition(value: &PartitionValue, boundaries: &[i64]) -> Option<u64> {
    let n = match value {
        PartitionValue::Integer(i) => *i,
        PartitionValue::Text(_) => return None,
    };

    if boundaries.is_empty() {
        return Some(0);
    }

    for (i, boundary) in boundaries.iter().enumerate() {
        if n < *boundary {
            return Some(i as u64);
        }
    }

    Some(boundaries.len() as u64)
}

pub fn calculate_hash(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub fn key_partition(value: &PartitionValue, columns: &[String], num_partitions: u64) -> u64 {
    match value {
        PartitionValue::Integer(n) => (*n).unsigned_abs() % num_partitions,
        PartitionValue::Text(h) => {
            let mut combined = *h;
            for col in columns.iter().skip(1) {
                combined = combined.wrapping_add(calculate_hash(col));
            }
            combined % num_partitions
        }
    }
}

pub fn list_partition(value: &PartitionValue, partitions: &[ListPartition]) -> Option<u64> {
    let n = match value {
        PartitionValue::Integer(i) => *i,
        PartitionValue::Text(_) => return None,
    };

    for partition in partitions {
        if partition.values.contains(&n) {
            return Some(partition.id);
        }
    }
    None
}

pub struct PartitionPruner {
    partition_key: PartitionKey,
}

impl PartitionPruner {
    pub fn new(partition_key: PartitionKey) -> Self {
        Self { partition_key }
    }

    pub fn prune_for_value(&self, value: &PartitionValue) -> Vec<u64> {
        if let Some(partition_id) = self.partition_key.partition(value) {
            vec![partition_id]
        } else {
            (0..self.partition_key.total_partitions()).collect()
        }
    }

    pub fn prune_for_range(&self, start: i64, end: i64) -> Vec<u64> {
        match &self.partition_key.strategy {
            PartitionStrategy::Range { boundaries } => {
                let mut relevant = Vec::new();
                for i in 0..=boundaries.len() {
                    let partition_start = if i == 0 { i64::MIN } else { boundaries[i - 1] };
                    let partition_end = if i >= boundaries.len() {
                        i64::MAX
                    } else {
                        boundaries[i]
                    };

                    if start <= partition_end && end >= partition_start {
                        relevant.push(i as u64);
                    }
                }
                relevant
            }
            _ => (0..self.partition_key.total_partitions()).collect(),
        }
    }

    pub fn prune_for_equality(&self, value: &PartitionValue) -> Vec<u64> {
        self.prune_for_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_partition_integer() {
        let value = PartitionValue::Integer(10);
        let shard = hash_partition(&value, 4);
        assert_eq!(shard, 2); // 10 % 4 = 2

        let shard2 = hash_partition(&value, 3);
        assert_eq!(shard2, 1); // 10 % 3 = 1
    }

    #[test]
    fn test_hash_partition_negative() {
        let value = PartitionValue::Integer(-10);
        let shard = hash_partition(&value, 4);
        assert_eq!(shard, 2); // |-10| % 4 = 2
    }

    #[test]
    fn test_range_partition() {
        let boundaries = vec![10, 20, 30];

        assert_eq!(
            range_partition(&PartitionValue::Integer(5), &boundaries),
            Some(0)
        );
        assert_eq!(
            range_partition(&PartitionValue::Integer(15), &boundaries),
            Some(1)
        );
        assert_eq!(
            range_partition(&PartitionValue::Integer(25), &boundaries),
            Some(2)
        );
        assert_eq!(
            range_partition(&PartitionValue::Integer(100), &boundaries),
            Some(3)
        );
    }

    #[test]
    fn test_range_partition_empty() {
        let value = PartitionValue::Integer(100);
        assert_eq!(range_partition(&value, &[]), Some(0));
    }

    #[test]
    fn test_partition_key_hash() {
        let key = PartitionKey::new_hash("user_id", 4);
        let shard = key.partition(&PartitionValue::Integer(7)).unwrap();
        assert_eq!(shard, 3); // 7 % 4 = 3
    }

    #[test]
    fn test_partition_key_range() {
        let key = PartitionKey::new_range("age", vec![18, 30, 65]);

        assert_eq!(key.partition(&PartitionValue::Integer(15)), Some(0));
        assert_eq!(key.partition(&PartitionValue::Integer(25)), Some(1));
        assert_eq!(key.partition(&PartitionValue::Integer(50)), Some(2));
        assert_eq!(key.partition(&PartitionValue::Integer(100)), Some(3));
    }

    #[test]
    fn test_calculate_hash() {
        let h1 = calculate_hash("hello");
        let h2 = calculate_hash("hello");
        let h3 = calculate_hash("world");

        assert_eq!(h1, h2); // same string = same hash
        assert_ne!(h1, h3); // different string = different hash
    }

    #[test]
    fn test_partition_value_from_i64() {
        let value = PartitionValue::from_i64(42);
        assert!(matches!(value, PartitionValue::Integer(42)));
    }

    #[test]
    fn test_partition_value_from_text() {
        let value = PartitionValue::from_text("hello");
        assert!(matches!(value, PartitionValue::Text(_)));
    }

    #[test]
    fn test_partition_value_abs_value() {
        let int_pos = PartitionValue::Integer(10);
        assert_eq!(int_pos.abs_value(), 10);

        let int_neg = PartitionValue::Integer(-10);
        assert_eq!(int_neg.abs_value(), 10);

        let text = PartitionValue::Text(42);
        assert_eq!(text.abs_value(), 42);
    }

    #[test]
    fn test_range_partition_text_returns_none() {
        let value = PartitionValue::Text(42);
        assert_eq!(range_partition(&value, &[10, 20]), None);
    }

    #[test]
    fn test_partition_strategy_default() {
        let strategy = PartitionStrategy::default();
        assert!(matches!(
            strategy,
            PartitionStrategy::Hash { num_shards: 4 }
        ));
    }

    #[test]
    fn test_partition_key_debug() {
        let key = PartitionKey::new_hash("id", 4);
        let debug = format!("{:?}", key);
        assert!(debug.contains("Hash"));
    }

    // KEY partition tests
    #[test]
    fn test_key_partition_integer() {
        let value = PartitionValue::Integer(15);
        let columns = vec!["region".to_string()];
        let shard = key_partition(&value, &columns, 4);
        assert_eq!(shard, 3); // 15 % 4 = 3
    }

    #[test]
    fn test_key_partition_multiple_columns() {
        let value = PartitionValue::Text(calculate_hash("user_id=123"));
        let columns = vec![
            "user_id".to_string(),
            "region".to_string(),
            "tenant".to_string(),
        ];
        let shard = key_partition(&value, &columns, 4);
        // Hash combining multiple columns
        assert!(shard < 4);
    }

    #[test]
    fn test_partition_key_key_strategy() {
        let key = PartitionKey::new_key(vec!["tenant_id".to_string(), "region".to_string()], 8);
        let shard = key.partition(&PartitionValue::Integer(20)).unwrap();
        assert_eq!(shard, 4); // 20 % 8 = 4
        assert_eq!(key.total_partitions(), 8);
    }

    #[test]
    fn test_partition_strategy_key_debug() {
        let strategy = PartitionStrategy::new_key(vec!["col1".to_string()], 4);
        let debug = format!("{:?}", strategy);
        assert!(debug.contains("Key"));
    }

    // LIST partition tests
    #[test]
    fn test_list_partition_exact_match() {
        let partitions = vec![
            ListPartition {
                id: 0,
                values: vec![1, 2, 3],
            },
            ListPartition {
                id: 1,
                values: vec![10, 20, 30],
            },
            ListPartition {
                id: 2,
                values: vec![100, 200, 300],
            },
        ];

        assert_eq!(
            list_partition(&PartitionValue::Integer(2), &partitions),
            Some(0)
        );
        assert_eq!(
            list_partition(&PartitionValue::Integer(20), &partitions),
            Some(1)
        );
        assert_eq!(
            list_partition(&PartitionValue::Integer(200), &partitions),
            Some(2)
        );
    }

    #[test]
    fn test_list_partition_no_match() {
        let partitions = vec![
            ListPartition {
                id: 0,
                values: vec![1, 2, 3],
            },
            ListPartition {
                id: 1,
                values: vec![10, 20, 30],
            },
        ];

        assert_eq!(
            list_partition(&PartitionValue::Integer(99), &partitions),
            None
        );
    }

    #[test]
    fn test_list_partition_text_returns_none() {
        let partitions = vec![ListPartition {
            id: 0,
            values: vec![1, 2, 3],
        }];
        assert_eq!(list_partition(&PartitionValue::Text(42), &partitions), None);
    }

    #[test]
    fn test_partition_key_list_strategy() {
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
        let key = PartitionKey::new_list("region", partitions.clone());

        assert_eq!(key.partition(&PartitionValue::Integer(3)), Some(0));
        assert_eq!(key.partition(&PartitionValue::Integer(6)), Some(1));
        assert_eq!(key.partition(&PartitionValue::Integer(9)), Some(2));
        assert_eq!(key.total_partitions(), 3);
    }

    #[test]
    fn test_partition_strategy_list_debug() {
        let partitions = vec![ListPartition {
            id: 0,
            values: vec![1, 2, 3],
        }];
        let strategy = PartitionStrategy::new_list(partitions);
        let debug = format!("{:?}", strategy);
        assert!(debug.contains("List"));
    }

    // PartitionPruner tests
    #[test]
    fn test_partition_pruner_hash() {
        let key = PartitionKey::new_hash("user_id", 4);
        let pruner = PartitionPruner::new(key);

        // Exact value prunes to single partition
        let result = pruner.prune_for_value(&PartitionValue::Integer(10));
        assert_eq!(result.len(), 1);

        // Another exact value prunes to its single partition
        let result = pruner.prune_for_equality(&PartitionValue::Integer(99));
        assert_eq!(result.len(), 1);
        assert_eq!(result, vec![3]); // 99 % 4 = 3
    }

    #[test]
    fn test_partition_pruner_range() {
        let key = PartitionKey::new_range("age", vec![18, 30, 65]);
        let pruner = PartitionPruner::new(key);

        // Range query prunes relevant partitions
        let result = pruner.prune_for_range(25, 50);
        // Partitions: P0=[MIN,18), P1=[18,30), P2=[30,65), P3=[65,MAX)
        // Range [25, 50] overlaps with P1 and P2
        assert!(result.contains(&1));
        assert!(result.contains(&2));
    }

    #[test]
    fn test_partition_pruner_key() {
        let key = PartitionKey::new_key(vec!["tenant_id".to_string()], 8);
        let pruner = PartitionPruner::new(key);

        let result = pruner.prune_for_value(&PartitionValue::Integer(16));
        assert_eq!(result.len(), 1); // Exact match prunes to 1
    }

    #[test]
    fn test_partition_pruner_list() {
        let partitions = vec![
            ListPartition {
                id: 0,
                values: vec![1, 2, 3],
            },
            ListPartition {
                id: 1,
                values: vec![4, 5, 6],
            },
        ];
        let key = PartitionKey::new_list("region", partitions);
        let pruner = PartitionPruner::new(key);

        // Known value prunes to specific partition
        let result = pruner.prune_for_value(&PartitionValue::Integer(3));
        assert_eq!(result, vec![0]);

        // Unknown value returns all partitions
        let result = pruner.prune_for_value(&PartitionValue::Integer(99));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_partition_pruner_equality() {
        let key = PartitionKey::new_hash("id", 4);
        let pruner = PartitionPruner::new(key);

        let result = pruner.prune_for_equality(&PartitionValue::Integer(5));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_list_partition_debug() {
        let partition = ListPartition {
            id: 1,
            values: vec![1, 2, 3],
        };
        let debug_str = format!("{:?}", partition);
        assert!(debug_str.contains("id: 1"));
        assert!(debug_str.contains("values"));
    }

    #[test]
    fn test_partition_strategy_default_hash() {
        let strategy = PartitionStrategy::default();
        match strategy {
            PartitionStrategy::Hash { num_shards } => assert_eq!(num_shards, 4),
            _ => panic!("Expected Hash strategy"),
        }
    }

    #[test]
    fn test_partition_strategy_hash() {
        let strategy = PartitionStrategy::Hash { num_shards: 8 };
        match strategy {
            PartitionStrategy::Hash { num_shards } => assert_eq!(num_shards, 8),
            _ => panic!("Expected Hash strategy"),
        }
    }

    #[test]
    fn test_partition_strategy_range() {
        let strategy = PartitionStrategy::Range {
            boundaries: vec![10, 20, 30],
        };
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("Range"));
    }

    #[test]
    fn test_partition_strategy_key() {
        let strategy = PartitionStrategy::new_key(vec!["col1".to_string()], 4);
        match strategy {
            PartitionStrategy::Key {
                columns,
                num_partitions,
            } => {
                assert_eq!(columns.len(), 1);
                assert_eq!(num_partitions, 4);
            }
            _ => panic!("Expected Key strategy"),
        }
    }

    #[test]
    fn test_partition_strategy_list() {
        let partitions = vec![ListPartition {
            id: 0,
            values: vec![1, 2],
        }];
        let strategy = PartitionStrategy::new_list(partitions);
        match strategy {
            PartitionStrategy::List { partitions } => assert_eq!(partitions.len(), 1),
            _ => panic!("Expected List strategy"),
        }
    }

    #[test]
    fn test_partition_key_new_key() {
        let key = PartitionKey::new_key(vec!["col1".to_string()], 4);
        assert_eq!(key.column, "col1");
        match key.strategy {
            PartitionStrategy::Key {
                columns,
                num_partitions,
            } => {
                assert_eq!(columns, vec!["col1"]);
                assert_eq!(num_partitions, 4);
            }
            _ => panic!("Expected Key strategy"),
        }
    }

    #[test]
    fn test_partition_key_debug_format() {
        let key = PartitionKey::new_hash("id", 4);
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("id"));
    }

    #[test]
    fn test_partition_value_integer() {
        let value = PartitionValue::Integer(42);
        match value {
            PartitionValue::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected Integer"),
        }
    }

    #[test]
    fn test_partition_value_text() {
        let value = PartitionValue::Text(42);
        match value {
            PartitionValue::Text(n) => assert_eq!(n, 42),
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_partition_value_debug() {
        let value = PartitionValue::Integer(42);
        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_hash_partition_negative_integer() {
        let value = PartitionValue::Integer(-42);
        let result = hash_partition(&value, 8);
        assert_eq!(result, 42 % 8);
    }

    #[test]
    fn test_range_partition_empty_boundaries() {
        let value = PartitionValue::Integer(100);
        let result = range_partition(&value, &[]);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_range_partition_text_value_returns_none() {
        let value = PartitionValue::Text(100);
        let result = range_partition(&value, &[10, 20, 30]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_range_partition_at_boundary() {
        let value = PartitionValue::Integer(10);
        let result = range_partition(&value, &[10, 20, 30]);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_range_partition_beyond_all_boundaries() {
        let value = PartitionValue::Integer(100);
        let result = range_partition(&value, &[10, 20, 30]);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_key_partition_with_multiple_columns() {
        let columns = vec!["col1".to_string(), "col2".to_string(), "col3".to_string()];
        let value = PartitionValue::Integer(10);
        let result = key_partition(&value, &columns, 8);
        assert!(result < 8);
    }

    #[test]
    fn test_key_partition_text_with_single_column() {
        let columns = vec!["col1".to_string()];
        let value = PartitionValue::Text(42);
        let result = key_partition(&value, &columns, 8);
        assert_eq!(result, 42 % 8);
    }

    #[test]
    fn test_list_partition_empty_partitions() {
        let value = PartitionValue::Integer(10);
        let result = list_partition(&value, &[]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_list_partition_text_value_returns_none() {
        let partitions = vec![ListPartition {
            id: 1,
            values: vec![1, 2, 3],
        }];
        let value = PartitionValue::Text(100);
        let result = list_partition(&value, &partitions);
        assert_eq!(result, None);
    }

    #[test]
    fn test_list_partition_not_found() {
        let partitions = vec![ListPartition {
            id: 1,
            values: vec![1, 2, 3],
        }];
        let value = PartitionValue::Integer(10);
        let result = list_partition(&value, &partitions);
        assert_eq!(result, None);
    }

    #[test]
    fn test_list_partition_found() {
        let partitions = vec![
            ListPartition {
                id: 0,
                values: vec![1, 2],
            },
            ListPartition {
                id: 1,
                values: vec![10, 20],
            },
            ListPartition {
                id: 2,
                values: vec![100, 200],
            },
        ];
        let value = PartitionValue::Integer(20);
        let result = list_partition(&value, &partitions);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_partition_pruner_new() {
        let key = PartitionKey::new_hash("id", 4);
        let pruner = PartitionPruner::new(key);
        assert_eq!(pruner.partition_key.total_partitions(), 4);
    }

    #[test]
    fn test_partition_pruner_prune_for_value_returns_single() {
        let key = PartitionKey::new_hash("id", 4);
        let pruner = PartitionPruner::new(key);
        let value = PartitionValue::Integer(10);
        let result = pruner.prune_for_value(&value);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_partition_pruner_prune_for_range_hash_strategy() {
        let key = PartitionKey::new_hash("id", 4);
        let pruner = PartitionPruner::new(key);
        let result = pruner.prune_for_range(0, 100);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_partition_pruner_prune_for_range_returns_relevant() {
        let key = PartitionKey::new_range("id", vec![10, 20, 30]);
        let pruner = PartitionPruner::new(key);
        let result = pruner.prune_for_range(15, 25);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_calculate_hash_deterministic() {
        let hash1 = calculate_hash("test");
        let hash2 = calculate_hash("test");
        let hash3 = calculate_hash("other");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
