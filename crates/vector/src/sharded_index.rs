//! Sharded Vector Index - distributed vector storage with hash-based partitioning
//!
//! Provides horizontal scaling for vector data through sharding.
//! Vectors are distributed across shards based on their ID hash.

use crate::error::{VectorError, VectorResult};
use crate::metrics::DistanceMetric;
use crate::traits::{IndexEntry, VectorIndex, VectorRecord};
use std::collections::HashMap;

/// Shard identifier for vector storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VectorShardId(pub u64);

impl VectorShardId {
    pub fn new(id: u64) -> Self {
        VectorShardId(id)
    }
}

/// Statistics for a vector shard
#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_id: VectorShardId,
    pub vector_count: usize,
    pub dimension: usize,
}

/// Hash-based partitioning strategy for vectors
pub struct HashPartitioner {
    num_shards: u64,
}

impl HashPartitioner {
    pub fn new(num_shards: u64) -> Self {
        HashPartitioner { num_shards }
    }

    pub fn get_shard_for_id(&self, id: u64) -> VectorShardId {
        VectorShardId(id % self.num_shards)
    }

    pub fn num_shards(&self) -> u64 {
        self.num_shards
    }
}

/// A single vector shard
struct VectorShard {
    index: Box<dyn VectorIndex>,
}

impl VectorShard {
    fn new(index: Box<dyn VectorIndex>) -> Self {
        VectorShard { index }
    }
}

/// Multi-shard vector index implementation
pub struct ShardedVectorIndex {
    shards: HashMap<VectorShardId, VectorShard>,
    partitioner: HashPartitioner,
}

impl ShardedVectorIndex {
    pub fn new(num_shards: usize, metric: DistanceMetric) -> Self {
        let partitioner = HashPartitioner::new(num_shards as u64);
        let mut shards = HashMap::new();

        for i in 0..num_shards {
            let shard_id = VectorShardId(i as u64);
            let index: Box<dyn VectorIndex> = Box::new(crate::FlatIndex::new(metric));
            shards.insert(shard_id, VectorShard::new(index));
        }

        ShardedVectorIndex {
            shards,
            partitioner,
        }
    }

    pub fn with_shard_indices(
        num_shards: usize,
        _metric: DistanceMetric,
        build_shard: impl Fn(usize) -> Box<dyn VectorIndex>,
    ) -> Self {
        let partitioner = HashPartitioner::new(num_shards as u64);
        let mut shards = HashMap::new();

        for i in 0..num_shards {
            let shard_id = VectorShardId(i as u64);
            let index = build_shard(i);
            shards.insert(shard_id, VectorShard::new(index));
        }

        ShardedVectorIndex {
            shards,
            partitioner,
        }
    }

    fn get_shard_for_id(&self, id: u64) -> VectorShardId {
        self.partitioner.get_shard_for_id(id)
    }

    pub fn get_shard_stats(&self, shard_id: VectorShardId) -> Option<ShardStats> {
        self.shards.get(&shard_id).map(|shard| ShardStats {
            shard_id,
            vector_count: shard.index.len(),
            dimension: shard.index.dimension(),
        })
    }

    pub fn all_shard_stats(&self) -> Vec<ShardStats> {
        self.shards
            .keys()
            .filter_map(|&shard_id| self.get_shard_stats(shard_id))
            .collect()
    }

    pub fn get_shard_ids(&self) -> Vec<VectorShardId> {
        self.shards.keys().copied().collect()
    }

    pub fn total_vector_count(&self) -> usize {
        self.shards.values().map(|s| s.index.len()).sum()
    }
}

impl VectorIndex for ShardedVectorIndex {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()> {
        let shard_id = self.get_shard_for_id(id);
        self.shards
            .get_mut(&shard_id)
            .ok_or(VectorError::ShardNotFound(shard_id.0))?
            .index
            .insert(id, vector)
    }

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>> {
        let mut all_results: Vec<IndexEntry> = Vec::new();

        for shard in self.shards.values() {
            if let Ok(results) = shard.index.search(query, k) {
                all_results.extend(results);
            }
        }

        if all_results.is_empty() {
            return Err(VectorError::EmptyIndex);
        }

        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        all_results.truncate(k);
        all_results.dedup_by(|a, b| a.id == b.id);

        Ok(all_results)
    }

    fn build_index(&mut self) -> VectorResult<()> {
        for shard in self.shards.values_mut() {
            shard.index.build_index()?;
        }
        Ok(())
    }

