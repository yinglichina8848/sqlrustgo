use crate::error::SpillResult;
use crate::memory_tracker::AdaptiveMemoryTracker;
use crate::partition_manager::PartitionManager;
use crate::r#trait::SpillingIterator;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildEntry<K, V> {
    pub key: K,
    pub values: Vec<V>,
}

pub struct HashJoinSpillOperator<
    K: Hash + Eq + Clone + serde::Serialize + serde::de::DeserializeOwned,
    V: Clone + serde::Serialize + serde::de::DeserializeOwned,
> {
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
    build_hash: HashMap<K, Vec<V>>,
    probe_buffer: Vec<(K, V)>,
    spilled_entries: Vec<(K, V)>,
}

impl<
        K: Hash + Eq + Clone + serde::Serialize + serde::de::DeserializeOwned,
        V: Clone + serde::Serialize + serde::de::DeserializeOwned,
    > HashJoinSpillOperator<K, V>
{
    pub fn new(tracker: Arc<AdaptiveMemoryTracker>) -> SpillResult<Self> {
        Ok(Self {
            tracker,
            partition_manager: PartitionManager::new()?,
            build_hash: HashMap::new(),
            probe_buffer: Vec::new(),
            spilled_entries: Vec::new(),
        })
    }

    pub fn add_build(&mut self, key: K, value: V) -> SpillResult<()> {
        if self.tracker.should_spill() {
            self.spill_build_side()?;
        }

        let entry = self.build_hash.entry(key).or_default();
        entry.push(value);

        let size = std::mem::size_of::<K>() + std::mem::size_of::<V>();
        let _ = self.tracker.allocate(size as u64);
        Ok(())
    }

    pub fn add_probe(&mut self, key: K, value: V) {
        self.probe_buffer.push((key, value));
    }

    fn spill_build_side(&mut self) -> SpillResult<()> {
        for (key, values) in self.build_hash.drain() {
            for value in values {
                self.spilled_entries.push((key.clone(), value));
            }
        }
        let _partition_id = self
            .partition_manager
            .write_partition(&self.spilled_entries)?;
        self.spilled_entries.clear();
        Ok(())
    }

    pub fn build_hash_map(&self) -> &HashMap<K, Vec<V>> {
        &self.build_hash
    }

    pub fn get_probe_buffer(&self) -> &[(K, V)] {
        &self.probe_buffer
    }
}

impl<
        K: Hash + Eq + Clone + serde::Serialize + serde::de::DeserializeOwned,
        V: Clone + serde::Serialize + serde::de::DeserializeOwned,
    > SpillingIterator for HashJoinSpillOperator<K, V>
{
    fn start_spill(&mut self) -> SpillResult<()> {
        self.spill_build_side()
    }

    fn is_spilling(&self) -> bool {
        self.partition_manager.num_partitions() > 0
    }

    fn num_partitions(&self) -> usize {
        self.partition_manager.num_partitions()
    }

    fn finish_spill(&mut self) {
        self.build_hash.clear();
        self.probe_buffer.clear();
        self.spilled_entries.clear();
        self.partition_manager.cleanup();
    }
}

impl<
        K: Hash + Eq + Clone + serde::Serialize + serde::de::DeserializeOwned,
        V: Clone + serde::Serialize + serde::de::DeserializeOwned,
    > Iterator for HashJoinSpillOperator<K, V>
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.spilled_entries.pop()
    }
}
