//! Partition strategies for sharding
//!
//! Supports both hash and range partitioning.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStrategy {
    Hash { num_shards: u64 },
    Range { boundaries: Vec<i64> },
}

impl Default for PartitionStrategy {
    fn default() -> Self {
        PartitionStrategy::Hash { num_shards: 4 }
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

    pub fn partition(&self, value: &PartitionValue) -> Option<u64> {
        match &self.strategy {
            PartitionStrategy::Hash { num_shards } => Some(hash_partition(value, *num_shards)),
            PartitionStrategy::Range { boundaries } => range_partition(value, boundaries),
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
        PartitionValue::Integer(n) => (*n as i64).abs() as u64 % num_shards,
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
}