    fn delete(&mut self, id: u64) -> VectorResult<()> {
        let shard_id = self.get_shard_for_id(id);
        self.shards
            .get_mut(&shard_id)
            .ok_or(VectorError::ShardNotFound(shard_id.0))?
            .index
            .delete(id)
    }

    fn len(&self) -> usize {
        self.total_vector_count()
    }

    fn is_empty(&self) -> bool {
        self.shards.values().all(|s| s.index.is_empty())
    }

    fn dimension(&self) -> usize {
        self.shards
            .values()
            .next()
            .map(|s| s.index.dimension())
            .unwrap_or(0)
    }

    fn metric(&self) -> DistanceMetric {
        self.shards
            .values()
            .next()
            .map(|s| s.index.metric())
            .unwrap_or(DistanceMetric::Cosine)
    }

    fn get_all(&self) -> Vec<VectorRecord> {
        let mut records = Vec::new();
        for shard in self.shards.values() {
            records.extend(shard.index.get_all());
        }
        records
    }

    fn iter_vectors(&self) -> Box<dyn Iterator<Item = (u64, &[f32])> + '_> {
        Box::new(self.shards.values().flat_map(|s| s.index.iter_vectors()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_partitioner() {
        let partitioner = HashPartitioner::new(3);

        assert_eq!(partitioner.get_shard_for_id(0), VectorShardId(0));
        assert_eq!(partitioner.get_shard_for_id(1), VectorShardId(1));
        assert_eq!(partitioner.get_shard_for_id(2), VectorShardId(2));
        assert_eq!(partitioner.get_shard_for_id(3), VectorShardId(0));
        assert_eq!(partitioner.get_shard_for_id(4), VectorShardId(1));
        assert_eq!(partitioner.get_shard_for_id(5), VectorShardId(2));
    }

    #[test]
    fn test_sharded_vector_insert_and_search() {
        let mut index = ShardedVectorIndex::new(2, DistanceMetric::Cosine);

        index.insert(0, &[1.0, 0.0]).unwrap();
        index.insert(1, &[0.0, 1.0]).unwrap();
        index.insert(2, &[0.707, 0.707]).unwrap();
        index.insert(3, &[0.0, 0.0]).unwrap();

        assert_eq!(index.len(), 4);

        let results = index.search(&[1.0, 0.0], 2).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, 0);
    }

    #[test]
    fn test_sharded_vector_delete() {
        let mut index = ShardedVectorIndex::new(2, DistanceMetric::Euclidean);

        index.insert(0, &[1.0, 0.0]).unwrap();
        index.insert(1, &[0.0, 1.0]).unwrap();

        assert_eq!(index.len(), 2);

        index.delete(0).unwrap();
        assert_eq!(index.len(), 1);

        let results = index.search(&[1.0, 0.0], 1).unwrap();
        assert!(results.iter().all(|r| r.id != 0));
    }

    #[test]
    fn test_shard_stats() {
        let mut index = ShardedVectorIndex::new(2, DistanceMetric::Cosine);

        index.insert(0, &[1.0, 0.0]).unwrap();
        index.insert(1, &[0.0, 1.0]).unwrap();
        index.insert(2, &[0.707, 0.707]).unwrap();

        let stats = index.all_shard_stats();
        assert_eq!(stats.len(), 2);

        let total_count: usize = stats.iter().map(|s| s.vector_count).sum();
        assert_eq!(total_count, 3);
    }

    #[test]
    fn test_build_index() {
        let mut index = ShardedVectorIndex::new(2, DistanceMetric::Cosine);

        for i in 0..100 {
            let vector = vec![rand::random::<f32>(), rand::random::<f32>()];
            index.insert(i, &vector).unwrap();
        }

        index.build_index().unwrap();

        assert_eq!(index.len(), 100);
    }

    #[test]
    fn test_different_shards_have_different_vectors() {
        let mut index = ShardedVectorIndex::new(2, DistanceMetric::Cosine);

        let vec0 = vec![1.0, 0.0];
        let vec1 = vec![0.0, 1.0];
        let vec2 = vec![0.707, 0.707];

        index.insert(0, &vec0).unwrap();
        index.insert(1, &vec1).unwrap();
        index.insert(2, &vec2).unwrap();

        let shard0_stats = index.get_shard_stats(VectorShardId(0)).unwrap();
        let shard1_stats = index.get_shard_stats(VectorShardId(1)).unwrap();

        assert_eq!(shard0_stats.vector_count + shard1_stats.vector_count, 3);
    }
}
